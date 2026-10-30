// Asset base struct and traits, referenced in registry macros.

use std::ops::Deref;

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

/// Asset contents and environment mappings.
///
/// Example contents and environment:
/// ```rust
/// const CONTENTS: &'static [&'static str] = &[
///     "path/to/content1",
///     "path/to/content2",
///     "path/to/content3"
/// ];
/// const ENVIRONMENT: &'static [(&'static str, &'static str)] = &[
///     ("ENV_VAR1", "path/to/content1"),
///     ("ENV_VAR2", "path/to/content2"),
/// ];
/// ```
pub trait Extractable: Retrievable {
    const CONTENTS: &'static [&'static str];
    const ENVIRONMENT: &'static [(&'static str, &'static str)];
}
