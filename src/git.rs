use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use git2::{ErrorCode, Repository, WorktreeAddOptions};
use log::{error, info};
use tmux_interface::{NewSession, NewWindow, Tmux, TmuxCommands};

use crate::{error::Error, settings::Settings};

#[derive(Debug)]
pub struct WorktreeContext {
    main_path: PathBuf,
    worktree_path: PathBuf,
    settings: Settings,
}

impl WorktreeContext {
    pub fn try_new(path: &Path, substitutions: &[String]) -> Result<Self, Error> {
        let repo = Repository::discover(std::env::current_dir().expect("Failed to get CWD"))
            .map_err(|e| Error::GitError { source: e })?;

        let main_path = repo
            .path()
            .parent()
            .expect("Invalid repo path")
            .to_path_buf();
        let worktree_path = Self::open_worktree(&repo, path)?;
        let settings = Settings::new(&main_path)?.substitute(substitutions)?;

        Ok(Self {
            main_path,
            worktree_path,
            settings,
        })
    }

    fn open_worktree(repo: &Repository, path: &Path) -> Result<PathBuf, Error> {
        match repo.worktree(
            &path
                .file_name()
                .expect("Invalid path")
                .display()
                .to_string(),
            &path,
            Some(
                &WorktreeAddOptions::new()
                    .checkout_existing(!fs::exists(path).expect("Failed to stat path")),
            ),
        ) {
            Ok(wt) => Ok(wt.path().to_path_buf()),
            Err(e) => match e.code() {
                ErrorCode::Exists => Ok(path.to_path_buf()),
                _ => {
                    return Err(Error::GitError { source: e });
                }
            },
        }
    }

    pub fn copy_sources(&self) {
        for file in &self.settings.copy {
            match std::fs::copy(self.main_path.join(&file), self.worktree_path.join(&file)) {
                Err(e) => error!("Error copying {}: {}", &file, e),
                Ok(_) => info!("Copied {}", &file),
            }
        }
    }

    pub fn link_sources(&self) {
        for file in &self.settings.link {
            match symlink_rs::symlink_auto(
                self.main_path.join(&file),
                self.worktree_path.join(&file),
            ) {
                Err(e) => error!("Error linking {}: {}", &file, e),
                Ok(_) => info!("Linked {}", &file),
            }
        }
    }

    pub fn spawn_commands(&self) {
        let procs = self
            .settings
            .commands
            .iter()
            .map(|cmd| {
                Command::new("sh")
                    .args(&["-c", &cmd])
                    .current_dir(&self.worktree_path)
                    .spawn()
                    .expect(&format!(
                        "Failed to spawn process in {}",
                        &self.worktree_path.display().to_string()
                    ))
            })
            .collect::<Vec<_>>();

        for mut proc in procs {
            proc.wait().expect("Failed to wait for status");
        }
    }

    pub fn spawn_tmux_session(&self) {
        if !self.settings.tmux.create_session {
            return;
        }

        let session_name = format!(
            "{} ({})",
            self.worktree_path
                .file_name()
                .expect("Failed to read worktree basename")
                .display()
                .to_string(),
            self.main_path
                .file_name()
                .expect("Failed to read main basename")
                .display()
                .to_string()
        );

        let mut commands = TmuxCommands::new();
        commands.push(
            NewSession::new()
                .session_name(&session_name)
                .detached()
                .start_directory(self.worktree_path.display().to_string()),
        );
        for command in &self.settings.tmux.additional_windows {
            let c = NewWindow::new()
                .target_window(&session_name)
                .start_directory(self.worktree_path.display().to_string())
                .shell_command(format!("{}; exec $SHELL", &command));
            commands.push(c);
        }

        match Tmux::with_commands(commands).status() {
            Ok(status) => match status.code() {
                Some(0) => info!("Tmux session spawned"),
                _ => error!("Error spawning tmux: {}", status),
            },
            Err(e) => error!("Error tmux command {}", e),
        }
    }
}
