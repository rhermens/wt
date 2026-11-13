use std::process::ExitCode;

use clap::Parser;
use log::error;
use wt::{cli::Args, git::WorktreeContext, settings};

fn main() -> ExitCode {
    settings::init_log();
    let args = Args::parse();

    let cwd = args
        .cwd
        .or_else(|| std::env::current_dir().ok())
        .expect("Unable to read cwd");
    let worktree = match WorktreeContext::try_new(&cwd, &args.path, &args.args) {
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
