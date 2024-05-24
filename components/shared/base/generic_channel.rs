/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Enum wrappers to be able to select different channel implementations at runtime.

use std::fmt;

use ipc_channel::router::ROUTER;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub enum GenericSender<T: Serialize> {
    Ipc(ipc_channel::ipc::IpcSender<T>),
    Crossbeam(crossbeam_channel::Sender<T>),
}

impl<T: Serialize> Serialize for GenericSender<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            GenericSender::Ipc(i) => i.serialize(s),
            GenericSender::Crossbeam(_) => unreachable!(),
        }
    }
}

impl<'a, T: Serialize> Deserialize<'a> for GenericSender<T> {
    fn deserialize<D>(d: D) -> Result<GenericSender<T>, D::Error>
    where
        D: Deserializer<'a>,
    {
        // Only ipc_channle will encounter deserialize scenario.
        ipc_channel::ipc::IpcSender::<T>::deserialize(d).map(|s| GenericSender::Ipc(s))
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
            GenericSender::Ipc(ref sender) => sender.send(msg).map_err(|_| SendError),
            GenericSender::Crossbeam(ref sender) => sender.send(msg).map_err(|_| SendError),
        }
    }
}

#[derive(Debug)]
pub struct SendError;
pub type SendResult = Result<(), SendError>;

#[derive(Debug)]
pub struct ReceiveError;
pub type ReceiveResult<T> = Result<T, ReceiveError>;

pub enum GenericReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    Ipc(ipc_channel::ipc::IpcReceiver<T>),
    Crossbeam(crossbeam_channel::Receiver<T>),
}

impl<T> GenericReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    pub fn recv(&self) -> ReceiveResult<T> {
        match *self {
            GenericReceiver::Ipc(ref receiver) => receiver.recv().map_err(|_| ReceiveError),
            GenericReceiver::Crossbeam(ref receiver) => receiver.recv().map_err(|_| ReceiveError),
        }
    }

    pub fn try_recv(&self) -> ReceiveResult<T> {
        match *self {
            GenericReceiver::Ipc(ref receiver) => receiver.try_recv().map_err(|_| ReceiveError),
            GenericReceiver::Crossbeam(ref receiver) => {
                receiver.try_recv().map_err(|_| ReceiveError)
            },
        }
    }

    pub fn into_inner(self) -> crossbeam_channel::Receiver<T>
    where
        T: Send + 'static,
    {
        match self {
            GenericReceiver::Ipc(receiver) => {
                ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(receiver)
            },
            GenericReceiver::Crossbeam(receiver) => receiver,
        }
    }
}

/// Creates a Servo channel that can select different channel implementations based on multiprocess
/// mode or not. If the scenario doesn't require message to pass process boundary, a simple
/// crossbeam channel is preferred.
pub fn channel<T>(multiprocess: bool) -> Option<(GenericSender<T>, GenericReceiver<T>)>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    if multiprocess {
        ipc_channel::ipc::channel()
            .map(|(tx, rx)| (GenericSender::Ipc(tx), GenericReceiver::Ipc(rx)))
            .ok()
    } else {
        let (tx, rx) = crossbeam_channel::unbounded();
        Some((GenericSender::Crossbeam(tx), GenericReceiver::Crossbeam(rx)))
    }
}
