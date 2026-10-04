// Asset retrieval for build.

use std::process::Command;

use crate::asset::*;
use crate::config::*;
use crate::consts::*;
use crate::error::*;

/// Retrieve an asset by name using the asset.sh script
pub fn asset_retrieve<T: Retrievable>(_: &T, config: &Config) -> IgnitionResult<String> {
    let asset_path_key = format!(
        "{}{}_PATH",
        ENV_METADATA_KEY_PREFIX,
        <T as Retrievable>::KEY.to_uppercase()
    );
    let mut output = Command::new(ASSET_SCRIPT_PATH)
        .args([
            config.bucket_url.clone(),
            <T as Retrievable>::KEY.to_string(),
            config.build_dir.clone(),
            config.cache_dir.clone(),
            config.asset_dir.clone(),
            config.target.clone(),
            asset_path_key,
        ])
        .spawn()
        .expect("asset.sh command failed to start");
    let _ = output.wait().expect("asset.sh command failed to complete");
    // TODO: parse output for asset path, or print straight to cargo metadata from script?
    Ok(String::new())
}

/// Prepare the asset.sh script by making it executable
pub fn asset_script() {
    let mut output = Command::new("chmod")
        .arg("+x")
        .arg(ASSET_SCRIPT_PATH)
        .spawn()
        .expect("'chmod +x <script-path>' failed");

    let _ = output.wait().expect("'chmod +x <script-path>' failed");
}
