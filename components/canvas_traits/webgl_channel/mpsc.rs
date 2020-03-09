/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};

#[macro_use]
macro_rules! unreachable_serializable {
    ($name:ident) => {
        impl<T> Serialize for $name<T> {
            fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                unreachable!();
            }
        }

        impl<'a, T> Deserialize<'a> for $name<T> {
            fn deserialize<D>(_: D) -> Result<$name<T>, D::Error>
            where
                D: Deserializer<'a>,
            {
                unreachable!();
            }
        }
    };
}

pub struct WebGLSender<T>(crossbeam_channel::Sender<T>);
pub struct WebGLReceiver<T>(crossbeam_channel::Receiver<T>);

impl<T> Clone for WebGLSender<T> {
    fn clone(&self) -> Self {
        WebGLSender(self.0.clone())
    }
}

impl<T> WebGLSender<T> {
    #[inline]
    pub fn send(&self, data: T) -> Result<(), crossbeam_channel::SendError<T>> {
        self.0.send(data)
    }
}

impl<T> WebGLReceiver<T> {
    #[inline]
    pub fn recv(&self) -> Result<T, crossbeam_channel::RecvError> {
        self.0.recv()
    }
    #[inline]
    pub fn try_recv(&self) -> Result<T, crossbeam_channel::TryRecvError> {
        self.0.try_recv()
    }
    pub fn into_inner(self) -> crossbeam_channel::Receiver<T> {
        self.0
    }
}

pub fn webgl_channel<T>() -> Result<(WebGLSender<T>, WebGLReceiver<T>), ()> {
    let (sender, receiver) = crossbeam_channel::unbounded();
    Ok((WebGLSender(sender), WebGLReceiver(receiver)))
}

unreachable_serializable!(WebGLReceiver);
unreachable_serializable!(WebGLSender);
