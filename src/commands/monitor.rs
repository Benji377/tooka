//! Monitor command implementation for managing background folder monitoring.
//!
//! This module provides subcommands to:
//! - Add folders to monitor
//! - Remove folders from monitoring
//! - List all monitored folders
//! - Show status and timing information
//! - Run the background daemon

use crate::cli;
use crate::common::config::Config;
use crate::common::monitoring::{MonitorConfig, MonitoredFolder, RunStatus};
use crate::core::sorter;
use crate::rules::rules_file::RulesFile;
use anyhow::Result;
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
    cli::info(&format!(
        "Removing folder from monitoring: {}",
        args.path
    ));

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
            println!("  {} {}", "Rules:".bright_white(), "All rules".bright_yellow());
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
            println!("  {} {}", "Last Run:".bright_white(), "Never".bright_black());
        }

        // Status
        let status_str = match &folder.last_status {
            RunStatus::NotRun => "Not run yet".bright_black().to_string(),
            RunStatus::Success { files_processed } => {
                format!("✓ Success ({} files)", files_processed).green().to_string()
            }
            RunStatus::Failed { error } => format!("✗ Failed: {}", error).red().to_string(),
        };
        println!("  {} {}", "Status:".bright_white(), status_str);

        // Time until next run
        if let Some(seconds) = monitor_config.time_until_next_run(&folder.path) {
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
        } else if monitor_config.is_due(folder) {
            println!(
                "  {} {}",
                "Next Run:".bright_white(),
                "Due now!".bright_green().bold()
            );
        } else {
            println!(
                "  {} {}",
                "Next Run:".bright_white(),
                format!(
                    "In {} seconds",
                    folder.interval_seconds
                )
                .bright_yellow()
            );
        }

        println!();
    }

    Ok(())
}

fn run_daemon(args: &DaemonRunArgs) -> Result<()> {
    // Check if daemon is already running
    if MonitorConfig::is_daemon_running() {
        cli::error("Monitoring daemon is already running");
        cli::info("Use 'tooka monitor daemon stop' to stop the running daemon first");
        return Ok(());
    }

    cli::header("⏰ Starting Monitoring Daemon");
    cli::info("The daemon will run continuously and sort folders at their scheduled intervals");
    cli::info("Press Ctrl+C to stop the daemon");
    println!();

    log::info!("Starting monitoring daemon with check interval {} seconds", args.check_interval);

    let mut monitor_config = MonitorConfig::load()?;
    let config = Config::load()?;

    if monitor_config.folders.is_empty() {
        cli::warning("No folders are currently being monitored");
        cli::info("Use 'tooka monitor add <path>' to add a folder");
        return Ok(());
    }

    // Write PID file
    MonitorConfig::write_pid()?;
    
    // Set up signal handler for graceful shutdown
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    
    if let Err(e) = ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    }) {
        cli::warning(&format!("Failed to set Ctrl-C handler: {}", e));
        log::warn!("Failed to set Ctrl-C handler: {}", e);
    }

    // Display initial status
    for folder in &monitor_config.folders {
        println!(
            "  📁 {} (every {} seconds)",
            folder.path.display().to_string().bright_cyan(),
            folder.interval_seconds
        );
    }
    println!();

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // Reload config each iteration to pick up changes
        monitor_config = MonitorConfig::load()?;

        for folder in monitor_config.folders.clone() {
            if monitor_config.is_due(&folder) {
                run_folder_sort(&mut monitor_config, &folder, &config)?;
            }
        }

        thread::sleep(Duration::from_secs(args.check_interval));
    }

    // Clean up PID file on exit
    println!();
    cli::info("Stopping monitoring daemon...");
    MonitorConfig::remove_pid()?;
    cli::success("Daemon stopped successfully");
    log::info!("Monitoring daemon stopped");

    Ok(())
}

