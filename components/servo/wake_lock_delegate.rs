/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::WebView;

/// A delegate that is responsible for acquiring and releasing screen wake locks.
/// Embedders should implement this trait to prevent the screen from sleeping while
/// a wake lock is held.
///
/// The Constellation tracks the aggregate lock count across all webviews and only
/// calls [`acquire`](WakeLockDelegate::acquire) on the 0→1 transition and
/// [`release`](WakeLockDelegate::release) on the N→0 transition, so these methods
/// act as fire-and-forget OS-level notifications rather than per-request callbacks.
///
/// <https://w3c.github.io/screen-wake-lock/>
pub trait WakeLockDelegate {
    /// Notify the embedder to acquire a screen wake lock, preventing the screen
    /// from sleeping. Called only when the aggregate lock count transitions from 0 to 1.
    fn acquire(&self, _webview: WebView) {}

    /// Notify the embedder to release a previously acquired screen wake lock,
    /// allowing the screen to sleep. Called only when the aggregate lock count
    /// transitions from N to 0.
    fn release(&self, _webview: WebView) {}
}

pub(crate) struct DefaultWakeLockDelegate;

impl WakeLockDelegate for DefaultWakeLockDelegate {}
