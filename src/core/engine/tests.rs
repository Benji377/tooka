//! End-to-end tests of the sort engine on real files in temporary folders.

use super::{
    Progress, Recovery, SortError,
    execute::{execute, run_task},
    journal::{self, CommitTask, Journal, Record, RunLock},
    plan::build_plan,
    recover, sort,
};
use crate::{
    core::sorter::collect_files,
    rules::rule::{
        Action, Conditions, CopyAction, DeleteAction, MoveAction, OnConflict, RenameAction, Rule,
    },
    utils::fsutil::Fingerprint,
};
use std::{collections::BTreeMap, fs, path::PathBuf};
use tempfile::{TempDir, tempdir};

/// A source folder, a destination folder and a state folder (journal/lock).
struct Env {
    _tmp: TempDir,
    root: PathBuf,
    src: PathBuf,
    dest: PathBuf,
    state: PathBuf,
}

impl Env {
    fn new() -> Self {
        let tmp = tempdir().unwrap();
        let root = std::path::absolute(tmp.path()).unwrap();
        let (src, dest, state) = (root.join("src"), root.join("dest"), root.join("state"));
        fs::create_dir_all(&src).unwrap();
        Self {
            _tmp: tmp,
            root,
            src,
            dest,
            state,
        }
    }

    fn write(&self, rel: &str, content: &str) -> PathBuf {
        let path = self.root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        path
    }

    fn sort(&self, rules: &[Rule]) -> Result<super::SortReport, SortError> {
        let files = collect_files(&self.src).unwrap();
        sort(
            &files,
            &self.src,
            rules,
            false,
            &self.state,
            &Progress::none(),
        )
    }

    /// Every file below the root (except the state folder) with its content.
    fn snapshot(&self) -> BTreeMap<String, String> {
        walkdir::WalkDir::new(&self.root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file() && !e.path().starts_with(&self.state))
            .map(|e| {
                let rel = e.path().strip_prefix(&self.root).unwrap();
                (
                    rel.to_string_lossy().replace('\\', "/"),
                    fs::read_to_string(e.path()).unwrap(),
                )
            })
            .collect()
    }

    fn dest_str(&self, sub: &str) -> String {
        self.dest.join(sub).to_string_lossy().into_owned()
    }
}

fn rule(id: &str, ext: &str, then: Vec<Action>, on_conflict: OnConflict) -> Rule {
    Rule {
        id: id.into(),
        name: id.into(),
        enabled: true,
        description: None,
        priority: 1,
        when: Conditions {
            any: None,
            filename: None,
            extensions: Some(vec![ext.into()]),
            path: None,
            size_kb: None,
            mime_type: None,
            created_date: None,
            modified_date: None,
            is_symlink: None,
            metadata: None,
        },
        then,
        on_conflict,
    }
}

fn move_to(to: String) -> Action {
    Action::Move(MoveAction {
        to,
        preserve_structure: false,
    })
}

fn map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

#[test]
fn successful_sort_moves_copies_and_renames() {
    let env = Env::new();
    env.write("src/a.txt", "A");
    env.write("src/sub/b.log", "B");
    env.write("src/c.bin", "C");

    let rules = [
        rule(
            "texts",
            "txt",
            vec![move_to(env.dest_str("texts"))],
            OnConflict::Abort,
        ),
        rule(
            "logs",
            "log",
            vec![
                Action::Copy(CopyAction {
                    to: env.dest_str("logs"),
                    preserve_structure: true,
                }),
                Action::Rename(RenameAction {
                    to: "{{filename}}-copy.log".into(),
                }),
            ],
            OnConflict::Abort,
        ),
    ];
    let report = env.sort(&rules).unwrap();

    assert!(report.commit_failures.is_empty());
    assert_eq!(
        env.snapshot(),
        map(&[
            ("dest/logs/sub/b-copy.log", "B"),
            ("dest/texts/a.txt", "A"),
            ("src/c.bin", "C"),
            ("src/sub/b.log", "B"),
        ])
    );
    assert!(
        !journal::exists(&env.state),
        "journal is removed after success"
    );
}

