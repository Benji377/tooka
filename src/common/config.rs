//! Configuration management for the Tooka application.
//!
//! This module defines the [`Config`] struct used to manage runtime configuration
//! including source folders, rules file paths, and logging directories.
//!
//! It provides functionality to load, save, reset, and display configuration
//! settings from a user-specific file (typically stored in `$HOME/.config/tooka/config.yml`).

use super::environment::{config_dir, data_dir, default_source_folder};
use crate::{
    core::context::{CONFIG_FILE_NAME, CONFIG_VERSION, DEFAULT_LOGS_FOLDER, RULES_FILE_NAME},
    core::error::TookaError,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Represents the user configuration for Tooka.
///
/// The configuration can be loaded from a YAML file, typically located in
/// the user's config directory (e.g., `$HOME/.config/tooka/config.yml`).
/// If the file doesn't exist, a default configuration is generated and saved.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Version of the configuration file
    pub version: usize,
    /// Folder that Tooka will sort files in
    pub source_folder: PathBuf,
    /// Path to the file containing all rules
    pub rules_file: PathBuf,
    /// Folder where Tooka will store logs
    pub logs_folder: PathBuf,
}

/// Default values for the configuration
impl Default for Config {
    /// Creates a default configuration for Tooka using fallback folder paths
    fn default() -> Self {
        log::debug!("Creating default configuration for Tooka");

        Self::new_with_fallbacks()
    }
}

/// Implementation of the Config struct
impl Config {
    /// Creates a new configuration with fallback paths
    fn new_with_fallbacks() -> Self {
        let data_dir = data_dir();
        Self {
            version: CONFIG_VERSION,
            source_folder: default_source_folder(),
            rules_file: data_dir.join(RULES_FILE_NAME),
            logs_folder: data_dir.join(DEFAULT_LOGS_FOLDER),
        }
    }

    /// Loads the Tooka configuration from the default file path.
    ///
    /// If the configuration file exists, it is parsed and returned.
    /// If it does not exist, a new configuration is created using default
    /// values and written to disk.
    ///
    /// # Errors
    /// Returns a [`TookaError`] if the configuration could not be loaded or saved.
    pub fn load() -> Result<Self, TookaError> {
        log::debug!("Loading configuration for Tooka");
        let config_path = Self::config_path();

        if config_path.exists() {
            let file = fs::File::open(&config_path)?;
            let reader = std::io::BufReader::new(file);
            let config: Config = serde_saphyr::from_reader(reader)?;
            Ok(config)
        } else {
            let config = Config::new_with_fallbacks();
            config.save()?;
            Ok(config)
        }
    }

    /// Saves the current configuration to the default path on disk.
    ///
    /// # Errors
    /// Returns a [`TookaError`] if the configuration could not be written to disk.
    pub fn save(&self) -> Result<(), TookaError> {
        let content = crate::utils::yaml::to_string(self)?;
        crate::utils::fsutil::write_atomic(&Self::config_path(), content.as_bytes())?;
        Ok(())
    }

    /// Returns the resolved path of the configuration file.
    ///
    /// # Errors
    /// Returns a [`TookaError`] if the path could not be determined or the file is missing.
    pub fn locate_config_file() -> Result<PathBuf, TookaError> {
        let config_path = Self::config_path();
        if config_path.exists() {
            Ok(config_path)
        } else {
            Err(TookaError::ConfigError("Config file not found".into()))
        }
    }

    /// Resets the configuration to default values and writes it to disk.
    ///
    /// This can be used to discard manual changes or recover from a corrupted config file.
    ///
    /// # Errors
    /// Returns a [`TookaError`] if the default configuration cannot be created or saved.
    pub fn reset_config(&mut self) -> Result<(), TookaError> {
        *self = Config::new_with_fallbacks();
        self.save()
    }

    /// Returns the current configuration as a YAML-formatted string.
    ///
    /// If serialization fails, a fallback error message is returned.
    pub fn show_config(&self) -> String {
        crate::utils::yaml::to_string(self).unwrap_or_else(|_| "Failed to serialize config".into())
    }

    /// Returns the path to the configuration file.
    fn config_path() -> PathBuf {
        config_dir().join(CONFIG_FILE_NAME)
    }
}
