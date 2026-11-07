//! Monitor command implementation for managing background folder monitoring.
//!
//! This module provides subcommands to:
//! - Add folders to monitor
//! - Remove folders from monitoring
//! - List all monitored folders
//! - Show status and timing information
//! - Run the background daemon

use crate::cli;
use crate::common::environment;
use crate::common::monitoring::{DaemonGuard, MonitorConfig, MonitoredFolder, RunStatus};
use crate::core::engine::{self, Progress, SortError};
use crate::core::{interrupt, sorter};
use anyhow::{Result, bail};
use chrono::Local;
use clap::{Args, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Args)]
#[command(about = "⏰ Manage background folder monitoring and automatic sorting")]
pub struct MonitorArgs {
    #[clap(subcommand)]
    pub command: MonitorCommands,
}

#[derive(Subcommand)]
pub enum MonitorCommands {
    /// Add a new folder to monitor
    Add(AddMonitorArgs),
    /// Remove a folder from monitoring
    Remove(RemoveMonitorArgs),
    /// List all monitored folders
    List,
    /// Show status of monitored folders
    Status,
    /// Manage the monitoring daemon
    Daemon(DaemonArgs),
}

#[derive(Args)]
pub struct AddMonitorArgs {
    /// Path to the folder to monitor
    #[arg(value_name = "PATH", help = "Path to the folder to monitor")]
    pub path: String,

    /// Interval in seconds between automatic sorts
    #[arg(
        short,
        long,
        default_value = "3600",
        help = "Interval in seconds between automatic sorts (default: 3600 = 1 hour)"
    )]
    pub interval: u64,

    /// Comma-separated rule IDs to apply (omit for all rules)
    #[arg(
        long,
        help = "Comma-separated list of rule IDs to execute (omit for all rules)"
    )]
    pub rules: Option<String>,
}

#[derive(Args)]
pub struct RemoveMonitorArgs {
    /// Path to the folder to stop monitoring
    #[arg(value_name = "PATH", help = "Path to the folder to stop monitoring")]
    pub path: String,
}

#[derive(Args)]
#[command(about = "Manage the monitoring daemon")]
pub struct DaemonArgs {
    #[clap(subcommand)]
    pub command: DaemonCommands,
}

#[derive(Subcommand)]
pub enum DaemonCommands {
    /// Run the monitoring daemon in the foreground
    Run(DaemonRunArgs),
    /// Stop the running monitoring daemon
    Stop,
    /// Check if the monitoring daemon is running
    Status,
}

#[derive(Args)]
pub struct DaemonRunArgs {
    /// Override check frequency in seconds (minimum time between checks)
    #[arg(
        short,
        long,
        default_value = "60",
        help = "How often to check for due folders in seconds (default: 60)"
    )]
    pub check_interval: u64,
}

pub fn run(args: &MonitorArgs) -> Result<()> {
    match &args.command {
        MonitorCommands::Add(add_args) => add_folder(add_args),
        MonitorCommands::Remove(remove_args) => remove_folder(remove_args),
        MonitorCommands::List => list_folders(),
        MonitorCommands::Status => show_status(),
        MonitorCommands::Daemon(daemon_args) => run_daemon_command(daemon_args),
    }
}

fn run_daemon_command(args: &DaemonArgs) -> Result<()> {
    match &args.command {
        DaemonCommands::Run(run_args) => run_daemon(run_args),
        DaemonCommands::Stop => stop_daemon(),
        DaemonCommands::Status => check_daemon_status(),
    }
}

fn add_folder(args: &AddMonitorArgs) -> Result<()> {
    cli::info(&format!("Adding folder to monitoring: {}", args.path));

    let path = PathBuf::from(&args.path);
    let rule_filter = args.rules.as_ref().map(|r| {
        r.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
    });

    let mut monitor_config = MonitorConfig::load()?;
    monitor_config.add_folder(path.clone(), args.interval, rule_filter)?;

    cli::success(&format!(
        "Folder '{}' added to monitoring (interval: {} seconds)",
        path.display(),
        args.interval
    ));

    log::info!(
        "Added folder '{}' to monitoring with interval {} seconds",
        path.display(),
        args.interval
    );

    Ok(())
}

