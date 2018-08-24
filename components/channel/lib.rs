extern crate crossbeam_channel;
extern crate ipc_channel;
extern crate serde;

use crossbeam_channel::{Receiver as BaseReceiver, Sender as BaseSender};
use ipc_channel::ipc::IpcReceiver;
use ipc_channel::router::ROUTER;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};


pub fn route_ipc_receiver_to_new_servo_receiver<T>(ipc_receiver: IpcReceiver<T>) -> Receiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize + Send + 'static
{
    let (servo_sender, servo_receiver) = channel();
    ROUTER.add_route(
        ipc_receiver.to_opaque(),
        Box::new(move |message| {
            drop(servo_sender.send(message.to::<T>().unwrap()))
        }),
    );
    servo_receiver
}

pub fn route_ipc_receiver_to_new_servo_sender<T>(ipc_receiver: IpcReceiver<T>, servo_sender: Sender<T>)
where
    T: for<'de> Deserialize<'de> + Serialize + Send + 'static
{
    ROUTER.add_route(
        ipc_receiver.to_opaque(),
        Box::new(move |message| {
            drop(servo_sender.send(message.to::<T>().unwrap()))
        }),
    )
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (base_sender, base_receiver) = crossbeam_channel::unbounded::<T>();
    let is_disconnected = Arc::new(AtomicBool::new(false));
    (Sender::new(base_sender, is_disconnected.clone()),
     Receiver::new(base_receiver, is_disconnected))
}

#[derive(Debug, PartialEq)]
pub enum ChannelError {
    ChannelClosedError
}

pub struct Receiver<T> {
    receiver: BaseReceiver<T>,
    is_disconnected: Arc<AtomicBool>,
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.is_disconnected.store(true, Ordering::SeqCst);
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver {
            receiver: self.receiver.clone(),
            is_disconnected: self.is_disconnected.clone(),
        }
    }
}

impl<T> Receiver<T> {
    pub fn new(receiver: BaseReceiver<T>, is_disconnected: Arc<AtomicBool>) -> Receiver<T> {
        Receiver {
            receiver,
            is_disconnected,
        }
    }

    pub fn recv(&self) -> Option<T> {
        self.receiver.recv()
    }

    pub fn try_recv(&self) -> Option<T> {
        self.receiver.try_recv()
    }

    pub fn len(&self) -> usize {
        self.receiver.len()
    }

    pub fn select(&self) -> &BaseReceiver<T> {
        &self.receiver
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.recv()
    }
}

impl<'a, T> IntoIterator for &'a Receiver<T> {
    type Item = T;
    type IntoIter = BaseReceiver<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.receiver.clone()
    }
}

pub struct Sender<T> {
    sender: BaseSender<T>,
    is_disconnected: Arc<AtomicBool>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender {
            sender: self.sender.clone(),
            is_disconnected: self.is_disconnected.clone(),
        }
    }
}

impl<T> Sender<T> {
    pub fn new(sender: BaseSender<T>, is_disconnected: Arc<AtomicBool>) -> Sender<T> {
        Sender {
            sender,
            is_disconnected,
        }
    }

    pub fn send(&self, msg: T) -> Result<(), ChannelError> {
        if self.is_disconnected.load(Ordering::SeqCst) {
            Err(ChannelError::ChannelClosedError)
        } else {
            Ok(self.sender.send(msg))
        }
    }

    pub fn len(&self) -> usize {
        self.sender.len()
    }

    pub fn select(&self) -> Option<&BaseSender<T>> {
        if self.is_disconnected.load(Ordering::SeqCst) {
            None
        } else {
            Some(&self.sender)
        }
    }
}
