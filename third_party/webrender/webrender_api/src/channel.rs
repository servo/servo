/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{Epoch, PipelineId};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::io::{self, Cursor, Error, ErrorKind, Read};
use std::mem;

pub use crossbeam_channel as crossbeam;

#[cfg(not(target_os = "windows"))]
pub use crossbeam_channel::{Sender, Receiver};

#[cfg(target_os = "windows")]
pub use std::sync::mpsc::{Sender, Receiver};

#[derive(Clone)]
pub struct Payload {
    /// An epoch used to get the proper payload for a pipeline id frame request.
    ///
    /// TODO(emilio): Is this still relevant? We send the messages for the same
    /// pipeline in order, so we shouldn't need it. Seems like this was only
    /// wallpapering (in most cases) the underlying problem in #991.
    pub epoch: Epoch,
    /// A pipeline id to key the payload with, along with the epoch.
    pub pipeline_id: PipelineId,
    pub display_list_data: Vec<u8>,
}

impl Payload {
    /// Convert the payload to a raw byte vector, in order for it to be
    /// efficiently shared via shmem, for example.
    /// This is a helper static method working on a slice.
    pub fn construct_data(epoch: Epoch, pipeline_id: PipelineId, dl_data: &[u8]) -> Vec<u8> {
        let mut data = Vec::with_capacity(
            mem::size_of::<u32>() + 2 * mem::size_of::<u32>() + mem::size_of::<u64>() + dl_data.len(),
        );
        data.write_u32::<LittleEndian>(epoch.0).unwrap();
        data.write_u32::<LittleEndian>(pipeline_id.0).unwrap();
        data.write_u32::<LittleEndian>(pipeline_id.1).unwrap();
        data.write_u64::<LittleEndian>(dl_data.len() as u64)
            .unwrap();
        data.extend_from_slice(dl_data);
        data
    }
    /// Convert the payload to a raw byte vector, in order for it to be
    /// efficiently shared via shmem, for example.
    pub fn to_data(&self) -> Vec<u8> {
        Self::construct_data(self.epoch, self.pipeline_id, &self.display_list_data)
    }

    /// Deserializes the given payload from a raw byte vector.
    pub fn from_data(data: &[u8]) -> Payload {
        let mut payload_reader = Cursor::new(data);
        let epoch = Epoch(payload_reader.read_u32::<LittleEndian>().unwrap());
        let pipeline_id = PipelineId(
            payload_reader.read_u32::<LittleEndian>().unwrap(),
            payload_reader.read_u32::<LittleEndian>().unwrap(),
        );

        let dl_size = payload_reader.read_u64::<LittleEndian>().unwrap() as usize;
        let mut built_display_list_data = vec![0; dl_size];
        payload_reader
            .read_exact(&mut built_display_list_data[..])
            .unwrap();

        assert_eq!(payload_reader.position(), data.len() as u64);

        Payload {
            epoch,
            pipeline_id,
            display_list_data: built_display_list_data,
        }
    }
}

pub type PayloadSender = MsgSender<Payload>;

pub type PayloadReceiver = MsgReceiver<Payload>;

pub struct MsgReceiver<T> {
    rx: Receiver<T>,
}

impl<T> MsgReceiver<T> {
    pub fn recv(&self) -> Result<T, Error> {
        self.rx.recv().map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))
    }

    pub fn to_crossbeam_receiver(self) -> Receiver<T> {
        self.rx
    }
}

#[derive(Clone)]
pub struct MsgSender<T> {
    tx: Sender<T>,
}

impl<T> MsgSender<T> {
    pub fn send(&self, data: T) -> Result<(), Error> {
        self.tx.send(data).map_err(|_| Error::new(ErrorKind::Other, "cannot send on closed channel"))
    }
}

pub fn payload_channel() -> Result<(PayloadSender, PayloadReceiver), Error> {
    let (tx, rx) = unbounded_channel();
    Ok((PayloadSender { tx }, PayloadReceiver { rx }))
}

pub fn msg_channel<T>() -> Result<(MsgSender<T>, MsgReceiver<T>), Error> {
    let (tx, rx) = unbounded_channel();
    Ok((MsgSender { tx }, MsgReceiver { rx }))
}

///
/// These serialize methods are needed to satisfy the compiler
/// which uses these implementations for the recording tool.
/// The recording tool only outputs messages that don't contain
/// Senders or Receivers, so in theory these should never be
/// called in the in-process config. If they are called,
/// there may be a bug in the messages that the replay tool is writing.
///

impl<T> Serialize for MsgSender<T> {
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        unreachable!();
    }
}

impl<'de, T> Deserialize<'de> for MsgSender<T> {
    fn deserialize<D>(_: D) -> Result<MsgSender<T>, D::Error>
                      where D: Deserializer<'de> {
        unreachable!();
    }
}

/// A create a channel intended for one-shot uses, for example the channels
/// created to block on a synchronous query and then discarded,
#[cfg(not(target_os = "windows"))]
pub fn single_msg_channel<T>() -> (Sender<T>, Receiver<T>) {
    crossbeam_channel::bounded(1)
}

/// A fast MPMC message channel that can hold a fixed number of messages.
///
/// If the channel is full, the sender will block upon sending extra messages
/// until the receiver has consumed some messages.
/// The capacity parameter should be chosen either:
///  - high enough to avoid blocking on the common cases,
///  - or, on the contrary, using the blocking behavior as a means to prevent
///    fast producers from building up work faster than it is consumed.
#[cfg(not(target_os = "windows"))]
pub fn fast_channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    crossbeam_channel::bounded(capacity)
}

/// Creates an MPMC channel that is a bit slower than the fast_channel but doesn't
/// have a limit on the number of messages held at a given time and therefore
/// doesn't block when sending.
#[cfg(not(target_os = "windows"))]
pub use crossbeam_channel::unbounded as unbounded_channel;


#[cfg(target_os = "windows")]
pub fn fast_channel<T>(_cap: usize) -> (Sender<T>, Receiver<T>) {
    std::sync::mpsc::channel()
}

#[cfg(target_os = "windows")]
pub fn unbounded_channel<T>() -> (Sender<T>, Receiver<T>) {
    std::sync::mpsc::channel()
}

#[cfg(target_os = "windows")]
pub fn single_msg_channel<T>() -> (Sender<T>, Receiver<T>) {
    std::sync::mpsc::channel()
}
