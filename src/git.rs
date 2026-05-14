use std::{path::PathBuf, process::Command};

use git2::{ErrorCode, Repository};
use log::{error, info};
use thiserror::Error;
use tmux_interface::{NewSession, NewWindow, Tmux, TmuxCommands};

#[derive(Debug)]
pub struct WorktreeContext {
    pub main_path: PathBuf,
    pub worktree_path: PathBuf,
}

#[derive(Debug, Error)]
pub enum WorktreeError {
    #[error("Invalid git state: {source}")]
    GitError {
        #[source]
        source: git2::Error,
    },

    #[error("Not a worktree")]
    NotAWorktree,

    #[error("Failed to open checkout: {source}")]
    FailedToOpenRepository {
        #[source]
        source: git2::Error,
    },

    #[error("Failed to read working directories")]
    InvalidWorkingDirectories,
}

impl WorktreeContext {
    pub fn try_from_hook() -> Result<Self, WorktreeError> {
        let repo = Repository::open_from_env()
            .map_err(|e| WorktreeError::FailedToOpenRepository { source: e })?;

        if !repo.is_worktree() {
            return Err(WorktreeError::NotAWorktree);
        }

        match (repo.commondir().parent(), repo.workdir()) {
            (Some(commondir_wd), Some(worktree_wd)) => Ok(Self {
                main_path: commondir_wd.to_path_buf(),
                worktree_path: worktree_wd.to_path_buf(),
            }),
            (_, _) => return Err(WorktreeError::InvalidWorkingDirectories),
        }
    }

    pub fn try_create_or_open(path: &PathBuf) -> Result<Self, WorktreeError> {
        let repo = Repository::discover(std::env::current_dir().expect("Failed to get CWD"))
            .map_err(|e| WorktreeError::GitError { source: e })?;

        let wt_path = match repo.worktree(
            &path
                .file_name()
                .expect("Invalid path")
                .display()
                .to_string(),
            &path,
            None,
        ) {
            Ok(wt) => wt.path().to_path_buf(),
            Err(e) => match e.code() {
                ErrorCode::Exists => path.clone(),
                _ => return Err(WorktreeError::GitError { source: e }),
            },
        };

        Ok(Self {
            main_path: repo
                .path()
                .parent()
                .expect("Invalid repo path")
                .to_path_buf(),
            worktree_path: wt_path,
        })
    }

    pub fn copy_sources(&self, sources: &Vec<String>) {
        for file in sources {
            match std::fs::copy(self.main_path.join(&file), self.worktree_path.join(&file)) {
                Err(e) => error!("Error copying {}: {}", &file, e),
                Ok(_) => info!("Copied {}", &file),
            }
        }
    }

    pub fn link_sources(&self, sources: &Vec<String>) {
        for file in sources {
            match symlink_rs::symlink_auto(
                self.main_path.join(&file),
                self.worktree_path.join(&file),
            ) {
                Err(e) => error!("Error linking {}: {}", &file, e),
                Ok(_) => info!("Linked {}", &file),
            }
        }
    }

    pub fn spawn_commands(&self, commands: &Vec<String>) {
        let procs = commands
            .into_iter()
            .map(|cmd| {
                Command::new("sh")
                    .args(&["-c", &cmd])
                    .current_dir(&self.worktree_path)
                    .spawn()
                    .expect("Failed to spawn process")
            })
            .collect::<Vec<_>>();

        for mut proc in procs {
            proc.wait().expect("Failed to wait for status");
        }
    }

    pub fn spawn_tmux_session(&self, windows: &Vec<String>) {
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
        for command in windows {
            let c = NewWindow::new()
                .target_window(&session_name)
                .start_directory(self.worktree_path.display().to_string())
                .shell_command(format!("{}; exec $SHELL", command));
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
