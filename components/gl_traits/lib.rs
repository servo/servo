/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![crate_name = "gl_traits"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

#[macro_use]
extern crate serde;

use canvas_traits::webgl::{WebGLMsg, WebGLSender};
use euclid::Size2D;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use std::rc::Rc;
use webrender_api::{ExternalImageId, PipelineId};

#[derive(Clone, Deserialize, Serialize)]
pub enum WebrenderImageHandlerChannel {
    WebGL(WebGLSender<WebGLMsg>),
}

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum WebrenderImageId {
    External(ExternalImageId),
    Output(PipelineId),
}

#[derive(Clone, Deserialize, Serialize)]
pub struct WebrenderImageHandler(pub WebrenderImageId, pub WebrenderImageHandlerChannel);

#[derive(Deserialize, Serialize)]
pub enum WebrenderImageHandlersMsg {
    Register(WebrenderImageHandler),
    Unregister(WebrenderImageId),
}

#[derive(Clone, Deserialize, Serialize)]
pub struct WebrenderImageHandlerLockChannel(
    pub IpcSender<Option<(u32, Size2D<i32>, Option<usize>)>>,
    pub Rc<IpcReceiver<Option<(u32, Size2D<i32>, Option<usize>)>>>,
);

impl WebrenderImageHandlerLockChannel {
    pub fn new() -> Self {
        let (sender, recv) = ipc::channel().expect("ipc channel failure");
        Self(sender, Rc::new(recv))
    }
}
