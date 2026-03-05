use crate::cli;
use crate::core::context;
use crate::rules::contradiction;
use anyhow::Result;
use clap::Args;
use std::fs;
use std::path::Path;

#[derive(Args)]
#[command(about = "📝 Add a new rule by importing a YAML file or scanning a directory")]
pub struct AddArgs {
    /// Path to the rule YAML file or directory containing YAML files
    #[arg(
        value_name = "PATH",
        help = "Path to the YAML file or directory containing YAML files with rule definitions"
    )]
    pub path: String,

    /// Optional flag to overwrite existing rules
    #[arg(
        long,
        default_value_t = false,
        help = "Overwrite existing rule if it already exists"
    )]
    pub overwrite: bool,
}

pub fn run(args: &AddArgs) -> Result<()> {
    let path = Path::new(&args.path);

    if path.is_file() {
        // Handle single file
        cli::info(&format!("📝 Adding rule from file: {}", args.path));
        log::info!("Adding rule from file: {}", args.path);

        let mut rf = context::get_locked_rules_file()?;

        // Check conflicts before adding
        check_and_display_conflicts(&args.path, &rf)?;

        rf.add_rule_from_file(&args.path, args.overwrite)
            .map_err(|e| anyhow::anyhow!("Failed to add rule from file: {}: {}", args.path, e))?;

        cli::success("Rule added successfully!");
        log::info!("Rule added successfully from file: {}", args.path);
    } else if path.is_dir() {
        // Handle directory
        cli::info(&format!(
            "📂 Scanning directory for YAML files: {}",
            args.path
        ));
        log::info!("Scanning directory for YAML files: {}", args.path);

        let yaml_files = find_yaml_files(path)?;

        if yaml_files.is_empty() {
            cli::warning("No YAML files found in the directory");
            log::warn!("No YAML files found in directory: {}", args.path);
            return Ok(());
        }

        cli::info(&format!("Found {} YAML files", yaml_files.len()));
        log::info!(
            "Found {} YAML files in directory: {}",
            yaml_files.len(),
            args.path
        );

        let mut rf = context::get_locked_rules_file()?;
        let mut added_count = 0;
        let mut failed_count = 0;
        let mut skipped_count = 0;

        for file_path in yaml_files {
            let file_path_str = file_path.to_string_lossy();
            let file_name = file_path.file_name().unwrap().to_string_lossy();
            log::info!("Processing file: {file_path_str}");

            match rf.add_rule_from_file(&file_path_str, args.overwrite) {
                Ok(()) => {
                    cli::success(&format!("  Added rules from: {file_name}"));
                    log::info!("Successfully added rules from: {file_path_str}");
                    added_count += 1;
                }
                Err(e) => {
                    if e.to_string().contains("already exists") && !args.overwrite {
                        cli::warning(&format!("  Skipped (rule exists): {file_name}"));
                        log::warn!("Skipped file due to existing rule: {file_path_str}");
                        skipped_count += 1;
                    } else {
                        cli::error(&format!("  Failed to add from: {file_name} - {e}"));
                        log::error!("Failed to add rules from: {file_path_str} - {e}");
                        failed_count += 1;
                    }
                }
            }
        }

        // Print summary
        cli::info(&format!(
            "📊 Summary: {added_count} added, {skipped_count} skipped, {failed_count} failed"
        ));
        log::info!(
            "Directory processing complete. Added: {added_count}, Skipped: {skipped_count}, Failed: {failed_count}"
        );

        if failed_count > 0 {
            return Err(anyhow::anyhow!("Failed to process {} files", failed_count));
        }
    } else {
        return Err(anyhow::anyhow!(
            "Path is neither a file nor a directory: {}",
            args.path
        ));
    }

    Ok(())
}

/// Find all YAML files in a directory (non-recursive)
fn find_yaml_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut yaml_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "yaml" || extension == "yml" {
                    yaml_files.push(path);
                }
            }
        }
    }

    // Sort files for consistent ordering
    yaml_files.sort();
    Ok(yaml_files)
}

/// Check for contradictions and conflicts in a rule file before adding
fn check_and_display_conflicts(
    file_path: &str,
    rules_file: &crate::rules::rules_file::RulesFile,
) -> Result<()> {
    use crate::rules::rule::Rule;

    // Parse the rule(s) from the file
    let rules = Rule::new_from_file(file_path)?;

    for rule in &rules {
        // Check for self-contradictions
        let self_conflicts = contradiction::check_self_contradiction(rule);
        for conflict in &self_conflicts {
            if conflict.level == contradiction::ConflictLevel::SelfContradiction {
                cli::error(&format!(
                    "Self-contradiction in rule '{}': {}",
                    conflict.rule_id, conflict.message
                ));
                return Err(anyhow::anyhow!(
                    "Rule '{}' has a self-contradiction",
                    rule.id
                ));
            }
        }

        // Check for conflicts with existing rules
        let rule_conflicts = contradiction::check_rule_conflicts(rule, &rules_file.rules);
        
        let mut potential_conflicts = Vec::new();
        let mut overlaps = Vec::new();

        for conflict in &rule_conflicts {
            match conflict.level {
                contradiction::ConflictLevel::PotentialConflict => {
                    potential_conflicts.push(conflict);
                }
                contradiction::ConflictLevel::Overlap => {
                    overlaps.push(conflict);
                }
                contradiction::ConflictLevel::SelfContradiction => {
                    // Should not happen here, but handle it just in case
                    cli::error(&format!(
                        "Self-contradiction in rule '{}': {}",
                        conflict.rule_id, conflict.message
                    ));
                }
            }
        }

        // Display potential conflicts as warnings
        if !potential_conflicts.is_empty() {
            cli::warning(&format!(
                "Rule '{}' has {} potential conflict(s):",
                rule.id,
                potential_conflicts.len()
            ));
            for conflict in potential_conflicts {
                if let Some(conflicting_id) = &conflict.conflicting_rule_id {
                    cli::warning(&format!("   - Conflicts with rule '{}': {}", 
                        conflicting_id, conflict.message));
                } else {
                    cli::warning(&format!("   - {}", conflict.message));
                }
            }
        }

        // Display overlaps as info (less critical)
        if !overlaps.is_empty() {
            cli::info(&format!(
                "Rule '{}' overlaps with {} existing rule(s)",
                rule.id,
                overlaps.len()
            ));
            for conflict in overlaps {
                if let Some(conflicting_id) = &conflict.conflicting_rule_id {
                    log::info!(
                        "Rule '{}' overlaps with '{}': {}",
                        rule.id,
                        conflicting_id,
                        conflict.message
                    );
                }
            }
        }
    }

    Ok(())
}
