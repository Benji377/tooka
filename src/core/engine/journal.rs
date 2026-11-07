//! The on-disk journal that makes a sort recoverable.
//!
//! Every file operation is written to the journal (and flushed to disk)
//! *before* it is performed. If Tooka crashes, is killed or loses power in
//! the middle of a sort, the journal tells `tooka recover` exactly what may
//! have happened, so it can be undone.
//!
//! The journal is a JSON Lines file: one [`Record`] per line. A record's
//! sequence number is its line index (the `Begin` record is 0).

use crate::utils::fsutil::{self, Fingerprint};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions, TryLockError},
    io::{self, Write},
    path::{Path, PathBuf},
};

/// Name of the journal file inside the state directory.
pub const JOURNAL_FILE: &str = "sort-journal.jsonl";
/// Name of the lock file that is held while a sort or recovery runs.
pub const LOCK_FILE: &str = "sort.lock";
/// Version of the journal format, written into the `Begin` record.
pub const JOURNAL_VERSION: u32 = 1;

/// One entry of the journal.
///
/// File operations are written as *intents* before they happen. Undo never
/// trusts that an intent completed; it inspects the filesystem instead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Record {
    /// Start of a sort run.
    Begin {
        version: u32,
        started: String,
        source: PathBuf,
    },
    /// A directory that did not exist is about to be created.
    DirCreate { path: PathBuf },
    /// An empty placeholder file is about to be created at `path` to reserve
    /// the destination.
    Claim { path: PathBuf },
    /// `from` is about to be renamed to `to` (same filesystem).
    Move { from: PathBuf, to: PathBuf },
    /// A move across filesystems: `from` is about to be copied into the
    /// claimed `to`. `from` is only removed after the whole sort succeeded.
    MoveViaCopy {
        from: PathBuf,
        to: PathBuf,
        source: Fingerprint,
    },
    /// `from` is about to be copied into the claimed `to`.
    Copy {
        from: PathBuf,
        to: PathBuf,
        source: Fingerprint,
    },
    /// The operation with sequence number `seq` failed without changing
    /// anything, so undo must ignore it.
    Void { seq: usize },
    /// All file operations succeeded; the run can no longer be undone.
    /// Lists the irreversible tasks that run from here on.
    CommitBegin { tasks: Vec<CommitTask> },
    /// Commit task number `index` was attempted.
    TaskDone { index: usize },
}

/// An irreversible step, performed only after all file operations succeeded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "task", rename_all = "snake_case")]
pub enum CommitTask {
    /// Remove the original of a cross-filesystem move, after checking that
    /// it did not change since it was copied to `copy`.
    RemoveSource {
        path: PathBuf,
        copy: PathBuf,
        source: Fingerprint,
    },
    /// Delete a file, either into the trash or permanently. `expected` is
    /// the file as it was when the sort reached it; a file that changed
    /// since then is kept.
    Delete {
        path: PathBuf,
        trash: bool,
        #[serde(default)]
        expected: Option<Fingerprint>,
    },
    /// Run an external command.
    Execute { command: String, args: Vec<String> },
}

impl std::fmt::Display for CommitTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RemoveSource { path, copy, .. } => write!(
                f,
                "remove the original {} (already copied to {})",
                path.display(),
                copy.display()
            ),
            Self::Delete {
                path, trash: true, ..
            } => write!(f, "move {} to the trash", path.display()),
            Self::Delete {
                path, trash: false, ..
            } => write!(f, "delete {}", path.display()),
            Self::Execute { command, args } => write!(f, "run `{command} {}`", args.join(" ")),
        }
    }
}

/// Exclusive lock on the state directory, held for the whole duration of a
/// sort or recovery. The operating system releases it automatically if the
/// process dies.
#[derive(Debug)]
pub struct RunLock {
    _file: File,
}

impl RunLock {
    /// Acquires the lock. Returns `Ok(None)` if another process holds it.
    pub fn try_acquire(state_dir: &Path) -> io::Result<Option<Self>> {
        fs::create_dir_all(state_dir)?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(state_dir.join(LOCK_FILE))?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Self { _file: file })),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(e)) => Err(e),
        }
    }
}

/// An open journal that records are appended to.
#[derive(Debug)]
pub struct Journal {
    path: PathBuf,
    file: File,
    records: Vec<Record>,
}

impl Journal {
    /// Creates a new journal. Fails if one already exists (an unfinished run
    /// must be recovered first).
    pub fn create(state_dir: &Path) -> io::Result<Self> {
        fs::create_dir_all(state_dir)?;
        let path = state_dir.join(JOURNAL_FILE);
        let file = OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(&path)?;
        fsutil::sync_dir(state_dir)?;
        Ok(Self {
            path,
            file,
            records: Vec::new(),
        })
    }

    /// Appends a record and flushes it to disk. Returns its sequence number.
    pub fn append(&mut self, record: Record) -> io::Result<usize> {
        let mut line = serde_json::to_vec(&record).map_err(io::Error::other)?;
        line.push(b'\n');
        self.file.write_all(&line)?;
        self.file.sync_data()?;
        self.records.push(record);
        Ok(self.records.len() - 1)
    }

    /// All records written so far.
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    /// Removes the journal after the run finished (or was fully undone).
    pub fn finish(self) -> io::Result<()> {
        let Self { path, file, .. } = self;
        drop(file);
        remove(&path)
    }
}

/// Path of the journal inside `state_dir`.
pub fn journal_path(state_dir: &Path) -> PathBuf {
    state_dir.join(JOURNAL_FILE)
}

