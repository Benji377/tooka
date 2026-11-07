//! Undoing the file operations of an unfinished sort.
//!
//! The same code is used for an in-process rollback (an operation failed or
//! the user pressed Ctrl+C) and for `tooka recover` after a crash. It walks
//! the journal backwards and decides every step from the *actual* state of
//! the filesystem, so it is safe to run more than once (e.g. if recovery
//! itself is interrupted).
//!
//! Safety rules — undo only ever removes:
//! * an **empty** placeholder at a path Tooka claimed, or
//! * a **copy** Tooka made, and only while the original still exists and
//!   is unchanged.
//!
//! Whenever the filesystem does not look like either "the operation
//! happened" or "it did not happen", the involved paths are left alone,
//! protected from any later undo step, and reported as a problem.

use super::journal::Record;
use crate::utils::fsutil::{self, Fingerprint, exists_no_follow, is_empty_file};
use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

/// Undoes all non-voided operations recorded before the commit point, in
/// reverse order. Returns the problems that need the user's attention; an
/// empty list means the filesystem is back to its state before the sort.
pub fn undo(records: &[Record]) -> Vec<String> {
    let end = records
        .iter()
        .position(|r| matches!(r, Record::CommitBegin { .. }))
        .unwrap_or(records.len());
    let voided: HashSet<usize> = records
        .iter()
        .filter_map(|r| match r {
            Record::Void { seq } => Some(*seq),
            _ => None,
        })
        .collect();

    let mut undo = Undo::default();
    for (seq, record) in records[..end].iter().enumerate().rev() {
        if voided.contains(&seq) {
            continue;
        }
        match record {
            Record::Move { from, to } => undo.move_back(from, to),
            Record::MoveViaCopy { from, to, source } | Record::Copy { from, to, source } => {
                undo.remove_copy(from, to, *source);
            }
            Record::Claim { path } => undo.release_claim(path),
            Record::DirCreate { path } => undo.remove_dir(path),
            Record::Begin { .. }
            | Record::Void { .. }
            | Record::CommitBegin { .. }
            | Record::TaskDone { .. } => {}
        }
    }
    undo.problems
}

#[derive(Default)]
struct Undo {
    /// Paths that hold user data whose origin is unclear; never removed.
    protected: HashSet<PathBuf>,
    problems: Vec<String>,
}

impl Undo {
    fn protect(&mut self, path: &Path, problem: String) {
        self.protected.insert(path.to_path_buf());
        log::error!("Undo problem: {problem}");
        self.problems.push(problem);
    }

    fn is_protected(&self, path: &Path) -> bool {
        self.protected.contains(path)
    }

    /// Undo of a rename from `from` to `to`.
    fn move_back(&mut self, from: &Path, to: &Path) {
        if self.is_protected(to) {
            return;
        }
        let at_to = exists_no_follow(to);
        let at_from = exists_no_follow(from);

        match (at_from, at_to) {
            // The move happened: put the file back.
            (false, true) => self.rename_back(from, to),
            // Both names exist. Either they are the same file (a rename that
            // only changed upper/lower case on a case-insensitive
            // filesystem), or the move never happened and `to` is still our
            // empty placeholder, or something unexpected happened.
            (true, true) => {
                if same_file::is_same_file(from, to).unwrap_or(false) {
                    if from != to {
                        self.rename_back(from, to);
                    }
                } else if !is_empty_file(to) {
                    self.protect(
                        to,
                        format!(
                            "{} was moved to {}, but a new file now exists at {}. Both were left untouched.",
                            from.display(),
                            to.display(),
                            from.display()
                        ),
                    );
                }
            }
            // The move never happened, or was already undone.
            (true, false) => {}
            (false, false) => self.problems.push(format!(
                "{} is neither at its original location nor at {}.",
                from.display(),
                to.display()
            )),
        }
    }

    fn rename_back(&mut self, from: &Path, to: &Path) {
        match fsutil::retry(|| fs::rename(to, from)) {
            Ok(()) => log::info!("Undo: moved {} back to {}", to.display(), from.display()),
            Err(e) => self.protect(
                to,
                format!(
                    "Could not move {} back to {}: {e}",
                    to.display(),
                    from.display()
                ),
            ),
        }
    }

    /// Undo of a copy from `from` into `to` (a copy action, or a move across
    /// filesystems). The copy is only removed while the original is still
    /// there and unchanged, so the content always survives somewhere.
    fn remove_copy(&mut self, from: &Path, to: &Path, source: Fingerprint) {
        if self.is_protected(to) || !exists_no_follow(to) {
            return;
        }
        let original_intact = Fingerprint::of(from).is_ok_and(|now| now == source);
        if !original_intact {
            self.protect(
                to,
                format!(
                    "{} is missing or changed since it was copied, so the copy at {} was kept.",
                    from.display(),
                    to.display()
                ),
            );
            return;
        }
        // `to` must still hold (the beginning of) our copy, not something
        // the user saved there in the meantime.
        if !fsutil::is_prefix_of(to, from).unwrap_or(false) {
            self.protect(
                to,
                format!(
                    "{} no longer matches the copy of {} that Tooka made, so it was kept.",
                    to.display(),
                    from.display()
                ),
            );
            return;
        }
        if let Err(e) = remove_file(to) {
            self.protect(
                to,
                format!("Could not remove the copy at {}: {e}", to.display()),
            );
        } else {
            log::info!("Undo: removed copy {}", to.display());
        }
    }

    /// Undo of a claim: remove the placeholder if it is still empty.
    fn release_claim(&mut self, path: &Path) {
        if self.is_protected(path) || !is_empty_file(path) {
            return;
        }
        if let Err(e) = remove_file(path) {
            self.problems.push(format!(
                "Could not remove the empty placeholder {}: {e}",
                path.display()
            ));
        }
    }

