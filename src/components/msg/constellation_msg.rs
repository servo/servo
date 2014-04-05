/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps reduce
/// coupling between these two components

use geom::rect::Rect;
use geom::size::Size2D;
use std::comm::{channel, Sender, Receiver};
use url::Url;

#[deriving(Clone)]
pub struct ConstellationChan(pub Sender<Msg>);

impl ConstellationChan {
    pub fn new() -> (Receiver<Msg>, ConstellationChan) {
        let (chan, port) = channel();
        (port, ConstellationChan(chan))
    }
}

#[deriving(Eq)]
pub enum IFrameSandboxState {
    IFrameSandboxed,
    IFrameUnsandboxed
}

// We pass this info to various tasks, so it lives in a separate, cloneable struct.
#[deriving(Clone)]
pub struct Failure {
    pub pipeline_id: PipelineId,
    pub subpage_id: Option<SubpageId>,
}

/// Messages from the compositor and script to the constellation.
pub enum Msg {
    ExitMsg,
    FailureMsg(Failure),
    InitLoadUrlMsg(Url),
    LoadCompleteMsg(PipelineId, Url),
    FrameRectMsg(PipelineId, SubpageId, Rect<f32>),
    LoadUrlMsg(PipelineId, Url),
    LoadIframeUrlMsg(Url, PipelineId, SubpageId, IFrameSandboxState),
    NavigateMsg(NavigationDirection),
    RendererReadyMsg(PipelineId),
    ResizedWindowMsg(Size2D<uint>),
}

/// Represents the two different ways to which a page can be navigated
#[deriving(Clone, Eq, Hash)]
pub enum NavigationType {
    Load,               // entered or clicked on a url
    Navigate,           // browser forward/back buttons
}

#[deriving(Clone, Eq, Hash)]
pub enum NavigationDirection {
    Forward,
    Back,
}

#[deriving(Clone, Eq, TotalEq, Hash, Encodable)]
pub struct PipelineId(pub uint);

#[deriving(Clone, Eq, TotalEq, Hash, Encodable)]
pub struct SubpageId(pub uint);
