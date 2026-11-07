//! Phases 2 and 3 of a sort: performing the plan.
//!
//! Phase 2 performs only reversible operations (create folders, move,
//! rename, copy), each one journaled before it happens. If anything fails,
//! or the user presses Ctrl+C, everything done so far is undone.
//!
//! Phase 3 (commit) runs only after phase 2 fully succeeded. It performs the
//! irreversible tasks: removing originals of cross-filesystem moves, deletes
//! and commands. Their failures are reported but do not undo the sort,
//! because the files are already in a consistent, sorted state.

use super::{
    SortError,
    journal::{CommitTask, Journal, Record},
    plan::{Plan, Step},
    undo,
};
use crate::{
    common::logger::log_file_operation,
    core::interrupt,
    utils::fsutil::{self, Fingerprint},
};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

/// Performs the reversible part of `plan`, then commits.
///
/// Returns the failures of commit tasks (which do not undo the sort).
pub fn execute(
    plan: &Plan,
    mut journal: Journal,
    on_file_done: &dyn Fn(),
) -> Result<Vec<String>, SortError> {
    journal
        .append(Record::Begin {
            version: super::journal::JOURNAL_VERSION,
            started: fsutil::now_rfc3339(),
            source: plan.source_root.clone(),
        })
        .map_err(SortError::Journal)?;

    let mut runner = Runner {
        journal,
        tasks: Vec::new(),
        touched_dirs: BTreeSet::new(),
    };

    for file_plan in plan.active_files() {
        if interrupt::is_interrupted() {
            return Err(runner.roll_back("the sort was interrupted".into()));
        }
        for step in &file_plan.steps {
            if let Err(e) = runner.perform(step) {
                let cause = format!("{}: {e}", file_plan.file.display());
                return Err(runner.roll_back(cause));
            }
        }
        on_file_done();
    }

    runner.commit()
}

struct Runner {
    journal: Journal,
    /// Irreversible tasks, in plan order (and chain order within a file).
    tasks: Vec<CommitTask>,
    /// Directories whose entries changed; flushed to disk before commit.
    touched_dirs: BTreeSet<PathBuf>,
}

impl Runner {
    fn log(&mut self, record: Record) -> io::Result<usize> {
        self.journal.append(record)
    }

    /// Marks the intent `seq` as "did not happen".
    fn void(&mut self, seq: usize) -> io::Result<()> {
        self.log(Record::Void { seq }).map(drop)
    }

    fn perform(&mut self, step: &Step) -> io::Result<()> {
        match step {
            Step::Move { from, to } => self.relocate(from, to, false),
            Step::Rename {
                from,
                to,
                case_only,
            } => self.relocate(from, to, *case_only),
            Step::Copy { from, to } => {
                self.create_parents(to)?;
                self.claim(to)?;
                let source = Fingerprint::of(from)?;
                self.log(Record::Copy {
                    from: from.clone(),
                    to: to.clone(),
                    source,
                })?;
                // On failure the partial copy is removed by undo.
                fsutil::copy_durable(from, to)?;
                self.touch(to);
                log_file_operation(&format!(
                    "[copy] '{}' to '{}'",
                    from.display(),
                    to.display()
                ));
                Ok(())
            }
            Step::Delete { path, trash } => {
                self.tasks.push(CommitTask::Delete {
                    path: path.clone(),
                    trash: *trash,
                    expected: Some(Fingerprint::of(path)?),
                });
                Ok(())
            }
            Step::Execute { command, args, .. } => {
                self.tasks.push(CommitTask::Execute {
                    command: command.clone(),
                    args: args.clone(),
                });
                Ok(())
            }
            Step::Skip { .. } | Step::Unchanged { .. } => Ok(()),
        }
    }

    /// Moves `from` to `to`. Falls back to copy + deferred removal when
    /// `to` is on another filesystem.
    fn relocate(&mut self, from: &Path, to: &Path, case_only: bool) -> io::Result<()> {
        self.create_parents(to)?;
        if !case_only {
            self.claim(to)?;
        }

        let seq = self.log(Record::Move {
            from: from.to_path_buf(),
            to: to.to_path_buf(),
        })?;
        match fsutil::retry(|| fs::rename(from, to)) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::CrossesDevices && !case_only => {
                self.void(seq)?;
                let source = Fingerprint::of(from)?;
                self.log(Record::MoveViaCopy {
                    from: from.to_path_buf(),
                    to: to.to_path_buf(),
                    source,
                })?;
                fsutil::copy_durable(from, to)?;
                self.tasks.push(CommitTask::RemoveSource {
                    path: from.to_path_buf(),
                    copy: to.to_path_buf(),
                    source,
                });
            }
            Err(e) => {
                self.void(seq)?;
                return Err(e);
            }
        }

