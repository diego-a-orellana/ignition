// Errors and return types.

use std::{env::VarError, path::PathBuf};

use thiserror::Error;

/// Error types.
#[derive(Error, Clone, Debug)]
pub enum IgnitionError {
    /// Asset archive extraction error.
    #[error("failed to extract archive {0}: {1}")]
    ArchiveError(PathBuf, String),
    /// Out directory error.
    #[error("OUT_DIR environment variable does not contain `/../build`: {0}")]
    BuildDirectoryError(String),
    /// Asset archive retrieved incompletely.
    #[error("incomplete download of {0}: expected {1} bytes, received {2}")]
    DownloadError(String, u64, u64),
    /// Environment variable error.
    #[error("environment variable error: {0}")]
    EnvironmentVariableError(#[from] VarError),
    /// Filesystem error.
    #[error("filesystem error at {0}: {1}")]
    FileSystemError(PathBuf, String),
    /// Path error.
    #[error("path error: {0}")]
    PathError(PathBuf),
    /// Remote asset request error.
    #[error("request error for {0}: {1}")]
    RequestError(String, String),
}

/// Result type for Ignition functions.
pub type IgnitionResult<T, E = IgnitionError> = std::result::Result<T, E>;
