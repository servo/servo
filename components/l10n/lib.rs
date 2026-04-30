/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module provides a simple API for loading and querying localization bundles
//! from `.ftl` (Fluent) files. It supports multiple named bundles (e.g. "servo"
//! for core, "servoshell" for the embedder) and automatic fallback to en-US.
//!
//! # Usage
//!
//! ```ignore
//! // Register a bundle (loads FTL files for current locale + en-US fallback)
//! servo_l10n::register("servo");
//!
//! // Look up a simple message
//! let text = servo_l10n::get("servo", "error-loading-page");
//!
//! // Or get a bundle handle for repeated use
//! let bundle = servo_l10n::bundle("servo");
//! let text = bundle.get("error-loading-page");
//!
//! // Parameterized messages
//! use servo_l10n::fluent_args;
//! let text = bundle.get_with_args("welcome", &l10n_args!["name" => "John"]);
//! ```

pub mod resources;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

pub use resources::{FileLocaleReader, set_locale_reader};

pub type FluentArgsTy<'a> = fluent_bundle::FluentArgs<'a>;
use fluent_bundle::concurrent::FluentBundle as ConcurrentFluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use log::warn;
use net_traits::get_current_locale;
use resources::load_bundle;
use unic_langid::LanguageIdentifier;

const FALLBACK_LOCALE: &str = "en-US";

/// Macro for convenient argument construction.
#[macro_export]
macro_rules! l10n_args {
    ( $($key:expr => $value:expr),* $(,)? ) => {{
        let mut args = servo_l10n::FluentArgsTy::new();
        $(
            args.set($key, $value);
        )*
        args
    }};
}

type BundleMap = HashMap<String, Arc<L10nBundle>>;

static BUNDLES: LazyLock<RwLock<BundleMap>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Create a FluentBundle from an FTL string for the given locale.
fn make_fluent_bundle(ftl_string: String, locale: &str) -> ConcurrentFluentBundle<FluentResource> {
    let langid: LanguageIdentifier = locale
        .parse()
        .unwrap_or_else(|_| panic!("Invalid locale identifier: {}", locale));
    let resource = FluentResource::try_new(ftl_string).expect("Failed to parse FTL resource");
    let mut bundle = ConcurrentFluentBundle::new_concurrent(vec![langid]);
    bundle
        .add_resource(resource)
        .expect("Failed to add FTL resource to bundle");
    bundle
}

/// A localization bundle for a specific domain (e.g. "servo", "servoshell").
///
/// Thread-safe and immutable after creation. Wraps a primary locale bundle
/// and an en-US fallback bundle. Message lookup tries the primary locale first,
/// then falls back to en-US.
pub struct L10nBundle {
    name: String,
    primary: Option<ConcurrentFluentBundle<FluentResource>>,
    fallback: ConcurrentFluentBundle<FluentResource>,
}

impl L10nBundle {
    /// Look up a simple message by key.
    ///
    /// # Panics
    /// Panics if the key is not found in either the primary locale or en-US fallback.
    pub fn get(&self, key: &str) -> String {
        self.get_with_args(key, &FluentArgs::new())
    }

    /// Look up a message by key with arguments for parameterized messages.
    ///
    /// # Panics
    /// Panics if the key is not found in either the primary locale or en-US fallback.
    pub fn get_with_args(&self, key: &str, args: &FluentArgs) -> String {
        // Try primary locale first
        if let Some(ref primary) = self.primary {
            if let Some(result) = Self::format_message(primary, key, args) {
                return result;
            }
        }

        // Fall back to en-US
        if let Some(result) = Self::format_message(&self.fallback, key, args) {
            return result;
        }

        panic!(
            "Missing l10n key '{}' in bundle '{}' (not found in {} fallback)",
            key, self.name, FALLBACK_LOCALE
        );
    }

    fn format_message(
        bundle: &ConcurrentFluentBundle<FluentResource>,
        key: &str,
        args: &FluentArgs,
    ) -> Option<String> {
        let message = bundle.get_message(key)?;
        let pattern = message.value()?;
        let mut errors = vec![];
        let value = bundle.format_pattern(pattern, Some(args), &mut errors);
        if !errors.is_empty() {
            warn!("L10n: errors formatting '{}': {:?}", key, errors);
        }
        Some(value.into_owned())
    }
}

/// Register a localization bundle by name.
///
/// Loads the FTL files for the current locale and en-US fallback from
/// the LocaleReader.
///
/// # Panics
/// Panics if the en-US fallback FTL file cannot be loaded.
pub fn register(bundle_name: &str) {
    let locale = &get_current_locale().0;

    // Load en-US fallback (required)
    let fallback_ftl = load_bundle(bundle_name, FALLBACK_LOCALE).unwrap_or_else(|| {
        panic!(
            "Missing required {} FTL content for bundle '{}'",
            FALLBACK_LOCALE, bundle_name
        )
    });
    let fallback = make_fluent_bundle(fallback_ftl, FALLBACK_LOCALE);

    // Load primary locale if it's different from the fallback.
    let primary = if locale != FALLBACK_LOCALE {
        match load_bundle(bundle_name, locale) {
            Some(ftl) => Some(make_fluent_bundle(ftl, locale)),
            None => {
                warn!(
                    "L10n: no {} locale file for bundle '{}', using {} only",
                    locale, bundle_name, FALLBACK_LOCALE
                );
                None
            },
        }
    } else {
        None
    };

    let l10n_bundle = Arc::new(L10nBundle {
        name: bundle_name.to_string(),
        primary,
        fallback,
    });

    BUNDLES
        .write()
        .expect("L10n bundle registry poisoned")
        .insert(bundle_name.to_string(), l10n_bundle);
}

/// Get a handle to a registered bundle.
///
/// # Panics
/// Panics if the bundle has not been registered via `register()`.
pub fn bundle(bundle_name: &str) -> Arc<L10nBundle> {
    Arc::clone(
        BUNDLES
            .read()
            .expect("L10n bundle registry poisoned")
            .get(bundle_name)
            .unwrap_or_else(|| {
                panic!(
                    "L10n bundle '{}' not registered. Call servo_l10n::register(\"{}\") first.",
                    bundle_name, bundle_name
                )
            }),
    )
}

/// Look up a simple message from a registered bundle.
///
/// Convenience function for occasional use. For repeated lookups in the same
/// bundle, prefer `bundle()` to get a handle.
///
/// # Panics
/// Panics if the bundle is not registered or the key is not found.
pub fn get(bundle_name: &str, key: &str) -> String {
    bundle(bundle_name).get(key)
}

/// Look up a parameterized message from a registered bundle.
///
/// # Panics
/// Panics if the bundle is not registered or the key is not found.
pub fn get_with_args(bundle_name: &str, key: &str, args: &FluentArgs) -> String {
    bundle(bundle_name).get_with_args(key, args)
}
