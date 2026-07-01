/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use embedder_traits::{WebViewPreference, WebViewPreferencesData, WebViewPreferencesId};
use servo_constellation_traits::EmbedderToConstellationMessage;

use crate::Servo;
use crate::proxies::ConstellationProxy;

/// The [`WebViewPreferences`] allows configuring preferences for
/// each `WebView` independently. The same [`WebViewPreferences`] instance
/// can be shared among multiple `WebView`s. `WebView`s that don't set a
/// custom [`WebViewPreferences`] during creation via `WebViewBuilder` will
/// use a shared "default" `WebViewPreferences` instance.
///
/// Changes to preferences made using the setters on [`WebViewPreferences`]
/// affect all `WebView`s that are associated with the [`WebViewPreferences`] .
/// The change will take effect when pages are reloaded.
#[derive(Clone)]
pub struct WebViewPreferences {
    pub(crate) id: WebViewPreferencesId,
    constellation_proxy: ConstellationProxy,
    data: RefCell<WebViewPreferencesData>,
}

impl WebViewPreferences {
    /// Create a new [`WebViewPreferences`] with all values set to their defaults.
    pub fn new(servo: &Servo) -> Self {
        Self::new_with_proxy(servo.constellation_proxy(), WebViewPreferencesId::next())
    }

    /// Create a new [`WebViewPreferences`] directly from a [`ConstellationProxy`].
    /// Used internally to create the DEFAULT preferences before `Servo` exists.
    pub(crate) fn new_with_proxy(
        constellation_proxy: &ConstellationProxy,
        id: WebViewPreferencesId,
    ) -> Self {
        // Send a message with empty preferences to register this id.
        constellation_proxy.send(EmbedderToConstellationMessage::SetWebViewPreferences(
            id,
            vec![],
        ));

        Self {
            id,
            constellation_proxy: constellation_proxy.clone(),
            data: RefCell::new(WebViewPreferencesData::default()),
        }
    }

    pub(crate) fn id(&self) -> WebViewPreferencesId {
        self.id
    }

    /// Get the default font size for proportional (variable-width) fonts, in CSS pixels.
    pub fn default_font_size(&self) -> i64 {
        self.data.borrow().default_font_size
    }

    /// Set the default font size for proportional (variable-width) fonts, in CSS pixels.
    pub fn set_default_font_size(&self, size: i64) {
        self.data.borrow_mut().default_font_size = size;
        self.constellation_proxy
            .send(EmbedderToConstellationMessage::SetWebViewPreferences(
                self.id,
                vec![WebViewPreference::DefaultFontSize(size)],
            ));
    }

    /// Get the default font size for monospace fonts, in CSS pixels.
    pub fn default_monospace_font_size(&self) -> i64 {
        self.data.borrow().default_monospace_font_size
    }

    /// Set the default font size for monospace fonts, in CSS pixels.
    pub fn set_default_monospace_font_size(&self, size: i64) {
        self.data.borrow_mut().default_monospace_font_size = size;
        self.constellation_proxy
            .send(EmbedderToConstellationMessage::SetWebViewPreferences(
                self.id,
                vec![WebViewPreference::DefaultMonospaceFontSize(size)],
            ));
    }
}

impl Drop for WebViewPreferences {
    fn drop(&mut self) {
        // The DEFAULT preferences don't have to be destroyed — they live as long
        // as Servo itself and DestroyWebViewPreferences for DEFAULT
        // during shutdown will not be processed as the script threads may
        // already be disconnected.
        if self.id != WebViewPreferencesId::DEFAULT {
            self.constellation_proxy.send(
                EmbedderToConstellationMessage::DestroyWebViewPreferences(self.id),
            );
        }
    }
}