#[test]
fn conflict_aborts_before_anything_changes() {
    let env = Env::new();
    env.write("src/a.txt", "new A");
    env.write("src/b.txt", "B");
    env.write("dest/texts/a.txt", "old A");
    let before = env.snapshot();

    let rules = [rule(
        "texts",
        "txt",
        vec![move_to(env.dest_str("texts"))],
        OnConflict::Abort,
    )];
    match env.sort(&rules) {
        Err(SortError::PlanIssues(issues)) => assert_eq!(issues.len(), 1),
        other => panic!("expected plan issues, got {other:?}"),
    }
    assert_eq!(env.snapshot(), before, "nothing may change");
    assert!(!journal::exists(&env.state));
}

#[test]
fn rename_policy_keeps_both_files() {
    let env = Env::new();
    env.write("src/a.txt", "new");
    env.write("dest/a.txt", "old");

    let rules = [rule(
        "texts",
        "txt",
        vec![move_to(env.dest_str(""))],
        OnConflict::Rename,
    )];
    env.sort(&rules).unwrap();
    assert_eq!(
        env.snapshot(),
        map(&[("dest/a (1).txt", "new"), ("dest/a.txt", "old")])
    );
}

/// A file appears at a destination after planning. The run must fail at
/// that point and undo everything it already did.
#[test]
fn failure_during_execution_rolls_back_everything() {
    let env = Env::new();
    env.write("src/1.txt", "one");
    env.write("src/2.txt", "two");
    env.write("src/3.txt", "three");
    let before = env.snapshot();

    let rules = [rule(
        "texts",
        "txt",
        vec![move_to(env.dest_str("t"))],
        OnConflict::Abort,
    )];
    let files = collect_files(&env.src).unwrap();
    let plan = build_plan(&files, &env.src, &rules, &|| {});
    assert!(plan.issues.is_empty());

    // Someone creates the destination of the last file meanwhile.
    env.write("dest/t/3.txt", "intruder");

    let journal = Journal::create(&env.state).unwrap();
    match execute(&plan, journal, &|| {}) {
        Err(SortError::RolledBack { cause }) => assert!(cause.contains("3.txt"), "{cause}"),
        other => panic!("expected a rollback, got {other:?}"),
    }

    let mut expected = before;
    expected.insert("dest/t/3.txt".into(), "intruder".into());
    assert_eq!(
        env.snapshot(),
        expected,
        "files 1 and 2 must be back in place"
    );
    assert!(!journal::exists(&env.state));
}

/// Simulates a crash after the first file was moved: the journal is left
/// behind, `sort` refuses to run, and `recover` restores the file.
#[test]
fn crash_is_recovered_from_the_journal() {
    let env = Env::new();
    let a = env.write("src/a.txt", "A");
    let dest_dir = env.dest.join("t");
    let b = dest_dir.join("a.txt");

    // What phase 2 would have done before the crash.
    let mut journal = Journal::create(&env.state).unwrap();
    journal
        .append(Record::Begin {
            version: journal::JOURNAL_VERSION,
            started: String::new(),
            source: env.src.clone(),
        })
        .unwrap();
    journal
        .append(Record::DirCreate {
            path: env.dest.clone(),
        })
        .unwrap();
    fs::create_dir(&env.dest).unwrap();
    journal
        .append(Record::DirCreate {
            path: dest_dir.clone(),
        })
        .unwrap();
    fs::create_dir(&dest_dir).unwrap();
    journal.append(Record::Claim { path: b.clone() }).unwrap();
    fs::write(&b, "").unwrap();
    journal
        .append(Record::Move {
            from: a.clone(),
            to: b.clone(),
        })
        .unwrap();
    fs::rename(&a, &b).unwrap();
    drop(journal); // crash: the journal stays on disk

    let rules = [rule(
        "texts",
        "txt",
        vec![move_to(env.dest_str("t"))],
        OnConflict::Abort,
    )];
    assert!(matches!(env.sort(&rules), Err(SortError::RecoveryPending)));

    assert!(matches!(recover(&env.state).unwrap(), Recovery::RolledBack));
    assert_eq!(env.snapshot(), map(&[("src/a.txt", "A")]));
    assert!(!env.dest.exists(), "created folders are removed again");
    assert!(matches!(
        recover(&env.state).unwrap(),
        Recovery::NothingToDo
    ));
}

