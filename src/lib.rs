// lib.rs
//
// This module contains metadata and environment variable-related logic.
// This is shared during execution of `ignition/build.rs` and dependent `build.rs` scripts.
//

use std::{collections::HashMap, env::{VarError, set_var, var}, ops::Deref, path::{Path, PathBuf}};

use thiserror::Error;

// Defaults and constants.
const DEFAULT_CACHE_DIR: &str = "cache";
const DEFAULT_ASSET_DIR: &str = "assets/dependencies";
pub const METADATA_KEY_PREFIX: &str = "DEP_IGNITION_SYS_";
pub const ASSET_SCRIPT_PATH: &str = "scripts/asset.sh";

/// Configuration for the build system.
#[derive(Debug)]
pub struct Config {
    // Root url of the bucket containing the assets
    pub bucket_url: String,
    // Build directory of the project (derived from OUT_DIR environment variable)
    pub build_dir: String,
    // Cache directory for the assets (configured or defaulting to DEFAULT_CACHE_DIR)
    pub cache_dir: String,
    // Asset directory for the assets (configured or defaulting to DEFAULT_ASSET_DIR)
    pub asset_dir: String,
    // Target for the project (default to system if not set)
    pub target: String,
}

impl Config {
    pub fn new() -> IgnitionResult<Self> {
        let bucket_url = var("IGNITION_BUCKET_URL")?;
        // require format: /../target/<target-triplet>/<build-type>/build/<ignition-build-id>/out
        let out_dir = var("OUT_DIR")?;
        let build_dir = out_dir
            .split(&"/build".to_string())
            .next()
            .ok_or(IgnitionError::BuildDirectoryError(out_dir.to_string()))?
            .to_string();
        // log these whether or not defaults used
        let cache_dir = var("IGNITION_CACHE_DIR").unwrap_or(DEFAULT_CACHE_DIR.to_string());
        println!("cache directory set to `{}`", &cache_dir);
        let asset_dir = var("IGNITION_ASSET_DIR").unwrap_or(DEFAULT_ASSET_DIR.to_string());
        println!("asset directory set to `{}`", &asset_dir);
        // optional
        let target = var("TARGET").unwrap_or(
            {
                println!("cargo::warning=target not set, using system target");
                "".to_string()
            }
        );
        Ok(
            Self {
                bucket_url,
                build_dir,
                cache_dir,
                asset_dir,
                target,
            }
        )
    }

    fn asset_path(&self) -> PathBuf {
        Path::new(self.build_dir.as_str()).join(self.asset_dir.as_str())
    }
}

/// Asset base struct and traits.
pub struct Asset { }

impl Asset {
    pub fn new() -> Self {
        Self { }
    }
}

pub trait Retrievable: Deref<Target = Asset> {
    const KEY: &'static str;
}

pub trait Extractable<const N: usize, const M: usize>: Retrievable {
    const CONTENTS: [&str; N];
    const ENVIRONMENT: [(&str, &str); M];
}

pub trait Targetable: Retrievable {
    fn target_exclusions(&self) -> Option<Vec<String>>;
    fn check_target_excluded(&self, config: &Config) -> bool;
}

impl <T: Retrievable>Targetable for T {

    fn target_exclusions(&self) -> Option<Vec<String>> {
        let env_var_str: String = format!("IGNITION_TARGET_EXCLUSIONS_{}", T::KEY.to_uppercase());
        if let Some(env_var_value) = var(env_var_str).ok() {
            let target_exclusions: Vec<String> = env_var_value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            println!("asset {} target exclusions: {:?}", T::KEY, target_exclusions);
            Some(target_exclusions)
        } else {
            println!("asset {} target exclusions not set", T::KEY);
            None
        }
    }

    fn check_target_excluded(&self, config: &Config) -> bool {
        let target: &str = config.target.as_str();
        if let Some(target_exclusions) = self.target_exclusions() {
            let is_excluded = target_exclusions.contains(&target.to_string());
            if is_excluded {
                println!("cargo::warning=asset {} excludes target {}", T::KEY, target);
            }
            is_excluded
        } else {
            false
        }
    }
}

