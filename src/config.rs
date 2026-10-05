// Configuration for build.

use std::{
    env::var,
    path::{Path, PathBuf},
};

use crate::consts::*;
use crate::error::*;

/// Configuration for the build system.
#[derive(Debug)]
pub struct Config {
    // Root url of the bucket containing the assets
    pub bucket_url: String,
    // Build directory of the project (derived from OUT_DIR environment variable)
    pub build_dir: String,
    // Asset directory for the assets (configured or defaulting to DEFAULT_ASSET_DIR)
    pub asset_dir: String,
    // Cache directory for the assets (configured or defaulting to DEFAULT_CACHE_DIR)
    pub cache_dir: String,
    // Target triplet (defaults to system if not set)
    pub target: TargetTriplet,
}

impl Config {
    pub fn new() -> IgnitionResult<Self> {
        // Asset source (root) url
        let bucket_url = var(ENV_BUCKET_URL)?;
        // Asset destination (root) path
        // NOTE: assume format `/../target/<target-triplet>/<build-type>/build/<ignition-build-id>/out`
        let out_dir = var(ENV_OUT_DIR)?;
        let build_dir = out_dir
            .split(&"/build".to_string())
            .next()
            .ok_or(IgnitionError::BuildDirectoryError(out_dir.to_string()))?
            .to_string();
        // Configurable directories (with defaults)
        let asset_dir = var(ENV_ASSET_DIR).unwrap_or(DEFAULT_ASSET_DIR.to_string());
        let cache_dir = var(ENV_CACHE_DIR).unwrap_or(DEFAULT_CACHE_DIR.to_string());
        // User-specified (or system default)
        let target = var(ENV_TARGET)?;
        Ok(Self {
            bucket_url,
            build_dir,
            cache_dir,
            asset_dir,
            target,
        })
    }

    pub fn asset_path(&self) -> PathBuf {
        Path::new(self.build_dir.as_str()).join(self.asset_dir.as_str())
    }
}
