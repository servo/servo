/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use constellation_traits::EmbedderToConstellationMessage;
use crossbeam_channel::{SendError, Sender};
use log::warn;

#[derive(Clone)]
pub(crate) struct ConstellationProxy {
    sender: Sender<EmbedderToConstellationMessage>,
    disconnected: Arc<AtomicBool>,
}

impl ConstellationProxy {
    pub fn new(sender: Sender<EmbedderToConstellationMessage>) -> Self {
        Self {
            sender,
            disconnected: Arc::default(),
        }
    }

    pub fn disconnected(&self) -> bool {
        self.disconnected.load(Ordering::SeqCst)
    }

    pub fn send(&self, msg: EmbedderToConstellationMessage) {
        if self.try_send(msg).is_err() {
            warn!("Lost connection to Constellation. Will report to embedder.")
        }
    }

    fn try_send(
        &self,
        msg: EmbedderToConstellationMessage,
    ) -> Result<(), SendError<EmbedderToConstellationMessage>> {
        if self.disconnected() {
            return Err(SendError(msg));
        }
        if let Err(error) = self.sender.send(msg) {
            self.disconnected.store(true, Ordering::SeqCst);
            return Err(error);
        }

        Ok(())
    }

    pub fn sender(&self) -> Sender<EmbedderToConstellationMessage> {
        self.sender.clone()
    }
}
