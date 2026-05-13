use clap::Parser;
use git2::Repository;
use log::{error, info};
use symlink_rs::symlink_auto;
use wtutils::load_settings;

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

    let repo = match Repository::open_from_env() {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to open repository: {}", e);
            return;
        }
    };

    if !repo.is_worktree() {
        info!("Not a worktree");
        return;
    }

    let (common_wd, worktree_wd) = match (repo.commondir().parent(), repo.workdir()) {
        (Some(commondir_wd), Some(worktree_wd)) => (commondir_wd, worktree_wd),
        (_, _) => {
            error!("Failed to read working directories");
            return;
        }
    };

    for file in settings.copy {
        match std::fs::copy(common_wd.join(&file), worktree_wd.join(&file)) {
            Err(e) => error!("Error copying {}: {}", &file, e),
            Ok(_) => info!("Copied {}", &file),
        }
    }

    for file in settings.link {
        match symlink_auto(common_wd.join(&file), worktree_wd.join(&file)) {
            Err(e) => error!("Error linking {}: {}", &file, e),
            Ok(_) => info!("Linked {}", &file),
        }
    }
}
