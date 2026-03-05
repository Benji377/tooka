//! Rule contradiction detection for Tooka.
//!
//! This module provides functionality to detect contradictions and conflicts in rules:
//! - Self-contradictions: A rule whose conditions can never be satisfied simultaneously
//! - Inter-rule conflicts: Multiple rules with overlapping conditions but potentially conflicting actions
//!
//! The conflict detection helps users identify problematic rule configurations early.

use crate::rules::rule::{Action, Conditions, DateRange, Range, Rule};
use crate::utils::date_parser::parse_date;
use std::collections::HashSet;

/// Represents the level of severity for a contradiction or conflict
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictLevel {
    /// Critical error: rule contradicts itself and can never match
    SelfContradiction,
    /// Warning: rule might conflict with another rule
    PotentialConflict,
    /// Info: rule overlaps with another but may be intentional
    Overlap,
}

/// Represents a detected contradiction or conflict
#[derive(Debug, Clone)]
pub struct Conflict {
    /// Level of severity
    pub level: ConflictLevel,
    /// Description of the conflict
    pub message: String,
    /// ID of the rule with the issue
    pub rule_id: String,
    /// Optional ID of conflicting rule (for inter-rule conflicts)
    pub conflicting_rule_id: Option<String>,
}

impl Conflict {
    fn new(level: ConflictLevel, message: String, rule_id: String) -> Self {
        Self {
            level,
            message,
            rule_id,
            conflicting_rule_id: None,
        }
    }

    fn with_conflicting_rule(mut self, conflicting_rule_id: String) -> Self {
        self.conflicting_rule_id = Some(conflicting_rule_id);
        self
    }
}

/// Checks a single rule for self-contradictions
///
/// Detects impossible condition combinations such as:
/// - Date ranges where `from` > `to`
/// - Size ranges where `min` > `max`
/// - Other logically impossible combinations
pub fn check_self_contradiction(rule: &Rule) -> Vec<Conflict> {
    let mut conflicts = Vec::new();

    // Check size_kb range
    if let Some(size) = &rule.when.size_kb {
        if let Some(conflict) = check_size_contradiction(size, &rule.id) {
            conflicts.push(conflict);
        }
    }

    // Check created_date range
    if let Some(date_range) = &rule.when.created_date {
        if let Some(conflict) = check_date_contradiction(date_range, "created_date", &rule.id) {
            conflicts.push(conflict);
        }
    }

    // Check modified_date range
    if let Some(date_range) = &rule.when.modified_date {
        if let Some(conflict) = check_date_contradiction(date_range, "modified_date", &rule.id) {
            conflicts.push(conflict);
        }
    }

    // Check if any=true with only one or no conditions
    if rule.when.any == Some(true) {
        let condition_count = count_conditions(&rule.when);
        if condition_count <= 1 {
            conflicts.push(Conflict::new(
                ConflictLevel::Overlap,
                format!(
                    "Rule has 'any: true' but only {} condition(s). Consider using 'any: false' (AND logic).",
                    condition_count
                ),
                rule.id.clone(),
            ));
        }
    }

    conflicts
}

/// Checks for size range contradictions
fn check_size_contradiction(size: &Range, rule_id: &str) -> Option<Conflict> {
    if let (Some(min), Some(max)) = (size.min, size.max) {
        if min > max {
            return Some(Conflict::new(
                ConflictLevel::SelfContradiction,
                format!(
                    "Size range is impossible: min ({} KB) > max ({} KB)",
                    min, max
                ),
                rule_id.to_string(),
            ));
        }
    }
    None
}

/// Checks for date range contradictions
fn check_date_contradiction(
    date_range: &DateRange,
    field_name: &str,
    rule_id: &str,
) -> Option<Conflict> {
    if let (Some(from_str), Some(to_str)) = (&date_range.from, &date_range.to) {
        match (parse_date(from_str), parse_date(to_str)) {
            (Ok(from), Ok(to)) => {
                if from > to {
                    return Some(Conflict::new(
                        ConflictLevel::SelfContradiction,
                        format!(
                            "{} range is impossible: from ({}) > to ({})",
                            field_name, from_str, to_str
                        ),
                        rule_id.to_string(),
                    ));
                }
            }
            (Err(e), _) => {
                return Some(Conflict::new(
                    ConflictLevel::SelfContradiction,
                    format!("{} 'from' date is invalid: {}", field_name, e),
                    rule_id.to_string(),
                ));
            }
            (_, Err(e)) => {
                return Some(Conflict::new(
                    ConflictLevel::SelfContradiction,
                    format!("{} 'to' date is invalid: {}", field_name, e),
                    rule_id.to_string(),
                ));
            }
        }
    }
    None
}

