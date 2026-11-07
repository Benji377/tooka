use crate::cli;
use crate::common::environment;
use crate::core::engine::{self, Recovery};
use anyhow::{Result, bail};
use clap::Args;

#[derive(Args)]
#[command(
    about = "🩹 Undo a sort that was interrupted (crash, power loss, killed process)",
    long_about = "Tooka records every file operation of a sort before performing it. \
If a sort could not finish, this command uses that record to put all files back \
where they were before the sort started."
)]
pub struct RecoverArgs {}

pub fn run(_args: &RecoverArgs) -> Result<()> {
    log::info!("Running recover");
    match engine::recover(&environment::data_dir())? {
        Recovery::NothingToDo => {
            cli::success("Nothing to recover: no sort was interrupted.");
        }
        Recovery::RolledBack => {
            cli::success(
                "The interrupted sort was undone. All files are back where they were before it started.",
            );
        }
        Recovery::Incomplete { problems } => {
            for problem in &problems {
                cli::error(problem);
            }
            bail!(
                "{} problem(s) prevented a complete recovery. Fix them and run `tooka recover` again; \
                 nothing that could be undone safely was left behind.",
                problems.len()
            );
        }
        Recovery::CommitInterrupted { unfinished } => {
            cli::success(
                "The interrupted sort had already sorted all files, so nothing needs to be undone.",
            );
            if unfinished.is_empty() {
                cli::info("All of its follow-up tasks had finished.");
            } else {
                cli::warning("These follow-up tasks may not have run; check them by hand:");
                for task in &unfinished {
                    cli::warning(&format!("  - {task}"));
                }
            }
        }
    }
    Ok(())
}
