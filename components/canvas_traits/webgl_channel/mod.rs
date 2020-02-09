/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Enum wrappers to be able to select different channel implementations at runtime.

mod ipc;
mod mpsc;

use crate::webgl::WebGLMsg;
use ipc_channel::ipc::IpcSender;
use serde::{Deserialize, Serialize};
use servo_config::opts;
use std::fmt;

lazy_static! {
    static ref IS_MULTIPROCESS: bool = { opts::multiprocess() };
}

#[derive(Deserialize, Serialize)]
pub enum WebGLSender<T: Serialize> {
    Ipc(ipc::WebGLSender<T>),
    Mpsc(mpsc::WebGLSender<T>),
}

impl<T> Clone for WebGLSender<T>
where
    T: Serialize,
{
    fn clone(&self) -> Self {
        match *self {
            WebGLSender::Ipc(ref chan) => WebGLSender::Ipc(chan.clone()),
            WebGLSender::Mpsc(ref chan) => WebGLSender::Mpsc(chan.clone()),
        }
    }
}

impl<T: Serialize> fmt::Debug for WebGLSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WebGLSender(..)")
    }
}

impl<T: Serialize> WebGLSender<T> {
    #[inline]
    pub fn send(&self, msg: T) -> WebGLSendResult {
        match *self {
            WebGLSender::Ipc(ref sender) => sender.send(msg).map_err(|_| ()),
            WebGLSender::Mpsc(ref sender) => sender.send(msg).map_err(|_| ()),
        }
    }
}

pub type WebGLSendResult = Result<(), ()>;

pub enum WebGLReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    Ipc(ipc::WebGLReceiver<T>),
    Mpsc(mpsc::WebGLReceiver<T>),
}

impl<T> WebGLReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    pub fn recv(&self) -> Result<T, ()> {
        match *self {
            WebGLReceiver::Ipc(ref receiver) => receiver.recv().map_err(|_| ()),
            WebGLReceiver::Mpsc(ref receiver) => receiver.recv().map_err(|_| ()),
        }
    }

    pub fn try_recv(&self) -> Result<T, ()> {
        match *self {
            WebGLReceiver::Ipc(ref receiver) => receiver.try_recv().map_err(|_| ()),
            WebGLReceiver::Mpsc(ref receiver) => receiver.try_recv().map_err(|_| ()),
        }
    }
}

pub fn webgl_channel<T>() -> Result<(WebGLSender<T>, WebGLReceiver<T>), ()>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    if *IS_MULTIPROCESS {
        ipc::webgl_channel()
            .map(|(tx, rx)| (WebGLSender::Ipc(tx), WebGLReceiver::Ipc(rx)))
            .map_err(|_| ())
    } else {
        mpsc::webgl_channel().map(|(tx, rx)| (WebGLSender::Mpsc(tx), WebGLReceiver::Mpsc(rx)))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGLChan(pub WebGLSender<WebGLMsg>);

impl WebGLChan {
    #[inline]
    pub fn send(&self, msg: WebGLMsg) -> WebGLSendResult {
        self.0.send(msg)
    }

    pub fn to_ipc(&self) -> IpcSender<WebGLMsg> {
        match self.0 {
            WebGLSender::Ipc(ref sender) => sender.clone(),
            WebGLSender::Mpsc(ref mpsc_sender) => {
                let (sender, receiver) =
                    ipc_channel::ipc::channel().expect("IPC Channel creation failed");
                let mpsc_sender = mpsc_sender.clone();
                ipc_channel::router::ROUTER.add_route(
                    receiver.to_opaque(),
                    Box::new(move |message| {
                        if let Ok(message) = message.to() {
                            let _ = mpsc_sender.send(message);
                        }
                    }),
                );
                sender
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebGLPipeline(pub WebGLChan);

impl WebGLPipeline {
    pub fn channel(&self) -> WebGLChan {
        self.0.clone()
    }
}
