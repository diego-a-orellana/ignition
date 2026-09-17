// Errors and return types.

use std::{env::VarError, path::PathBuf};

use thiserror::Error;

/// Error types.
#[derive(Error, Clone, Debug)]
pub enum IgnitionError {
    /// Out directory error.
    #[error("OUT_DIR environment variable does not contain `/../build`: {0}")]
    BuildDirectoryError(String),
    /// Deserialization error.
    #[error("deserialization error: {0}")]
    DeserializationError(String),
    /// Environment variable error.
    #[error("environment variable error: {0}")]
    EnvironmentVariableError(#[from] VarError),
    /// Path error.
    #[error("path error: {0}")]
    PathError(PathBuf),
    /// Target alias key not found (e.g. 'architecture', 'vendor', 'operating_system', or 'environment')
    #[error("target alias key not found: {0}")]
    TargetAliasKeyNotFoundError(String),
    /// Target configuration (myaml string)
    #[error("target configuration key not found: {0}")]
    TargetConfigurationKeyNotFoundError(String),
}

impl From<serde_yaml::Error> for IgnitionError {
    fn from(err: serde_yaml::Error) -> Self {
        IgnitionError::DeserializationError(err.to_string())
    }
}

/// Result type for Ignition functions.
pub type IgnitionResult<T, E = IgnitionError> = std::result::Result<T, E>;
