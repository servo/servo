/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use compositing_traits::ConstellationMsg;
use crossbeam_channel::{SendError, Sender};
use log::warn;

#[derive(Clone)]
pub(crate) struct ConstellationProxy {
    sender: Sender<ConstellationMsg>,
    disconnected: Arc<AtomicBool>,
}

impl ConstellationProxy {
    pub fn new(sender: Sender<ConstellationMsg>) -> Self {
        Self {
            sender,
            disconnected: Arc::default(),
        }
    }

    pub fn disconnected(&self) -> bool {
        self.disconnected.load(Ordering::SeqCst)
    }

    pub fn send(&self, msg: ConstellationMsg) {
        if self.try_send(msg).is_err() {
            warn!("Lost connection to Constellation. Will report to embedder.")
        }
    }

    fn try_send(&self, msg: ConstellationMsg) -> Result<(), SendError<ConstellationMsg>> {
        if self.disconnected() {
            return Err(SendError(msg));
        }
        if let Err(error) = self.sender.send(msg) {
            self.disconnected.store(true, Ordering::SeqCst);
            return Err(error);
        }

        Ok(())
    }

    pub fn sender(&self) -> Sender<ConstellationMsg> {
        self.sender.clone()
    }
}
