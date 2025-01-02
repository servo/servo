/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::mpsc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

pub struct GLPlayerSender<T>(mpsc::Sender<T>);
pub struct GLPlayerReceiver<T>(mpsc::Receiver<T>);

impl<T> Clone for GLPlayerSender<T> {
    fn clone(&self) -> Self {
        GLPlayerSender(self.0.clone())
    }
}

impl<T> GLPlayerSender<T> {
    #[inline]
    pub fn send(&self, data: T) -> Result<(), mpsc::SendError<T>> {
        self.0.send(data)
    }
}

impl<T> GLPlayerReceiver<T> {
    #[inline]
    pub fn recv(&self) -> Result<T, mpsc::RecvError> {
        self.0.recv()
    }
}

pub fn glplayer_channel<T>() -> Result<(GLPlayerSender<T>, GLPlayerReceiver<T>), ()> {
    let (sender, receiver) = mpsc::channel();
    Ok((GLPlayerSender(sender), GLPlayerReceiver(receiver)))
}

unreachable_serializable!(GLPlayerReceiver);
unreachable_serializable!(GLPlayerSender);
