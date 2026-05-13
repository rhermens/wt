use clap::Parser;
use git2::Repository;
use log::{error, info};
use symlink_rs::symlink_auto;
use wtutils::{git::WorktreeCheckoutAction, settings::load_settings};

#[derive(Parser, Debug)]
struct Args {
    pub previous_head: String,
    pub new_head: String,
    pub is_branch: String,
}

fn main() {
    let settings = match load_settings() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to load settings: {}", e);
            return;
        }
    };

    let worktree = match WorktreeCheckoutAction::try_from_checkout() {
        Ok(w) => w,
        Err(e) => {
            error!("Failed to open repository: {}", e);
            return;
        }
    };

    for file in settings.copy {
        match std::fs::copy(
            worktree.common_wd.join(&file),
            worktree.worktree_wd.join(&file),
        ) {
            Err(e) => error!("Error copying {}: {}", &file, e),
            Ok(_) => info!("Copied {}", &file),
        }
    }

    for file in settings.link {
        match symlink_auto(
            worktree.common_wd.join(&file),
            worktree.worktree_wd.join(&file),
        ) {
            Err(e) => error!("Error linking {}: {}", &file, e),
            Ok(_) => info!("Linked {}", &file),
        }
    }
}