fn remove_folder(args: &RemoveMonitorArgs) -> Result<()> {
    cli::info(&format!("Removing folder from monitoring: {}", args.path));

    let path = PathBuf::from(&args.path);
    let mut monitor_config = MonitorConfig::load()?;
    monitor_config.remove_folder(&path)?;

    cli::success(&format!(
        "Folder '{}' removed from monitoring",
        path.display()
    ));

    log::info!("Removed folder '{}' from monitoring", path.display());

    Ok(())
}

fn list_folders() -> Result<()> {
    let monitor_config = MonitorConfig::load()?;

    if monitor_config.folders.is_empty() {
        cli::warning("No folders are currently being monitored");
        cli::info("Use 'tooka monitor add <path>' to add a folder");
        return Ok(());
    }

    cli::header("📋 Monitored Folders");

    for (idx, folder) in monitor_config.folders.iter().enumerate() {
        println!("{}", format!("Folder {}", idx + 1).bright_cyan().bold());
        println!("  {} {}", "Path:".bright_white(), folder.path.display());
        println!(
            "  {} {} seconds ({} minutes)",
            "Interval:".bright_white(),
            folder.interval_seconds,
            folder.interval_seconds / 60
        );

        if let Some(rules) = &folder.rule_filter {
            println!(
                "  {} {}",
                "Rules:".bright_white(),
                rules.join(", ").bright_yellow()
            );
        } else {
            println!(
                "  {} {}",
                "Rules:".bright_white(),
                "All rules".bright_yellow()
            );
        }

        println!();
    }

    println!(
        "{}",
        format!("Total: {} folder(s)", monitor_config.folders.len())
            .bright_green()
            .bold()
    );

    Ok(())
}

fn show_status() -> Result<()> {
    let monitor_config = MonitorConfig::load()?;

    if monitor_config.folders.is_empty() {
        cli::warning("No folders are currently being monitored");
        cli::info("Use 'tooka monitor add <path>' to add a folder");
        return Ok(());
    }

    cli::header("📊 Monitoring Status");

    for (idx, folder) in monitor_config.folders.iter().enumerate() {
        println!("{}", format!("Folder {}", idx + 1).bright_cyan().bold());
        println!("  {} {}", "Path:".bright_white(), folder.path.display());

        // Last run information
        if let Some(last_run) = folder.last_run {
            let local_time = last_run.with_timezone(&Local);
            println!(
                "  {} {}",
                "Last Run:".bright_white(),
                local_time.format("%Y-%m-%d %H:%M:%S")
            );
        } else {
            println!(
                "  {} {}",
                "Last Run:".bright_white(),
                "Never".bright_black()
            );
        }

        // Status
        let status_str = match &folder.last_status {
            RunStatus::NotRun => "Not run yet".bright_black().to_string(),
            RunStatus::Success { files_processed } => {
                format!("✓ Success ({} files)", files_processed)
                    .green()
                    .to_string()
            }
            RunStatus::Failed { error } => format!("✗ Failed: {}", error).red().to_string(),
        };
        println!("  {} {}", "Status:".bright_white(), status_str);

        // Time until next run
        if let Some(seconds) = MonitorConfig::time_until_next_run(folder).filter(|s| *s > 0) {
            let minutes = seconds / 60;
            let hours = minutes / 60;
            let remaining_minutes = minutes % 60;

            let time_str = if hours > 0 {
                format!("{} hours, {} minutes", hours, remaining_minutes)
            } else {
                format!("{} minutes", minutes)
            };

            println!(
                "  {} {}",
                "Next Run:".bright_white(),
                time_str.bright_yellow()
            );
        } else if MonitorConfig::is_due(folder) {
            println!(
                "  {} {}",
                "Next Run:".bright_white(),
                "Due now!".bright_green().bold()
            );
        } else {
            println!(
                "  {} {}",
                "Next Run:".bright_white(),
                format!("In {} seconds", folder.interval_seconds).bright_yellow()
            );
        }

        println!();
    }

    Ok(())
}

