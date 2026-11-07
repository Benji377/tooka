//! The sort engine: plan, execute, commit, and recover.
//!
//! A sort either completes, or is undone completely, or (if even undoing
//! fails) tells the user exactly which files need attention. See
//! [`plan`] for phase 1, [`execute`] for phases 2 and 3, [`journal`] for the
//! on-disk record that makes crashes recoverable, and [`undo`] for the
//! rollback rules.

pub mod execute;
pub mod journal;
pub mod plan;
pub mod undo;

#[cfg(test)]
mod tests;

use crate::{core::sorter::MatchResult, rules::rule::Rule};
use journal::{CommitTask, Journal, Record, RunLock};
use plan::{Plan, PlanIssue, Step};
use std::{
    fmt::Write as _,
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Why a sort did not complete. Every variant says what state the files are
/// in, so the user is never left guessing.
#[derive(Debug, Error)]
pub enum SortError {
    #[error("{}", format_issues(.0))]
    PlanIssues(Vec<PlanIssue>),

    #[error("The sort was interrupted before anything was changed.")]
    InterruptedWhilePlanning,

    #[error(
        "Sorting failed: {cause}\nAll changes were undone; your files are exactly as they were before."
    )]
    RolledBack { cause: String },

    #[error("{}", format_incomplete(.cause, .problems))]
    RollbackIncomplete {
        cause: String,
        problems: Vec<String>,
    },

    #[error(
        "Another Tooka sort (or the monitoring daemon) is currently running. Try again when it has finished."
    )]
    Busy,

    #[error(
        "A previous sort was interrupted before it finished, so some files may not be where they belong.\nRun `tooka recover` to restore them, then sort again."
    )]
    RecoveryPending,

    #[error("Could not write the sort journal, so nothing was changed: {0}")]
    Journal(io::Error),

    #[error("Could not access the sort journal or lock in Tooka's data folder: {0}")]
    State(io::Error),

    #[error("Nothing was changed: {0}")]
    Setup(String),
}

fn format_issues(issues: &[PlanIssue]) -> String {
    let mut out = format!(
        "Nothing was changed, because {} problem(s) were found:",
        issues.len()
    );
    for issue in issues {
        let _ = write!(out, "\n  - {issue}");
    }
    out
}

fn format_incomplete(cause: &str, problems: &[String]) -> String {
    let mut out = format!(
        "Sorting failed: {cause}\nMost changes were undone, but these need your attention:"
    );
    for problem in problems {
        let _ = write!(out, "\n  - {problem}");
    }
    out.push_str(
        "\nThe sort journal was kept. After fixing the above, run `tooka recover` to finish undoing the sort.",
    );
    out
}

/// Result of a successful sort.
#[derive(Debug, Default)]
pub struct SortReport {
    /// One row per performed (or, in a dry run, planned) action.
    pub results: Vec<MatchResult>,
    /// Notes that did not stop the sort (e.g. files skipped by `on_conflict: skip`).
    pub warnings: Vec<String>,
    /// Irreversible tasks (delete, commands, removing originals of moves
    /// across drives) that failed after all files were sorted.
    pub commit_failures: Vec<String>,
}

/// Progress callbacks used by the CLI's progress bars.
pub struct Progress<'a> {
    /// Called once per file after it was matched against the rules.
    pub matched: &'a (dyn Fn() + Sync),
    /// Called with the number of files that will change, before executing.
    pub executing: &'a dyn Fn(usize),
    /// Called once per file after its operations were performed.
    pub file_done: &'a dyn Fn(),
}

impl Progress<'static> {
    /// No progress reporting.
    pub fn none() -> Self {
        Self {
            matched: &|| {},
            executing: &|_| {},
            file_done: &|| {},
        }
    }
}

