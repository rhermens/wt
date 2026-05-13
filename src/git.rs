use std::path::PathBuf;

use git2::Repository;
use thiserror::Error;

pub struct WorktreeCheckoutAction {
    pub common_wd: PathBuf,
    pub worktree_wd: PathBuf,
}

#[derive(Debug, Error)]
pub enum WorktreeError {
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

impl WorktreeCheckoutAction {
    pub fn try_from_checkout() -> Result<Self, WorktreeError> {
        let repo = Repository::open_from_env()
            .map_err(|e| WorktreeError::FailedToOpenRepository { source: e })?;

        if !repo.is_worktree() {
            return Err(WorktreeError::NotAWorktree);
        }

        match (repo.commondir().parent(), repo.workdir()) {
            (Some(commondir_wd), Some(worktree_wd)) => Ok(Self {
                common_wd: commondir_wd.to_path_buf(),
                worktree_wd: worktree_wd.to_path_buf(),
            }),
            (_, _) => return Err(WorktreeError::InvalidWorkingDirectories),
        }
    }
}
