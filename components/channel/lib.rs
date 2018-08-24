extern crate crossbeam_channel;

use crossbeam_channel::{Receiver, Sender};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};


pub fn channel<T>() -> (ServoSender<T>, ServoReceiver<T>) {
    let (base_sender, base_receiver) = crossbeam_channel::unbounded::<T>();
    let is_disconnected = Arc::new(AtomicBool::new(false));
    (ServoSender::new(base_sender, is_disconnected.clone()),
     ServoReceiver::new(base_receiver, is_disconnected))
}

#[derive(Debug, PartialEq)]
pub enum ChannelError {
    ChannelClosedError
}

pub struct ServoReceiver<T> {
    receiver: Receiver<T>,
    is_disconnected: Arc<AtomicBool>,
}

impl<T> Drop for ServoReceiver<T> {
    fn drop(&mut self) {
        self.is_disconnected.store(true, Ordering::SeqCst);
    }
}

impl<T> ServoReceiver<T> {
    pub fn new(receiver: Receiver<T>, is_disconnected: Arc<AtomicBool>) -> ServoReceiver<T> {
        ServoReceiver {
            receiver,
            is_disconnected,
        }
    }

    pub fn recv(&self) -> Option<T> {
        self.receiver.recv()
    }

    pub fn select(&self) -> &Receiver<T> {
        &self.receiver
    }
}

pub struct ServoSender<T> {
    sender: Sender<T>,
    is_disconnected: Arc<AtomicBool>,
}

impl<T> ServoSender<T> {
    pub fn new(sender: Sender<T>, is_disconnected: Arc<AtomicBool>) -> ServoSender<T> {
        ServoSender {
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

    pub fn select(&self) -> &Sender<T> {
        &self.sender
    }
}
