/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps reduce
/// coupling between these two components

use std::comm::{Chan, SharedChan};
use extra::url::Url;
use extra::future::Future;
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
    ExitMsg(Chan<()>),
    InitLoadUrlMsg(Url),
    LoadUrlMsg(PipelineId, Url, Future<Size2D<uint>>),
    LoadIframeUrlMsg(Url, PipelineId, SubpageId, Future<Size2D<uint>>),
    NavigateMsg(NavigationDirection),
    RendererReadyMsg(PipelineId),
    ResizedWindowBroadcast(Size2D<uint>),
}

/// Represents the two different ways to which a page can be navigated
#[deriving(Clone, Eq, IterBytes)]
enum NavigationType {
    Load,               // entered or clicked on a url
    Navigate,           // browser forward/back buttons
}

#[deriving(Clone, Eq, IterBytes)]
pub enum NavigationDirection {
    Forward,
    Back,
}

#[deriving(Clone, Eq, IterBytes)]
pub struct PipelineId(uint);
#[deriving(Clone, Eq, IterBytes)]
pub struct SubpageId(uint);
