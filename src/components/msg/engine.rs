/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to engine. Using this abstract interface helps reduce
/// coupling between these two components

use std::comm::{Chan, SharedChan};
use extra::net::url::Url;
use compositor::CompositorToken;

pub use compositor;

#[deriving(Clone)]
pub struct EngineChan {
    chan: SharedChan<Msg>,
}

impl EngineChan {
    pub fn new(chan: Chan<Msg>) -> EngineChan {
        EngineChan {
            chan: SharedChan::new(chan),
        }
    }
    pub fn send(&self, msg: Msg) {
        self.chan.send(msg);
    }
}

pub enum Msg {
    LoadUrlMsg(Url),
    ExitMsg(Chan<()>),
    RendererReadyMsg(uint),
    TokenSurrenderMsg(~CompositorToken),
}

