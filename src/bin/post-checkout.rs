use std::process::Command;

use clap::Parser;
use git_worktree_utils::{git::WorktreeCheckoutAction, settings};
use log::{error, info};
use symlink_rs::symlink_auto;

#[derive(Parser, Debug)]
struct Args {
    pub previous_head: String,
    pub new_head: String,
    pub is_branch: String,
}

fn main() {
    settings::init_log();
    let worktree = match WorktreeCheckoutAction::try_from_checkout() {
        Ok(w) => w,
        Err(e) => {
            error!("Failed to open repository: {}", e);
            return;
        }
    };

    let settings = match settings::load_settings(&worktree.common_wd) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to load settings: {}", e);
            return;
        }
    };

    if let Some(copies) = settings.copy {
        for file in copies {
            match std::fs::copy(
                worktree.common_wd.join(&file),
                worktree.worktree_wd.join(&file),
            ) {
                Err(e) => error!("Error copying {}: {}", &file, e),
                Ok(_) => info!("Copied {}", &file),
            }
        }
    }

    if let Some(links) = settings.link {
        for file in links {
            match symlink_auto(
                worktree.common_wd.join(&file),
                worktree.worktree_wd.join(&file),
            ) {
                Err(e) => error!("Error linking {}: {}", &file, e),
                Ok(_) => info!("Linked {}", &file),
            }
        }
    }

    if let Some(commands) = settings.commands {
        for command in commands {
            match Command::new("sh").arg("-c").arg(&command).output() {
                Ok(output) => {
                    error!("{}", String::from_utf8_lossy(&output.stderr));
                    info!("{}", String::from_utf8_lossy(&output.stdout));
                }
                Err(e) => error!("Error executing command {}", e),
            }
        }
    }
}
