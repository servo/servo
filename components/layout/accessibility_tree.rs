/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvError, SendError, Sender, TrySendError};
use log::info;
use malloc_size_of_derive::MallocSizeOf;
use parking_lot::RwLock;
use webrender_api::crossbeam_channel;

use crate::FragmentTree;

#[derive(MallocSizeOf)]
pub(crate) struct AccessibilityTree {}

pub(crate) struct AccessibilityThread {
    input_channel: Arc<DebouncingChannel>,
    // TODO: don’t leak the thread
    join_handle: JoinHandle<()>,
}

impl AccessibilityThread {
    pub(crate) fn new() -> Self {
        let input_channel = Arc::new(DebouncingChannel::new());
        Self {
            input_channel: input_channel.clone(),
            join_handle: thread::spawn(move || {
                while let Ok(fragment_tree) = input_channel.recv() {
                    info!("Thinking...");
                    std::thread::sleep(Duration::from_secs(1));
                    info!(
                        "Fragment tree has {} top-level fragments",
                        fragment_tree.root_fragments.len()
                    );
                }
            }),
        }
    }

    pub(crate) fn send(&self, fragment_tree: Arc<FragmentTree>) {
        // If this fails, the channel is disconnected (so the thread is gone).
        let _ = self.input_channel.send(fragment_tree);
    }
}

/// A debouncing [`crossbeam_channel`].
struct DebouncingChannel {
    sender: Sender<Arc<FragmentTree>>,
    receiver: Receiver<Arc<FragmentTree>>,
    last_input: RwLock<Option<Arc<FragmentTree>>>,
}

impl DebouncingChannel {
    fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(0);
        Self {
            sender,
            receiver,
            last_input: Default::default(),
        }
    }
    /// Send a fragment tree, or fail if the channel is disconnected.
    ///
    /// If the channel is not currently in `recv()`, store the value for later. If a value has
    /// already been stored, overwrite that value.
    fn send(&self, fragment_tree: Arc<FragmentTree>) -> Result<(), SendError<Arc<FragmentTree>>> {
        match self.sender.try_send(fragment_tree) {
            Ok(()) => Ok(()),
            Err(TrySendError::Disconnected(value)) => Err(SendError(value)),
            Err(TrySendError::Full(value)) => {
                *self.last_input.write() = Some(value);
                Ok(())
            },
        }
    }
    /// Receive a fragment tree, or fail if the channel is disconnected.
    ///
    /// If there was a value stored by [Self::send()], return that instead, without trying to
    /// receive on the channel. Either way, fragment trees are never delivered out of order.
    fn recv(&self) -> Result<Arc<FragmentTree>, RecvError> {
        // TODO: try to receive on the channel, and if successful, disregard the stored value.
        // otherwise the latency of the final fragment tree in a burst may be suboptimal.
        if let Some(last_input) = self.last_input.write().take() {
            Ok(last_input)
        } else {
            self.receiver.recv()
        }
    }
}
