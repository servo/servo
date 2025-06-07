/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::BrowsingContextId;
use embedder_traits::Theme;

use crate::session_history::JointSessionHistory;

/// The `Constellation`'s view of a `WebView` in the embedding layer. This tracks all of the
/// `Constellation` state for this `WebView`.
pub(crate) struct ConstellationWebView {
    /// The currently focused browsing context in this webview for key events.
    /// The focused pipeline is the current entry of the focused browsing
    /// context.
    pub focused_browsing_context_id: BrowsingContextId,

    /// The joint session history for this webview.
    pub session_history: JointSessionHistory,

    /// The [`Theme`] that this [`ConstellationWebView`] uses. This is communicated to all
    /// `ScriptThread`s so that they know how to render the contents of a particular `WebView.
    theme: Theme,
}

impl ConstellationWebView {
    pub(crate) fn new(focused_browsing_context_id: BrowsingContextId) -> Self {
        Self {
            focused_browsing_context_id,
            session_history: JointSessionHistory::new(),
            theme: Theme::Light,
        }
    }

    /// Set the [`Theme`] on this [`ConstellationWebView`] returning true if the theme changed.
    pub(crate) fn set_theme(&mut self, new_theme: Theme) -> bool {
        let old_theme = std::mem::replace(&mut self.theme, new_theme);
        old_theme != self.theme
    }

    /// Get the [`Theme`] of this [`ConstellationWebView`].
    pub(crate) fn theme(&self) -> Theme {
        self.theme
    }
}
