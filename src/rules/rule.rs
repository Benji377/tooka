//! Defines the core data structures representing file operation rules in Tooka.
//! Includes rule conditions, actions, and validation logic ensuring rule correctness.
//! Supports complex matching criteria such as filename patterns, metadata, size, dates, etc.

use std::{fs, path::Path};

use crate::core::error::RuleValidationError;
use crate::utils::date_parser::parse_date;
use serde::{Deserialize, Serialize};

/// Represents a rule for file operations, specifying when it applies and what actions to take.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// Unique identifier for the rule.
    pub id: String,
    /// Human-readable name of the rule.
    pub name: String,
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Optional detailed description.
    pub description: Option<String>,
    /// Rule priority (higher is more important).
    pub priority: u32,
    /// Conditions to match files for this rule.
    pub when: Conditions,
    /// Actions to perform when conditions match.
    pub then: Vec<Action>,
    /// What to do when a destination of this rule already exists.
    #[serde(default, skip_serializing_if = "OnConflict::is_default")]
    pub on_conflict: OnConflict,
}

/// What a rule does when the destination of a move, copy or rename already exists
/// (or is already the destination of another file in the same sort run).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OnConflict {
    /// Stop the whole sort before anything is changed and report the conflict.
    #[default]
    Abort,
    /// Leave this file untouched and continue with the other files.
    Skip,
    /// Keep both files by giving the new one a free name such as `name (1).ext`.
    Rename,
}

impl OnConflict {
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// Contains matching criteria to determine when a rule applies.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Conditions {
    /// If true, matches if any condition is true (logical OR); otherwise all must match (AND).
    #[serde(default)]
    pub any: Option<bool>,
    /// Regex pattern to match against the filename.
    pub filename: Option<String>,
    /// List of file extensions to match.
    #[serde(default)]
    pub extensions: Option<Vec<String>>,
    /// Glob pattern for file path matching.
    pub path: Option<String>,
    /// File size range in KB.
    pub size_kb: Option<Range>,
    /// MIME type filter.
    pub mime_type: Option<String>,
    /// Date range when the file was created.
    pub created_date: Option<DateRange>,
    /// Date range when the file was modified.
    pub modified_date: Option<DateRange>,
    /// Whether the file is a symbolic link.
    pub is_symlink: Option<bool>,
    /// Additional metadata fields for matching.
    #[serde(default)]
    pub metadata: Option<Vec<MetadataField>>,
}

/// Represents a single metadata field to match against
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MetadataField {
    /// Metadata field key (e.g., "EXIF:DateTime")
    pub key: String,
    /// Optional value to match against the field
    pub value: Option<String>,
}

/// Represents a data range for matching files
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Range {
    /// Minimum size in KB (inclusive)
    pub min: Option<u64>,
    /// Maximum size in KB (inclusive)
    pub max: Option<u64>,
}

/// Represents a date range for matching files
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DateRange {
    /// Optional start date in RFC3339 format (inclusive)
    pub from: Option<String>,
    /// Optional end date in RFC3339 format (inclusive)
    pub to: Option<String>,
}

/// Represents an action to perform when a rule matches
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Action {
    /// Move the file to a new location
    Move(MoveAction),
    /// Copy the file to a new location
    Copy(CopyAction),
    /// Rename the file to a new name
    Rename(RenameAction),
    /// Delete the file, optionally moving it to trash
    Delete(DeleteAction),
    /// Executes a CLI command or script
    Execute(ExecuteAction),
    /// Skip the file without any action
    Skip,
}

/// Represents a move action, specifying the destination path and whether to preserve structure
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MoveAction {
    /// Destination path where the file should be moved
    pub to: String,
    /// If true, preserves the directory structure relative to the source path
    #[serde(default)]
    pub preserve_structure: bool,
}

/// Represents a copy action, specifying the destination path and whether to preserve structure
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CopyAction {
    /// Destination path where the file should be copied
    pub to: String,
    /// If true, preserves the directory structure relative to the source path
    #[serde(default)]
    pub preserve_structure: bool,
}

/// Represents a rename action, specifying the new name for the file
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RenameAction {
    /// New name for the file, can include metadata placeholders
    pub to: String,
}

/// Represents a delete action, specifying whether to move the file to trash
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DeleteAction {
    /// If true (the default), moves the file to the trash instead of
    /// deleting it permanently. Only an explicit `trash: false` deletes
    /// permanently.
    #[serde(default = "default_trash")]
    pub trash: bool,
}

fn default_trash() -> bool {
    true
}

/// Represents an execute action, specifying the command to run and its arguments
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExecuteAction {
    /// Command to execute, can be a shell command or script
    pub command: String,
    /// Arguments to pass to the command
    pub args: Vec<String>,
}

