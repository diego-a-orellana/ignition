// build.rs
//
// On crate compilation, assets are retrieved and environment variables are exported as cargo metadata.
// On dependent compilation, cargo metadata are retrieved and environment variables are set.
//

include!("src/lib.rs");

/// Main entry point, run configured from user-specified environment variables and defaults
fn main() {
    // force re-run by pointing to a non-existent file
    println!("cargo::rerun-if-changed=NULL");
    let config = Config::new().expect("failed to create Ignition config");
    println!("{:?}", config);
    retrieve_assets(&config).expect("failed to retrieve assets");
}
