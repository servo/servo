/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcSender, OpaqueIpcSender};
use ipc_channel::router::ROUTER;
use opts;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::intrinsics::type_name;
use std::io::{Error, ErrorKind};
use std::marker::Reflect;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};

lazy_static! {
    static ref IN_PROCESS_SENDERS: Mutex<HashMap<usize, OpaqueSender>> =
        Mutex::new(HashMap::new());
}

static NEXT_SENDER_ID: AtomicUsize = ATOMIC_USIZE_INIT;

pub enum OptionalIpcSender<T> where T: Deserialize + Serialize + Send + Any {
    OutOfProcess(IpcSender<T>),
    InProcess(Sender<T>),
}

impl<T> OptionalIpcSender<T> where T: Deserialize + Serialize + Send + Any {
    pub fn send(&self, value: T) -> Result<(), Error> {
        match *self {
            OptionalIpcSender::OutOfProcess(ref ipc_sender) => ipc_sender.send(value),
            OptionalIpcSender::InProcess(ref sender) => {
                sender.send(value).map_err(|_| Error::new(ErrorKind::Other, "MPSC send failed"))
            }
        }
    }

    pub fn to_opaque(self) -> OptionalOpaqueIpcSender {
        match self {
            OptionalIpcSender::OutOfProcess(ipc_sender) => {
                OptionalOpaqueIpcSender::OutOfProcess(ipc_sender.to_opaque())
            }
            OptionalIpcSender::InProcess(sender) => {
                OptionalOpaqueIpcSender::InProcess(OpaqueSender::new(sender))
            }
        }
    }
}

impl<T> Clone for OptionalIpcSender<T> where T: Deserialize + Serialize + Send + Any {
    fn clone(&self) -> OptionalIpcSender<T> {
        match *self {
            OptionalIpcSender::OutOfProcess(ref ipc_sender) => {
                OptionalIpcSender::OutOfProcess((*ipc_sender).clone())
            }
            OptionalIpcSender::InProcess(ref sender) => {
                OptionalIpcSender::InProcess((*sender).clone())
            }
        }
    }
}

impl<T> Deserialize for OptionalIpcSender<T> where T: Deserialize + Serialize + Send + Any {
    fn deserialize<D>(deserializer: &mut D)
                      -> Result<OptionalIpcSender<T>, D::Error> where D: Deserializer {
        if opts::multiprocess() {
            return Ok(OptionalIpcSender::OutOfProcess(try!(Deserialize::deserialize(
                            deserializer))))
        }
        let id: usize = try!(Deserialize::deserialize(deserializer));
        let sender = IN_PROCESS_SENDERS.lock().unwrap().remove(&id).unwrap();
        Ok(OptionalIpcSender::InProcess(sender.to().unwrap()))
     }
}

impl<T> Serialize for OptionalIpcSender<T> where T: Deserialize + Serialize + Send + Any {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        match *self {
            OptionalIpcSender::OutOfProcess(ref ipc_sender) => ipc_sender.serialize(serializer),
            OptionalIpcSender::InProcess(ref sender) => {
                let id = NEXT_SENDER_ID.fetch_add(1, Ordering::SeqCst);
                IN_PROCESS_SENDERS.lock()
                                  .unwrap()
                                  .insert(id, OpaqueSender::new((*sender).clone()));
                id.serialize(serializer)
            }
        }
    }
}

#[derive(Clone)]
pub enum OptionalOpaqueIpcSender {
    OutOfProcess(OpaqueIpcSender),
    InProcess(OpaqueSender),
}

impl OptionalOpaqueIpcSender {
    pub fn to<T>(self) -> OptionalIpcSender<T>
                 where T: Deserialize + Serialize + Send + Any + 'static {
        match self {
            OptionalOpaqueIpcSender::OutOfProcess(ipc_sender) => {
                OptionalIpcSender::OutOfProcess(ipc_sender.to())
            }
            OptionalOpaqueIpcSender::InProcess(sender) => {
                OptionalIpcSender::InProcess(sender.to().unwrap())
            }
        }
    }
}

