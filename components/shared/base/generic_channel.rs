/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Enum wrappers to be able to select different channel implementations at runtime.

use std::fmt;
use std::fmt::Display;

use ipc_channel::ipc::IpcError;
use ipc_channel::router::ROUTER;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

static GENERIC_CHANNEL_USAGE_ERROR_PANIC_MSG: &str = "May not send a crossbeam channel over an IPC channel. \
     Please also convert the ipc-channel you want to send this GenericReceiver over \
     into a GenericChannel.";

pub enum GenericSender<T: Serialize> {
    Ipc(ipc_channel::ipc::IpcSender<T>),
    /// A crossbeam-channel. To keep the API in sync with the Ipc variant when using a Router,
    /// which propagates the IPC error, the inner type is a Result.
    /// In the IPC case, the Router deserializes the message, which can fail, and sends
    /// the result to a crossbeam receiver.
    /// The crossbeam channel does not involve serializing, so we can't have this error,
    /// but replicating the API allows us to have one channel type as the receiver
    /// after routing the receiver .
    Crossbeam(crossbeam_channel::Sender<Result<T, ipc_channel::Error>>),
}

impl<T: Serialize> Serialize for GenericSender<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            GenericSender::Ipc(i) => i.serialize(s),
            GenericSender::Crossbeam(_) => panic!("{GENERIC_CHANNEL_USAGE_ERROR_PANIC_MSG}"),
        }
    }
}

impl<'a, T: Serialize> Deserialize<'a> for GenericSender<T> {
    fn deserialize<D>(d: D) -> Result<GenericSender<T>, D::Error>
    where
        D: Deserializer<'a>,
    {
        // Only ipc_channel will encounter deserialize scenario.
        ipc_channel::ipc::IpcSender::<T>::deserialize(d).map(GenericSender::Ipc)
    }
}

impl<T> Clone for GenericSender<T>
where
    T: Serialize,
{
    fn clone(&self) -> Self {
        match *self {
            GenericSender::Ipc(ref chan) => GenericSender::Ipc(chan.clone()),
            GenericSender::Crossbeam(ref chan) => GenericSender::Crossbeam(chan.clone()),
        }
    }
}

impl<T: Serialize> fmt::Debug for GenericSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sender(..)")
    }
}

impl<T: Serialize> GenericSender<T> {
    #[inline]
    pub fn send(&self, msg: T) -> SendResult {
        match *self {
            GenericSender::Ipc(ref sender) => sender
                .send(msg)
                .map_err(|e| SendError::SerializationError(format!("{e}"))),
            GenericSender::Crossbeam(ref sender) => {
                sender.send(Ok(msg)).map_err(|_| SendError::Disconnected)
            },
        }
    }
}

impl<T: Serialize> MallocSizeOf for GenericSender<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

#[derive(Debug)]
pub enum SendError {
    Disconnected,
    SerializationError(String),
}

impl Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

pub type SendResult = Result<(), SendError>;

#[derive(Debug)]
pub enum ReceiveError {
    DeserializationFailed(String),
    /// Io Error. May occur when using IPC.
    Io(std::io::Error),
    /// The channel was closed.
    Disconnected,
}

impl From<IpcError> for ReceiveError {
    fn from(e: IpcError) -> Self {
        match e {
            IpcError::Disconnected => ReceiveError::Disconnected,
            IpcError::Bincode(reason) => ReceiveError::DeserializationFailed(reason.to_string()),
            IpcError::Io(reason) => ReceiveError::Io(reason),
        }
    }
}

impl From<crossbeam_channel::RecvError> for ReceiveError {
    fn from(_: crossbeam_channel::RecvError) -> Self {
        ReceiveError::Disconnected
    }
}

pub enum TryReceiveError {
    Empty,
    ReceiveError(ReceiveError),
}

impl From<ipc_channel::ipc::TryRecvError> for TryReceiveError {
    fn from(e: ipc_channel::ipc::TryRecvError) -> Self {
        match e {
            ipc_channel::ipc::TryRecvError::Empty => TryReceiveError::Empty,
            ipc_channel::ipc::TryRecvError::IpcError(inner) => {
                TryReceiveError::ReceiveError(inner.into())
            },
        }
    }
}