/// Validates the rule's fields and consistency.
///
/// Checks for required fields, duplicate metadata keys, valid size ranges,
/// valid date formats, and valid action configurations.
///
/// Returns an error if validation fails.
impl Rule {
    /// Constructs rules from a YAML file.
    /// Supports both single-rule files and multi-rule files (under `rules:` key).
    pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Self>, RuleValidationError> {
        let content = fs::read_to_string(path)
            .map_err(|e| RuleValidationError::InvalidFormat(format!("Failed to read file: {e}")))?;

        if is_multi_rule_document(&content) {
            // Multiple rules
            let parsed: Result<RulesWrapper, _> = serde_saphyr::from_str(&content);
            match parsed {
                Ok(wrapper) => Ok(wrapper.rules),
                Err(e) => Err(RuleValidationError::InvalidFormat(format!(
                    "YAML parsing failed: {e}"
                ))),
            }
        } else {
            // Single rule
            let rule: Result<Rule, _> = serde_saphyr::from_str(&content);
            match rule {
                Ok(r) => Ok(vec![r]),
                Err(e) => Err(RuleValidationError::InvalidFormat(format!(
                    "YAML parsing failed: {e}"
                ))),
            }
        }
    }
    /// Validates the rule, with an optional `deep` check for logic and content consistency.
    ///
    /// If `deep` is `false`, only structural deserialization is considered valid.
    /// If `deep` is `true`, content checks like required fields, logic checks, and date format are applied.
    pub fn validate(&self, deep: bool) -> Result<(), RuleValidationError> {
        if !deep {
            return Ok(()); // Structural deserialization has already passed
        }

        if self.id.trim().is_empty() {
            return Err(RuleValidationError::MissingId);
        }

        if self.name.trim().is_empty() {
            return Err(RuleValidationError::MissingName(self.id.clone()));
        }

        if self.then.is_empty() {
            return Err(RuleValidationError::NoActions(self.id.clone()));
        }

        if let Some(pattern) = &self.when.filename
            && let Err(e) = regex::Regex::new(pattern)
        {
            return Err(RuleValidationError::InvalidCondition(
                self.id.clone(),
                format!("Invalid filename regex '{pattern}': {e}"),
            ));
        }

        if let Some(pattern) = &self.when.path
            && let Err(e) = glob::Pattern::new(pattern)
        {
            return Err(RuleValidationError::InvalidCondition(
                self.id.clone(),
                format!("Invalid path glob '{pattern}': {e}"),
            ));
        }

        if let Some(metadata) = &self.when.metadata {
            let mut keys = std::collections::HashSet::new();
            for field in metadata {
                if !keys.insert(&field.key) {
                    return Err(RuleValidationError::InvalidCondition(
                        self.id.clone(),
                        format!("Duplicate metadata key '{}'", field.key),
                    ));
                }
                if let Some(pattern) = &field.value
                    && let Err(e) = glob::Pattern::new(pattern)
                {
                    return Err(RuleValidationError::InvalidCondition(
                        self.id.clone(),
                        format!(
                            "Invalid glob '{pattern}' for metadata key '{}': {e}",
                            field.key
                        ),
                    ));
                }
            }
        }

        if let Some(size) = &self.when.size_kb
            && let (Some(min), Some(max)) = (size.min, size.max)
            && min > max
        {
            return Err(RuleValidationError::InvalidCondition(
                self.id.clone(),
                "Invalid size_kb range: min > max".into(),
            ));
        }

        for (label, date_range) in [
            ("created_date", &self.when.created_date),
            ("modified_date", &self.when.modified_date),
        ] {
            if let Some(range) = date_range {
                if let Some(from) = &range.from
                    && let Err(e) = parse_date(from)
                {
                    return Err(RuleValidationError::InvalidCondition(
                        self.id.clone(),
                        format!("Invalid {label} 'from' date: {e}"),
                    ));
                }
                if let Some(to) = &range.to
                    && let Err(e) = parse_date(to)
                {
                    return Err(RuleValidationError::InvalidCondition(
                        self.id.clone(),
                        format!("Invalid {label} 'to' date: {e}"),
                    ));
                }
            }
        }

        if let Some(value) = self.action_validation() {
            return value;
        }

        Ok(())
    }

