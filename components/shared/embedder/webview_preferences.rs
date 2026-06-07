/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicU32, Ordering};

use serde::{Deserialize, Serialize};

static NEXT_WEBVIEW_PREFERENCES_ID: AtomicU32 = AtomicU32::new(1);

/// An opaque identifier for a `WebViewPreferences` type in Servo.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct WebViewPreferencesId(pub u32);

impl WebViewPreferencesId {
    /// The ID for the default preferences used by Servo when creating a
    /// `WebView` that doesn't set a custom `WebViewPreferences` in
    /// `WebViewBuilder`.
    pub const DEFAULT: Self = Self(0);

    /// Generate the next unique [`WebViewPreferencesId`].
    pub fn next() -> Self {
        Self(NEXT_WEBVIEW_PREFERENCES_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// The backing preference data for a [`WebViewPreferencesId`].
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct WebViewPreferencesData {
    /// The default font size for proportional (variable-width) fonts, in CSS pixels.
    pub default_font_size: i64,
    /// The default font size for monospace fonts, in CSS pixels.
    pub default_monospace_font_size: i64,
}

impl Default for WebViewPreferencesData {
    fn default() -> Self {
        Self {
            default_font_size: 16,
            default_monospace_font_size: 13,
        }
    }
}

impl WebViewPreferencesData {
    /// Create a new [`WebViewPreferencesData`] with all values set to their defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a list of (partial) preference updates to this `WebViewPreferencesData`.
    pub fn apply_preference_updates(&mut self, preference_updates: &[WebViewPreference]) {
        for preference in preference_updates {
            match *preference {
                WebViewPreference::DefaultFontSize(size) => self.default_font_size = size,
                WebViewPreference::DefaultMonospaceFontSize(size) => {
                    self.default_monospace_font_size = size
                },
            }
        }
    }
}

/// A single preference value that can be sent to constellation and script threads
/// during updates.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum WebViewPreference {
    DefaultFontSize(i64),
    DefaultMonospaceFontSize(i64),
}