impl From<crossbeam_channel::TryRecvError> for TryReceiveError {
    fn from(e: crossbeam_channel::TryRecvError) -> Self {
        match e {
            crossbeam_channel::TryRecvError::Empty => TryReceiveError::Empty,
            crossbeam_channel::TryRecvError::Disconnected => {
                TryReceiveError::ReceiveError(ReceiveError::Disconnected)
            },
        }
    }
}

pub type ReceiveResult<T> = Result<T, ReceiveError>;
pub type TryReceiveResult<T> = Result<T, TryReceiveError>;

pub enum GenericReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    Ipc(ipc_channel::ipc::IpcReceiver<T>),
    Crossbeam(RoutedReceiver<T>),
}

impl<T> GenericReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    #[inline]
    pub fn recv(&self) -> ReceiveResult<T> {
        match *self {
            GenericReceiver::Ipc(ref receiver) => Ok(receiver.recv()?),
            GenericReceiver::Crossbeam(ref receiver) => {
                // `recv()` returns an error if the channel is disconnected
                let msg = receiver.recv()?;
                // `msg` must be `ok` because the corresponding [`GenericSender::Crossbeam`] will
                // unconditionally send an `Ok(T)`
                Ok(msg.expect("Infallible"))
            },
        }
    }

    #[inline]
    pub fn try_recv(&self) -> TryReceiveResult<T> {
        match *self {
            GenericReceiver::Ipc(ref receiver) => Ok(receiver.try_recv()?),
            GenericReceiver::Crossbeam(ref receiver) => {
                let msg = receiver.try_recv()?;
                Ok(msg.expect("Infallible"))
            },
        }
    }

    /// Route to a crossbeam receiver, preserving any errors.
    ///
    /// For `Crossbeam` receivers this is a no-op, while for `Ipc` receivers
    /// this creates a route.
    #[inline]
    pub fn route_preserving_errors(self) -> RoutedReceiver<T>
    where
        T: Send + 'static,
    {
        match self {
            GenericReceiver::Ipc(ipc_receiver) => {
                let (crossbeam_sender, crossbeam_receiver) = crossbeam_channel::unbounded();
                let crossbeam_sender_clone = crossbeam_sender.clone();
                ROUTER.add_typed_route(
                    ipc_receiver,
                    Box::new(move |message| {
                        let _ = crossbeam_sender_clone.send(message);
                    }),
                );
                crossbeam_receiver
            },
            GenericReceiver::Crossbeam(receiver) => receiver,
        }
    }
}

impl<T> Serialize for GenericReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            GenericReceiver::Ipc(receiver) => receiver.serialize(s),
            GenericReceiver::Crossbeam(_) => panic!("{GENERIC_CHANNEL_USAGE_ERROR_PANIC_MSG}"),
        }
    }
}

impl<'a, T> Deserialize<'a> for GenericReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    fn deserialize<D>(d: D) -> Result<GenericReceiver<T>, D::Error>
    where
        D: Deserializer<'a>,
    {
        // Only ipc_channel will encounter deserialize scenario.
        ipc_channel::ipc::IpcReceiver::<T>::deserialize(d).map(GenericReceiver::Ipc)
    }
}

/// Creates a Servo channel that can select different channel implementations based on multiprocess
/// mode or not. If the scenario doesn't require message to pass process boundary, a simple
/// crossbeam channel is preferred.
pub fn channel<T>() -> Option<(GenericSender<T>, GenericReceiver<T>)>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    if servo_config::opts::get().multiprocess || servo_config::opts::get().force_ipc {
        ipc_channel::ipc::channel()
            .map(|(tx, rx)| (GenericSender::Ipc(tx), GenericReceiver::Ipc(rx)))
            .ok()
    } else {
        let (tx, rx) = crossbeam_channel::unbounded();
        Some((GenericSender::Crossbeam(tx), GenericReceiver::Crossbeam(rx)))
    }
}
pub type RoutedReceiver<T> = crossbeam_channel::Receiver<Result<T, ipc_channel::Error>>;
