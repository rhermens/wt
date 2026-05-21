use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use git2::{ErrorCode, Repository, WorktreeAddOptions};
use log::{info, warn};
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
        let cwd = std::env::current_dir().map_err(|e| Error::IoError {
            path: PathBuf::from("."),
            source: e,
        })?;

        let repo = Repository::discover(cwd).map_err(|e| Error::GitError { source: e })?;

        let main_path = repo
            .workdir()
            .ok_or_else(|| Error::InvalidPath(repo.path().to_path_buf()))?
            .to_path_buf();

        let worktree_path = Self::open_worktree(&repo, path)?;
        let settings = Settings::new(&main_path)?.substitute(substitutions)?;

        Ok(Self {
            main_path,
            worktree_path,
            settings,
        })
    }

    fn ensure_parent_directory(target_path: &Path) -> Result<(), Error> {
        let parent = target_path
            .parent()
            .ok_or_else(|| Error::InvalidPath(target_path.to_path_buf()))?;

        if std::fs::exists(parent).map_err(|e| Error::IoError {
            path: parent.to_path_buf(),
            source: e,
        })? {
            return Ok(());
        }

        std::fs::create_dir_all(parent).map_err(|e| Error::IoError {
            path: parent.to_path_buf(),
            source: e,
        })
    }

    fn open_worktree(repo: &Repository, path: &Path) -> Result<PathBuf, Error> {
        let name = path
            .file_name()
            .ok_or_else(|| Error::InvalidPath(path.to_path_buf()))?
            .to_string_lossy()
            .into_owned();

        let already_exists = fs::exists(path).map_err(|e| Error::IoError {
            path: path.to_path_buf(),
            source: e,
        })?;

        Self::ensure_parent_directory(&path)?;
        match repo.worktree(
            &name,
            path,
            Some(WorktreeAddOptions::new().checkout_existing(!already_exists)),
        ) {
            Ok(wt) => Ok(wt.path().to_path_buf()),
            Err(e) => match e.code() {
                ErrorCode::Exists => {
                    warn!("Worktree exists: {}", e);
                    Ok(path.to_path_buf())
                }
                _ => Err(Error::GitError { source: e }),
            },
        }
    }

    pub fn copy_sources(&self) -> Vec<Error> {
        self.settings
            .copy
            .iter()
            .filter_map(|file| {
                let src = self.main_path.join(file);
                let dst = self.worktree_path.join(file);
                match std::fs::copy(&src, &dst) {
                    Ok(_) => {
                        info!("Copied {}", file);
                        None
                    }
                    Err(e) => Some(Error::CopyError {
                        path: src,
                        source: e,
                    }),
                }
            })
            .collect()
    }

    pub fn link_sources(&self) -> Vec<Error> {
        self.settings
            .link
            .iter()
            .filter_map(|file| {
                let src = self.main_path.join(file);
                let dst = self.worktree_path.join(file);
                match symlink_rs::symlink_auto(&src, &dst) {
                    Ok(_) => {
                        info!("Linked {}", file);
                        None
                    }
                    Err(e) => Some(Error::LinkError {
                        path: src,
                        source: e,
                    }),
                }
            })
            .collect()
    }

    pub fn spawn_commands(&self) -> Vec<Error> {
        self.settings
            .commands
            .iter()
            .filter_map(|cmd| {
                let spawn_result = Command::new("sh")
                    .args(["-c", cmd])
                    .current_dir(&self.worktree_path)
                    .spawn()
                    .map_err(|e| Error::CommandError { source: e });

                match spawn_result {
                    Err(e) => Some(e),
                    Ok(mut child) => match child.wait() {
                        Err(e) => Some(Error::CommandError { source: e }),
                        Ok(status) if !status.success() => {
                            warn!("Command exited with {}: {}", status, cmd);
                            None
                        }
                        Ok(_) => None,
                    },
                }
            })
            .collect()
    }

    pub fn spawn_tmux_session(&self) -> Vec<Error> {
        if !self.settings.tmux.create_session {
            return vec![];
        }

        let worktree_name = match self.worktree_path.file_name() {
            Some(n) => n.to_string_lossy().into_owned(),
            None => return vec![Error::InvalidPath(self.worktree_path.clone())],
        };

        let main_name = match self.main_path.file_name() {
            Some(n) => n.to_string_lossy().into_owned(),
            None => return vec![Error::InvalidPath(self.main_path.clone())],
        };

        let session_name = format!("{} ({})", worktree_name, main_name);
        let start_dir = self.worktree_path.display().to_string();

        let mut commands = TmuxCommands::new();
        commands.push(
            NewSession::new()
                .session_name(&session_name)
                .detached()
                .start_directory(&start_dir),
        );
        for command in &self.settings.tmux.additional_windows {
            commands.push(
                NewWindow::new()
                    .target_window(&session_name)
                    .start_directory(&start_dir)
                    .shell_command(format!("{}; exec $SHELL", command)),
            );
        }

        match Tmux::with_commands(commands).status() {
            Err(e) => vec![Error::TmuxError(e.to_string())],
            Ok(status) if status.success() => {
                info!("Tmux session spawned");
                vec![]
            }
            Ok(status) => {
                warn!("Tmux exited with {}", status);
                vec![]
            }
        }
    }
}
