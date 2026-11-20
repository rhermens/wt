use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use git2::{ErrorCode, Repository, WorktreeAddOptions};
use log::{info, warn};

use crate::{error::Error, settings::Settings};

#[derive(Debug)]
pub struct WorktreeContext {
    pub main_path: PathBuf,
    pub worktree_path: PathBuf,
    pub settings: Settings,
}

#[derive(Debug)]
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
    pub fn try_new(cwd: &Path, path: &Path, args: &[String]) -> Result<Self, Error> {
        let repo = Repository::discover(cwd).map_err(|e| Error::GitError { source: e })?;

        let main_path = repo
            .workdir()
            .ok_or_else(|| Error::InvalidPath(repo.path().to_path_buf()))?
            .to_path_buf();

        let settings = Settings::new(&main_path, &path, args)?;
        let target_path = WorktreeTargetPath::try_new(
            &settings
                .worktrees_directory
                .as_deref()
                .or_else(|| repo.workdir())
                .expect("failed to read path")
                .join(path),
        )?;
        target_path.ensure()?;
        let worktree_path = Self::open_worktree(&repo, &target_path)?;

        Ok(Self {
            main_path,
            worktree_path,
            settings,
        })
    }

    fn open_worktree(
        repo: &Repository,
        worktree_path: &WorktreeTargetPath,
    ) -> Result<PathBuf, Error> {
        match repo.worktree(
            &worktree_path.basename,
            &worktree_path.path,
            Some(WorktreeAddOptions::new().checkout_existing(!worktree_path.is_existing)),
        ) {
            Ok(wt) => Ok(wt.path().to_path_buf()),
            Err(e) => match (e.code(), e.raw_class()) {
                (ErrorCode::Exists, _) => {
                    warn!("Worktree exists: {}", e);
                    Ok(worktree_path.path.to_path_buf())
                }
                (ErrorCode::GenericError, 32) => {
                    warn!("Worktree exists: {}", e);
                    Ok(worktree_path.path.to_path_buf())
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
}
