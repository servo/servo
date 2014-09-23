/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps reduce
//! coupling between these two components

use geom::rect::Rect;
use geom::size::TypedSize2D;
use geom::scale_factor::ScaleFactor;
use layers::geometry::DevicePixel;
use servo_util::geometry::{PagePx, ViewportPx};
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

#[deriving(PartialEq)]
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

pub struct WindowSizeData {
    /// The size of the initial layout viewport, before parsing an
    /// http://www.w3.org/TR/css-device-adapt/#initial-viewport
    pub initial_viewport: TypedSize2D<ViewportPx, f32>,

    /// The "viewing area" in page px. See `PagePx` documentation for details.
    pub visible_viewport: TypedSize2D<PagePx, f32>,

    /// The resolution of the window in dppx, not including any "pinch zoom" factor.
    pub device_pixel_ratio: ScaleFactor<ViewportPx, DevicePixel, f32>,
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
    ResizedWindowMsg(WindowSizeData),
}

/// Represents the two different ways to which a page can be navigated
#[deriving(Clone, PartialEq, Hash)]
pub enum NavigationType {
    Load,               // entered or clicked on a url
    Navigate,           // browser forward/back buttons
}

#[deriving(Clone, PartialEq, Hash)]
pub enum NavigationDirection {
    Forward,
    Back,
}

#[deriving(Clone, PartialEq, Eq, Hash)]
pub struct PipelineId(pub uint);

#[deriving(Clone, PartialEq, Eq, Hash)]
pub struct SubpageId(pub uint);
