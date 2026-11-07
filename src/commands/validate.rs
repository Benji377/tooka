use crate::cli;
use crate::rules::rule::Rule;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "✅ Validate a rule YAML file (structure, patterns, dates and actions)")]
pub struct ValidateArgs {
    /// Path to the rule YAML file
    #[arg(value_name = "FILE", help = "Path to the YAML file to validate")]
    pub file: String,

    /// Kept so existing scripts keep working; validation is always complete.
    #[arg(long, hide = true)]
    pub deep: bool,
}

pub fn run(args: &ValidateArgs) -> Result<()> {
    log::info!("Validating rule from file: {}", args.file);

    // Deserialize the file (already validates structure)
    let rules = Rule::new_from_file(&args.file)
        .map_err(|e| anyhow::anyhow!("Failed to load rule from file: {}: {}", args.file, e))?;

    log::info!("Loaded {} rules from file: {}", rules.len(), args.file);
    println!("Loaded {} rules from file: {}", rules.len(), args.file);

    // Check the content too (patterns, dates, action order), exactly as
    // `tooka add` and `tooka sort` do.
    let mut err_count = 0;
    for rule in rules {
        if let Err(e) = rule.validate(true) {
            log::error!("Rule '{}' is invalid: {}", rule.name, e);
            cli::error(&format!("Rule '{}' is invalid: {}", rule.name, e));
            err_count += 1;
        } else {
            log::info!("Rule '{}' is valid", rule.name);
            cli::success(&format!("Rule '{}' is valid", rule.name));
        }
    }

    if err_count > 0 {
        log::error!("Validation completed with {err_count} errors");
        println!("Validation completed with {err_count} errors");
        return Err(anyhow::anyhow!(
            "Validation failed with {} errors",
            err_count
        ));
    }

    log::info!("All rules are valid");
    cli::success("All rules are valid");

    Ok(())
}
