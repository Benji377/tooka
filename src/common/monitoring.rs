//! Monitoring configuration and daemon state.
//!
//! The monitoring daemon periodically sorts configured folders. This module
//! stores which folders are monitored (`monitoring.yml` in the config
//! directory) and tracks whether the daemon is running.
//!
//! "Is the daemon running?" is answered with an operating-system file lock
//! held by the daemon (`monitor.lock`): the OS releases it automatically
//! when the daemon exits or crashes, so a stale state is impossible. The PID
//! file is only used to know which process to stop.

use crate::{common::environment, core::error::TookaError, utils::fsutil};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions, TryLockError},
    io,
    path::{Path, PathBuf},
};

const MONITOR_FILE_NAME: &str = "monitoring.yml";
const PID_FILE_NAME: &str = "monitor.pid";
const LOCK_FILE_NAME: &str = "monitor.lock";
const STOP_FILE_NAME: &str = "monitor.stop";

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
    /// Loads the monitoring configuration, creating an empty one if needed.
    ///
    /// Tooka versions before 1.2.0 stored this file in `~/.config/tooka`
    /// on every platform. If it only exists there, it is migrated.
    pub fn load() -> Result<Self, TookaError> {
        let config_path = Self::config_path();

        if !config_path.exists()
            && let Some(legacy) = Self::legacy_config_path()
            && legacy != config_path
            && legacy.is_file()
        {
            log::info!(
                "Migrating monitoring config from {} to {}",
                legacy.display(),
                config_path.display()
            );
            let content = fs::read_to_string(&legacy)?;
            let config: MonitorConfig = serde_saphyr::from_str(&content)?;
            config.save()?;
            return Ok(config);
        }

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            Ok(serde_saphyr::from_str(&content)?)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Saves the monitoring configuration (atomically).
    pub fn save(&self) -> Result<(), TookaError> {
        let content = crate::utils::yaml::to_string(self)?;
        fsutil::write_atomic(&Self::config_path(), content.as_bytes())?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        environment::config_dir().join(MONITOR_FILE_NAME)
    }

    /// Location used before 1.2.0, unless an explicit config dir is set.
    fn legacy_config_path() -> Option<PathBuf> {
        if std::env::var_os("TOOKA_CONFIG_DIR").is_some() {
            return None;
        }
        Some(
            environment::home_dir()
                .join(".config")
                .join("tooka")
                .join(MONITOR_FILE_NAME),
        )
    }

    /// Path of the file holding the daemon's PID.
    pub fn pid_file_path() -> PathBuf {
        environment::config_dir().join(PID_FILE_NAME)
    }

    /// Reads the daemon PID, if a PID file exists.
    pub fn read_pid() -> Result<Option<u32>, TookaError> {
        let content = match fs::read_to_string(Self::pid_file_path()) {
            Ok(c) => c,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        content
            .trim()
            .parse::<u32>()
            .map(Some)
            .map_err(|e| TookaError::ConfigError(format!("Invalid PID in file: {e}")))
    }

    /// Path of the file that asks a running daemon to stop.
    pub fn stop_request_path() -> PathBuf {
        environment::config_dir().join(STOP_FILE_NAME)
    }

    /// Asks the running daemon to stop at its next safe point.
    pub fn request_stop() -> Result<(), TookaError> {
        fsutil::write_atomic(&Self::stop_request_path(), b"stop")?;
        Ok(())
    }

    /// Returns true if a monitoring daemon currently holds the daemon lock.
    pub fn is_daemon_running() -> bool {
        let Ok(file) = open_lock_file(&environment::config_dir()) else {
            return false;
        };
        matches!(file.try_lock(), Err(TryLockError::WouldBlock))
    }

    /// Adds a folder to monitor. The path is stored as an absolute path, so
    /// it does not depend on where the daemon is started.
    pub fn add_folder(
        &mut self,
        path: PathBuf,
        interval_seconds: u64,
        rule_filter: Option<Vec<String>>,
    ) -> Result<(), TookaError> {
        if !path.is_dir() {
            return Err(TookaError::ConfigError(format!(
                "Path '{}' does not exist or is not a directory",
                path.display()
            )));
        }
        if interval_seconds == 0 {
            return Err(TookaError::ConfigError(
                "The interval must be at least 1 second".into(),
            ));
        }
        let path = std::path::absolute(&path)?;

        if self.folders.iter().any(|f| f.path == path) {
            return Err(TookaError::ConfigError(format!(
                "Folder '{}' is already being monitored",
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

        self.save()
    }

    /// Removes a monitored folder.
    pub fn remove_folder(&mut self, path: &Path) -> Result<(), TookaError> {
        let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
        let initial_len = self.folders.len();
        self.folders
            .retain(|f| f.path != path && f.path != absolute);

        if self.folders.len() == initial_len {
            return Err(TookaError::ConfigError(format!(
                "Folder '{}' is not being monitored",
                path.display()
            )));
        }

        self.save()
    }

    /// Updates the status of a monitored folder after a run.
    pub fn update_status(&mut self, path: &Path, status: RunStatus) -> Result<(), TookaError> {
        let folder = self
            .folders
            .iter_mut()
            .find(|f| f.path == path)
            .ok_or_else(|| {
                TookaError::ConfigError(format!(
                    "Folder '{}' is not being monitored",
                    path.display()
                ))
            })?;
        folder.last_run = Some(Utc::now());
        folder.last_status = status;
        self.save()
    }

    /// Seconds until the folder's next run (0 if it is due), or `None` if
    /// it never ran.
    pub fn time_until_next_run(folder: &MonitoredFolder) -> Option<i64> {
        folder.last_run.map(|last| {
            let interval = i64::try_from(folder.interval_seconds).unwrap_or(i64::MAX);
            let next_run = last + chrono::Duration::seconds(interval);
            (next_run - Utc::now()).num_seconds().max(0)
        })
    }

    /// Checks if a folder is due for sorting.
    pub fn is_due(folder: &MonitoredFolder) -> bool {
        Self::time_until_next_run(folder).is_none_or(|secs| secs == 0)
    }
}

fn open_lock_file(dir: &Path) -> io::Result<File> {
    fs::create_dir_all(dir)?;
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join(LOCK_FILE_NAME))
}

/// Proof that this process is the (only) monitoring daemon. Holding it keeps
/// the daemon lock; dropping it removes the PID file.
#[derive(Debug)]
pub struct DaemonGuard {
    _lock: File,
}

impl DaemonGuard {
    /// Becomes the daemon. Returns `Ok(None)` if another daemon is running.
    pub fn acquire() -> Result<Option<Self>, TookaError> {
        let dir = environment::config_dir();
        let lock = open_lock_file(&dir)?;
        // `is_daemon_running` holds the lock for a moment while checking, so
        // retry briefly before concluding that another daemon runs.
        let mut attempts = 0;
        loop {
            match lock.try_lock() {
                Ok(()) => break,
                Err(TryLockError::WouldBlock) if attempts < 10 => {
                    attempts += 1;
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(TryLockError::WouldBlock) => return Ok(None),
                Err(TryLockError::Error(e)) => return Err(e.into()),
            }
        }
        // A stop request left over from an earlier daemon is not for us.
        let _ = fs::remove_file(dir.join(STOP_FILE_NAME));
        fsutil::write_atomic(
            &dir.join(PID_FILE_NAME),
            std::process::id().to_string().as_bytes(),
        )?;
        Ok(Some(Self { _lock: lock }))
    }
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(MonitorConfig::pid_file_path());
    }
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunStatus::NotRun => write!(f, "Not run yet"),
            RunStatus::Success { files_processed } => {
                write!(f, "Success ({files_processed} files processed)")
            }
            RunStatus::Failed { error } => write!(f, "Failed: {error}"),
        }
    }
}
