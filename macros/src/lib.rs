// lib.rs
//
// Code generation for the asset registry.
// Asset types are emitted at compile time from `config/registry.yaml` of the dependent crate.
//

use std::{collections::BTreeMap, env::var, fs::read_to_string, path::PathBuf};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use serde::Deserialize;
use syn::{LitStr, parse_macro_input};

const ENV_MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";
const FEATURE_FLAG_PREFIX: &str = "download-";

/// Single asset entry of the registry, keyed by its (unused) crate identifier.
#[derive(Deserialize)]
struct RegistryEntry {
    /// Rust type name generated for the asset.
    name: String,
    /// Asset key, as used for retrieval, metadata prefixes and the `download-<key>` feature.
    key: String,
    /// Expected contents on extraction of the asset archive.
    #[serde(default)]
    contents: Vec<String>,
    /// Mapping of environment variables to asset-relative content paths.
    #[serde(default)]
    environment: Vec<BTreeMap<String, String>>,
}

/// Generate an asset type for every entry of the registry at `<CARGO_MANIFEST_DIR>/<path>`.
///
/// Each entry emits a tuple struct over `Asset` implementing `Retrievable` and `Extractable`.
/// A `retrieve_assets` entry point is emitted alongside them, retrieving each asset whose
/// `download-<key>` feature is enabled.
#[proc_macro]
pub fn register_assets(input: TokenStream) -> TokenStream {
    let registry_path = parse_macro_input!(input as LitStr).value();
    match registry(&registry_path) {
        Ok(tokens) => tokens.into(),
        Err(err) => quote! { ::std::compile_error!(#err); }.into(),
    }
}

/// Read, deserialize and expand the registry.
fn registry(registry_path: &str) -> Result<TokenStream2, String> {
    let path = PathBuf::from(
        var(ENV_MANIFEST_DIR).map_err(|err| format!("{} error: {}", ENV_MANIFEST_DIR, err))?,
    )
    .join(registry_path);
    let registry_str = read_to_string(&path)
        .map_err(|err| format!("failed to read registry {}: {}", path.display(), err))?;
    // Deserialized as an ordered map so that generated code is stable across builds
    let registry: BTreeMap<String, RegistryEntry> = serde_yaml::from_str(&registry_str)
        .map_err(|err| format!("failed to deserialize registry {}: {}", path.display(), err))?;

    let names = registry
        .values()
        .map(|entry| identifier(&entry.name))
        .collect::<Result<Vec<Ident>, String>>()?;
    let assets = registry.values().zip(&names).map(asset);
    let retrieval = retrieval(registry.values().zip(&names));

    // Cargo does not track files read during macro expansion, so the registry is also included
    // as a (discarded) string to register it as a dependency of the compilation
    let path_str = path.to_string_lossy();
    Ok(quote! {
        const _: &str = ::std::include_str!(#path_str);
        #(#assets)*
        #retrieval
    })
}

/// Expand a single registry entry into its asset type and trait implementations.
fn asset((entry, name): (&RegistryEntry, &Ident)) -> TokenStream2 {
    let key = &entry.key;
    let contents = &entry.contents;
    let environment = entry
        .environment
        .iter()
        .flat_map(|mapping| mapping.iter())
        .map(|(env_var, content)| quote! { (#env_var, #content) })
        .collect::<Vec<TokenStream2>>();

    quote! {
        #[derive(Default)]
        pub struct #name(crate::asset::Asset);

        impl #name {
            pub fn new() -> Self {
                Self(crate::asset::Asset::new())
            }
        }

        impl ::std::ops::Deref for #name {
            type Target = crate::asset::Asset;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl crate::asset::Retrievable for #name {
            const KEY: &'static str = #key;
        }

        impl crate::asset::Extractable for #name {
            const CONTENTS: &'static [&'static str] = &[#(#contents),*];
            const ENVIRONMENT: &'static [(&'static str, &'static str)] = &[#(#environment),*];
        }
    }
}

/// Expand the retrieval entry point, one feature-gated block per registry entry.
fn retrieval<'a>(entries: impl Iterator<Item = (&'a RegistryEntry, &'a Ident)>) -> TokenStream2 {
    let blocks = entries.map(|(entry, name)| {
        let feature = format!("{}{}", FEATURE_FLAG_PREFIX, entry.key);
        quote! {
            #[cfg(feature = #feature)]
            {
                let asset = #name::new();
                let _ = crate::retrieve::asset_retrieve(&asset, config)?;
                let _ = crate::environment_variables(&asset, Some(config))?;
            }
        }
    });

    quote! {
        /// Retrieve every asset whose `download-<key>` feature is enabled and export its
        /// environment variables as cargo metadata.
        // `config` is unused when no asset feature is enabled
        #[allow(unused_variables)]
        pub fn retrieve_assets(config: &crate::config::Config) -> crate::error::IgnitionResult<()> {
            crate::retrieve::asset_script();
            #(#blocks)*
            Ok(())
        }
    }
}

/// Validate a registry `name` before using it as a type identifier.
fn identifier(name: &str) -> Result<Ident, String> {
    let valid = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if valid {
        Ok(Ident::new(name, Span::call_site()))
    } else {
        Err(format!("invalid asset name (not an identifier): {}", name))
    }
}
