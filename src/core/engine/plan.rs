//! Phase 1 of a sort: building the plan.
//!
//! Planning never changes the filesystem. It matches every file against the
//! rules, simulates each rule's chain of actions to compute the exact final
//! paths, and collects every problem it can find up front (existing
//! destinations, two files with the same destination, invalid rules,
//! missing commands, ...). A sort with any problem stops here, before a
//! single file is touched.

use crate::{
    common::environment,
    core::interrupt,
    file::file_match,
    rules::rule::{Action, OnConflict, Rule},
    utils::{fsutil, rename_pattern},
};
use rayon::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::{Path, PathBuf},
};

/// One resolved step of a file's plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Move the file to another location (`move` action).
    Move { from: PathBuf, to: PathBuf },
    /// Rename the file within its folder (`rename` action).
    /// `case_only` is set when only upper/lower case changes on a
    /// filesystem that ignores case, where `to` already "exists".
    Rename {
        from: PathBuf,
        to: PathBuf,
        case_only: bool,
    },
    /// Copy the file; later actions of the rule continue on the copy.
    Copy { from: PathBuf, to: PathBuf },
    /// Delete the file (performed after all moves and copies succeeded).
    Delete { path: PathBuf, trash: bool },
    /// Run a command (performed after all moves and copies succeeded).
    Execute {
        command: String,
        args: Vec<String>,
        path: PathBuf,
    },
    /// The rule's `skip` action.
    Skip { path: PathBuf },
    /// A move/copy/rename whose destination is the file's current path.
    Unchanged { path: PathBuf, action: &'static str },
}

/// What happens to one collected file.
#[derive(Debug, Clone)]
pub struct FilePlan {
    /// The file as it was found in the source folder.
    pub file: PathBuf,
    /// The rule that matched, if any.
    pub rule_id: Option<String>,
    /// Steps to perform, in order. Empty if nothing happens to the file.
    pub steps: Vec<Step>,
    /// Set when the file is left alone because of `on_conflict: skip`.
    pub skipped_reason: Option<String>,
}

/// A problem that prevents the sort from running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanIssue {
    pub file: Option<PathBuf>,
    pub rule_id: Option<String>,
    pub message: String,
}

impl fmt::Display for PlanIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.rule_id, &self.file) {
            (Some(rule), Some(file)) => {
                write!(f, "{} (rule '{rule}'): {}", file.display(), self.message)
            }
            (Some(rule), None) => write!(f, "rule '{rule}': {}", self.message),
            (None, Some(file)) => write!(f, "{}: {}", file.display(), self.message),
            (None, None) => write!(f, "{}", self.message),
        }
    }
}

/// The complete plan of a sort run.
#[derive(Debug, Clone, Default)]
pub struct Plan {
    /// Absolute path of the folder being sorted.
    pub source_root: PathBuf,
    pub files: Vec<FilePlan>,
    /// Problems that stop the sort. Empty means the plan can be executed.
    pub issues: Vec<PlanIssue>,
    /// Notes worth showing that do not stop the sort.
    pub warnings: Vec<String>,
    /// Set if the user pressed Ctrl+C while planning.
    pub interrupted: bool,
}

impl Plan {
    /// Plans that actually change something.
    pub fn active_files(&self) -> impl Iterator<Item = &FilePlan> {
        self.files.iter().filter(|f| {
            f.skipped_reason.is_none()
                && f.steps
                    .iter()
                    .any(|s| !matches!(s, Step::Skip { .. } | Step::Unchanged { .. }))
        })
    }
}

