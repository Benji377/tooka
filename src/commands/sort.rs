use std::path::PathBuf;

use crate::cli;
use crate::common::config::Config;
use crate::core::engine::{Progress, SortReport};
use crate::core::{interrupt, report, sorter};
use anyhow::{Result, bail};
use clap::Args;
use colored::Colorize;
use indicatif::ProgressBar;

#[derive(Args)]
#[command(about = "🚀 Sort files in the source folder using defined rules")]
pub struct SortArgs {
    /// Override default source folder
    #[arg(long, help = "Override the default source folder path")]
    pub source: Option<String>,
    /// Comma-separated rule IDs to run
    #[arg(
        long,
        help = "Comma-separated list of rule IDs to execute (use '<all>' for all rules)"
    )]
    pub rules: Option<String>,
    /// Output report format: pdf, csv, json
    #[arg(
        long,
        help = "Generate a report in the specified format (pdf, csv, json)"
    )]
    pub report: Option<String>,
    /// Output directory for the report
    #[arg(long, help = "Directory where the report will be saved")]
    pub output: Option<String>,
    /// Simulate the sorting without making changes
    #[arg(
        long,
        default_value_t = false,
        help = "Preview what would happen without actually moving files"
    )]
    pub dry_run: bool,
}

pub fn run(args: SortArgs) -> Result<()> {
    if args.dry_run {
        cli::warning("🔍 Running in dry-run mode - no files will be changed");
    } else {
        cli::info("🚀 Starting file sorting...");
    }

    log::info!(
        "Running sort with source: {:?}, rules: {:?}, dry_run: {}",
        args.source,
        args.rules,
        args.dry_run
    );

    let config = Config::load()?;
    let source_path = match args.source.as_deref() {
        None | Some("<default>") => config.source_folder.clone(),
        Some(source) => PathBuf::from(source),
    };

    let rule_filter: Option<Vec<String>> = args
        .rules
        .as_deref()
        .filter(|r| *r != "<all>")
        .map(|r| r.split(',').map(|s| s.trim().to_string()).collect());

    // Ctrl+C now stops the sort cleanly (rolling it back) instead of
    // killing Tooka in the middle of moving files.
    interrupt::install_handler();

    let pb = ProgressBar::new(0);
    pb.set_style(cli::progress_style());
    pb.set_message("matching rules");
    let matched = || pb.inc(1);
    let executing = |count: usize| {
        pb.set_position(0);
        pb.set_length(count as u64);
        pb.set_message("sorting");
    };
    let file_done = || pb.inc(1);

    let result = sorter::sort_folder(
        &source_path,
        rule_filter.as_deref(),
        args.dry_run,
        &Progress {
            matched: &matched,
            executing: &executing,
            file_done: &file_done,
        },
    );
    pb.finish_and_clear();

    let report = result?;
    print_report(&report);

    if let Some(report_type) = &args.report {
        log::info!("Generating report of type: {report_type}");
        let output_dir = args.output.as_ref().map_or_else(
            || {
                std::env::current_dir().unwrap_or_else(|_| {
                    cli::warning("Cannot get current working directory, using '.'");
                    PathBuf::from(".")
                })
            },
            PathBuf::from,
        );

        report::generate_report(report_type, &output_dir, &report.results)?;
        cli::success(&format!(
            "Report generated successfully in {}",
            output_dir.display()
        ));
    }

    if !report.commit_failures.is_empty() {
        bail!(
            "All files were sorted, but {} follow-up task(s) failed (see above).",
            report.commit_failures.len()
        );
    }
    if args.dry_run {
        cli::success("Dry run complete - nothing was changed.");
    } else {
        cli::success("Sorting completed successfully!");
    }
    Ok(())
}

fn print_report(report: &SortReport) {
    let changed: Vec<_> = report
        .results
        .iter()
        .filter(|r| r.action != "skip" && r.action != "unchanged")
        .collect();
    log::info!("Sort finished with {} action(s)", changed.len());

    if changed.is_empty() {
        cli::info("No files matched the sorting rules.");
    } else {
        cli::header("📁 Sorted Files");
        println!(
            "{} | {} | {} | {} | {}",
            "File".bright_cyan().bold(),
            "Action".bright_cyan().bold(),
            "Matched Rule".bright_cyan().bold(),
            "Current Path".bright_cyan().bold(),
            "New Path".bright_cyan().bold()
        );
        println!("{}", "─".repeat(120).bright_black());
        for result in changed {
            println!(
                "{:<30} | {:<8} | {:<20} | {} | {}",
                result.file_name.bright_white(),
                result.action.magenta(),
                result.matched_rule_id.green(),
                result.current_path.display().to_string().yellow(),
                result.new_path.display().to_string().blue()
            );
        }
    }

    for warning in &report.warnings {
        cli::warning(warning);
    }
    for failure in &report.commit_failures {
        cli::error(failure);
    }
}