/// Counts the number of non-None conditions
fn count_conditions(conditions: &Conditions) -> usize {
    let mut count = 0;
    if conditions.filename.is_some() {
        count += 1;
    }
    if conditions.extensions.as_ref().is_some_and(|e| !e.is_empty()) {
        count += 1;
    }
    if conditions.path.is_some() {
        count += 1;
    }
    if conditions.size_kb.is_some() {
        count += 1;
    }
    if conditions.mime_type.is_some() {
        count += 1;
    }
    if conditions.created_date.is_some() {
        count += 1;
    }
    if conditions.modified_date.is_some() {
        count += 1;
    }
    if conditions.is_symlink.is_some() {
        count += 1;
    }
    if conditions
        .metadata
        .as_ref()
        .is_some_and(|m| !m.is_empty())
    {
        count += 1;
    }
    count
}

/// Checks if a new rule conflicts with existing rules
///
/// Detects potential conflicts where:
/// - Rules have overlapping conditions
/// - Rules perform different actions on the same files
/// - Priority might not be what the user expects
pub fn check_rule_conflicts(new_rule: &Rule, existing_rules: &[Rule]) -> Vec<Conflict> {
    let mut conflicts = Vec::new();

    for existing_rule in existing_rules {
        // Skip disabled rules
        if !existing_rule.enabled {
            continue;
        }

        // Skip if it's the same rule (during overwrite)
        if existing_rule.id == new_rule.id {
            continue;
        }

        // Check if conditions overlap
        if conditions_overlap(&new_rule.when, &existing_rule.when) {
            // Determine conflict level based on actions and priority
            let conflict_level = determine_conflict_level(new_rule, existing_rule);

            if conflict_level != ConflictLevel::Overlap
                || new_rule.priority == existing_rule.priority
            {
                let message = format_conflict_message(new_rule, existing_rule, conflict_level);
                conflicts.push(
                    Conflict::new(conflict_level, message, new_rule.id.clone())
                        .with_conflicting_rule(existing_rule.id.clone()),
                );
            }
        }
    }

    conflicts
}

/// Determines if two condition sets can potentially match the same files
fn conditions_overlap(cond1: &Conditions, cond2: &Conditions) -> bool {
    // If both use OR logic, overlap is more likely
    let both_any = cond1.any.unwrap_or(false) && cond2.any.unwrap_or(false);

    // Check each condition type for overlap
    let filename_overlap = check_filename_overlap(&cond1.filename, &cond2.filename);
    let extension_overlap =
        check_extension_overlap(cond1.extensions.as_ref(), cond2.extensions.as_ref());
    let path_overlap = check_path_overlap(&cond1.path, &cond2.path);
    let size_overlap = check_size_overlap(&cond1.size_kb, &cond2.size_kb);
    let mime_overlap = check_mime_overlap(&cond1.mime_type, &cond2.mime_type);
    let symlink_overlap = check_symlink_overlap(&cond1.is_symlink, &cond2.is_symlink);

    // For OR logic, any overlap means potential conflict
    if both_any {
        return filename_overlap
            || extension_overlap
            || path_overlap
            || size_overlap
            || mime_overlap
            || symlink_overlap;
    }

    // For AND logic, all specified conditions must potentially overlap
    // If no conditions are specified, assume overlap
    let mut all_overlap = true;
    let mut has_conditions = false;

    if cond1.filename.is_some() || cond2.filename.is_some() {
        has_conditions = true;
        all_overlap = all_overlap && filename_overlap;
    }

    if cond1.extensions.is_some() || cond2.extensions.is_some() {
        has_conditions = true;
        all_overlap = all_overlap && extension_overlap;
    }

    if cond1.path.is_some() || cond2.path.is_some() {
        has_conditions = true;
        all_overlap = all_overlap && path_overlap;
    }

    if cond1.size_kb.is_some() || cond2.size_kb.is_some() {
        has_conditions = true;
        all_overlap = all_overlap && size_overlap;
    }

    if cond1.mime_type.is_some() || cond2.mime_type.is_some() {
        has_conditions = true;
        all_overlap = all_overlap && mime_overlap;
    }

    if cond1.is_symlink.is_some() || cond2.is_symlink.is_some() {
        has_conditions = true;
        all_overlap = all_overlap && symlink_overlap;
    }

    has_conditions && all_overlap
}

/// Checks if two filename patterns can match the same files
fn check_filename_overlap(pattern1: &Option<String>, pattern2: &Option<String>) -> bool {
    match (pattern1, pattern2) {
        (None, _) | (_, None) => true, // If one is unspecified, could overlap
        (Some(p1), Some(p2)) => {
            // Simple heuristic: if patterns are different, they might still overlap
            // Complex regex overlap detection would require full regex analysis
            p1 == p2 || p1.contains('*') || p2.contains('*') || p1.contains('.') || p2.contains('.')
        }
    }
}

