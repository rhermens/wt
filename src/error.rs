use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid git state: {source}")]
    GitError {
        #[source]
        source: git2::Error,
    },

    #[error("Settings error: {source}")]
    InvalidSettings {
        #[source]
        source: config::ConfigError,
    },

    #[error("Missing substitution values")]
    MissingSubstitutions,

    #[error("IO error for '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid path: '{0}' has no usable file name")]
    InvalidPath(PathBuf),

    #[error("Tmux error: {0}")]
    TmuxError(String),
}