#[test]
fn crash_during_commit_reports_unfinished_tasks() {
    let env = Env::new();
    let mut journal = Journal::create(&env.state).unwrap();
    let tasks = vec![
        CommitTask::Delete {
            path: env.root.join("x"),
            trash: false,
            expected: None,
        },
        CommitTask::Execute {
            command: "notify".into(),
            args: vec![],
        },
    ];
    journal.append(Record::CommitBegin { tasks }).unwrap();
    journal.append(Record::TaskDone { index: 0 }).unwrap();
    drop(journal);

    match recover(&env.state).unwrap() {
        Recovery::CommitInterrupted { unfinished } => {
            assert_eq!(unfinished.len(), 1);
            assert!(matches!(unfinished[0], CommitTask::Execute { .. }));
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(!journal::exists(&env.state));
}

#[test]
fn a_running_sort_blocks_other_sorts() {
    let env = Env::new();
    env.write("src/a.txt", "A");
    let _held = RunLock::try_acquire(&env.state).unwrap().unwrap();
    let rules = [rule(
        "texts",
        "txt",
        vec![move_to(env.dest_str(""))],
        OnConflict::Abort,
    )];
    assert!(matches!(env.sort(&rules), Err(SortError::Busy)));
    assert!(matches!(recover(&env.state), Err(SortError::Busy)));
}

#[test]
fn dry_run_changes_nothing_and_reports_the_plan() {
    let env = Env::new();
    env.write("src/a.txt", "A");
    let before = env.snapshot();
    let rules = [rule(
        "texts",
        "txt",
        vec![move_to(env.dest_str(""))],
        OnConflict::Abort,
    )];
    let files = collect_files(&env.src).unwrap();
    let report = sort(
        &files,
        &env.src,
        &rules,
        true,
        &env.state,
        &Progress::none(),
    )
    .unwrap();
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].action, "move");
    assert_eq!(env.snapshot(), before);
    assert!(!env.state.join(journal::JOURNAL_FILE).exists());
}

#[test]
fn delete_after_copy_deletes_the_copy() {
    let env = Env::new();
    env.write("src/a.tmp", "A");
    let rules = [rule(
        "tmp",
        "tmp",
        vec![
            Action::Copy(CopyAction {
                to: env.dest_str("backup"),
                preserve_structure: false,
            }),
            Action::Delete(DeleteAction { trash: false }),
        ],
        OnConflict::Abort,
    )];
    env.sort(&rules).unwrap();
    // The chain continues on the copy, so the copy is what gets deleted;
    // the original stays.
    assert_eq!(env.snapshot(), map(&[("src/a.tmp", "A")]));
}

#[cfg(unix)]
#[test]
fn execute_receives_the_path_and_failures_do_not_roll_back() {
    let env = Env::new();
    env.write("src/a.txt", "A");
    let marker = env.root.join("marker");
    let rules = [rule(
        "texts",
        "txt",
        vec![
            move_to(env.dest_str("")),
            Action::Execute(crate::rules::rule::ExecuteAction {
                command: "sh".into(),
                args: vec![
                    "-c".into(),
                    format!("echo \"$0\" > '{}'", marker.display()),
                    "{{path}}".into(),
                ],
            }),
            Action::Execute(crate::rules::rule::ExecuteAction {
                command: "false".into(),
                args: vec![],
            }),
        ],
        OnConflict::Abort,
    )];

    let report = env.sort(&rules).unwrap();
    assert_eq!(
        fs::read_to_string(&marker).unwrap().trim(),
        env.dest.join("a.txt").to_string_lossy()
    );
    assert_eq!(
        report.commit_failures.len(),
        1,
        "{:?}",
        report.commit_failures
    );
    assert!(env.dest.join("a.txt").exists(), "the move is kept");
}

