// Centralized constants and supported types.

use std::collections::HashMap;

// Configuration files
// NOTE: `config/registry.yaml` is read at compile time by the `register_assets!` macro
pub(crate) const CFG_TARGET_STR: &str = include_str!("../config/target.yaml");
pub(crate) const CFG_TARGET_ALIAS_KEY: &str = "alias";

// Environment variables
pub(crate) const ENV_ASSET_DIR: &str = "IGNITION_ASSET_DIR";
pub(crate) const ENV_BUCKET_URL: &str = "IGNITION_BUCKET_URL";
pub(crate) const ENV_CACHE_DIR: &str = "IGNITION_CACHE_DIR";
pub(crate) const ENV_OUT_DIR: &str = "OUT_DIR";
pub(crate) const ENV_TARGET: &str = "TARGET";

// Environment variable prefixes (for export and additional configuration)
pub(crate) const ENV_METADATA_KEY_PREFIX: &str = "DEP_IGNITION_SYS_";

// Target triplet parts
pub(crate) const TARGET_ARCHITECTURE_KEY: &str = "architecture";
pub(crate) const TARGET_VENDOR_KEY: &str = "vendor";
pub(crate) const TARGET_OPERATING_SYSTEM_KEY: &str = "operating_system";
pub(crate) const TARGET_ENVIRONMENT_KEY: &str = "environment";

// Defaults
pub(crate) const DEFAULT_ASSET_DIR: &str = "assets/dependencies";
pub(crate) const DEFAULT_CACHE_DIR: &str = "cache";
pub(crate) const ASSET_SCRIPT_PATH: &str = "scripts/asset.sh";

// Supporting types
pub(crate) type TargetConfiguration = HashMap<String, HashMap<TargetTriplet, TargetTripletMap>>;
pub(crate) type TargetTriplet = String;
pub(crate) type TargetTripletMap = HashMap<String, Option<String>>;
