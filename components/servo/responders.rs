/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crossbeam_channel::{Receiver, Sender, TryRecvError, unbounded};
use log::warn;

use crate::ServoError;

/// Sender for errors raised by delegate request objects.
///
/// This allows errors to be raised asynchronously.
pub(crate) struct ServoErrorSender {
    sender: Sender<ServoError>,
}

impl ServoErrorSender {
    pub(crate) fn raise_response_send_error(&self, error: bincode::Error) {
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
