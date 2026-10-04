// lib.rs
//
// This module contains metadata and environment variable-related logic.
// Shared during execution of `ignition/build.rs` and dependent `build.rs` scripts.
//

pub mod asset;
pub mod config;
pub mod consts;
pub mod error;
pub mod target;

use std::{
    collections::HashMap,
    env::{set_var, var},
};

use asset::*;
use config::*;
use consts::*;
use error::*;
use ignition_macros::register_assets;

// fn asset_retrieve_2<T: Retrievable>(_: &T, config: &Config) -> IgnitionResult<String> {
//     let asset_path_key = format!(
//         "DEP_IGNITION_SYS_{}_PATH",
//         <T as Retrievable>::KEY.to_uppercase()
//     );
//     let target: Target = (&config.target).try_into()?;
//     let target_parts = target_parts_aliased(&target);
//     let mut output = Command::new(ASSET_SCRIPT_PATH)
//         .args([
//             config.bucket_url.clone(),
//             <T as Retrievable>::KEY.to_string(),
//             config.build_dir.clone(),
//             config.cache_dir.clone(),
//             config.asset_dir.clone(),
//             config.target.clone(),
//             asset_path_key,
//         ])
//         .spawn()
//         .expect("asset.sh command failed to start");
//     let _ = output.wait().expect("asset.sh command failed to complete");
//     // TODO: parse output for asset path, or print straight to cargo metadata from script?
//     Ok(String::new())
// }

/// Either retrieve (`config` not provided) or set (`config` provided) environment variables for a particular asset.
///
/// First argument (always provided) as `Asset` tuple struct implementing trait `Extractable`.
///
/// For an asset exporting two environment variables, the return will always be a HashMap of
/// environment variables and corresponding absolute paths:
/// ```rust
/// HashMap<String, String>
/// {
///     "ENV_VAR1": "absolute/path/to/content1",
///     "ENV_VAR2": "absolute/path/to/content2"
/// }
/// ```
///
/// Retrieving and setting are blind (not validated), so possible to overwrite or return empty strings.
///
pub fn environment_variables<T: Extractable>(
    _: &T,
    config: Option<&Config>,
) -> IgnitionResult<HashMap<String, String>> {
    // Environment variables to retrieve or set
    let mut env_vars = HashMap::new();

    // If config provided: check that all asset contents are present
    if let Some(cfg) = config {
        for cont in T::CONTENTS.iter() {
            let cont_path = cfg.asset_path().join(cont);
            if !cont_path.exists() {
                println!(
                    "cargo::error=asset {} missing content at {}",
                    T::KEY,
                    cont_path.to_string_lossy()
                );
            }
        }
    }

    // Either set (config provided) or retrieve (config not provided) env vars
    for (env_var, cont) in T::ENVIRONMENT.iter() {
        // (1) directory provided, so export <ENV_VAR> as cargo metadata for use in other crates
        // NOTE: also checks that content exists (may be distinct from T::CONTENTS)
        if let Some(cfg) = config {
            let cont_path = cfg.asset_path().join(cont);
            if !cont_path.exists() {
                println!(
                    "cargo::error=asset {} missing content for {} at {}",
                    T::KEY,
                    env_var,
                    cont_path.to_string_lossy()
                );
            } else {
                let cont_path_str = cont_path
                    .to_str()
                    .ok_or(IgnitionError::PathError(cont_path.clone()))?;
                println!("cargo::metadata={}={}", env_var, cont_path_str);
                env_vars.insert(env_var.to_string(), cont_path_str.to_string());
            }

        // (2) directory not provided, so retrieve DEP_IGNITION_SYS_<ENV_VAR> and set <ENV_VAR>
        } else {
            let env_var_value = var(ENV_METADATA_KEY_PREFIX.to_string() + env_var)?;
            unsafe {
                set_var(env_var, &env_var_value);
            }
            env_vars.insert(env_var.to_string(), env_var_value);
        }
    }

    Ok(env_vars)
}

// Asset types generated from the registry
register_assets!("config/registry.yaml");
