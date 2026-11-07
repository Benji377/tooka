use crate::rules::template::generate_rule_template_yaml;
use anyhow::{Result, anyhow};
use clap::Args;

#[derive(Args)]
#[command(about = "📋 Generate a template rule YAML file")]
pub struct TemplateArgs {
    /// Output file path
    #[arg(long, help = "Output file path (defaults to 'rule_template.yaml')")]
    pub output: Option<String>,
}

pub fn run(args: TemplateArgs) -> Result<()> {
    let output_path = args
        .output
        .unwrap_or_else(|| "rule_template.yaml".to_string());

    log::info!("Generating rule template YAML to {output_path}");

    let rule_template = generate_rule_template_yaml()
        .map_err(|e| anyhow!("Failed to generate rule template: {}", e))?;

    // Never overwrite an existing file.
    let mut file = std::fs::File::create_new(&output_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            anyhow!("'{output_path}' already exists; choose another path with --output")
        } else {
            anyhow!("Failed to write rule template to file: {e}")
        }
    })?;
    std::io::Write::write_all(&mut file, rule_template.as_bytes())
        .map_err(|e| anyhow!("Failed to write rule template to file: {e}"))?;

    println!("Rule template YAML generated successfully at {output_path}");

    Ok(())
}
