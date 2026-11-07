//! Process-wide handling of Ctrl+C and termination signals.
//!
//! Instead of killing the process mid-operation, a signal only sets a flag.
//! Long-running work (sorting, the monitoring daemon) checks the flag at safe
//! points and stops cleanly, rolling back an unfinished sort.

use std::sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);
static HANDLER: OnceLock<()> = OnceLock::new();

/// Installs the signal handler once per process. Safe to call repeatedly.
///
/// With the `termination` feature of `ctrlc`, this also covers SIGTERM
/// (used by `tooka monitor daemon stop`) and SIGHUP on Unix.
pub fn install_handler() {
    HANDLER.get_or_init(|| {
        if let Err(e) = ctrlc::set_handler(|| INTERRUPTED.store(true, Ordering::SeqCst)) {
            log::warn!("Failed to install the Ctrl+C handler: {e}");
        }
    });
}

/// Asks the process to stop at the next safe point, exactly like Ctrl+C.
pub fn request() {
    INTERRUPTED.store(true, Ordering::SeqCst);
}

/// Returns true once the user asked the process to stop.
pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}