/// Builds the plan for sorting `files` (collected from `source_root`) with
/// `rules`, which must already be filtered to enabled rules and sorted by
/// priority. `on_matched` is called once per file after matching.
pub fn build_plan(
    files: &[PathBuf],
    source_root: &Path,
    rules: &[Rule],
    on_matched: &(dyn Fn() + Sync),
) -> Plan {
    let source_root = absolute(source_root);
    let mut plan = Plan {
        source_root: source_root.clone(),
        ..Plan::default()
    };

    for rule in rules {
        if let Err(e) = rule.validate(true) {
            plan.issues.push(PlanIssue {
                file: None,
                rule_id: Some(rule.id.clone()),
                message: e.to_string(),
            });
        }
        check_commands(rule, &mut plan);
    }

    // Files already inside a destination folder of a rule are not sorted
    // again; otherwise every run would move them once more (nesting folders
    // with preserve_structure, or piling up "(1)" copies).
    let excluded_roots: Vec<PathBuf> = rules
        .iter()
        .flat_map(|rule| rule.then.iter())
        .filter_map(|action| match action {
            Action::Move(a) => Some(resolve_target_dir(&a.to)),
            Action::Copy(a) => Some(resolve_target_dir(&a.to)),
            _ => None,
        })
        .filter(|dir| dir.starts_with(&source_root) && *dir != source_root)
        .collect();

    if source_root.to_str().is_none() {
        plan.issues.push(PlanIssue {
            file: Some(source_root.clone()),
            rule_id: None,
            message:
                "the source folder's path is not valid Unicode, which Tooka cannot record safely"
                    .into(),
        });
        return plan;
    }

    let mut files: Vec<PathBuf> = files
        .iter()
        .map(|f| absolute(f))
        .filter(|f| !excluded_roots.iter().any(|root| f.starts_with(root)))
        .filter(|f| {
            // The journal stores paths as text; a file it cannot record
            // could not be undone, so it is left alone.
            let ok = f.to_str().is_some();
            if !ok {
                plan.warnings.push(format!(
                    "Skipped {}: the name is not valid Unicode",
                    f.display()
                ));
            }
            ok
        })
        .collect();
    files.sort();

    // Matching reads file metadata (and EXIF data), so it runs in parallel.
    let matched: Vec<Option<&Rule>> = files
        .par_iter()
        .map(|file| {
            let rule = if interrupt::is_interrupted() {
                None
            } else {
                rules
                    .iter()
                    .find(|rule| file_match::match_rule_matcher(file, &rule.when))
            };
            on_matched();
            rule
        })
        .collect();

    if interrupt::is_interrupted() {
        plan.interrupted = true;
        return plan;
    }

    // Destinations reserved by earlier files of this run.
    let mut reserved = HashSet::new();
    for (file, rule) in files.into_iter().zip(matched) {
        let file_plan = match rule {
            None => FilePlan {
                file,
                rule_id: None,
                steps: Vec::new(),
                skipped_reason: None,
            },
            Some(rule) => plan_file(file, rule, &source_root, &mut reserved, &mut plan),
        };
        plan.files.push(file_plan);
    }

    plan
}

