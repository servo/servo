/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use ipc_channel::ipc::{IpcReceiverSet, IpcSelectionResult};
use serde::{Deserialize, Serialize};

use crate::generic_channel::{GenericReceiver, GenericReceiverVariants};

/// A GenericReceiverSet. Allows you to wait on multiple GenericReceivers.
/// Automatically selects either Ipc or crossbeam depending on multiprocess mode.
/// # Examples
/// ```
/// # let mut rx_set = GenericReceiverSet::new();
/// # let private_channel = generic_channel::channel().unwrap();
/// # let public_channel = generic_channel::channel().unwrap();
/// # let reporter_channel = generic_channel::channel().unwrap();
/// # let private_id = rx_set.add(private_receiver);
/// # let public_id = rx_set.add(public_receiver);
/// # let reporter_id = rx_set.add(memory_reporter);
/// # for received in rx_set.select().into_iter() {
/// #     match received {
/// #         GenericSelectionResult::ChannelClosed(_) => continue,
/// #         GenericSelectionResult::Error => println!("Found selection error"),
/// #         GenericSelectionResult::MessageReceived(id, msg) => {
/// #     }
/// # }
/// ```
pub struct GenericReceiverSet<T: Serialize + for<'de> Deserialize<'de>>(
    GenericReceiverSetVariants<T>,
);

impl<T: Serialize + for<'de> Deserialize<'de>> Default for GenericReceiverSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
enum GenericReceiverSetVariants<T: for<'de> Deserialize<'de>> {
    Ipc(ipc_channel::ipc::IpcReceiverSet),
    Crossbeam(Vec<crossbeam_channel::Receiver<Result<T, ipc_channel::IpcError>>>),
}

#[cfg(test)]
pub fn create_ipc_receiver_set<T: Serialize + for<'de> serde::Deserialize<'de>>()
-> GenericReceiverSet<T> {
    GenericReceiverSet(GenericReceiverSetVariants::Ipc(
        IpcReceiverSet::new().expect("Could not create ipc receiver"),
    ))
}

#[cfg(test)]
pub fn create_crossbeam_receiver_set<T: Serialize + for<'de> serde::Deserialize<'de>>()
-> GenericReceiverSet<T> {
    GenericReceiverSet(GenericReceiverSetVariants::Crossbeam(vec![]))
}

/// Result for readable events returned from [GenericReceiverSet::select].
#[derive(Debug, PartialEq)]
pub enum GenericSelectionResult<T> {
    /// A message received from the [`GenericReceiver`],
    /// identified by the `u64` value and Deserialized into a 'T'.
    MessageReceived(u64, T),
    /// The channel has been closed for the [GenericReceiver] identified by the `u64` value.
    ChannelClosed(u64),
    /// An error occured decoding the message.
    Error(String),
}

impl<T: Serialize + for<'de> serde::Deserialize<'de>> From<IpcSelectionResult>
    for GenericSelectionResult<T>
{
    fn from(value: IpcSelectionResult) -> Self {
        match value {
            IpcSelectionResult::MessageReceived(channel_id, ipc_message) => {
                match ipc_message.to() {
                    Ok(value) => GenericSelectionResult::MessageReceived(channel_id, value),
                    Err(error) => GenericSelectionResult::Error(error.to_string()),
                }
            },
            IpcSelectionResult::ChannelClosed(channel_id) => {
                GenericSelectionResult::ChannelClosed(channel_id)
            },
        }
    }
}

impl<T: Serialize + for<'de> Deserialize<'de>> GenericReceiverSet<T> {
    /// Create a new ReceiverSet.
    pub fn new() -> GenericReceiverSet<T> {
        if servo_config::opts::get().multiprocess || servo_config::opts::get().force_ipc {
            GenericReceiverSet(GenericReceiverSetVariants::Ipc(
                IpcReceiverSet::new().expect("Could not create ipc receiver"),
            ))
        } else {
            GenericReceiverSet(GenericReceiverSetVariants::Crossbeam(vec![]))
        }
    }

    /// Add a receiver to the set.
    pub fn add(&mut self, receiver: GenericReceiver<T>) -> u64 {
        match (&mut self.0, receiver.0) {
            (
                GenericReceiverSetVariants::Ipc(ipc_receiver_set),
                GenericReceiverVariants::Ipc(ipc_receiver),
            ) => ipc_receiver_set
                .add(ipc_receiver)
                .expect("Could not add channel"),
            (GenericReceiverSetVariants::Ipc(_), GenericReceiverVariants::Crossbeam(_)) => {
                unreachable!()
            },
            (GenericReceiverSetVariants::Crossbeam(_), GenericReceiverVariants::Ipc(_)) => {
                unreachable!()
            },
            (
                GenericReceiverSetVariants::Crossbeam(receivers),
                GenericReceiverVariants::Crossbeam(receiver),
            ) => {
                let index = receivers.len() as u64;
                receivers.push(receiver);
                index
            },
        }
    }

    /// Block until at least one of the Receivers receives a message and return a vector of the received messages.
    pub fn select(&mut self) -> Vec<GenericSelectionResult<T>> {
        match &mut self.0 {
            GenericReceiverSetVariants::Ipc(ipc_receiver_set) => ipc_receiver_set
                .select()
                .map(|result_value| {
                    result_value
                        .into_iter()
                        .map(|selection_result| selection_result.into())
                        .collect()
                })
                .unwrap_or_else(|e| vec![GenericSelectionResult::Error(e.to_string())]),
            GenericReceiverSetVariants::Crossbeam(receivers) => {
                let mut sel = crossbeam_channel::Select::new();
                // we need to add all the receivers to the set
                let _selected_receivers = receivers
                    .iter()
                    .map(|receiver| sel.recv(receiver))
                    .collect::<Vec<usize>>();
                let selector = sel.select();
                let index = selector.index();
                let Some(receiver) = receivers.get(index) else {
                    return vec![GenericSelectionResult::ChannelClosed(index as u64)];
                };
                let Ok(result) = selector.recv(receiver) else {
                    return vec![GenericSelectionResult::ChannelClosed(index as u64)];
                };

                vec![GenericSelectionResult::MessageReceived(
                    index.try_into().unwrap(),
                    result.unwrap(),
                )]
            },
        }
    }
}