#[test]
fn original_of_a_cross_drive_move_is_kept_if_it_changed() {
    let env = Env::new();
    let original = env.write("src/a.txt", "v1");
    let copy = env.write("dest/a.txt", "v1");
    let source = Fingerprint::of(&original).unwrap();
    let task = CommitTask::RemoveSource {
        path: original.clone(),
        copy: copy.clone(),
        source,
    };

    // Changed after the copy was made (e.g. a download still being written).
    fs::write(&original, "v2 - longer").unwrap();
    assert!(run_task(&task).is_err());
    assert!(original.exists());

    // Unchanged: the original is removed.
    fs::write(&original, "v1").unwrap();
    let task = CommitTask::RemoveSource {
        path: original.clone(),
        copy,
        source: Fingerprint::of(&original).unwrap(),
    };
    run_task(&task).unwrap();
    assert!(!original.exists());
}

#[test]
fn skip_policy_leaves_the_file_and_sorts_the_rest() {
    let env = Env::new();
    env.write("src/a.txt", "new");
    env.write("src/b.txt", "B");
    env.write("dest/a.txt", "old");
    let rules = [rule(
        "texts",
        "txt",
        vec![move_to(env.dest_str(""))],
        OnConflict::Skip,
    )];
    let report = env.sort(&rules).unwrap();
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(
        env.snapshot(),
        map(&[
            ("dest/a.txt", "old"),
            ("dest/b.txt", "B"),
            ("src/a.txt", "new")
        ])
    );
}

#[test]
fn unmatched_and_already_sorted_files_are_left_alone() {
    let env = Env::new();
    env.write("src/a.txt", "A");
    env.write("src/keep.bin", "K");
    let rules = [rule(
        "texts",
        "txt",
        vec![move_to(env.src.to_string_lossy().into_owned())],
        OnConflict::Abort,
    )];
    let report = env.sort(&rules).unwrap();
    let actions: Vec<_> = report.results.iter().map(|r| r.action.as_str()).collect();
    assert_eq!(actions, vec!["unchanged", "skip"]);
    assert_eq!(
        env.snapshot(),
        map(&[("src/a.txt", "A"), ("src/keep.bin", "K")])
    );
}

fn set_readonly(path: &std::path::Path) {
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(path, perms).unwrap();
}

/// Regression: copying a read-only file failed (the copy could not be
/// opened to set its timestamps), which rolled back the whole sort.
#[test]
fn read_only_files_can_be_copied_and_the_copy_undone() {
    let env = Env::new();
    let original = env.write("src/a.txt", "A");
    set_readonly(&original);
    let rules = [rule(
        "texts",
        "txt",
        vec![Action::Copy(CopyAction {
            to: env.dest_str(""),
            preserve_structure: false,
        })],
        OnConflict::Abort,
    )];
    env.sort(&rules).unwrap();
    let copy = env.dest.join("a.txt");
    assert_eq!(fs::read_to_string(&copy).unwrap(), "A");
    assert!(fs::metadata(&copy).unwrap().permissions().readonly());

    // Undo removes the read-only copy again (needs care on Windows).
    let problems = super::undo::undo(&[
        Record::Claim { path: copy.clone() },
        Record::Copy {
            from: original.clone(),
            to: copy.clone(),
            source: Fingerprint::of(&original).unwrap(),
        },
    ]);
    assert!(problems.is_empty(), "{problems:?}");
    assert!(!copy.exists());
}

