/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use embedder_traits::{WebResourceRequest, WebResourceResponseMsg};
use ipc_channel::ipc::IpcSender;

use crate::webview_delegate::AllowOrDenyRequest;
use crate::{Servo, WebView};

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
    /// Potentially intercept a resource request. If not handled, the request will not be intercepted.
    ///
    /// Note: If this request is associated with a `WebView`,  the `WebViewDelegate` will
    /// receive this notification first and have a chance to intercept the request.
    ///
    /// TODO: This API needs to be reworked to match the new model of how responses are sent.
    fn intercept_web_resource_load(
        &self,
        _webview: Option<WebView>,
        _request: &WebResourceRequest,
        response_sender: IpcSender<WebResourceResponseMsg>,
    ) {
        let _ = response_sender.send(WebResourceResponseMsg::None);
    }
}

pub(crate) struct DefaultServoDelegate;
impl ServoDelegate for DefaultServoDelegate {}