/// Returns true if an unfinished journal exists.
pub fn exists(state_dir: &Path) -> bool {
    journal_path(state_dir).exists()
}

/// Reads the journal of an unfinished run, if there is one.
///
/// Every record is written as one line ending in `\n`. A crash can leave
/// the last record half-written (no trailing newline); it is ignored,
/// because its operation was never started (the intent is always flushed
/// before the operation runs). Any complete line that cannot be read is an
/// error, since guessing could make recovery do the wrong thing.
pub fn read(state_dir: &Path) -> io::Result<Option<Vec<Record>>> {
    let path = journal_path(state_dir);
    let content = match fs::read(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };

    let mut lines: Vec<&[u8]> = content.split(|b| *b == b'\n').collect();
    // After the final newline `split` yields an empty slice; without a final
    // newline the last element is an incomplete write.
    if let Some(last) = lines.pop()
        && !last.is_empty()
    {
        log::warn!("Ignoring incomplete last line of the sort journal");
    }

    let mut records = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        match serde_json::from_slice::<Record>(line) {
            Ok(record) => records.push(record),
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "the sort journal {} is damaged at line {}: {e}",
                        path.display(),
                        i + 1
                    ),
                ));
            }
        }
    }
    Ok(Some(records))
}

/// Deletes the journal file of `state_dir`.
pub fn discard(state_dir: &Path) -> io::Result<()> {
    remove(&journal_path(state_dir))
}

fn remove(path: &Path) -> io::Result<()> {
    match fsutil::retry(|| fs::remove_file(path)) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    if let Some(dir) = path.parent() {
        fsutil::sync_dir(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn journal_round_trip() {
        let dir = tempdir().unwrap();
        let mut journal = Journal::create(dir.path()).unwrap();
        let begin = Record::Begin {
            version: JOURNAL_VERSION,
            started: "now".into(),
            source: "/src".into(),
        };
        let claim = Record::Claim {
            path: "/dest/a".into(),
        };
        assert_eq!(journal.append(begin.clone()).unwrap(), 0);
        assert_eq!(journal.append(claim.clone()).unwrap(), 1);
        assert!(
            Journal::create(dir.path()).is_err(),
            "only one journal at a time"
        );

        let read_back = read(dir.path()).unwrap().unwrap();
        assert_eq!(read_back, vec![begin, claim]);

        journal.finish().unwrap();
        assert!(read(dir.path()).unwrap().is_none());
    }

    /// Every record and task type must survive a write/read cycle;
    /// `tooka recover` only ever sees what was read back from disk.
    #[test]
    fn every_record_type_reads_back() {
        let dir = tempdir().unwrap();
        let fp = Fingerprint {
            size: 42,
            modified_ns: Some(1_700_000_000_123_456_789),
        };
        let records = vec![
            Record::Begin {
                version: JOURNAL_VERSION,
                started: "now".into(),
                source: "/src".into(),
            },
            Record::DirCreate { path: "/d".into() },
            Record::Claim {
                path: "/d/a".into(),
            },
            Record::Move {
                from: "/src/a".into(),
                to: "/d/a".into(),
            },
            Record::MoveViaCopy {
                from: "/src/b".into(),
                to: "/d/b".into(),
                source: fp,
            },
            Record::Copy {
                from: "/src/c".into(),
                to: "/d/c".into(),
                source: Fingerprint {
                    size: 0,
                    modified_ns: None,
                },
            },
            Record::Void { seq: 3 },
            Record::CommitBegin {
                tasks: vec![
                    CommitTask::RemoveSource {
                        path: "/src/b".into(),
                        copy: "/d/b".into(),
                        source: fp,
                    },
                    CommitTask::Delete {
                        path: "/d/c".into(),
                        trash: true,
                        expected: Some(fp),
                    },
                    CommitTask::Execute {
                        command: "echo".into(),
                        args: vec!["x".into()],
                    },
                ],
            },
            Record::TaskDone { index: 0 },
        ];
        let mut journal = Journal::create(dir.path()).unwrap();
        for record in &records {
            journal.append(record.clone()).unwrap();
        }
        assert_eq!(read(dir.path()).unwrap().unwrap(), records);
    }

    #[test]
    fn truncated_last_line_is_ignored_but_damage_elsewhere_is_an_error() {
        let dir = tempdir().unwrap();
        let path = journal_path(dir.path());
        fs::write(
            &path,
            "{\"op\":\"claim\",\"path\":\"/a\"}\n{\"op\":\"claim\",\"pa",
        )
        .unwrap();
        assert_eq!(read(dir.path()).unwrap().unwrap().len(), 1);

        // A complete last line that cannot be read is damage, not a torn write.
        fs::write(
            &path,
            "{\"op\":\"claim\",\"path\":\"/a\"}\n{\"op\":\"nope\"}\n",
        )
        .unwrap();
        assert!(read(dir.path()).is_err());

        fs::write(&path, "{\"op\":\"cla\n{\"op\":\"claim\",\"path\":\"/a\"}\n").unwrap();
        assert!(read(dir.path()).is_err());
    }

    #[test]
    fn lock_is_exclusive() {
        let dir = tempdir().unwrap();
        let first = RunLock::try_acquire(dir.path()).unwrap();
        assert!(first.is_some());
        assert!(RunLock::try_acquire(dir.path()).unwrap().is_none());
        drop(first);
        assert!(RunLock::try_acquire(dir.path()).unwrap().is_some());
    }
}