fn run_daemon(args: &DaemonRunArgs) -> Result<()> {
    let Some(_guard) = DaemonGuard::acquire()? else {
        bail!(
            "Monitoring daemon is already running. Use 'tooka monitor daemon stop' to stop it first."
        );
    };

    let monitor_config = MonitorConfig::load()?;
    if monitor_config.folders.is_empty() {
        cli::warning("No folders are currently being monitored");
        cli::info("Use 'tooka monitor add <path>' to add a folder");
        return Ok(());
    }

    interrupt::install_handler();
    watch_for_stop_request();

    cli::header("⏰ Starting Monitoring Daemon");
    cli::info("The daemon will run continuously and sort folders at their scheduled intervals");
    cli::info("Press Ctrl+C to stop the daemon");
    println!();
    log::info!(
        "Starting monitoring daemon with check interval {} seconds",
        args.check_interval
    );

    for folder in &monitor_config.folders {
        println!(
            "  📁 {} (every {} seconds)",
            folder.path.display().to_string().bright_cyan(),
            folder.interval_seconds
        );
    }
    println!();

    while !interrupt::is_interrupted() {
        // Reload each iteration to pick up folders added or removed meanwhile.
        match MonitorConfig::load() {
            Ok(config) => {
                for folder in config.folders.iter().filter(|f| MonitorConfig::is_due(f)) {
                    if interrupt::is_interrupted() {
                        break;
                    }
                    run_folder_sort(folder);
                }
            }
            Err(e) => {
                cli::error(&format!("Could not read the monitoring configuration: {e}"));
                log::error!("Could not read the monitoring configuration: {e}");
            }
        }
        sleep_unless_interrupted(Duration::from_secs(args.check_interval.max(1)));
    }

    println!();
    cli::success("Daemon stopped");
    log::info!("Monitoring daemon stopped");
    Ok(())
}

/// Turns a stop request from `tooka monitor daemon stop` into the same
/// clean shutdown as Ctrl+C (works on every platform, unlike signals).
fn watch_for_stop_request() {
    let path = MonitorConfig::stop_request_path();
    thread::spawn(move || {
        loop {
            if path.exists() {
                log::info!("Stop requested");
                let _ = std::fs::remove_file(&path);
                interrupt::request();
                return;
            }
            thread::sleep(Duration::from_millis(250));
        }
    });
}

/// Sleeps for `duration`, waking up early when the user asks to stop.
fn sleep_unless_interrupted(duration: Duration) {
    let step = Duration::from_millis(250);
    let mut slept = Duration::ZERO;
    while slept < duration && !interrupt::is_interrupted() {
        thread::sleep(step);
        slept += step;
    }
}

