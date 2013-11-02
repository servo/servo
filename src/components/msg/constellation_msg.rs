/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps reduce
/// coupling between these two components

use std::comm::{Chan, SharedChan};
use extra::url::Url;
use extra::future::Future;
use azure::azure_hl::Color;
use geom::size::Size2D;
use geom::rect::Rect;

#[deriving(Clone)]
pub struct ConstellationChan(SharedChan<Msg>);

impl ConstellationChan {
    pub fn new(chan: Chan<Msg>) -> ConstellationChan {
        ConstellationChan(SharedChan::new(chan))
    }
}

#[deriving(Eq)]
pub enum IFrameSandboxState {
    IFrameSandboxed,
    IFrameUnsandboxed
}

/// Messages from the compositor to the constellation.
pub enum Msg {
    ExitMsg(Chan<()>),
    FailureMsg(PipelineId, Option<SubpageId>),
    InitLoadUrlMsg(Url),
    FrameRectMsg(PipelineId, SubpageId, Rect<f32>),
    LoadUrlMsg(PipelineId, Url, Future<Size2D<uint>>),
    LoadIframeUrlMsg(Url, PipelineId, SubpageId, Future<Size2D<uint>>, IFrameSandboxState),
    NavigateMsg(NavigationDirection),
    RendererReadyMsg(PipelineId),
    ResizedWindowMsg(Size2D<uint>),
}

/// Represents the two different ways to which a page can be navigated
#[deriving(Clone, Eq, IterBytes)]
pub enum NavigationType {
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
