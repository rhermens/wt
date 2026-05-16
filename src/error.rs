use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
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

    #[error("Settings error: {source}")]
    InvalidSettings {
        #[source]
        source: config::ConfigError,
    },

    #[error("Missing substitution values")]
    MissingSubstitutions,
}