/// Checks if two extension lists can match the same files
fn check_extension_overlap(
    exts1: Option<&Vec<String>>,
    exts2: Option<&Vec<String>>,
) -> bool {
    match (exts1, exts2) {
        (None, _) | (_, None) => true,
        (Some(e1), Some(e2)) => {
            let set1: HashSet<_> = e1.iter().collect();
            let set2: HashSet<_> = e2.iter().collect();
            !set1.is_disjoint(&set2)
        }
    }
}

/// Checks if two path patterns can match the same files
fn check_path_overlap(path1: &Option<String>, path2: &Option<String>) -> bool {
    match (path1, path2) {
        (None, _) | (_, None) => true,
        (Some(p1), Some(p2)) => {
            // Simple heuristic: paths might overlap if they share a prefix or use wildcards
            p1 == p2 || p1.contains('*') || p2.contains('*') || p1.starts_with(p2) || p2.starts_with(p1)
        }
    }
}

/// Checks if two size ranges can match the same files
fn check_size_overlap(range1: &Option<Range>, range2: &Option<Range>) -> bool {
    match (range1, range2) {
        (None, _) | (_, None) => true,
        (Some(r1), Some(r2)) => {
            let min1 = r1.min.unwrap_or(0);
            let max1 = r1.max.unwrap_or(u64::MAX);
            let min2 = r2.min.unwrap_or(0);
            let max2 = r2.max.unwrap_or(u64::MAX);

            // Ranges overlap if they intersect
            !(max1 < min2 || max2 < min1)
        }
    }
}

/// Checks if two MIME type patterns can match the same files
fn check_mime_overlap(mime1: &Option<String>, mime2: &Option<String>) -> bool {
    match (mime1, mime2) {
        (None, _) | (_, None) => true,
        (Some(m1), Some(m2)) => {
            // Check for wildcard matches (e.g., "image/*")
            m1 == m2
                || m1.ends_with("/*")
                || m2.ends_with("/*")
                || (m1.ends_with("/*") && m2.starts_with(&m1[..m1.len() - 1]))
                || (m2.ends_with("/*") && m1.starts_with(&m2[..m2.len() - 1]))
        }
    }
}

/// Checks if two symlink conditions can match the same files
fn check_symlink_overlap(sym1: &Option<bool>, sym2: &Option<bool>) -> bool {
    match (sym1, sym2) {
        (None, _) | (_, None) => true,
        (Some(s1), Some(s2)) => s1 == s2,
    }
}

/// Determines the conflict level between two rules
fn determine_conflict_level(rule1: &Rule, rule2: &Rule) -> ConflictLevel {
    // If priorities are the same, this is more concerning
    if rule1.priority == rule2.priority {
        return ConflictLevel::PotentialConflict;
    }

    // Check if actions are conflicting
    let actions_conflict = check_actions_conflict(&rule1.then, &rule2.then);

    if actions_conflict {
        ConflictLevel::PotentialConflict
    } else {
        ConflictLevel::Overlap
    }
}

/// Checks if two action sets are conflicting
fn check_actions_conflict(actions1: &[Action], actions2: &[Action]) -> bool {
    // Extract primary action types
    let has_move1 = actions1.iter().any(|a| matches!(a, Action::Move(_)));
    let has_move2 = actions2.iter().any(|a| matches!(a, Action::Move(_)));

    let has_delete1 = actions1.iter().any(|a| matches!(a, Action::Delete(_)));
    let has_delete2 = actions2.iter().any(|a| matches!(a, Action::Delete(_)));

    let has_rename1 = actions1.iter().any(|a| matches!(a, Action::Rename(_)));
    let has_rename2 = actions2.iter().any(|a| matches!(a, Action::Rename(_)));

    // Conflicting if both try to move, delete, or rename
    // Also conflicting if one moves and the other deletes (mutually exclusive file operations)
    (has_move1 && has_move2)
        || (has_delete1 && has_delete2)
        || (has_rename1 && has_rename2)
        || (has_move1 && has_delete2)
        || (has_move2 && has_delete1)
}