/// Sorts `files` from `source_root` with `rules` (enabled and sorted by
/// priority). With `dry_run`, only the plan is computed and returned.
///
/// `state_dir` holds the journal and lock; it is normally the data
/// directory.
pub fn sort(
    files: &[PathBuf],
    source_root: &Path,
    rules: &[Rule],
    dry_run: bool,
    state_dir: &Path,
    progress: &Progress<'_>,
) -> Result<SortReport, SortError> {
    if dry_run {
        let plan = plan::build_plan(files, source_root, rules, progress.matched);
        if plan.interrupted {
            return Err(SortError::InterruptedWhilePlanning);
        }
        if !plan.issues.is_empty() {
            return Err(SortError::PlanIssues(plan.issues));
        }
        let mut warnings = plan.warnings.clone();
        if journal::exists(state_dir) {
            warnings.push(
                "A previous sort was interrupted; run `tooka recover` before sorting for real."
                    .into(),
            );
        }
        return Ok(SortReport {
            results: results_of(&plan),
            warnings,
            commit_failures: Vec::new(),
        });
    }

    // Hold the lock for the whole run so the daemon and a manual sort can
    // never work on files at the same time.
    let _lock = RunLock::try_acquire(state_dir)
        .map_err(SortError::State)?
        .ok_or(SortError::Busy)?;
    if journal::exists(state_dir) {
        return Err(SortError::RecoveryPending);
    }

    let plan = plan::build_plan(files, source_root, rules, progress.matched);
    if plan.interrupted {
        return Err(SortError::InterruptedWhilePlanning);
    }
    if !plan.issues.is_empty() {
        return Err(SortError::PlanIssues(plan.issues));
    }

    let results = results_of(&plan);
    let active = plan.active_files().count();
    if active == 0 {
        return Ok(SortReport {
            results,
            warnings: plan.warnings,
            commit_failures: Vec::new(),
        });
    }

    (progress.executing)(active);
    let journal = Journal::create(state_dir).map_err(SortError::Journal)?;
    let commit_failures = execute::execute(&plan, journal, progress.file_done)?;

    Ok(SortReport {
        results,
        warnings: plan.warnings,
        commit_failures,
    })
}

/// What `tooka recover` did.
#[derive(Debug)]
pub enum Recovery {
    /// There was no unfinished sort.
    NothingToDo,
    /// The unfinished sort was undone completely.
    RolledBack,
    /// Some changes could not be undone; the journal was kept.
    Incomplete { problems: Vec<String> },
    /// The sort had already sorted all files and was finishing its
    /// irreversible tasks; these may not have run.
    CommitInterrupted { unfinished: Vec<CommitTask> },
}

/// Recovers from a sort that did not finish (crash, kill, power loss).
pub fn recover(state_dir: &Path) -> Result<Recovery, SortError> {
    let _lock = RunLock::try_acquire(state_dir)
        .map_err(SortError::State)?
        .ok_or(SortError::Busy)?;

    let Some(records) = journal::read(state_dir).map_err(SortError::State)? else {
        return Ok(Recovery::NothingToDo);
    };

    let commit = records.iter().find_map(|r| match r {
        Record::CommitBegin { tasks } => Some(tasks.clone()),
        _ => None,
    });

    if let Some(tasks) = commit {
        let done: std::collections::HashSet<usize> = records
            .iter()
            .filter_map(|r| match r {
                Record::TaskDone { index } => Some(*index),
                _ => None,
            })
            .collect();
        let unfinished: Vec<CommitTask> = tasks
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !done.contains(i))
            .map(|(_, t)| t)
            .collect();
        for task in &unfinished {
            log::warn!("Recover: task may not have run: {task}");
        }
        journal::discard(state_dir).map_err(SortError::State)?;
        return Ok(Recovery::CommitInterrupted { unfinished });
    }

    let problems = undo::undo(&records);
    if problems.is_empty() {
        journal::discard(state_dir).map_err(SortError::State)?;
        Ok(Recovery::RolledBack)
    } else {
        Ok(Recovery::Incomplete { problems })
    }
}

/// Turns a plan into report rows (one per action, one per untouched file).
fn results_of(plan: &Plan) -> Vec<MatchResult> {
    let mut rows = Vec::new();
    for file_plan in &plan.files {
        let file_name = file_plan
            .file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let rule_id = file_plan.rule_id.clone().unwrap_or_else(|| "none".into());
        let row = |action: &str, current: &Path, new: &Path| MatchResult {
            file_name: file_name.clone(),
            action: action.into(),
            matched_rule_id: rule_id.clone(),
            current_path: current.to_path_buf(),
            new_path: new.to_path_buf(),
        };

        if file_plan.steps.is_empty() {
            rows.push(row("skip", &file_plan.file, &file_plan.file));
            continue;
        }
        for step in &file_plan.steps {
            rows.push(match step {
                Step::Move { from, to } => row("move", from, to),
                Step::Rename { from, to, .. } => row("rename", from, to),
                Step::Copy { from, to } => row("copy", from, to),
                Step::Delete { path, .. } => row("delete", path, Path::new("[deleted]")),
                Step::Execute { path, .. } => row("execute", path, path),
                Step::Skip { path } => row("skip", path, path),
                Step::Unchanged { path, .. } => row("unchanged", path, path),
            });
        }
    }
    rows
}
