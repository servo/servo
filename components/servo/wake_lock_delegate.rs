/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::GenericSender;
use embedder_traits::AllowOrDeny;

use crate::WebView;

/// A delegate that is responsible for acquiring and releasing screen wake locks.
/// Embedders should implement this trait to prevent the screen from sleeping while
/// a wake lock is held.
///
/// <https://w3c.github.io/screen-wake-lock/>
pub trait WakeLockDelegate {
    /// A request to acquire a screen wake lock. The embedder should respond via
    /// `response` with [`AllowOrDeny::Allow`] to grant the lock (and prevent the
    /// screen from sleeping until [`WakeLockDelegate::release`] is called), or
    /// [`AllowOrDeny::Deny`] to reject the request.
    fn acquire(&self, _webview: WebView, response: GenericSender<AllowOrDeny>) {
        let _ = response.send(AllowOrDeny::Deny);
    }

    /// A request to release a previously acquired screen wake lock. The embedder
    /// may allow the screen to sleep again.
    fn release(&self, _webview: WebView) {}
}

pub(crate) struct DefaultWakeLockDelegate;

impl WakeLockDelegate for DefaultWakeLockDelegate {}
