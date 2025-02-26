/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crossbeam_channel::{Receiver, Sender, TryRecvError, unbounded};
use log::warn;

use crate::Servo;
use crate::responders::DelegateErrorSender;
use crate::webview_delegate::{AllowOrDenyRequest, WebResourceLoad};

#[derive(Debug)]
pub enum ServoError {
    /// The channel to the off-the-main-thread web engine has been lost. No further
    /// attempts to communicate will happen. This is an unrecoverable error in Servo.
    LostConnectionWithBackend,
    /// The devtools server, used to expose pages to remote web inspectors has failed
    /// to start.
    DevtoolsFailedToStart,
    /// Failed to send response to delegate request.
    ResponseSend(bincode::Error),
}

/// Channel for errors raised by [`WebViewDelegate`] request objects.
///
/// This allows errors to be raised asynchronously.
pub(crate) struct ServoErrorChannel {
    sender: Sender<ServoError>,
    receiver: Receiver<ServoError>,
}

impl Default for ServoErrorChannel {
    fn default() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}

impl ServoErrorChannel {
    pub(crate) fn sender(&self) -> ServoErrorSender {
        ServoErrorSender {
            sender: self.sender.clone(),
        }
    }
    pub(crate) fn try_recv(&self) -> Option<ServoError> {
        match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(error) => {
                debug_assert_eq!(error, TryRecvError::Empty);
                None
            },
        }
    }
}

/// Sender for [`ServoError`] that identifies the [`WebView`] in question.
pub(crate) struct ServoErrorSender {
    sender: Sender<ServoError>,
}

impl DelegateErrorSender for ServoErrorSender {
    fn raise_response_send_error(&self, error: bincode::Error) {
        if let Err(error) = self.sender.send(ServoError::ResponseSend(error)) {
            warn!("Failed to send Servo error: {error:?}");
        }
    }
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