/// Asset: OpenCv
pub struct OpenCv(Asset);

impl OpenCv {
    pub fn new() -> Self {
        Self(Asset::new())
    }
}

impl Deref for OpenCv {
    type Target = Asset;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Retrievable for OpenCv {
    const KEY: &'static str = "opencv";
}

impl Extractable<2, 2> for OpenCv {
    const CONTENTS: [&str; 2] = [
        "opencv/lib",
        "opencv/opencv4",
    ];
    const ENVIRONMENT: [(&str, &str); 2] = [
        ("OPENCV_LINK_PATHS", "opencv/lib"),
        ("OPENCV_INCLUDE_PATHS", "opencv/opencv4"),
    ];
}

/// Asset: ONNXRuntime
pub struct ONNXRuntime(Asset);

impl ONNXRuntime {
    pub fn new() -> Self {
        Self(Asset::new())
    }
}

impl Deref for ONNXRuntime {
    type Target = Asset;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Retrievable for ONNXRuntime {
    const KEY: &'static str = "onnxruntime";
}

impl Extractable<1, 1> for ONNXRuntime {
    const CONTENTS: [&str; 1] = [
        "onnxruntime"
    ];
    const ENVIRONMENT: [(&str, &str); 1] = [
        ("ORT_LIB_LOCATION", "onnxruntime")
    ];
}

/// Error types.
#[derive(Error, Clone, Debug)]
pub enum IgnitionError {

    /// Out directory error.
    #[error("OUT_DIR environment variable does not contain `/../build`: {0}")]
    BuildDirectoryError(String),
    /// Environment variable error.
    #[error("environment variable error: {0}")]
    EnvironmentVariableError(#[from] VarError),
    /// Path error.
    #[error("path error: {0}")]
    PathError(PathBuf),
}

/// Result type for Ignition functions.
pub type IgnitionResult<T, E = IgnitionError> = std::result::Result<T, E>;

/// Either retrieve (`config` not provided) or set (`config` provided) environment variables for a particular asset.
///
/// First argument (always provided) as `Asset` tuple struct implementing trait `Extractable`.
/// Example contents and environment:
/// ```rust
/// const CONTENTS: [&str; 3] = [
///     "path/to/content1",
///     "path/to/content2",
///     "path/to/content3"
/// ];
/// const ENVIRONMENT: [(&str, &str); 2] = [
///     ("ENV_VAR1", "path/to/content1"),
///     ("ENV_VAR2", "path/to/content2"),
/// ];
/// ```
///
/// From example above, the return will always be a HashMap of environment variables and corresponding absolute paths:
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
/// NOTE: T::CONTENTS may be disjoint from T::ENVIRONMENT. When Config is provided, path existence for each checked independently.
pub fn environment_variables<T: Extractable<N, M>, const N: usize, const M: usize>(
    _: &T,
    config: Option<&Config>,
) -> IgnitionResult<HashMap<String, String>> {

    // Environment variables to retrieve or set
    let mut env_vars = HashMap::new();

    // If config provided: check that all contents are present
    if let Some(cfg) = config {
        for cont in T::CONTENTS.iter() {
            let cont_path = cfg.asset_path().join(cont);
            if !cont_path.exists() {
                println!("cargo::error=asset {} missing content at {}", T::KEY, cont_path.to_string_lossy());
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
                println!("cargo::error=asset {} missing content for {} at {}", T::KEY, env_var, cont_path.to_string_lossy());
            }
            else {
                let cont_path_str = cont_path.to_str().ok_or(IgnitionError::PathError(cont_path.clone()))?;
                println!("cargo::metadata={}={}", env_var, cont_path_str);
                env_vars.insert(env_var.to_string(), cont_path_str.to_string());
            }

        // (2) directory not provided, so retrieve DEP_IGNITION_SYS_<ENV_VAR> and set <ENV_VAR>
        } else {
            let env_var_value = var(METADATA_KEY_PREFIX.to_string() + env_var)?;
            unsafe {
                set_var(env_var, &env_var_value);
            }
            env_vars.insert(env_var.to_string(), env_var_value);
        }
    }

    Ok(env_vars)
}
