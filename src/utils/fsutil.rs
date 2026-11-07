//! Low-level filesystem helpers used by the sort engine and config storage.
//!
//! These wrap `std::fs` with the guarantees Tooka relies on: never overwrite
//! an existing file, make copies durable before the original is removed,
//! write config files atomically, and tolerate the short-lived file locks
//! that antivirus scanners and indexers take on Windows.

use std::{
    fs::{self, File, FileTimes, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Runs a filesystem operation, retrying briefly when Windows reports that
/// the file is temporarily locked by another process (sharing or lock
/// violation, access denied). On other platforms the operation runs once.
pub fn retry<T>(mut op: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    const DELAYS_MS: [u64; 6] = [50, 100, 200, 300, 500, 800];
    let mut attempt = 0;
    loop {
        match op() {
            Err(e) if is_transient_lock_error(&e) && attempt < DELAYS_MS.len() => {
                log::debug!("Retrying after transient lock error: {e}");
                std::thread::sleep(Duration::from_millis(DELAYS_MS[attempt]));
                attempt += 1;
            }
            result => return result,
        }
    }
}

#[cfg(windows)]
fn is_transient_lock_error(e: &io::Error) -> bool {
    // ERROR_ACCESS_DENIED (5), ERROR_SHARING_VIOLATION (32), ERROR_LOCK_VIOLATION (33)
    matches!(e.raw_os_error(), Some(5 | 32 | 33))
}

#[cfg(not(windows))]
fn is_transient_lock_error(_e: &io::Error) -> bool {
    false
}

/// Atomically reserves `path` by creating an empty file there.
///
/// Fails with `ErrorKind::AlreadyExists` if anything exists at `path`, so a
/// file that appeared after planning can never be overwritten.
pub fn claim(path: &Path) -> io::Result<()> {
    retry(|| File::create_new(path)).map(drop)
}

/// Copies `from` into the already claimed `to`, keeps the original's access
/// and modification times, and flushes the data to disk before returning.
///
/// Returns the number of bytes copied.
pub fn copy_durable(from: &Path, to: &Path) -> io::Result<u64> {
    let bytes = retry(|| fs::copy(from, to))?;
    let source_meta = fs::metadata(from)?;
    // `fs::copy` also copies permissions. A read-only copy cannot be opened
    // for writing, so keep it writable until timestamps and data are set.
    let readonly = source_meta.permissions().readonly();
    if readonly {
        let mut perms = fs::metadata(to)?.permissions();
        make_writable(&mut perms);
        fs::set_permissions(to, perms)?;
    }
    let file = retry(|| OpenOptions::new().write(true).open(to))?;
    let mut times = FileTimes::new();
    if let Ok(accessed) = source_meta.accessed() {
        times = times.set_accessed(accessed);
    }
    if let Ok(modified) = source_meta.modified() {
        times = times.set_modified(modified);
    }
    // Keeping timestamps is best effort (some filesystems refuse it), but
    // the data must be on disk before the original may be removed.
    if let Err(e) = file.set_times(times) {
        log::warn!("Could not keep timestamps on {}: {e}", to.display());
    }
    file.sync_all()?;
    drop(file);
    if readonly {
        fs::set_permissions(to, source_meta.permissions())?;
    }
    Ok(bytes)
}

#[cfg(unix)]
fn make_writable(perms: &mut fs::Permissions) {
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(perms.mode() | 0o200); // owner write only
}

#[cfg(not(unix))]
#[allow(clippy::permissions_set_readonly_false)] // Windows: only the read-only attribute
fn make_writable(perms: &mut fs::Permissions) {
    perms.set_readonly(false);
}

/// Removes a file. A missing file is an error (as with `fs::remove_file`).
///
/// On Windows, read-only files cannot be deleted, so the read-only
/// attribute is cleared first when needed (on Unix, deleting depends only
/// on the folder's permissions).
pub fn remove_file(path: &Path) -> io::Result<()> {
    match retry(|| fs::remove_file(path)) {
        #[cfg(windows)]
        Err(e)
            if e.kind() == io::ErrorKind::PermissionDenied
                && fs::symlink_metadata(path).is_ok_and(|m| m.permissions().readonly()) =>
        {
            let mut perms = fs::symlink_metadata(path)?.permissions();
            make_writable(&mut perms);
            fs::set_permissions(path, perms)?;
            retry(|| fs::remove_file(path))
        }
        other => other,
    }
}

/// Returns true if the content of `part` is equal to, or the beginning of,
/// the content of `whole` (e.g. a complete or an interrupted copy).
pub fn is_prefix_of(part: &Path, whole: &Path) -> io::Result<bool> {
    use std::io::Read;
    let (mut a, mut b) = (File::open(part)?, File::open(whole)?);
    if a.metadata()?.len() > b.metadata()?.len() {
        return Ok(false);
    }
    let (mut buf_a, mut buf_b) = (vec![0u8; 64 * 1024], vec![0u8; 64 * 1024]);
    loop {
        let n = a.read(&mut buf_a)?;
        if n == 0 {
            return Ok(true);
        }
        b.read_exact(&mut buf_b[..n])?;
        if buf_a[..n] != buf_b[..n] {
            return Ok(false);
        }
    }
}

/// Flushes a directory entry to disk so that a rename or file creation in it
/// survives a power loss. Only meaningful (and possible) on Unix.
pub fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(dir)?.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
    }
    Ok(())
}

