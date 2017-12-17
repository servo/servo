/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crossbeam_channel::{self, Sender, Receiver};
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
                            where D: Deserializer<'a> {
                unreachable!();
            }
        }
    };
}

#[derive(Clone)]
pub struct WebGLSender<T>(Sender<T>);
pub struct WebGLReceiver<T>(Receiver<T>);

impl<T> WebGLSender<T> {
    #[inline]
    pub fn send(&self, data: T) {
        self.0.send(data)
    }
}

impl<T> WebGLReceiver<T> {
    #[inline]
    pub fn recv(&self) -> Option<T> {
        self.0.recv()
    }
}

pub fn webgl_channel<T>() -> Result<(WebGLSender<T>, WebGLReceiver<T>), ()> {
    let (sender, receiver) = crossbeam_channel::unbounded();
    Ok((WebGLSender(sender), WebGLReceiver(receiver)))
}

unreachable_serializable!(WebGLReceiver);
unreachable_serializable!(WebGLSender);
