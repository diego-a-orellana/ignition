// Assets (from registry) for build.

use std::{env::var, ops::Deref};

use crate::consts::*;

#[derive(Default)]
pub struct Asset {}

impl Asset {
    pub fn new() -> Self {
        Self {}
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
    fn check_target_excluded(&self, target: &TargetTriplet) -> bool;
}

impl<T: Retrievable> Targetable for T {
    fn target_exclusions(&self) -> Option<Vec<String>> {
        let env_var_str: String =
            format!("{}{}", ENV_TARGET_EXCLUSIONS_PREFIX, T::KEY.to_uppercase());
        if let Ok(env_var_value) = var(env_var_str) {
            let target_exclusions: Vec<String> = env_var_value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            println!(
                "asset {} target exclusions: {:?}",
                T::KEY,
                target_exclusions
            );
            Some(target_exclusions)
        } else {
            println!("asset {} target exclusions not set", T::KEY);
            None
        }
    }

    fn check_target_excluded(&self, target: &TargetTriplet) -> bool {
        if let Some(target_exclusions) = self.target_exclusions() {
            let is_excluded = target_exclusions.contains(target);
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
#[derive(Default)]
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
    const CONTENTS: [&str; 2] = ["opencv/lib", "opencv/opencv4"];
    const ENVIRONMENT: [(&str, &str); 2] = [
        ("OPENCV_LINK_PATHS", "opencv/lib"),
        ("OPENCV_INCLUDE_PATHS", "opencv/opencv4"),
    ];
}

/// Asset: ONNXRuntime
#[derive(Default)]
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
    const CONTENTS: [&str; 1] = ["onnxruntime"];
    const ENVIRONMENT: [(&str, &str); 1] = [("ORT_LIB_LOCATION", "onnxruntime")];
}

// /// Asset registry from YAML file.
// /// NOTE: entries [to be] referenced in macro, structs for each generated procedurally.
// fn asset_registry() -> IgnitionResult<TargetConfiguration> {
//     // Ok(serde_yaml::from_str::<_>(CFG_REGISTRY_STR)?)
//     todo!()
// }
