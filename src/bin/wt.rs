use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use log::error;
use wt::{git::WorktreeContext, settings};

#[derive(Parser, Debug)]
struct Args {
    #[arg(index = 1, required = true)]
    pub path: PathBuf,

    #[arg(index = 2, num_args = 1..)]
    pub args: Vec<String>,
}

fn main() -> ExitCode {
    settings::init_log();
    let args = Args::parse();

    let worktree = match WorktreeContext::try_new(&args.path, &args.args) {
        Ok(wt) => wt,
        Err(wt::error::Error::LuaError { source }) => {
            error!("{}", source);
            return ExitCode::FAILURE;
        }
        Err(e) => {
            error!("Failed to open worktree: {:?}", e);
            return ExitCode::FAILURE;
        }
    };

    for e in worktree
        .copy_sources()
        .into_iter()
        .chain(worktree.link_sources())
        .chain(worktree.spawn_commands())
        .chain(worktree.spawn_tmux_session())
    {
        error!("{}", e);
    }

    ExitCode::SUCCESS
}
