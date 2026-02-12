// build.rs
//
// This crate performs pre-compilation asset retrieval and cargo metadata setting.
// During runtime of a dependent `build.rs`, this crate will retrieve cargo metadata and export environment variables.
//
use std::{env::var, process::Command};

const ASSET_SCRIPT_PATH: &str = "scripts/asset.sh";
const DEFAULT_CACHE_PATH: &str = "cache";
const DEFAULT_DIRECTORY_PATH: &str = "assets/dependencies";

include!("src/lib.rs");

/// Entry point for all asset retrieval and environment variable setting
fn asset(
    var_bucket_url: &str,
    build_dir: &str,
    var_cache_path: &str,
    var_directory_path: &str,
    target: &str,
) {
    asset_script();

    #[cfg(feature = "download-opencv")]
    #[allow(clippy::needless_borrow)]
    asset_opencv(
        &var_bucket_url,
        &build_dir,
        &var_cache_path,
        &var_directory_path,
        &target,
    );

    #[cfg(feature = "download-onnxruntime")]
    #[allow(clippy::needless_borrow)]
    asset_onnxruntime(
        &var_bucket_url,
        &build_dir,
        &var_cache_path,
        &var_directory_path,
        &target,
    );
}

/// Retrieve OpenCV asset and set environment variables
#[cfg(feature = "download-opencv")]
fn asset_opencv(
    var_bucket_url: &str,
    build_dir: &str,
    cache_path: &str,
    directory_path: &str,
    target: &str,
) {
    asset_retrieve(
        var_bucket_url,
        "opencv",
        build_dir,
        cache_path,
        directory_path,
        target,
    );
    let _ = environment_variables(
        "opencv",
        Some(&std::path::Path::new(&build_dir).join(directory_path)),
    );
}

/// Retrieve Onnruuntime asset and set environment variable
#[cfg(feature = "download-onnxruntime")]
fn asset_onnxruntime(
    var_bucket_url: &str,
    build_dir: &str,
    cache_path: &str,
    directory_path: &str,
    target: &str,
) {
    asset_retrieve(
        var_bucket_url,
        "onnxruntime",
        build_dir,
        cache_path,
        directory_path,
        target,
    );
    let _ = environment_variables(
        "onnxruntime",
        Some(&std::path::Path::new(&build_dir).join(directory_path)),
    );
}

/// Retrieve an asset by name using the asset.sh script
fn asset_retrieve(
    var_bucket_url: &str,
    asset: &str,
    build_dir: &str,
    cache_path: &str,
    directory_path: &str,
    target: &str,
) {
    let mut output = Command::new(ASSET_SCRIPT_PATH)
        .args([
            var_bucket_url,
            asset,
            build_dir,
            cache_path,
            directory_path,
            target,
        ])
        .spawn()
        .expect("asset.sh command failed to start");
    let _ = output.wait().expect("asset.sh command failed to complete");
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

/// Main entry point
fn main() {
    // force re-run by pointing to a non-existent file
    println!("cargo:rerun-if-changed=NULL");

    // require definition and format: /../target/<target-triplet>/<build-type>/build/<ignition-build-id>/out
    let out_dir = var("OUT_DIR").unwrap();
    let build_dir = out_dir.split(&"/build".to_string()).next().unwrap();

    // parse target
    let target = std::env::var("TARGET").unwrap_or("".to_string());

    // retrieve assets and set environment variables (note: target exclusion)
    #[cfg(any(feature = "download-opencv", feature = "download-onnxruntime"))]
    if !(target.starts_with("aarch64-") && target.contains("linux")) {
        asset(
            &var("IGNITION_BUCKET_URL").expect("IGNITION_BUCKET_URL environment variable error"),
            build_dir,
            &var("IGNITION_CACHE_PATH").unwrap_or(DEFAULT_CACHE_PATH.to_string()),
            &var("IGNITION_DIRECTORY_PATH").unwrap_or(DEFAULT_DIRECTORY_PATH.to_string()),
            &target,
        );
    }
}
