// build.rs
//
// On crate compilation, assets are retrieved and environment variables are exported as cargo metadata.
// On dependent compilation, cargo metadata are retrieved and environment variables are set.
//

use std::process::Command;

include!("src/lib.rs");

/// Entry point for asset retrieval and environment variable setting
fn asset(config: Config) -> IgnitionResult<()> {
    asset_script();

    #[cfg(feature = "download-opencv")]
    asset_opencv(&config)?;

    #[cfg(feature = "download-onnxruntime")]
    asset_onnxruntime(&config)?;

    Ok(())
}

/// Retrieve OpenCV asset and set environment variables
#[cfg(feature = "download-opencv")]
fn asset_opencv(config: &Config) -> IgnitionResult<()> {
    let asset = OpenCv::new();
    let _ = asset_retrieve(&asset, config)?;
    let _ = environment_variables(&asset, Some(config))?;
    Ok(())
}

/// Retrieve Onnxruntime asset and set environment variable
#[cfg(feature = "download-onnxruntime")]
fn asset_onnxruntime(config: &Config) -> IgnitionResult<()> {
    let asset = ONNXRuntime::new();
    let _ = asset_retrieve(&asset, config)?;
    let _ = environment_variables(&asset, Some(config))?;
    Ok(())
}

/// Retrieve an asset by name using the asset.sh script
fn asset_retrieve<T: Retrievable>(_: &T, config: &Config) -> IgnitionResult<String> {
    let asset_path_key = format!(
        "DEP_IGNITION_SYS_{}_PATH",
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
fn asset_script() {
    let mut output = Command::new("chmod")
        .arg("+x")
        .arg(ASSET_SCRIPT_PATH)
        .spawn()
        .expect("'chmod +x <script-path>' failed");

    let _ = output.wait().expect("'chmod +x <script-path>' failed");
}

/// Main entry point, run configured from user-specified environment variables and defaults
fn main() {
    // force re-run by pointing to a non-existent file
    println!("cargo::rerun-if-changed=NULL");
    let config = Config::new().expect("failed to create Ignition config");
    println!("{:?}", config);
    asset(config).expect("failed to retrieve assets");
}