/// Simulates `rule`'s actions on `file` and resolves every destination.
fn plan_file(
    file: PathBuf,
    rule: &Rule,
    source_root: &Path,
    reserved: &mut HashSet<String>,
    plan: &mut Plan,
) -> FilePlan {
    let mut file_plan = FilePlan {
        file: file.clone(),
        rule_id: Some(rule.id.clone()),
        steps: Vec::new(),
        skipped_reason: None,
    };
    // Destinations of this file only; merged into `reserved` at the end so
    // that a skipped file does not block anything.
    let mut mine: HashSet<String> = HashSet::new();
    let relative_dir = file
        .parent()
        .and_then(|p| p.strip_prefix(source_root).ok())
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let mut metadata: Option<HashMap<String, String>> = None;
    let mut current = file.clone();

    let issue = |plan: &mut Plan, message: String| {
        plan.issues.push(PlanIssue {
            file: Some(file.clone()),
            rule_id: Some(rule.id.clone()),
            message,
        });
    };

    for action in &rule.then {
        let (wanted, action_name) = match action {
            Action::Move(a) => (
                destination(&a.to, a.preserve_structure, &relative_dir, &current),
                "move",
            ),
            Action::Copy(a) => (
                destination(&a.to, a.preserve_structure, &relative_dir, &current),
                "copy",
            ),
            Action::Rename(a) => {
                let meta = metadata.get_or_insert_with(|| {
                    rename_pattern::extract_metadata(&file).unwrap_or_default()
                });
                let new_name = rename_pattern::evaluate_template(&a.to, &current, meta);
                if let Err(e) = fsutil::validate_file_name(&new_name) {
                    issue(plan, format!("invalid name from rename template: {e}"));
                    return file_plan;
                }
                (current.with_file_name(new_name), "rename")
            }
            Action::Delete(a) => {
                file_plan.steps.push(Step::Delete {
                    path: current.clone(),
                    trash: a.trash,
                });
                break;
            }
            Action::Execute(a) => {
                let path_str = current.to_string_lossy();
                file_plan.steps.push(Step::Execute {
                    command: a.command.clone(),
                    args: a
                        .args
                        .iter()
                        .map(|arg| arg.replace("{{path}}", &path_str))
                        .collect(),
                    path: current.clone(),
                });
                continue;
            }
            Action::Skip => {
                file_plan.steps.push(Step::Skip {
                    path: current.clone(),
                });
                continue;
            }
        };

        if wanted == current {
            file_plan.steps.push(Step::Unchanged {
                path: current.clone(),
                action: action_name,
            });
            continue;
        }

        // A rename that only changes case on a case-insensitive filesystem:
        // the "existing" destination is the file itself.
        let case_only = current == file
            && action_name == "rename"
            && fsutil::exists_no_follow(&wanted)
            && same_file::is_same_file(&current, &wanted).unwrap_or(false);

        if let Err(message) = check_parent(&wanted) {
            issue(plan, message);
            return file_plan;
        }

        let taken = |p: &Path| {
            let key = path_key(p);
            (!case_only && fsutil::exists_no_follow(p))
                || reserved.contains(&key)
                || mine.contains(&key)
        };

        let target = if taken(&wanted) {
            match rule.on_conflict {
                OnConflict::Abort => {
                    issue(
                        plan,
                        format!(
                            "{} already exists (or is the destination of another file). \
                             Set `on_conflict: skip` or `on_conflict: rename` on the rule to allow this.",
                            wanted.display()
                        ),
                    );
                    return file_plan;
                }
                OnConflict::Skip => {
                    let reason = format!("{} already exists", wanted.display());
                    plan.warnings.push(format!(
                        "Skipped {} (rule '{}'): {reason}",
                        file.display(),
                        rule.id
                    ));
                    file_plan.steps.clear();
                    file_plan.skipped_reason = Some(reason);
                    return file_plan;
                }
                OnConflict::Rename => match free_name(&wanted, &taken) {
                    Some(free) => free,
                    None => {
                        issue(
                            plan,
                            format!("no free name found next to {}", wanted.display()),
                        );
                        return file_plan;
                    }
                },
            }
        } else {
            wanted
        };

        mine.insert(path_key(&target));
        let step = match action_name {
            "move" => Step::Move {
                from: current.clone(),
                to: target.clone(),
            },
            "copy" => Step::Copy {
                from: current.clone(),
                to: target.clone(),
            },
            _ => Step::Rename {
                from: current.clone(),
                to: target.clone(),
                case_only: case_only && target.parent() == current.parent(),
            },
        };
        file_plan.steps.push(step);
        current = target;
    }

    reserved.extend(mine);
    file_plan
}

/// Records a problem for every `execute` command that cannot be found.
fn check_commands(rule: &Rule, plan: &mut Plan) {
    for action in &rule.then {
        if let Action::Execute(a) = action {
            match fsutil::command_exists(&a.command) {
                Some(true) => {}
                Some(false) => plan.issues.push(PlanIssue {
                    file: None,
                    rule_id: Some(rule.id.clone()),
                    message: format!("command '{}' was not found", a.command),
                }),
                None => plan.warnings.push(format!(
                    "Rule '{}': could not verify that command '{}' exists",
                    rule.id, a.command
                )),
            }
        }
    }
}

/// Fails if a parent of `path` exists but is not a directory.
fn check_parent(path: &Path) -> Result<(), String> {
    let mut dir = path.parent();
    while let Some(d) = dir {
        if let Ok(meta) = std::fs::metadata(d) {
            return if meta.is_dir() {
                Ok(())
            } else {
                Err(format!(
                    "cannot create {} because {} is a file, not a folder",
                    path.display(),
                    d.display()
                ))
            };
        }
        dir = d.parent();
    }
    Ok(())
}

