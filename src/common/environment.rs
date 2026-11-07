//! Resolution of the directories Tooka uses.
//!
//! Every path can be overridden with an environment variable, which is also
//! what the integration tests use to run Tooka in an isolated directory:
//!
//! | Variable              | Default (Linux)                | Used for                         |
//! |-----------------------|--------------------------------|----------------------------------|
//! | `TOOKA_CONFIG_DIR`    | `~/.config/tooka`              | `tooka.yaml`, monitoring config  |
//! | `TOOKA_DATA_DIR`      | `~/.local/share/tooka`         | rules, logs, sort journal        |
//! | `TOOKA_SOURCE_FOLDER` | the user's Downloads directory | default folder to sort           |
//!
//! On macOS and Windows the defaults follow the platform conventions
//! (via the `directories` crate).

use crate::core::context::{APP_NAME, APP_ORG, APP_QUALIFIER};
use directories::{BaseDirs, ProjectDirs, UserDirs};
use std::{
    env,
    path::{Path, PathBuf},
};

/// Returns the user's home directory.
///
/// Falls back to `$HOME`, then to the current directory, if the platform
/// cannot tell us.
pub fn home_dir() -> PathBuf {
    if let Some(dirs) = BaseDirs::new() {
        return dirs.home_dir().to_path_buf();
    }
    env::var_os("HOME").map_or_else(
        || {
            log::warn!("Could not determine the home directory; using the current directory");
            PathBuf::from(".")
        },
        PathBuf::from,
    )
}

/// Directory holding `tooka.yaml` and the monitoring configuration.
pub fn config_dir() -> PathBuf {
    dir_with_env("TOOKA_CONFIG_DIR", ProjectDirs::config_dir, ".config")
}

/// Directory holding the rules file, logs and the sort journal.
pub fn data_dir() -> PathBuf {
    dir_with_env("TOOKA_DATA_DIR", ProjectDirs::data_dir, ".local/share")
}

/// Returns the default folder to sort, usually the user's Downloads directory.
///
/// Uses `TOOKA_SOURCE_FOLDER` if set, otherwise the system's Downloads
/// directory, otherwise `~/Downloads`.
pub fn default_source_folder() -> PathBuf {
    if let Some(path) = env::var_os("TOOKA_SOURCE_FOLDER") {
        return PathBuf::from(path);
    }

    if let Some(user_dirs) = UserDirs::new()
        && let Some(downloads) = user_dirs.download_dir()
    {
        return downloads.to_path_buf();
    }

    let fallback = home_dir().join("Downloads");
    log::warn!(
        "Could not find the user's Downloads directory; using {}",
        fallback.display()
    );
    fallback
}

/// Resolves a directory from an environment variable, the platform's project
/// directories, or `~/<fallback_subdir>/tooka`, in that order.
fn dir_with_env(
    env_var: &str,
    project_dir: fn(&ProjectDirs) -> &Path,
    fallback_subdir: &str,
) -> PathBuf {
    if let Some(path) = env::var_os(env_var) {
        return PathBuf::from(path);
    }

    if let Some(proj_dirs) = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME) {
        return project_dir(&proj_dirs).to_path_buf();
    }

    let fallback = home_dir().join(fallback_subdir).join(APP_NAME);
    log::warn!("Using fallback for {env_var}: {}", fallback.display());
    fallback
}
