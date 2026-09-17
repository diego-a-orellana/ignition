// Targets (and aliases) for build.

use std::{iter::zip, ops::Deref};

use crate::consts::*;
use crate::error::*;

/// Target as tuple struct of triplet and alias.
/// Assumes Rust target triplet format, architecture-vendor-operating_system-[environment].
/// Alias as per-part optional values.
#[derive(Debug)]
pub struct Target(TargetTriplet, TargetAlias);

impl Deref for Target {
    type Target = TargetTriplet;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Target {
    pub fn architecture(&self) -> &str {
        self.deref().split('-').collect::<Vec<&str>>()[0]
    }

    pub fn vendor(&self) -> &str {
        self.deref().split('-').collect::<Vec<&str>>()[1]
    }

    pub fn operating_system(&self) -> &str {
        self.deref().split('-').collect::<Vec<&str>>()[2]
    }

    pub fn environment(&self) -> Option<&str> {
        self.deref()
            .split('-')
            .collect::<Vec<&str>>()
            .get(3)
            .copied()
    }

    pub fn alias(&self) -> &TargetAlias {
        &self.1
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.deref().split('-')
    }
}

impl TryFrom<&TargetTriplet> for Target {
    type Error = IgnitionError;
    fn try_from(target: &TargetTriplet) -> IgnitionResult<Self> {
        let alias = TargetAlias::try_from(target)?;
        Ok(Self(target.clone(), alias))
    }
}

/// Target alias as per-part optional values.
#[derive(Default, Debug)]
pub struct TargetAlias {
    pub architecture: Option<String>,
    pub vendor: Option<String>,
    pub operating_system: Option<String>,
    pub environment: Option<String>,
}

impl TargetAlias {
    pub fn new(
        architecture: Option<String>,
        vendor: Option<String>,
        operating_system: Option<String>,
        environment: Option<String>,
    ) -> Self {
        Self {
            architecture,
            vendor,
            operating_system,
            environment,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Option<&str>> {
        [
            self.architecture.as_deref(),
            self.vendor.as_deref(),
            self.operating_system.as_deref(),
            self.environment.as_deref(),
        ]
        .into_iter()
    }
}

impl TryFrom<&TargetTriplet> for TargetAlias {
    type Error = IgnitionError;
    fn try_from(target: &TargetTriplet) -> IgnitionResult<Self> {
        let mut target_alias = TargetAlias::default();
        if let Some(alias_map) = target_config()?
            .get(CFG_TARGET_ALIAS_KEY)
            .ok_or(IgnitionError::TargetConfigurationKeyNotFoundError(
                CFG_TARGET_ALIAS_KEY.to_string(),
            ))?
            .get(target)
        {
            target_alias.architecture = alias_map
                .get(TARGET_ARCHITECTURE_KEY)
                .ok_or(IgnitionError::TargetAliasKeyNotFoundError(
                    TARGET_ARCHITECTURE_KEY.to_string(),
                ))?
                .clone();
            target_alias.vendor = alias_map
                .get(TARGET_VENDOR_KEY)
                .ok_or(IgnitionError::TargetAliasKeyNotFoundError(
                    TARGET_VENDOR_KEY.to_string(),
                ))?
                .clone();
            target_alias.operating_system = alias_map
                .get(TARGET_OPERATING_SYSTEM_KEY)
                .ok_or(IgnitionError::TargetAliasKeyNotFoundError(
                    TARGET_OPERATING_SYSTEM_KEY.to_string(),
                ))?
                .clone();
            target_alias.environment = alias_map
                .get(TARGET_ENVIRONMENT_KEY)
                .ok_or(IgnitionError::TargetAliasKeyNotFoundError(
                    TARGET_ENVIRONMENT_KEY.to_string(),
                ))?
                .clone();
        }
        Ok(target_alias)
    }
}

/// Target configuration from YAML file.
/// NOTE: select targets only, entries may be missing for specific targets.
fn target_config() -> IgnitionResult<TargetConfiguration> {
    Ok(serde_yaml::from_str::<TargetConfiguration>(CFG_TARGET_STR)?)
}

/// Target parts with consideration of optional aliases.
pub fn target_parts_aliased(target: &Target) -> Vec<&str> {
    let mut parts = Vec::new();
    for (part, optional_alias) in zip(target.iter(), target.alias().iter()) {
        if let Some(alias_part) = optional_alias {
            parts.push(alias_part);
        } else {
            parts.push(part);
        }
    }
    parts
}