/// After a crash, the user edited the copy before running recover: the
/// edited file must survive.
#[test]
fn recover_keeps_a_copy_the_user_changed() {
    let env = Env::new();
    let original = env.write("src/a.txt", "original");
    let copy = env.write("dest/a.txt", "edited by the user");
    let problems = super::undo::undo(&[
        Record::Claim { path: copy.clone() },
        Record::Copy {
            from: original.clone(),
            to: copy.clone(),
            source: Fingerprint::of(&original).unwrap(),
        },
    ]);
    assert_eq!(problems.len(), 1);
    assert_eq!(fs::read_to_string(&copy).unwrap(), "edited by the user");
}

/// An interrupted copy (the beginning of the original) is cleaned up.
#[test]
fn recover_removes_a_partial_copy() {
    let env = Env::new();
    let original = env.write("src/a.txt", "complete content");
    let copy = env.write("dest/a.txt", "complete");
    let problems = super::undo::undo(&[
        Record::Claim { path: copy.clone() },
        Record::Copy {
            from: original.clone(),
            to: copy.clone(),
            source: Fingerprint::of(&original).unwrap(),
        },
    ]);
    assert!(problems.is_empty(), "{problems:?}");
    assert!(!copy.exists());
}

#[test]
fn delete_keeps_a_file_that_changed_during_the_sort() {
    let env = Env::new();
    let path = env.write("src/a.tmp", "v1");
    let task = CommitTask::Delete {
        path: path.clone(),
        trash: false,
        expected: Some(Fingerprint::of(&path).unwrap()),
    };
    fs::write(&path, "v2 - changed").unwrap();
    assert!(run_task(&task).is_err());
    assert!(path.exists());
}

/// Works on case-sensitive (Linux) and case-insensitive (macOS, Windows)
/// filesystems alike.
#[test]
fn rename_that_only_changes_case() {
    let env = Env::new();
    env.write("src/report.txt", "R");
    let rules = [rule(
        "upper",
        "txt",
        vec![Action::Rename(RenameAction {
            to: "REPORT.txt".into(),
        })],
        OnConflict::Abort,
    )];
    env.sort(&rules).unwrap();
    let names: Vec<_> = fs::read_dir(&env.src)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["REPORT.txt"]);
}

/// A file whose name cannot be stored in the journal must not block the
/// other files (Linux allows such names; macOS and Windows do not).
#[cfg(target_os = "linux")]
#[test]
fn files_with_non_unicode_names_are_skipped() {
    use std::os::unix::ffi::OsStrExt;
    let env = Env::new();
    env.write("src/ok.txt", "ok");
    fs::write(
        env.src.join(std::ffi::OsStr::from_bytes(b"bad\xff.txt")),
        "x",
    )
    .unwrap();
    let rules = [rule(
        "texts",
        "txt",
        vec![move_to(env.dest_str(""))],
        OnConflict::Abort,
    )];
    let report = env.sort(&rules).unwrap();
    assert_eq!(report.warnings.len(), 1, "{:?}", report.warnings);
    assert!(env.dest.join("ok.txt").exists());
}

/// A move to another filesystem followed by a rename: the original must be
/// removed once the copy reached its final name. Needs a second
/// filesystem; uses /dev/shm (tmpfs) on Linux when it differs from the
/// temporary folder's filesystem, and is skipped otherwise.
#[cfg(target_os = "linux")]
#[test]
fn move_across_filesystems_then_rename() {
    use std::os::unix::fs::MetadataExt;
    let env = Env::new();
    let other = std::path::Path::new("/dev/shm");
    let Ok(other_meta) = fs::metadata(other) else {
        return;
    };
    if other_meta.dev() == fs::metadata(&env.root).unwrap().dev() {
        eprintln!("skipped: /dev/shm is on the same filesystem as the temp folder");
        return;
    }
    let far = tempfile::tempdir_in(other).unwrap();
    env.write("src/a.txt", "A");
    let rules = [rule(
        "texts",
        "txt",
        vec![
            move_to(far.path().to_string_lossy().into_owned()),
            Action::Rename(RenameAction {
                to: "renamed.txt".into(),
            }),
        ],
        OnConflict::Abort,
    )];
    let report = env.sort(&rules).unwrap();
    assert!(
        report.commit_failures.is_empty(),
        "{:?}",
        report.commit_failures
    );
    assert!(!env.src.join("a.txt").exists(), "original removed");
    assert_eq!(
        fs::read_to_string(far.path().join("renamed.txt")).unwrap(),
        "A"
    );
}