/// Formats a conflict message with details about the conflicting rules
fn format_conflict_message(rule1: &Rule, rule2: &Rule, level: ConflictLevel) -> String {
    let priority_info = if rule1.priority > rule2.priority {
        format!(
            "New rule has higher priority ({} > {}), so it will be applied first.",
            rule1.priority, rule2.priority
        )
    } else if rule1.priority < rule2.priority {
        format!(
            "Existing rule '{}' has higher priority ({} > {}), so your new rule may never match.",
            rule2.id, rule2.priority, rule1.priority
        )
    } else {
        "Both rules have the same priority. Rule order is non-deterministic.".to_string()
    };

    match level {
        ConflictLevel::PotentialConflict => {
            format!(
                "Rule potentially conflicts with existing rule '{}' ({}). {}",
                rule2.id, rule2.name, priority_info
            )
        }
        ConflictLevel::Overlap => {
            format!(
                "Rule overlaps with existing rule '{}' ({}). {}",
                rule2.id, rule2.name, priority_info
            )
        }
        ConflictLevel::SelfContradiction => {
            unreachable!("Self-contradiction should not reach here")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::rule::{Conditions, DeleteAction, MoveAction, Rule};

    fn create_test_rule(id: &str, priority: u32, conditions: Conditions) -> Rule {
        Rule {
            id: id.to_string(),
            name: format!("Test Rule {}", id),
            enabled: true,
            description: None,
            priority,
            when: conditions,
            then: vec![Action::Move(MoveAction {
                to: "/test".to_string(),
                preserve_structure: false,
            })],
        }
    }

    #[test]
    fn test_size_contradiction() {
        let rule = create_test_rule(
            "test",
            1,
            Conditions {
                any: None,
                filename: None,
                extensions: None,
                path: None,
                size_kb: Some(Range {
                    min: Some(100),
                    max: Some(50),
                }),
                mime_type: None,
                created_date: None,
                modified_date: None,
                is_symlink: None,
                metadata: None,
            },
        );

        let conflicts = check_self_contradiction(&rule);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].level, ConflictLevel::SelfContradiction);
    }

    #[test]
    fn test_date_contradiction() {
        let rule = create_test_rule(
            "test",
            1,
            Conditions {
                any: None,
                filename: None,
                extensions: None,
                path: None,
                size_kb: None,
                mime_type: None,
                created_date: Some(DateRange {
                    from: Some("2024-12-31".to_string()),
                    to: Some("2024-01-01".to_string()),
                }),
                modified_date: None,
                is_symlink: None,
                metadata: None,
            },
        );

        let conflicts = check_self_contradiction(&rule);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].level, ConflictLevel::SelfContradiction);
    }

    #[test]
    fn test_extension_overlap() {
        let cond1 = Conditions {
            any: None,
            filename: None,
            extensions: Some(vec!["jpg".to_string(), "png".to_string()]),
            path: None,
            size_kb: None,
            mime_type: None,
            created_date: None,
            modified_date: None,
            is_symlink: None,
            metadata: None,
        };

        let cond2 = Conditions {
            any: None,
            filename: None,
            extensions: Some(vec!["png".to_string(), "gif".to_string()]),
            path: None,
            size_kb: None,
            mime_type: None,
            created_date: None,
            modified_date: None,
            is_symlink: None,
            metadata: None,
        };

        assert!(conditions_overlap(&cond1, &cond2));
    }

    #[test]
    fn test_no_extension_overlap() {
        let cond1 = Conditions {
            any: None,
            filename: None,
            extensions: Some(vec!["jpg".to_string()]),
            path: None,
            size_kb: None,
            mime_type: None,
            created_date: None,
            modified_date: None,
            is_symlink: None,
            metadata: None,
        };

        let cond2 = Conditions {
            any: None,
            filename: None,
            extensions: Some(vec!["pdf".to_string()]),
            path: None,
            size_kb: None,
            mime_type: None,
            created_date: None,
            modified_date: None,
            is_symlink: None,
            metadata: None,
        };

        assert!(!conditions_overlap(&cond1, &cond2));
    }

    #[test]
    fn test_size_range_overlap() {
        let range1 = Some(Range {
            min: Some(50),
            max: Some(150),
        });
        let range2 = Some(Range {
            min: Some(100),
            max: Some(200),
        });

        assert!(check_size_overlap(&range1, &range2));
    }

    #[test]
    fn test_size_range_no_overlap() {
        let range1 = Some(Range {
            min: Some(50),
            max: Some(100),
        });
        let range2 = Some(Range {
            min: Some(150),
            max: Some(200),
        });

        assert!(!check_size_overlap(&range1, &range2));
    }

    #[test]
    fn test_rule_conflict_detection() {
        let rule1 = create_test_rule(
            "new",
            5,
            Conditions {
                any: None,
                filename: None,
                extensions: Some(vec!["jpg".to_string()]),
                path: None,
                size_kb: None,
                mime_type: None,
                created_date: None,
                modified_date: None,
                is_symlink: None,
                metadata: None,
            },
        );

        let rule2 = Rule {
            id: "existing".to_string(),
            name: "Existing Rule".to_string(),
            enabled: true,
            description: None,
            priority: 3,
            when: Conditions {
                any: None,
                filename: None,
                extensions: Some(vec!["jpg".to_string()]),
                path: None,
                size_kb: None,
                mime_type: None,
                created_date: None,
                modified_date: None,
                is_symlink: None,
                metadata: None,
            },
            then: vec![Action::Delete(DeleteAction { trash: true })],
        };

        let conflicts = check_rule_conflicts(&rule1, &[rule2]);
        assert!(!conflicts.is_empty());
    }
}
