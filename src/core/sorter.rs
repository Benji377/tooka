//! Collecting the files to sort, and the result rows used by reports.
//!
//! The sorting itself (plan, execute, undo) lives in [`crate::core::engine`].

use super::error::TookaError;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// One row of a sort report: an action performed (or planned) on a file.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    /// File name matched by the rule.
    pub file_name: String,
    /// Action performed on the file (move, copy, rename, delete, execute,
    /// skip, unchanged).
    pub action: String,
    /// ID of the rule that matched, or `none`.
    pub matched_rule_id: String,
    /// The file's path before the action.
    pub current_path: PathBuf,
    /// The file's path after the action.
    pub new_path: PathBuf,
}

/// Recursively collects all regular files below `dir`, sorted by path.
///
/// Symbolic links are not followed and not collected. Entries that cannot
/// be read are logged and skipped.
pub fn collect_files(dir: &Path) -> Result<Vec<PathBuf>, TookaError> {
    if !dir.is_dir() {
        return Err(TookaError::ConfigError(format!(
            "Path '{}' does not exist or is not a directory.",
            dir.display()
        )));
    }

    let mut files: Vec<PathBuf> = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(e) if e.file_type().is_file() => Some(e.into_path()),
            Ok(_) => None,
            Err(err) => {
                log::warn!("Error reading directory entry: {err}");
                None
            }
        })
        .collect();
    files.sort();
    Ok(files)
}

/// Sorts `source` with the rules from the rules file.
///
/// `rule_filter` restricts the run to the given rule IDs (all enabled rules
/// if `None`). Used by both `tooka sort` and the monitoring daemon.
pub fn sort_folder(
    source: &Path,
    rule_filter: Option<&[String]>,
    dry_run: bool,
    progress: &super::engine::Progress<'_>,
) -> Result<super::engine::SortReport, super::engine::SortError> {
    use super::engine::{self, SortError};
    use crate::{common::environment, rules::rules_file::RulesFile};

    let state_dir = environment::data_dir();
    // Report a running or interrupted sort before anything else (e.g.
    // before complaining about the rules). `engine::sort` checks this again
    // while holding the lock for the whole run.
    if !dry_run {
        let lock = engine::journal::RunLock::try_acquire(&state_dir).map_err(SortError::State)?;
        if lock.is_none() {
            return Err(SortError::Busy);
        }
        if engine::journal::exists(&state_dir) {
            return Err(SortError::RecoveryPending);
        }
    }

    let setup = |e: TookaError| SortError::Setup(e.to_string());
    let rules = RulesFile::load()
        .and_then(|r| r.optimized_with_filter(rule_filter))
        .map_err(setup)?;
    let files = collect_files(source).map_err(setup)?;
    engine::sort(&files, source, &rules.rules, dry_run, &state_dir, progress)
}
