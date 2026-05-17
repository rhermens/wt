use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use log::error;
use wt::{git::WorktreeContext, settings};

#[derive(Parser, Debug)]
struct Args {
    #[arg(index = 1, required = true)]
    pub path: PathBuf,

    #[arg(index = 2, num_args = 1..)]
    pub command_substitutions: Vec<String>,
}

fn main() -> ExitCode {
    settings::init_log();
    let args = Args::parse();

    let worktree = match WorktreeContext::try_new(&args.path, &args.command_substitutions) {
        Ok(wt) => wt,
        Err(e) => {
            error!("Failed to open worktree: {}", e);
            return ExitCode::FAILURE;
        }
    };

    worktree.copy_sources();
    worktree.link_sources();
    worktree.spawn_commands();
    worktree.spawn_tmux_session();

    ExitCode::SUCCESS
}