    /// Undo of a directory creation: remove it if it is empty. A directory
    /// that is not empty is simply kept; that never loses data.
    fn remove_dir(&mut self, path: &Path) {
        match fs::remove_dir(path) {
            Ok(()) => log::info!("Undo: removed directory {}", path.display()),
            Err(e) => log::debug!("Kept directory {}: {e}", path.display()),
        }
    }
}

fn remove_file(path: &Path) -> io::Result<()> {
    match fsutil::remove_file(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn fp(path: &Path) -> Fingerprint {
        Fingerprint::of(path).unwrap()
    }

    #[test]
    fn completed_move_is_moved_back() {
        let dir = tempdir().unwrap();
        let (a, b) = (dir.path().join("a.txt"), dir.path().join("sub/a.txt"));
        fs::write(&a, "data").unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(&b, "").unwrap();
        fs::rename(&a, &b).unwrap();

        let problems = undo(&[
            Record::DirCreate { path: sub.clone() },
            Record::Claim { path: b.clone() },
            Record::Move {
                from: a.clone(),
                to: b.clone(),
            },
        ]);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(fs::read_to_string(&a).unwrap(), "data");
        assert!(!sub.exists(), "created directory is removed again");
    }

    #[test]
    fn move_that_never_happened_only_removes_placeholder() {
        let dir = tempdir().unwrap();
        let (a, b) = (dir.path().join("a.txt"), dir.path().join("b.txt"));
        fs::write(&a, "data").unwrap();
        fs::write(&b, "").unwrap(); // claimed placeholder

        let problems = undo(&[
            Record::Claim { path: b.clone() },
            Record::Move {
                from: a.clone(),
                to: b.clone(),
            },
        ]);
        assert!(problems.is_empty(), "{problems:?}");
        assert!(a.exists());
        assert!(!b.exists());
    }

    /// The scenario from the design review: after the move, a new file
    /// appears at the original location. The moved original must survive.
    #[test]
    fn moved_original_is_never_deleted_when_source_reappears() {
        let dir = tempdir().unwrap();
        let (a, b) = (dir.path().join("a.txt"), dir.path().join("b.txt"));
        fs::write(&b, "original content").unwrap(); // moved original
        fs::write(&a, "new download").unwrap(); // reappeared

        let problems = undo(&[
            Record::Claim { path: b.clone() },
            Record::Move {
                from: a.clone(),
                to: b.clone(),
            },
        ]);
        assert_eq!(problems.len(), 1);
        assert_eq!(fs::read_to_string(&b).unwrap(), "original content");
        assert_eq!(fs::read_to_string(&a).unwrap(), "new download");
    }

    #[test]
    fn copy_is_removed_only_while_original_is_unchanged() {
        let dir = tempdir().unwrap();
        let (a, b) = (dir.path().join("a.txt"), dir.path().join("b.txt"));
        fs::write(&a, "data").unwrap();
        fs::write(&b, "data").unwrap();
        let source = fp(&a);
        let records = [
            Record::Claim { path: b.clone() },
            Record::Copy {
                from: a.clone(),
                to: b.clone(),
                source,
            },
        ];

        // Original gone: keep the copy.
        fs::rename(&a, dir.path().join("elsewhere")).unwrap();
        let problems = undo(&records);
        assert_eq!(problems.len(), 1);
        assert!(b.exists());

        // Original back and unchanged: remove the copy.
        fs::rename(dir.path().join("elsewhere"), &a).unwrap();
        let problems = undo(&records);
        assert!(problems.is_empty(), "{problems:?}");
        assert!(!b.exists());
        assert!(a.exists());
    }

    #[test]
    fn chain_copy_then_move_is_undone_in_reverse() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        let c = dir.path().join("c.txt");
        fs::write(&a, "data").unwrap();
        let source = fp(&a);
        fs::copy(&a, &b).unwrap();
        fs::write(&c, "").unwrap();
        fs::rename(&b, &c).unwrap();

        let problems = undo(&[
            Record::Claim { path: b.clone() },
            Record::Copy {
                from: a.clone(),
                to: b.clone(),
                source,
            },
            Record::Claim { path: c.clone() },
            Record::Move {
                from: b.clone(),
                to: c.clone(),
            },
        ]);
        assert!(problems.is_empty(), "{problems:?}");
        assert!(a.exists());
        assert!(!b.exists());
        assert!(!c.exists());
    }

    #[test]
    fn voided_and_committed_records_are_ignored() {
        let dir = tempdir().unwrap();
        let b = dir.path().join("b.txt");
        fs::write(&b, "").unwrap(); // someone else's empty file

        let problems = undo(&[Record::Claim { path: b.clone() }, Record::Void { seq: 0 }]);
        assert!(problems.is_empty());
        assert!(b.exists(), "a voided claim must not remove the file");

        let problems = undo(&[
            Record::CommitBegin { tasks: vec![] },
            Record::Claim { path: b.clone() },
        ]);
        assert!(problems.is_empty());
        assert!(b.exists(), "nothing after the commit point is undone");
    }

    #[test]
    fn undo_is_idempotent() {
        let dir = tempdir().unwrap();
        let (a, b) = (dir.path().join("a.txt"), dir.path().join("b.txt"));
        fs::write(&b, "data").unwrap();
        let records = [
            Record::Claim { path: b.clone() },
            Record::Move {
                from: a.clone(),
                to: b.clone(),
            },
        ];
        assert!(undo(&records).is_empty());
        assert!(undo(&records).is_empty());
        assert_eq!(fs::read_to_string(&a).unwrap(), "data");
    }
}