/// A minimal JPEG whose EXIF data says it was taken with a "Google" camera
/// on 2024-05-01 (little-endian TIFF: IFD0 with Make and a pointer to the
/// EXIF IFD, which holds DateTimeOriginal).
fn jpeg_with_exif() -> Vec<u8> {
    fn entry(tiff: &mut Vec<u8>, tag: u16, kind: u16, count: u32, value: u32) {
        tiff.extend(tag.to_le_bytes());
        tiff.extend(kind.to_le_bytes());
        tiff.extend(count.to_le_bytes());
        tiff.extend(value.to_le_bytes());
    }
    const ASCII: u16 = 2;
    const LONG: u16 = 4;
    let make = b"Google\0";
    let taken = b"2024:05:01 12:30:00\0";
    let mut tiff = b"II*\0".to_vec();
    tiff.extend(8u32.to_le_bytes()); // IFD0 offset
    // IFD0 at 8: 2 entries -> ends at 8 + 2 + 24 + 4 = 38
    tiff.extend(2u16.to_le_bytes());
    entry(&mut tiff, 0x010F, ASCII, make.len() as u32, 38); // Make
    entry(&mut tiff, 0x8769, LONG, 1, 46); // EXIF IFD pointer
    tiff.extend(0u32.to_le_bytes());
    tiff.extend(make);
    tiff.push(0); // pad to 46
    // EXIF IFD at 46: 1 entry -> data at 46 + 2 + 12 + 4 = 64
    tiff.extend(1u16.to_le_bytes());
    entry(&mut tiff, 0x9003, ASCII, taken.len() as u32, 64); // DateTimeOriginal
    tiff.extend(0u32.to_le_bytes());
    tiff.extend(taken);

    let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE1];
    jpeg.extend(((2 + 6 + tiff.len()) as u16).to_be_bytes());
    jpeg.extend(b"Exif\0\0");
    jpeg.extend(tiff);
    jpeg.extend([0xFF, 0xD9]);
    jpeg
}

/// Photo rules depend on the EXIF library; this catches a dependency
/// update that changes how EXIF data is read or shown.
#[test]
fn photos_are_matched_and_renamed_by_exif_data() {
    let env = Env::new();
    fs::write(env.src.join("IMG_1.jpg"), jpeg_with_exif()).unwrap();
    fs::write(env.src.join("IMG_2.jpg"), b"\xFF\xD8\xFF\xD9").unwrap(); // no EXIF

    let mut photos = rule(
        "photos",
        "jpg",
        vec![
            move_to(env.dest_str("photos")),
            Action::Rename(RenameAction {
                to: "{{metadata.EXIF:DateTime|date:%Y-%m-%d}}_{{metadata.EXIF:Make}}_{{filename}}.jpg"
                    .into(),
            }),
        ],
        OnConflict::Abort,
    );
    photos.when.metadata = Some(vec![
        crate::rules::rule::MetadataField {
            key: "EXIF:Make".into(),
            value: Some("Google".into()),
        },
        crate::rules::rule::MetadataField {
            key: "EXIF:DateTimeOriginal".into(),
            value: None,
        },
    ]);
    env.sort(&[photos]).unwrap();
    assert!(env.dest.join("photos/2024-05-01_Google_IMG_1.jpg").exists());
    assert!(env.src.join("IMG_2.jpg").exists(), "no EXIF, no match");
}
