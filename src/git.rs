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

struct WorktreeTargetPath {
    path: PathBuf,
    basename: String,
    parent_directory: PathBuf,
    is_existing: bool,
}

impl WorktreeTargetPath {
    fn try_new(target: &Path) -> Result<Self, Error> {
        let basename = target
            .file_name()
            .ok_or_else(|| Error::InvalidPath(target.to_path_buf()))?
            .to_string_lossy()
            .into_owned();

        let is_existing = fs::exists(target).map_err(|e| Error::IoError {
            path: target.to_path_buf(),
            source: e,
        })?;

        let parent_directory = target
            .parent()
            .ok_or_else(|| Error::InvalidPath(target.to_path_buf()))?
            .to_path_buf();

        Ok(Self {
            path: target.to_path_buf(),
            basename,
            parent_directory,
            is_existing,
        })
    }

    fn ensure(&self) -> Result<(), Error> {
        if std::fs::exists(&self.parent_directory).map_err(|e| Error::IoError {
            path: self.parent_directory.clone(),
            source: e,
        })? {
            return Ok(());
        }

        std::fs::create_dir_all(&self.parent_directory).map_err(|e| Error::IoError {
            path: self.parent_directory.clone(),
            source: e,
        })
    }
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
        let settings = Settings::new(&main_path, &worktree_path, substitutions)?;

        Ok(Self {
            main_path,
            worktree_path,
            settings,
        })
    }

    fn open_worktree(repo: &Repository, path: &Path) -> Result<PathBuf, Error> {
        let worktree_path = WorktreeTargetPath::try_new(path)?;
        worktree_path.ensure()?;

        match repo.worktree(
            &worktree_path.basename,
            &worktree_path.path,
            Some(WorktreeAddOptions::new().checkout_existing(!worktree_path.is_existing)),
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
            .flat_map(|op| op.into_pathbufs(&self.main_path))
            .filter_map(|src| {
                let dst = self.worktree_path.join(&src);

                match std::fs::copy(&src, &dst) {
                    Ok(_) => {
                        info!("Copied {}", src.to_string_lossy());
                        None
                    }
                    Err(e) => Some(Error::CopyError {
                        path: src,
                        dest: dst,
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
            .flat_map(|op| op.into_pathbufs(&self.main_path))
            .filter_map(|src| {
                let dst = self.worktree_path.join(&src);

                match symlink_rs::symlink_auto(&src, &dst) {
                    Ok(_) => {
                        info!("Linked {}", src.to_string_lossy());
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
