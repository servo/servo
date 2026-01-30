/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::generic_channel::{
    GenericReceiverVariants, GenericSenderVariants, GenericSenderVisitor, ReceiveResult, SendError,
    SendResult, serialize_generic_sender_variants,
};

/// The oneshot sender struct
pub struct GenericOneshotSender<T: Serialize>(GenericSenderVariants<T>);

impl<T: Serialize> GenericOneshotSender<T> {
    #[inline]
    /// Send a message across the channel
    pub fn send(self, msg: T) -> SendResult {
        match self.0 {
            GenericSenderVariants::Ipc(ref sender) => sender
                .send(msg)
                .map_err(|e| SendError::SerializationError(format!("{e}"))),
            GenericSenderVariants::Crossbeam(ref sender) => {
                sender.send(Ok(msg)).map_err(|_| SendError::Disconnected)
            },
        }
    }
}

impl<T> GenericOneshotReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    #[inline]
    /// Receive a message across the channel
    pub fn recv(self) -> ReceiveResult<T> {
        match self.0 {
            GenericReceiverVariants::Ipc(ref receiver) => Ok(receiver.recv()?),
            GenericReceiverVariants::Crossbeam(ref receiver) => {
                // `recv()` returns an error if the channel is disconnected
                let msg = receiver.recv()?;
                // `msg` must be `ok` because the corresponding [`GenericSender::Crossbeam`] will
                // unconditionally send an `Ok(T)`
                Ok(msg.expect("Infallible"))
            },
        }
    }
}

/// The oneshot receiver struct
pub struct GenericOneshotReceiver<T: Serialize + for<'de> Deserialize<'de>>(
    GenericReceiverVariants<T>,
);

/// Creates a oneshot generic channel used to send only a single message, similar tokio::sync::oneshot.
/// This is not the same as ipc_channel::oneshot.
/// The send and receive methods will consume the Sender/Receiver.
/// We will automatically select ipc or crossbeam channels.
pub fn oneshot<T>() -> Option<(GenericOneshotSender<T>, GenericOneshotReceiver<T>)>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    if servo_config::opts::get().multiprocess || servo_config::opts::get().force_ipc {
        ipc_channel::ipc::channel()
            .map(|(tx, rx)| {
                (
                    GenericOneshotSender(GenericSenderVariants::Ipc(tx)),
                    GenericOneshotReceiver(GenericReceiverVariants::Ipc(rx)),
                )
            })
            .ok()
    } else {
        let (rx, tx) = crossbeam_channel::bounded(1);
        Some((
            GenericOneshotSender(GenericSenderVariants::Crossbeam(rx)),
            GenericOneshotReceiver(GenericReceiverVariants::Crossbeam(tx)),
        ))
    }
}

impl<T: Serialize> Serialize for GenericOneshotSender<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serialize_generic_sender_variants(&self.0, s)
    }
}

impl<'a, T: Serialize + Deserialize<'a>> Deserialize<'a> for GenericOneshotSender<T> {
    fn deserialize<D>(d: D) -> Result<GenericOneshotSender<T>, D::Error>
    where
        D: Deserializer<'a>,
    {
        d.deserialize_enum(
            "GenericSender",
            &["Ipc", "Crossbeam"],
            GenericSenderVisitor {
                marker: PhantomData,
            },
        )
        .map(|variant| GenericOneshotSender(variant))
    }
}

impl<T: Serialize> fmt::Debug for GenericOneshotSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sender(..)")
    }
}