/// Writes `contents` to `path` atomically: readers see either the old or the
/// new file, never a half-written one, even if Tooka is killed mid-write.
pub fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;

    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?;
    let mut tmp_name = file_name.to_os_string();
    tmp_name.push(format!(".tmp-{}", std::process::id()));
    let tmp_path = parent.join(tmp_name);

    let result = (|| {
        let mut file = File::create(&tmp_path)?;
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        retry(|| fs::rename(&tmp_path, path))?;
        sync_dir(parent)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

/// Returns true if `path` is an existing regular file of zero bytes.
pub fn is_empty_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.is_file() && m.len() == 0)
}

/// Returns true if something (file, directory or symlink) exists at `path`.
pub fn exists_no_follow(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

/// Size and modification time of a file, used to detect whether a file
/// changed between two points in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Fingerprint {
    pub size: u64,
    /// Modification time in nanoseconds since the Unix epoch, if available.
    /// (`u64` covers dates until the year 2554; the journal format cannot
    /// store `u128`.)
    pub modified_ns: Option<u64>,
}

impl Fingerprint {
    pub fn of(path: &Path) -> io::Result<Self> {
        let meta = fs::metadata(path)?;
        let modified_ns = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .and_then(|d| u64::try_from(d.as_nanos()).ok());
        Ok(Self {
            size: meta.len(),
            modified_ns,
        })
    }
}

/// Returns the current time as an RFC 3339 string (for journal records).
pub fn now_rfc3339() -> String {
    chrono::DateTime::<chrono::Local>::from(SystemTime::now()).to_rfc3339()
}

/// Checks that `name` can be used as a single file name on this platform.
pub fn validate_file_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("the new file name is empty".into());
    }
    if name == "." || name == ".." {
        return Err(format!("'{name}' is not a valid file name"));
    }
    if name.contains(['/', '\\']) {
        return Err(format!("'{name}' contains a path separator"));
    }
    if name.chars().any(char::is_control) {
        return Err(format!("'{name}' contains control characters"));
    }
    #[cfg(windows)]
    {
        if name.contains(['<', '>', ':', '"', '|', '?', '*']) {
            return Err(format!(
                "'{name}' contains a character that Windows does not allow (<>:\"|?*)"
            ));
        }
        if name.ends_with(['.', ' ']) {
            return Err(format!(
                "'{name}' ends with a dot or space, which Windows strips"
            ));
        }
        let stem = name
            .split('.')
            .next()
            .unwrap_or("")
            .trim_end()
            .to_ascii_uppercase();
        let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || ((stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.len() == 4
                && stem.as_bytes()[3].is_ascii_digit());
        if reserved {
            return Err(format!("'{name}' is a reserved device name on Windows"));
        }
    }
    Ok(())
}

/// Best-effort check whether `command` can be started by
/// `std::process::Command`. Returns `None` when the answer is uncertain.
///
/// On Unix the lookup mirrors `execvp`: a command containing a path
/// separator is used as is, otherwise each `PATH` entry is searched for an
/// executable file. On Windows, `Command` also searches system directories
/// and appends `.exe`, so a miss there is reported as uncertain.
pub fn command_exists(command: &str) -> Option<bool> {
    let path = Path::new(command);
    if path.components().count() > 1 || path.is_absolute() {
        return Some(is_executable(path));
    }

    let found = std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths)
            .any(|dir| candidates(&dir, command).iter().any(|c| is_executable(c)))
    });

    if found {
        Some(true)
    } else if cfg!(windows) {
        None
    } else {
        Some(false)
    }
}

fn candidates(dir: &Path, command: &str) -> Vec<PathBuf> {
    let mut list = vec![dir.join(command)];
    if cfg!(windows) && Path::new(command).extension().is_none() {
        list.push(dir.join(format!("{command}.exe")));
    }
    list
}

fn is_executable(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn claim_refuses_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("taken.txt");
        fs::write(&path, "keep me").unwrap();
        let err = claim(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(&path).unwrap(), "keep me");
    }

    #[test]
    fn copy_durable_keeps_content_and_mtime() {
        let dir = tempdir().unwrap();
        let from = dir.path().join("a.txt");
        let to = dir.path().join("b.txt");
        fs::write(&from, "hello").unwrap();
        let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
        File::options()
            .write(true)
            .open(&from)
            .unwrap()
            .set_modified(old)
            .unwrap();
        claim(&to).unwrap();
        assert_eq!(copy_durable(&from, &to).unwrap(), 5);
        assert_eq!(fs::read_to_string(&to).unwrap(), "hello");
        assert_eq!(fs::metadata(&to).unwrap().modified().unwrap(), old);
    }

    #[test]
    fn write_atomic_replaces_content() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        write_atomic(&path, b"one").unwrap();
        write_atomic(&path, b"two").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "two");
        assert_eq!(
            fs::read_dir(dir.path()).unwrap().count(),
            1,
            "no temp file left"
        );
    }

    #[test]
    fn file_name_validation() {
        assert!(validate_file_name("photo 2024.jpg").is_ok());
        assert!(validate_file_name("").is_err());
        assert!(validate_file_name("..").is_err());
        assert!(validate_file_name("a/b.txt").is_err());
        assert!(validate_file_name("a\\b.txt").is_err());
        #[cfg(windows)]
        {
            assert!(validate_file_name("con.txt").is_err());
            assert!(validate_file_name("report.").is_err());
            assert!(validate_file_name("what?.txt").is_err());
        }
    }

    #[test]
    fn command_lookup() {
        assert_eq!(
            command_exists("definitely-not-a-real-command-xyz"),
            if cfg!(windows) { None } else { Some(false) }
        );
        #[cfg(unix)]
        assert_eq!(command_exists("sh"), Some(true));
    }
}
