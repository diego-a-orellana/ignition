// Centralized constants and supported types.

// Configuration files
// NOTE: `config/registry.yaml` is read at compile time by the `register_assets!` macro

// Environment variables
pub(crate) const ENV_ASSET_DIR: &str = "IGNITION_ASSET_DIR";
pub(crate) const ENV_BUCKET_URL: &str = "IGNITION_BUCKET_URL";
pub(crate) const ENV_CACHE_DIR: &str = "IGNITION_CACHE_DIR";
pub(crate) const ENV_OUT_DIR: &str = "OUT_DIR";
pub(crate) const ENV_TARGET: &str = "TARGET";

// Environment variable prefixes (for export and additional configuration)
pub(crate) const ENV_METADATA_KEY_PREFIX: &str = "DEP_IGNITION_SYS_";

// Asset archive retrieval
// NOTE: cargo prefixes exported metadata keys, so `<ASSET>_ASSET_PATH` is read as
// `DEP_IGNITION_SYS_<ASSET>_ASSET_PATH` by dependent build scripts
#[cfg(feature = "download")]
pub(crate) const ARCHIVE_EXTENSION: &str = ".tar.gz";
#[cfg(feature = "download")]
pub(crate) const ARCHIVE_PARTIAL_EXTENSION: &str = ".part";
#[cfg(feature = "download")]
pub(crate) const METADATA_PATH_KEY_SUFFIX: &str = "_ASSET_PATH";

// Defaults
pub(crate) const DEFAULT_ASSET_DIR: &str = "assets/dependencies";
pub(crate) const DEFAULT_CACHE_DIR: &str = "cache";

// Supporting types
pub(crate) type TargetTriplet = String;
