use std::path::PathBuf;

use clap::Parser;
use git_worktree_utils::{git::WorktreeContext, settings};
use log::error;

#[derive(Parser, Debug)]
struct Args {
    pub path: PathBuf,
}

fn main() {
    settings::init_log();
    let args = Args::parse();

    let worktree = match WorktreeContext::try_create_or_open(&args.path) {
        Ok(wt) => wt,
        Err(e) => {
            return error!("Failed to create worktree: {}", e);
        }
    };

    let settings = match settings::load_settings(&worktree.main_path) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to load settings: {}", e);
            return;
        }
    };

    worktree.copy_sources(&settings.copy);
    worktree.link_sources(&settings.link);
    worktree.spawn_commands(&settings.commands);

    if settings.tmux.create_session {
        worktree.spawn_tmux_session(&settings.tmux.additional_windows);
    }
}