    fn action_validation(&self) -> Option<Result<(), RuleValidationError>> {
        let invalid = |i: usize, msg: &str| {
            Some(Err(RuleValidationError::InvalidAction(
                self.id.clone(),
                i,
                msg.into(),
            )))
        };
        // Index of the first `execute` action. Commands run after all file
        // operations of a sort, so a later move/copy/rename would make the
        // `{{path}}` they received point to a file that is no longer there.
        let mut first_execute: Option<usize> = None;

        for (i, action) in self.then.iter().enumerate() {
            match action {
                Action::Move(MoveAction { to, .. }) | Action::Copy(CopyAction { to, .. }) => {
                    if to.trim().is_empty() {
                        return invalid(i, "Missing destination path");
                    }
                }
                Action::Rename(inner) => {
                    if inner.to.trim().is_empty() {
                        return invalid(i, "Missing rename target path");
                    }
                    if inner.to.contains(['/', '\\']) {
                        return invalid(
                            i,
                            "Rename target must be a file name, not a path (use a move action to change folders)",
                        );
                    }
                }
                Action::Delete(_) => {
                    if i + 1 < self.then.len() {
                        return invalid(i, "Delete must be the last action of a rule");
                    }
                }
                Action::Execute(inner) => {
                    if inner.command.trim().is_empty() {
                        return invalid(i, "Missing command to execute");
                    }
                    first_execute.get_or_insert(i);
                }
                Action::Skip => {}
            }

            if let Some(exec_index) = first_execute
                && matches!(
                    action,
                    Action::Move(_) | Action::Copy(_) | Action::Rename(_)
                )
            {
                return invalid(
                    i,
                    &format!(
                        "Move, copy and rename must come before execute (action {exec_index}), because commands run after all file operations"
                    ),
                );
            }
        }
        None
    }
}

/// Returns true if a YAML document holds several rules under `rules:`
/// (as opposed to a single rule). Comments, blank lines and the `---`
/// document marker before it are ignored.
pub(crate) fn is_multi_rule_document(content: &str) -> bool {
    content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#') && *line != "---")
        .is_some_and(|line| line.starts_with("rules:"))
}

/// Wrapper for multi-rule YAML files
#[derive(Debug, Serialize, Deserialize)]
struct RulesWrapper {
    pub rules: Vec<Rule>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule_with(then: Vec<Action>) -> Rule {
        Rule {
            id: "r".into(),
            name: "R".into(),
            enabled: true,
            description: None,
            priority: 1,
            when: Conditions {
                any: None,
                filename: None,
                extensions: None,
                path: None,
                size_kb: None,
                mime_type: None,
                created_date: None,
                modified_date: None,
                is_symlink: None,
                metadata: None,
            },
            then,
            on_conflict: OnConflict::default(),
        }
    }

    fn mv() -> Action {
        Action::Move(MoveAction {
            to: "/dest".into(),
            preserve_structure: false,
        })
    }

    fn exec() -> Action {
        Action::Execute(ExecuteAction {
            command: "echo".into(),
            args: vec!["{{path}}".into()],
        })
    }

    #[test]
    fn execute_must_come_after_file_operations() {
        assert!(rule_with(vec![mv(), exec()]).validate(true).is_ok());
        assert!(rule_with(vec![exec(), mv()]).validate(true).is_err());
    }

    #[test]
    fn delete_must_be_last() {
        let delete = Action::Delete(DeleteAction { trash: true });
        assert!(rule_with(vec![mv(), delete.clone()]).validate(true).is_ok());
        assert!(rule_with(vec![delete, mv()]).validate(true).is_err());
    }

    #[test]
    fn rename_target_must_be_a_file_name() {
        let rename = |to: &str| Action::Rename(RenameAction { to: to.into() });
        assert!(
            rule_with(vec![rename("{{filename}}-x.txt")])
                .validate(true)
                .is_ok()
        );
        assert!(
            rule_with(vec![rename("sub/{{filename}}")])
                .validate(true)
                .is_err()
        );
    }

    #[test]
    fn delete_uses_the_trash_unless_told_otherwise() {
        let parse = |yaml: &str| match serde_saphyr::from_str::<Action>(yaml).unwrap() {
            Action::Delete(d) => d.trash,
            other => panic!("unexpected {other:?}"),
        };
        assert!(parse("action: delete\n"));
        assert!(parse("action: delete\ntrash: true\n"));
        assert!(!parse("action: delete\ntrash: false\n"));
    }

    #[test]
    fn multi_rule_files_may_start_with_comments() {
        assert!(is_multi_rule_document(
            "# my rules\n\n---\nrules:\n- id: a\n"
        ));
        assert!(!is_multi_rule_document("# one rule\nid: a\n"));
    }

    #[test]
    fn invalid_patterns_are_reported() {
        let mut rule = rule_with(vec![mv()]);
        rule.when.filename = Some("([unclosed".into());
        assert!(rule.validate(true).is_err());

        let mut rule = rule_with(vec![mv()]);
        rule.when.path = Some("[!".into());
        assert!(rule.validate(true).is_err());
    }

    #[test]
    fn on_conflict_defaults_to_abort_and_is_not_written_when_default() {
        let yaml = "id: r\nname: R\nenabled: true\ndescription: null\npriority: 1\nwhen:\n  extensions: [pdf]\nthen:\n- action: skip\n";
        let rule: Rule = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(rule.on_conflict, OnConflict::Abort);
        let written = crate::utils::yaml::to_string(&rule).unwrap();
        assert!(!written.contains("on_conflict"), "{written}");

        let rule: Rule = serde_saphyr::from_str(&format!("{yaml}on_conflict: rename\n")).unwrap();
        assert_eq!(rule.on_conflict, OnConflict::Rename);
    }
}
