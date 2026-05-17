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
}
