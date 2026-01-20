// lib.rs
//
// This module contains metadata and environment variable-related logic.
// This is shared during execution of `ignition/build.rs` and dependent `build.rs` scripts.
//

use std::{collections::HashMap, path::Path};

use serde::Deserialize;
use serde_json::from_str;
use thiserror::Error;

// `config/environment.json` not available at runtime, so need to include contents as a string
const ENVIRONMENT_CONFIG: &str = include_str!("../config/environment.json");
const METADATA_KEY_PREFIX: &str = "DEP_IGNITION_SYS_";

/// Error type for Ignition functions.
#[derive(Error, Clone, Debug)]
pub enum IgnitionError {
    /// Bad hash mapping in configuration file.
    #[error("invalid hash key: {0}")]
    BadHashMapKeyError(String),
    /// Environment variable error.
    #[error("environment variable error: {0}")]
    EnvironmentVariableError(#[from] std::env::VarError),
    /// Any error arising from usage of serde_json for deserializing configuration string.
    #[error("failed to deserialize configuration string: {0}")]
    ConfigurationDeserializationError(String),
}

/// Convert serde_json::Error to IgnitionError.
impl From<serde_json::Error> for IgnitionError {
    fn from(err: serde_json::Error) -> Self {
        IgnitionError::ConfigurationDeserializationError(err.to_string())
    }
}

/// Environment configuration for a particular asset.
#[derive(Deserialize)]
pub struct AssetEnvironment {
    /// List of contents expected on extaction of asset archive.
    #[serde(default)]
    pub contents: Vec<String>,
    /// Mapping of contents to their corresponding environment variables.
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

/// Result type for Ignition functions.
pub type IgnitionResult<T, E = IgnitionError> = std::result::Result<T, E>;

/// Determine environment variables for a particular asset.
///
/// Assuming environment.json formatted as:
/// ```json
/// {
///     ...
///     "asset": {
///         "contents": [
///             "path/to/content1",
///             "path/to/content2"
///         ],
///         "environment": {
///             "path/to/content1": "ENV_VAR1",
///             "path/to/content2": "ENV_VAR2"
///         }
///     }
///     ...
/// }
/// ```
///
/// Return is :
/// ```rust
/// HashMap<String, String>
/// {
///     "ENV_VAR1": "absolute/path/to/content1",
///     "ENV_VAR2": "absolute/path/to/content2"
/// }
/// ```
///
/// The optional <directory_path> parameter determines if these environment variables are set OR retrieved.
/// In either case, the operation is blind -- set/get not validated, so possible to overwrite or return empty strings.
pub fn environment_variables(
    asset: &str,
    directory_path: Option<&Path>,
) -> IgnitionResult<HashMap<String, String>> {
    let env_cfg: HashMap<String, AssetEnvironment> = from_str(ENVIRONMENT_CONFIG)?;
    let asset_cfg = env_cfg
        .get(asset)
        .ok_or(IgnitionError::BadHashMapKeyError(asset.to_string()))?;
    let mut env_vars = HashMap::new();
    for cont in asset_cfg.contents.iter() {
        let env_var = asset_cfg
            .environment
            .get(cont)
            .ok_or(IgnitionError::BadHashMapKeyError(cont.to_string()))?;
        // directory provided, so export <ENV_VAR> as cargo metadata for use in other crates
        if let Some(directory_path) = directory_path {
            let cont_path = directory_path.join(cont);
            if cont_path.exists() {
                let cont_path_str = cont_path.to_string_lossy();
                println!("cargo::metadata={}={}", env_var, cont_path_str);
                env_vars.insert(env_var.to_string(), cont_path_str.to_string());
            }
        // directory not provided, so retrieve DEP_IGNITION_<ENV_VAR> and set <ENV_VAR>
        } else {
            let env_var_value = std::env::var(METADATA_KEY_PREFIX.to_string() + env_var)?;
            unsafe {
                std::env::set_var(env_var, &env_var_value);
            }
            env_vars.insert(env_var.to_string(), env_var_value);
        }
    }
    Ok(env_vars)
}