impl Deserialize for OptionalOpaqueIpcSender {
    fn deserialize<D>(deserializer: &mut D)
                      -> Result<OptionalOpaqueIpcSender, D::Error> where D: Deserializer {
        if opts::multiprocess() {
            return Ok(OptionalOpaqueIpcSender::OutOfProcess(try!(Deserialize::deserialize(
                            deserializer))))
        }
        let id: usize = try!(Deserialize::deserialize(deserializer));
        let sender = IN_PROCESS_SENDERS.lock().unwrap().remove(&id).unwrap();
        Ok(OptionalOpaqueIpcSender::InProcess(sender))
     }
}

impl Serialize for OptionalOpaqueIpcSender {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        match *self {
            OptionalOpaqueIpcSender::OutOfProcess(ref ipc_sender) => {
                ipc_sender.serialize(serializer)
            }
            OptionalOpaqueIpcSender::InProcess(ref sender) => {
                let id = NEXT_SENDER_ID.fetch_add(1, Ordering::SeqCst);
                IN_PROCESS_SENDERS.lock().unwrap().insert(id, (*sender).clone());
                id.serialize(serializer)
            }
        }
    }
}

#[derive(Clone)]
pub struct OpaqueSender {
    sender: Sender<()>,
    id: TypeId,
}

impl OpaqueSender {
    fn new<T>(sender: Sender<T>) -> OpaqueSender where T: 'static + Reflect + Send {
        unsafe {
            OpaqueSender {
                sender: mem::transmute::<_, Sender<()>>(sender),
                id: TypeId::of::<T>(),
            }
        }
    }

    fn to<T>(self) -> Option<Sender<T>> where T: 'static + Reflect + Send {
        unsafe {
            if self.id != TypeId::of::<T>() {
                None
            } else {
                Some(mem::transmute::<_, Sender<T>>(self.sender))
            }
        }
    }
}

pub fn optional_ipc_channel<T>() -> (OptionalIpcSender<T>, Receiver<T>)
                                 where T: Deserialize + Serialize + Send + Any {
    if opts::multiprocess() {
        let (ipc_sender, ipc_receiver) = ipc::channel().unwrap();
        let receiver = ROUTER.route_ipc_receiver_to_new_mpsc_receiver(ipc_receiver);
        (OptionalIpcSender::OutOfProcess(ipc_sender), receiver)
    } else {
        let (sender, receiver) = mpsc::channel();
        (OptionalIpcSender::InProcess(sender), receiver)
    }
}

pub struct Unserializable<T>(T);

impl<T> Unserializable<T> {
    pub fn new(inner: T) -> Unserializable<T> {
        Unserializable(inner)
    }

    pub fn get(self) -> T {
        self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for Unserializable<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unserializable({:?})", self.0)
    }
}

impl<T: Clone> Clone for Unserializable<T> {
    fn clone(&self) -> Self {
        Unserializable(self.0.clone())
    }
}

impl<T: Copy> Copy for Unserializable<T> {}

impl<T: PartialEq> PartialEq for Unserializable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: Eq> Eq for Unserializable<T> {}

impl<T> Serialize for Unserializable<T> {
    fn serialize<S>(&self, _: &mut S) -> Result<(), S::Error> where S: Serializer {
        panic!("Can't serialize a `Unserializable({})` struct", unsafe { type_name::<T>() });
    }
}

impl<T> Deserialize for Unserializable<T> {
    fn deserialize<D>(_: &mut D) -> Result<Unserializable<T>, D::Error> where D: Deserializer {
        panic!("Can't deserialize a `Unserializable({})` struct", unsafe { type_name::<T>() });
    }
}

impl<T> Deref for Unserializable<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Unserializable<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
