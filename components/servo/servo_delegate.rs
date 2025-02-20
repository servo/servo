/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::webview_delegate::{AllowOrDenyRequest, WebResourceLoad};
use crate::Servo;

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub enum ServoError {
    /// The channel to the off-the-main-thread web engine has been lost. No further
    /// attempts to communicate will happen. This is an unrecoverable error in Servo.
    LostConnectionWithBackend,
    /// The devtools server, used to expose pages to remote web inspectors has failed
    /// to start.
    DevtoolsFailedToStart,
}

pub trait ServoDelegate {
    /// Notification that Servo has received a major error.
    fn notify_error(&self, _servo: &Servo, _error: ServoError) {}
    /// Report that the DevTools server has started on the given `port`. The `token` that
    /// be used to bypass the permission prompt from the DevTools client.
    fn notify_devtools_server_started(&self, _servo: &Servo, _port: u16, _token: String) {}
    /// Request a DevTools connection from a DevTools client. Typically an embedder application
    /// will show a permissions prompt when this happens to confirm a connection is allowed.
    fn request_devtools_connection(&self, _servo: &Servo, _request: AllowOrDenyRequest) {}
    /// Triggered when Servo will load a web (HTTP/HTTPS) resource. The load may be
    /// intercepted and alternate contents can be loaded by the client by calling
    /// [`WebResourceLoad::intercept`]. If not handled, the load will continue as normal.
    ///
    /// Note: This delegate method is called for all resource loads not associated with a
    /// [`WebView`].  For loads associated with a [`WebView`], Servo  will call
    /// [`crate::WebViewDelegate::load_web_resource`].
    fn load_web_resource(&self, _load: WebResourceLoad) {}
}

pub(crate) struct DefaultServoDelegate;
impl ServoDelegate for DefaultServoDelegate {}