/// Finds `name (1).ext`, `name (2).ext`, ... that is not taken.
fn free_name(wanted: &Path, taken: &dyn Fn(&Path) -> bool) -> Option<PathBuf> {
    let stem = wanted.file_stem()?.to_string_lossy().into_owned();
    let ext = wanted
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (1..10_000)
        .map(|n| wanted.with_file_name(format!("{stem} ({n}){ext}")))
        .find(|candidate| !taken(candidate))
}

/// Key used to detect two destinations that are the same file. Windows and
/// macOS filesystems ignore case by default, so case is folded there.
fn path_key(path: &Path) -> String {
    let s = path.to_string_lossy();
    if cfg!(any(windows, target_os = "macos")) {
        s.to_lowercase()
    } else {
        s.into_owned()
    }
}

/// Computes where a move or copy action puts `current`.
fn destination(to: &str, preserve_structure: bool, relative_dir: &Path, current: &Path) -> PathBuf {
    let dir = resolve_target_dir(to);
    let name = current.file_name().unwrap_or_default();
    if preserve_structure {
        dir.join(relative_dir).join(name)
    } else {
        dir.join(name)
    }
}

/// Resolves the `to` folder of a move or copy action to an absolute path.
///
/// * `./...` is relative to the current working directory,
/// * `~/...` is relative to the home directory,
/// * anything else is an absolute path (a missing leading `/` is added).
pub fn resolve_target_dir(to: &str) -> PathBuf {
    if to.starts_with('.') {
        absolute(Path::new(to))
    } else if let Some(rest) = to.strip_prefix('~') {
        environment::home_dir().join(rest.trim_start_matches(['/', '\\']))
    } else {
        PathBuf::from("/").join(to.trim_start_matches('/'))
    }
}

