//! Monitoring configuration and state management.
//!
//! This module manages the background monitoring system that automatically
//! sorts files in specified folders at configured intervals.

use crate::core::error::TookaError;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Represents a single monitored folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredFolder {
    /// Path to the folder being monitored
    pub path: PathBuf,
    /// Interval in seconds between checks
    pub interval_seconds: u64,
    /// Last time the folder was sorted
    pub last_run: Option<DateTime<Utc>>,
    /// Status of the last run
    pub last_status: RunStatus,
    /// Optional rule IDs to apply (None means all rules)
    pub rule_filter: Option<Vec<String>>,
}

/// Status of the last monitoring run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunStatus {
    /// Never run yet
    NotRun,
    /// Last run succeeded
    Success { files_processed: usize },
    /// Last run failed
    Failed { error: String },
}

/// Configuration for all monitored folders
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitorConfig {
    /// Version of the monitoring config format
    pub version: usize,
    /// List of monitored folders
    pub folders: Vec<MonitoredFolder>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            version: 1,
            folders: Vec::new(),
        }
    }
}

impl MonitorConfig {
    /// Load monitoring configuration from disk
    pub fn load() -> Result<Self, TookaError> {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: MonitorConfig = serde_yaml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save monitoring configuration to disk
    pub fn save(&self) -> Result<(), TookaError> {
        let config_path = Self::config_path();
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_yaml::to_string(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    /// Get the path to the monitoring configuration file
    fn config_path() -> PathBuf {
        let home_dir = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        
        let config_dir = home_dir.join(".config").join("tooka");
        config_dir.join("monitoring.yml")
    }

    /// Get the path to the PID file
    pub fn pid_file_path() -> PathBuf {
        let home_dir = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        
        let config_dir = home_dir.join(".config").join("tooka");
        config_dir.join("monitor.pid")
    }

    /// Write the daemon PID to a file
    pub fn write_pid() -> Result<(), TookaError> {
        let pid_path = Self::pid_file_path();
        
        if let Some(parent) = pid_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let pid = std::process::id();
        fs::write(&pid_path, pid.to_string())?;
        Ok(())
    }

    /// Remove the PID file
    pub fn remove_pid() -> Result<(), TookaError> {
        let pid_path = Self::pid_file_path();
        if pid_path.exists() {
            fs::remove_file(&pid_path)?;
        }
        Ok(())
    }

    /// Read the daemon PID from the file
    pub fn read_pid() -> Result<Option<u32>, TookaError> {
        let pid_path = Self::pid_file_path();
        
        if !pid_path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&pid_path)?;
        let pid = content.trim().parse::<u32>()
            .map_err(|e| TookaError::ConfigError(format!("Invalid PID in file: {}", e)))?;
        
        Ok(Some(pid))
    }

    /// Check if the daemon is running
    pub fn is_daemon_running() -> bool {
        if let Ok(Some(pid)) = Self::read_pid() {
            // Check if the process is actually running
            #[cfg(unix)]
            {
                use std::process::Command;
                if let Ok(output) = Command::new("ps")
                    .arg("-p")
                    .arg(pid.to_string())
                    .output()
                {
                    return output.status.success();
                }
            }
            
            #[cfg(not(unix))]
            {
                // On Windows, we can use different approach
                // For now, just assume if PID file exists, it's running
                return true;
            }
        }
        false
    }

    /// Add a new folder to monitor
    pub fn add_folder(
        &mut self,
        path: PathBuf,
        interval_seconds: u64,
        rule_filter: Option<Vec<String>>,
    ) -> Result<(), TookaError> {
        // Check if folder already exists
        if self.folders.iter().any(|f| f.path == path) {
            return Err(TookaError::ConfigError(format!(
                "Folder '{}' is already being monitored",
                path.display()
            )));
        }

        // Verify the path exists and is a directory
        if !path.exists() {
            return Err(TookaError::ConfigError(format!(
                "Path '{}' does not exist",
                path.display()
            )));
        }

        if !path.is_dir() {
            return Err(TookaError::ConfigError(format!(
                "Path '{}' is not a directory",
                path.display()
            )));
        }

        self.folders.push(MonitoredFolder {
            path,
            interval_seconds,
            last_run: None,
            last_status: RunStatus::NotRun,
            rule_filter,
        });

        self.save()?;
        Ok(())
    }

    /// Remove a monitored folder
    pub fn remove_folder(&mut self, path: &PathBuf) -> Result<(), TookaError> {
        let initial_len = self.folders.len();
        self.folders.retain(|f| &f.path != path);

        if self.folders.len() == initial_len {
            return Err(TookaError::ConfigError(format!(
                "Folder '{}' is not being monitored",
                path.display()
            )));
        }

        self.save()?;
        Ok(())
    }

    /// Update the status of a monitored folder after a run
    pub fn update_status(
        &mut self,
        path: &PathBuf,
        status: RunStatus,
    ) -> Result<(), TookaError> {
        if let Some(folder) = self.folders.iter_mut().find(|f| &f.path == path) {
            folder.last_run = Some(Utc::now());
            folder.last_status = status;
            self.save()?;
            Ok(())
        } else {
            Err(TookaError::ConfigError(format!(
                "Folder '{}' is not being monitored",
                path.display()
            )))
        }
    }

    /// Get time until next run for a folder
    pub fn time_until_next_run(&self, path: &PathBuf) -> Option<i64> {
        self.folders
            .iter()
            .find(|f| &f.path == path)
            .and_then(|folder| {
                folder.last_run.map(|last| {
                    let next_run = last + chrono::Duration::seconds(folder.interval_seconds as i64);
                    let now = Utc::now();
                    (next_run - now).num_seconds().max(0)
                })
            })
    }

    /// Check if a folder is due for sorting
    pub fn is_due(&self, folder: &MonitoredFolder) -> bool {
        match folder.last_run {
            None => true, // Never run
            Some(last) => {
                let elapsed = Utc::now().signed_duration_since(last);
                elapsed.num_seconds() >= folder.interval_seconds as i64
            }
        }
    }
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunStatus::NotRun => write!(f, "Not run yet"),
            RunStatus::Success { files_processed } => {
                write!(f, "Success ({} files processed)", files_processed)
            }
            RunStatus::Failed { error } => write!(f, "Failed: {}", error),
        }
    }
}