fn run_folder_sort(
    monitor_config: &mut MonitorConfig,
    folder: &MonitoredFolder,
    _config: &Config,
) -> Result<()> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    cli::info(&format!(
        "[{}] Sorting folder: {}",
        timestamp,
        folder.path.display()
    ));

    log::info!("Running scheduled sort for folder: {}", folder.path.display());

    // Load rules
    let rules_file = match RulesFile::load() {
        Ok(rf) => rf,
        Err(e) => {
            let error_msg = format!("Failed to load rules: {}", e);
            cli::error(&error_msg);
            monitor_config.update_status(&folder.path, RunStatus::Failed { error: error_msg })?;
            return Ok(());
        }
    };

    // Get optimized rules with filter
    let optimized_rules = match rules_file.optimized_with_filter(folder.rule_filter.as_deref()) {
        Ok(rules) => rules,
        Err(e) => {
            let error_msg = format!("Failed to optimize rules: {}", e);
            cli::error(&error_msg);
            monitor_config.update_status(&folder.path, RunStatus::Failed { error: error_msg })?;
            return Ok(());
        }
    };

    // Collect files
    let files = match sorter::collect_files(&folder.path) {
        Ok(f) => f,
        Err(e) => {
            let error_msg = format!("Failed to collect files: {}", e);
            cli::error(&error_msg);
            monitor_config.update_status(&folder.path, RunStatus::Failed { error: error_msg })?;
            return Ok(());
        }
    };

    // Sort files
    match sorter::sort_files(&files, &folder.path, &optimized_rules, false, Option::<fn()>::None) {
        Ok(results) => {
            let files_processed = results.len();
            cli::success(&format!(
                "[{}] Sorted {} file(s) in {}",
                timestamp,
                files_processed,
                folder.path.display()
            ));
            log::info!(
                "Successfully sorted {} files in folder: {}",
                files_processed,
                folder.path.display()
            );
            monitor_config.update_status(
                &folder.path,
                RunStatus::Success { files_processed },
            )?;
        }
        Err(e) => {
            let error_msg = format!("Sorting failed: {}", e);
            cli::error(&format!("[{}] {}", timestamp, error_msg));
            log::error!("Failed to sort folder {}: {}", folder.path.display(), e);
            monitor_config.update_status(&folder.path, RunStatus::Failed { error: error_msg })?;
        }
    }

    Ok(())
}

fn stop_daemon() -> Result<()> {
    cli::info("🛑 Stopping monitoring daemon...");
    
    if !MonitorConfig::is_daemon_running() {
        cli::warning("Monitoring daemon is not running");
        return Ok(());
    }

    match MonitorConfig::read_pid()? {
        Some(pid) => {
            log::info!("Stopping monitoring daemon with PID {}", pid);
            
            #[cfg(unix)]
            {
                use std::process::Command;
                
                // Send SIGTERM signal
                let result = Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .status();
                
                match result {
                    Ok(status) if status.success() => {
                        // Wait a bit for the daemon to shut down gracefully
                        thread::sleep(Duration::from_millis(500));
                        
                        // Check if it's still running
                        if MonitorConfig::is_daemon_running() {
                            cli::warning("Daemon still running, sending stronger signal...");
                            let _ = Command::new("kill")
                                .arg("-KILL")
                                .arg(pid.to_string())
                                .status();
                            thread::sleep(Duration::from_millis(200));
                        }
                        
                        // Clean up PID file if daemon didn't
                        let _ = MonitorConfig::remove_pid();
                        
                        cli::success("Monitoring daemon stopped successfully");
                        log::info!("Monitoring daemon stopped");
                    }
                    Ok(_) => {
                        cli::error(&format!("Failed to stop daemon with PID {}", pid));
                        // Clean up stale PID file
                        let _ = MonitorConfig::remove_pid();
                    }
                    Err(e) => {
                        cli::error(&format!("Failed to send signal to daemon: {}", e));
                        // Clean up stale PID file
                        let _ = MonitorConfig::remove_pid();
                    }
                }
            }
            
            #[cfg(not(unix))]
            {
                // On Windows, we need a different approach
                cli::warning("Stopping daemon on Windows is not yet implemented");
                cli::info("Please stop the daemon manually using Task Manager or Ctrl+C");
            }
        }
        None => {
            cli::warning("No PID file found for running daemon");
        }
    }

    Ok(())
}

fn check_daemon_status() -> Result<()> {
    if MonitorConfig::is_daemon_running() {
        if let Ok(Some(pid)) = MonitorConfig::read_pid() {
            cli::success(&format!("Monitoring daemon is running (PID: {})", pid));
            log::info!("Monitoring daemon is running with PID {}", pid);
        } else {
            cli::success("Monitoring daemon is running");
        }
    } else {
        cli::info("Monitoring daemon is not running");
        log::info!("Monitoring daemon is not running");
    }

    Ok(())
}