        // Guard against platforms silently changing the name (e.g. Windows
        // stripping trailing dots): undo needs the file at exactly `to`.
        if !fsutil::exists_no_follow(to) {
            return Err(io::Error::other(format!(
                "{} did not end up at {}",
                from.display(),
                to.display()
            )));
        }
        // If `from` is the copy of an earlier cross-filesystem move, its
        // original may only be removed once the copy is verified at its
        // final location.
        for task in &mut self.tasks {
            if let CommitTask::RemoveSource { copy, .. } = task
                && copy == from
            {
                *copy = to.to_path_buf();
            }
        }
        self.touch(from);
        self.touch(to);
        let action = if case_only { "rename" } else { "move" };
        log_file_operation(&format!(
            "[{action}] '{}' to '{}'",
            from.display(),
            to.display()
        ));
        Ok(())
    }

    /// Reserves `path` with an empty placeholder; fails if it is taken.
    fn claim(&mut self, path: &Path) -> io::Result<()> {
        let seq = self.log(Record::Claim {
            path: path.to_path_buf(),
        })?;
        if let Err(e) = fsutil::claim(path) {
            self.void(seq)?;
            return Err(if e.kind() == io::ErrorKind::AlreadyExists {
                io::Error::new(
                    e.kind(),
                    format!("{} appeared after planning", path.display()),
                )
            } else {
                e
            });
        }
        Ok(())
    }

    /// Creates the missing parent folders of `path`, top-down, journaling
    /// each one so that undo can remove them again.
    fn create_parents(&mut self, path: &Path) -> io::Result<()> {
        let mut missing = Vec::new();
        let mut dir = path.parent();
        while let Some(d) = dir {
            if fsutil::exists_no_follow(d) {
                break;
            }
            missing.push(d.to_path_buf());
            dir = d.parent();
        }
        for d in missing.into_iter().rev() {
            let seq = self.log(Record::DirCreate { path: d.clone() })?;
            match fs::create_dir(&d) {
                Ok(()) => self.touch(&d),
                // Created by someone else in the meantime: not ours to remove.
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists && d.is_dir() => {
                    self.void(seq)?;
                }
                Err(e) => {
                    self.void(seq)?;
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    fn touch(&mut self, path: &Path) {
        if let Some(parent) = path.parent() {
            self.touched_dirs.insert(parent.to_path_buf());
        }
    }

    /// Undoes everything and turns the failure into the matching error.
    fn roll_back(self, cause: String) -> SortError {
        log::error!("Sort failed, rolling back: {cause}");
        log_file_operation(&format!("[rollback] {cause}"));
        let problems = undo::undo(self.journal.records());
        if problems.is_empty() {
            match self.journal.finish() {
                Ok(()) => SortError::RolledBack { cause },
                Err(e) => SortError::RollbackIncomplete {
                    cause,
                    problems: vec![format!(
                        "All changes were undone, but the journal could not be removed: {e}"
                    )],
                },
            }
        } else {
            SortError::RollbackIncomplete { cause, problems }
        }
    }

    /// Phase 3: make the result durable, then run the irreversible tasks.
    fn commit(mut self) -> Result<Vec<String>, SortError> {
        // Copies are already synced; flush the directory entries of every
        // rename and new file before anything is removed.
        let flush_error = self.touched_dirs.iter().find_map(|dir| {
            fsutil::sync_dir(dir)
                .err()
                .map(|e| format!("could not flush {}: {e}", dir.display()))
        });
        if let Some(cause) = flush_error {
            return Err(self.roll_back(cause));
        }

        let tasks = std::mem::take(&mut self.tasks);
        if let Err(e) = self.log(Record::CommitBegin {
            tasks: tasks.clone(),
        }) {
            return Err(self.roll_back(format!("could not write the journal: {e}")));
        }

        let mut failures = Vec::new();
        for (index, task) in tasks.iter().enumerate() {
            match run_task(task) {
                Ok(()) => log_file_operation(&format!("[done] {task}")),
                Err(e) => {
                    log::error!("Commit task failed ({task}): {e}");
                    failures.push(format!("Could not {task}: {e}"));
                }
            }
            if let Err(e) = self.log(Record::TaskDone { index }) {
                log::error!("Could not write the journal: {e}");
            }
        }

        if let Err(e) = self.journal.finish() {
            failures.push(format!("Could not remove the sort journal: {e}"));
        }
        Ok(failures)
    }
}

/// Performs one irreversible commit task.
pub(super) fn run_task(task: &CommitTask) -> io::Result<()> {
    match task {
        CommitTask::RemoveSource { path, copy, source } => {
            // Only remove the original if neither it nor the copy changed;
            // otherwise keep both rather than lose an edit.
            let original_now = Fingerprint::of(path)?;
            if original_now != *source {
                return Err(io::Error::other(format!(
                    "{} changed after it was copied to {}; both were kept",
                    path.display(),
                    copy.display()
                )));
            }
            let copy_size = fs::metadata(copy)?.len();
            if copy_size != source.size {
                return Err(io::Error::other(format!(
                    "the copy at {} is incomplete; the original {} was kept",
                    copy.display(),
                    path.display()
                )));
            }
            fsutil::remove_file(path)
        }
        CommitTask::Delete {
            path,
            trash,
            expected,
        } => {
            if let Some(expected) = expected
                && Fingerprint::of(path)? != *expected
            {
                return Err(io::Error::other(format!(
                    "{} changed during the sort, so it was kept",
                    path.display()
                )));
            }
            if *trash {
                trash::delete(path).map_err(|e| io::Error::other(e.to_string()))
            } else {
                fsutil::remove_file(path)
            }
        }
        CommitTask::Execute { command, args } => {
            let output = Command::new(command).args(args).output()?;
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stderr = stderr.trim();
                Err(io::Error::other(if stderr.is_empty() {
                    format!("exited with {}", output.status)
                } else {
                    format!("exited with {}: {stderr}", output.status)
                }))
            }
        }
    }
}