/// Makes `path` absolute without touching the filesystem (no symlink
/// resolution), so paths in the plan and journal never depend on the
/// current directory.
fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::rule::{Conditions, CopyAction, MoveAction, RenameAction};
    use std::fs;
    use tempfile::tempdir;

    fn rule(id: &str, then: Vec<Action>, on_conflict: OnConflict) -> Rule {
        Rule {
            id: id.into(),
            name: id.into(),
            enabled: true,
            description: None,
            priority: 1,
            when: Conditions {
                any: None,
                filename: None,
                extensions: Some(vec!["txt".into()]),
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

    fn move_to(dir: &Path) -> Action {
        Action::Move(MoveAction {
            to: dir.to_string_lossy().into_owned(),
            preserve_structure: false,
        })
    }

    fn plan_for(src: &Path, rules: &[Rule]) -> Plan {
        let files = crate::core::sorter::collect_files(src).unwrap();
        build_plan(&files, src, rules, &|| {})
    }

    #[test]
    fn existing_destination_aborts_by_default() {
        let tmp = tempdir().unwrap();
        let (src, dest) = (tmp.path().join("src"), tmp.path().join("dest"));
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::write(src.join("a.txt"), "new").unwrap();
        fs::write(dest.join("a.txt"), "old").unwrap();

        let plan = plan_for(&src, &[rule("r", vec![move_to(&dest)], OnConflict::Abort)]);
        assert_eq!(plan.issues.len(), 1, "{:?}", plan.issues);
        assert!(plan.issues[0].message.contains("already exists"));
    }

    #[test]
    fn skip_and_rename_policies() {
        let tmp = tempdir().unwrap();
        let (src, dest) = (tmp.path().join("src"), tmp.path().join("dest"));
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dest).unwrap();
        fs::write(src.join("a.txt"), "new").unwrap();
        fs::write(dest.join("a.txt"), "old").unwrap();

        let plan = plan_for(&src, &[rule("r", vec![move_to(&dest)], OnConflict::Skip)]);
        assert!(plan.issues.is_empty());
        assert!(plan.files[0].skipped_reason.is_some());
        assert_eq!(plan.active_files().count(), 0);

        let plan = plan_for(&src, &[rule("r", vec![move_to(&dest)], OnConflict::Rename)]);
        assert!(plan.issues.is_empty());
        assert_eq!(
            plan.files[0].steps,
            vec![Step::Move {
                from: absolute(&src.join("a.txt")),
                to: absolute(&dest.join("a (1).txt")),
            }]
        );
    }

    #[test]
    fn two_files_with_the_same_destination_conflict() {
        let tmp = tempdir().unwrap();
        let (src, dest) = (tmp.path().join("src"), tmp.path().join("dest"));
        fs::create_dir_all(src.join("x")).unwrap();
        fs::create_dir_all(src.join("y")).unwrap();
        fs::write(src.join("x/a.txt"), "1").unwrap();
        fs::write(src.join("y/a.txt"), "2").unwrap();

        let plan = plan_for(&src, &[rule("r", vec![move_to(&dest)], OnConflict::Abort)]);
        assert_eq!(plan.issues.len(), 1, "{:?}", plan.issues);

        let plan = plan_for(&src, &[rule("r", vec![move_to(&dest)], OnConflict::Rename)]);
        assert!(plan.issues.is_empty());
        let targets: Vec<_> = plan
            .files
            .iter()
            .flat_map(|f| &f.steps)
            .filter_map(|s| match s {
                Step::Move { to, .. } => to.file_name().map(|n| n.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect();
        assert_eq!(targets, vec!["a.txt", "a (1).txt"]);
    }

    #[test]
    fn files_already_in_a_destination_inside_the_source_are_not_resorted() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(src.join("texts")).unwrap();
        fs::write(src.join("new.txt"), "1").unwrap();
        fs::write(src.join("texts/old.txt"), "2").unwrap();

        let plan = plan_for(
            &src,
            &[rule(
                "r",
                vec![move_to(&src.join("texts"))],
                OnConflict::Abort,
            )],
        );
        assert!(plan.issues.is_empty(), "{:?}", plan.issues);
        assert_eq!(plan.files.len(), 1);
        assert!(plan.files[0].file.ends_with("new.txt"));
    }

    #[test]
    fn chain_reserves_intermediate_destinations_and_renames_in_place() {
        let tmp = tempdir().unwrap();
        let (src, dest) = (tmp.path().join("src"), tmp.path().join("dest"));
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.txt"), "1").unwrap();

        let plan = plan_for(
            &src,
            &[rule(
                "r",
                vec![
                    Action::Copy(CopyAction {
                        to: dest.to_string_lossy().into_owned(),
                        preserve_structure: false,
                    }),
                    Action::Rename(RenameAction {
                        to: "renamed_{{filename}}.txt".into(),
                    }),
                ],
                OnConflict::Abort,
            )],
        );
        assert!(plan.issues.is_empty(), "{:?}", plan.issues);
        assert_eq!(
            plan.files[0].steps,
            vec![
                Step::Copy {
                    from: absolute(&src.join("a.txt")),
                    to: absolute(&dest.join("a.txt")),
                },
                Step::Rename {
                    from: absolute(&dest.join("a.txt")),
                    to: absolute(&dest.join("renamed_a.txt")),
                    case_only: false,
                },
            ]
        );
    }

    #[test]
    fn rename_template_producing_a_path_is_rejected() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.txt"), "1").unwrap();

        // The template itself passes validation; only its result is a path.
        let plan = plan_for(
            &src,
            &[rule(
                "r",
                vec![Action::Rename(RenameAction { to: "..".into() })],
                OnConflict::Abort,
            )],
        );
        assert_eq!(plan.issues.len(), 1, "{:?}", plan.issues);
    }

    #[test]
    fn execute_gets_the_path_placeholder_and_unknown_commands_are_reported() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.txt"), "1").unwrap();

        let exec = |command: &str| {
            Action::Execute(crate::rules::rule::ExecuteAction {
                command: command.into(),
                args: vec!["--file={{path}}".into()],
            })
        };
        let plan = plan_for(
            &src,
            &[rule(
                "r",
                vec![exec("no-such-command-xyz")],
                OnConflict::Abort,
            )],
        );
        if cfg!(windows) {
            assert!(!plan.warnings.is_empty());
        } else {
            assert_eq!(plan.issues.len(), 1, "{:?}", plan.issues);
        }

        let plan = plan_for(&src, &[rule("r", vec![exec("sh")], OnConflict::Abort)]);
        let expected = format!("--file={}", absolute(&src.join("a.txt")).display());
        match &plan.files[0].steps[0] {
            Step::Execute { args, .. } => assert_eq!(args, &vec![expected]),
            other => panic!("unexpected step {other:?}"),
        }
    }
}
