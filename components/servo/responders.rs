/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::{GenericSender, SendError, SendResult};
use crossbeam_channel::{Receiver, Sender, TryRecvError, unbounded};
use log::warn;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender as TokioSender;

use crate::ServoError;

/// Sender for errors raised by delegate request objects.
///
/// This allows errors to be raised asynchronously.
pub(crate) struct ServoErrorSender {
    sender: Sender<ServoError>,
}

impl ServoErrorSender {
    pub(crate) fn raise_response_send_error(&self, error: SendError) {
        if let Err(error) = self.sender.send(ServoError::ResponseFailedToSend(error)) {
            warn!("Failed to send Servo error: {error:?}");
        }
    }
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

pub(crate) trait AbstractSender {
    type Message;
    fn send(&self, value: Self::Message) -> SendResult;
}

impl<T: Serialize> AbstractSender for GenericSender<T> {
    type Message = T;
    fn send(&self, value: T) -> SendResult {
        GenericSender::send(self, value)
    }
}

impl<T> AbstractSender for TokioSender<T> {
    type Message = T;
    fn send(&self, value: T) -> SendResult {
        TokioSender::send(self, value).map_err(|_| SendError::Disconnected)
    }
}

/// Sends a response over an IPC channel, or a default response on [`Drop`] if no response was sent.
pub(crate) struct IpcResponder<T> {
    response_sender: Box<dyn AbstractSender<Message = T>>,
    response_sent: bool,
    /// Always present, except when taken by [`Drop`].
    default_response: Option<T>,
}

impl<T: Serialize + 'static> IpcResponder<T> {
    pub(crate) fn new(response_sender: GenericSender<T>, default_response: T) -> Self {
        Self {
            response_sender: Box::new(response_sender),
            response_sent: false,
            default_response: Some(default_response),
        }
    }
}

impl<T: 'static> IpcResponder<T> {
    pub(crate) fn new_same_process(response_sender: TokioSender<T>, default_response: T) -> Self {
        Self {
            response_sender: Box::new(response_sender),
            response_sent: false,
            default_response: Some(default_response),
        }
    }
}

impl<T> IpcResponder<T> {
    pub(crate) fn send(&mut self, response: T) -> SendResult {
        let result = self.response_sender.send(response);
        self.response_sent = true;
        result
    }
}

impl<T> Drop for IpcResponder<T> {
    fn drop(&mut self) {
        if !self.response_sent {
            let response = self
                .default_response
                .take()
                .expect("Guaranteed by inherent impl");
            // Don’t notify embedder about send errors for the default response,
            // since they didn’t send anything and probably don’t care.
            let _ = self.response_sender.send(response);
        }
    }
}