fn run_folder_sort(folder: &MonitoredFolder) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    cli::info(&format!(
        "[{timestamp}] Sorting folder: {}",
        folder.path.display()
    ));
    log::info!(
        "Running scheduled sort for folder: {}",
        folder.path.display()
    );

    let status = match sorter::sort_folder(
        &folder.path,
        folder.rule_filter.as_deref(),
        false,
        &Progress::none(),
    ) {
        Ok(report) => {
            let changed = report
                .results
                .iter()
                .filter(|r| r.action != "skip" && r.action != "unchanged")
                .count();
            for warning in &report.warnings {
                cli::warning(warning);
            }
            for failure in &report.commit_failures {
                cli::error(failure);
            }
            cli::success(&format!(
                "[{timestamp}] Sorted {changed} file(s) in {}",
                folder.path.display()
            ));
            if report.commit_failures.is_empty() {
                RunStatus::Success {
                    files_processed: changed,
                }
            } else {
                RunStatus::Failed {
                    error: format!(
                        "files were sorted, but {} follow-up task(s) failed: {}",
                        report.commit_failures.len(),
                        report.commit_failures.join("; ")
                    ),
                }
            }
        }
        Err(SortError::Busy) => {
            // A manual sort is running; try again at the next check.
            cli::info("Another sort is running; will try again later");
            return;
        }
        Err(e) => {
            cli::error(&format!("[{timestamp}] {e}"));
            log::error!("Failed to sort folder {}: {e}", folder.path.display());
            RunStatus::Failed {
                error: e.to_string(),
            }
        }
    };

    // Reload before saving, so changes made while sorting are kept.
    let result =
        MonitorConfig::load().and_then(|mut config| config.update_status(&folder.path, status));
    if let Err(e) = result {
        log::warn!(
            "Could not save the status of {}: {e}",
            folder.path.display()
        );
    }
}

fn stop_daemon() -> Result<()> {
    cli::info("🛑 Stopping monitoring daemon...");

    if !MonitorConfig::is_daemon_running() {
        cli::warning("Monitoring daemon is not running");
        return Ok(());
    }

    // Ask politely: the daemon stops at its next safe point and undoes a
    // sort in progress, exactly as with Ctrl+C.
    MonitorConfig::request_stop()?;
    if wait_until_stopped(Duration::from_secs(30)) {
        cli::success("Monitoring daemon stopped");
        return Ok(());
    }

    // A sort that copies a large file can take a while to reach a safe
    // point; killing it now would leave the sort for `tooka recover`.
    if engine::journal::exists(&environment::data_dir()) {
        cli::info("The daemon is finishing a sort; waiting up to 10 more minutes...");
        if wait_until_stopped(Duration::from_secs(600)) {
            cli::success("Monitoring daemon stopped");
            return Ok(());
        }
    }

    let Some(pid) = MonitorConfig::read_pid()? else {
        bail!(
            "The daemon does not respond and its PID file is missing; stop it with your task manager"
        );
    };
    cli::warning(&format!(
        "The daemon (PID {pid}) does not respond; forcing it to stop"
    ));
    log::warn!("Force-stopping monitoring daemon with PID {pid}");
    force_kill(pid)?;

    if wait_until_stopped(Duration::from_secs(5)) {
        let _ = std::fs::remove_file(MonitorConfig::stop_request_path());
        cli::success("Monitoring daemon stopped");
        cli::info("If it was in the middle of a sort, run `tooka recover` to undo that sort.");
        Ok(())
    } else {
        bail!("Could not stop the monitoring daemon (PID {pid})")
    }
}

#[cfg(unix)]
fn force_kill(pid: u32) -> Result<()> {
    let status = std::process::Command::new("kill")
        .args(["-KILL", &pid.to_string()])
        .status()?;
    if !status.success() {
        bail!("Failed to stop the daemon (PID {pid})");
    }
    Ok(())
}

#[cfg(windows)]
fn force_kill(pid: u32) -> Result<()> {
    let status = std::process::Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .status()?;
    if !status.success() {
        bail!("taskkill failed with {status}");
    }
    Ok(())
}

fn wait_until_stopped(timeout: Duration) -> bool {
    let step = Duration::from_millis(200);
    let mut waited = Duration::ZERO;
    while waited < timeout {
        if !MonitorConfig::is_daemon_running() {
            return true;
        }
        thread::sleep(step);
        waited += step;
    }
    !MonitorConfig::is_daemon_running()
}

fn check_daemon_status() -> Result<()> {
    if MonitorConfig::is_daemon_running() {
        match MonitorConfig::read_pid() {
            Ok(Some(pid)) => cli::success(&format!("Monitoring daemon is running (PID: {pid})")),
            _ => cli::success("Monitoring daemon is running"),
        }
    } else {
        cli::info("Monitoring daemon is not running");
    }
    Ok(())
}
