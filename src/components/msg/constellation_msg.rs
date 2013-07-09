/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps reduce
/// coupling between these two components

use std::comm::{Chan, SharedChan};
use extra::net::url::Url;
use geom::size::Size2D;

#[deriving(Clone)]
pub struct ConstellationChan {
    chan: SharedChan<Msg>,
}

impl ConstellationChan {
    pub fn new(chan: Chan<Msg>) -> ConstellationChan {
        ConstellationChan {
            chan: SharedChan::new(chan),
        }
    }
    pub fn send(&self, msg: Msg) {
        self.chan.send(msg);
    }
}

pub enum Msg {
    LoadUrlMsg(Url),
    NavigateMsg(NavigationDirection),
    ExitMsg(Chan<()>),
    RendererReadyMsg(uint),
    CompositorAck(uint),
    ResizedWindowBroadcast(Size2D<uint>),
}

/// Represents the two different ways to which a page can be navigated
enum NavigationType {
    Load,               // entered or clicked on a url
    Navigate,           // browser forward/back buttons
}

pub enum NavigationDirection {
    Forward,
    Back,
}
