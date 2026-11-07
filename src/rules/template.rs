use crate::{
    core::error::TookaError,
    rules::rule::{Action, Conditions, MetadataField, MoveAction, OnConflict, Range, Rule},
};

/// Explains the optional settings that are not part of the example above.
const TEMPLATE_FOOTER: &str = "\
# Optional: what to do when a destination file already exists
# (abort = stop the whole sort without changing anything, the default;
#  skip = leave this file alone; rename = keep both as 'name (1).ext').
# on_conflict: abort
#
# Full reference: https://github.com/Benji377/tooka/blob/main/docs/rules.md
";

/// Generates a YAML template for a Tooka rule.
pub fn generate_rule_template_yaml() -> Result<String, TookaError> {
    let rule = Rule {
        id: "example_rule".to_string(),
        name: "Example Rule".to_string(),
        enabled: true,
        description: Some("Describe what this rule does".to_string()),
        priority: 1,
        when: Conditions {
            any: Some(false),
            filename: Some(r"^.*\.jpg$".to_string()),
            extensions: Some(vec!["jpg".to_string(), "jpeg".to_string()]),
            path: None,
            size_kb: Some(Range {
                min: Some(10),
                max: Some(5000),
            }),
            mime_type: Some("image/jpeg".to_string()),
            created_date: None,
            modified_date: None,
            is_symlink: None,
            metadata: Some(vec![MetadataField {
                key: "EXIF:DateTimeOriginal".to_string(),
                value: None,
            }]),
        },
        then: vec![Action::Move(MoveAction {
            to: "/path/to/destination".to_string(),
            preserve_structure: false,
        })],
        on_conflict: OnConflict::Abort,
    };

    let yaml = crate::utils::yaml::to_string(&rule)?;
    Ok(format!("{yaml}{TEMPLATE_FOOTER}"))
}
