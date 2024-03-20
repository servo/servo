/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Enum wrappers to be able to select different channel implementations at runtime.

mod ipc;
mod mpsc;

use std::fmt;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use servo_config::opts;

use crate::GLPlayerMsg;

lazy_static! {
    static ref IS_MULTIPROCESS: bool = opts::multiprocess();
}

#[derive(Deserialize, Serialize)]
pub enum GLPlayerSender<T: Serialize> {
    Ipc(ipc::GLPlayerSender<T>),
    Mpsc(mpsc::GLPlayerSender<T>),
}

impl<T> Clone for GLPlayerSender<T>
where
    T: Serialize,
{
    fn clone(&self) -> Self {
        match *self {
            GLPlayerSender::Ipc(ref chan) => GLPlayerSender::Ipc(chan.clone()),
            GLPlayerSender::Mpsc(ref chan) => GLPlayerSender::Mpsc(chan.clone()),
        }
    }
}

impl<T: Serialize> fmt::Debug for GLPlayerSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GLPlayerSender(..)")
    }
}

impl<T: Serialize> GLPlayerSender<T> {
    #[inline]
    pub fn send(&self, msg: T) -> GLPlayerSendResult {
        match *self {
            GLPlayerSender::Ipc(ref sender) => sender.send(msg).map_err(|_| ()),
            GLPlayerSender::Mpsc(ref sender) => sender.send(msg).map_err(|_| ()),
        }
    }
}

pub type GLPlayerSendResult = Result<(), ()>;

pub enum GLPlayerReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    Ipc(ipc::GLPlayerReceiver<T>),
    Mpsc(mpsc::GLPlayerReceiver<T>),
}

impl<T> GLPlayerReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    pub fn recv(&self) -> Result<T, ()> {
        match *self {
            GLPlayerReceiver::Ipc(ref receiver) => receiver.recv().map_err(|_| ()),
            GLPlayerReceiver::Mpsc(ref receiver) => receiver.recv().map_err(|_| ()),
        }
    }

    #[allow(clippy::wrong_self_convention)] // It is an alias to the underlying module
    pub fn to_opaque(self) -> ipc_channel::ipc::OpaqueIpcReceiver {
        match self {
            GLPlayerReceiver::Ipc(receiver) => receiver.to_opaque(),
            _ => unreachable!(),
        }
    }
}

pub fn glplayer_channel<T>() -> Option<(GLPlayerSender<T>, GLPlayerReceiver<T>)>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    // Let's use Ipc until we move the Player instance into GPlayerThread
    if true {
        ipc::glplayer_channel()
            .map(|(tx, rx)| (GLPlayerSender::Ipc(tx), GLPlayerReceiver::Ipc(rx)))
            .ok()
    } else {
        mpsc::glplayer_channel()
            .map(|(tx, rx)| (GLPlayerSender::Mpsc(tx), GLPlayerReceiver::Mpsc(rx)))
            .ok()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GLPlayerChan(pub GLPlayerSender<GLPlayerMsg>);

impl GLPlayerChan {
    #[inline]
    pub fn send(&self, msg: GLPlayerMsg) -> GLPlayerSendResult {
        self.0.send(msg)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GLPlayerPipeline(pub GLPlayerChan);

impl GLPlayerPipeline {
    pub fn channel(&self) -> GLPlayerChan {
        self.0.clone()
    }
}
