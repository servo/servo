/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(custom_derive)]
#![feature(plugin)]
#![feature(plugin)]
#![plugin(plugins)]

#![deny(unsafe_code)]
#![plugin(serde_macros)]

extern crate app_units;

extern crate azure;
extern crate euclid;
#[cfg(not(target_os = "windows"))]
extern crate gaol;
extern crate gfx;
extern crate gfx_traits;
extern crate gleam;
extern crate image;
extern crate ipc_channel;
extern crate layers;
extern crate layout_traits;
#[macro_use]
extern crate log;
extern crate msg;
extern crate net_traits;
#[macro_use]
extern crate profile_traits;
extern crate script_traits;
extern crate serde;
extern crate style_traits;
extern crate time;
extern crate url;
#[macro_use]
extern crate util;
extern crate webrender;
extern crate webrender_traits;

pub use compositor_thread::{CompositorEventListener, CompositorProxy, CompositorThread};
use euclid::size::{Size2D, TypedSize2D};
use gfx::paint_thread::ChromeToPaintMsg;
use gfx_traits::Epoch;
use ipc_channel::ipc::{IpcSender};
use layout_traits::LayoutControlChan;
use msg::constellation_msg::{FrameId, Key, KeyState, KeyModifiers, LoadData};
use msg::constellation_msg::{NavigationDirection, PipelineId};
use msg::constellation_msg::{WebDriverCommandMsg, WindowSizeData, WindowSizeType};
use script_traits::ConstellationControlMsg;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use url::Url;
use util::geometry::PagePx;

mod compositor;
mod compositor_layer;
pub mod compositor_thread;
mod delayed_composition;
#[cfg(not(target_os = "windows"))]
pub mod sandboxing;
mod surface_map;
mod touch;
pub mod windowing;

/// Specifies whether the script or layout thread needs to be ticked for animation.
#[derive(Deserialize, Serialize)]
pub enum AnimationTickType {
    Script,
    Layout,
}

/// Messages from the compositor to the constellation.
#[derive(Deserialize, Serialize)]
pub enum CompositorMsg {
    Exit,
    FrameSize(PipelineId, Size2D<f32>),
    /// Request that the constellation send the FrameId corresponding to the document
    /// with the provided pipeline id
    GetFrame(PipelineId, IpcSender<Option<FrameId>>),
    /// Request that the constellation send the current pipeline id for the provided frame
    /// id, or for the root frame if this is None, over a provided channel.
    /// Also returns a boolean saying whether the document has finished loading or not.
    GetPipeline(Option<FrameId>, IpcSender<Option<(PipelineId, bool)>>),
    /// Requests that the constellation inform the compositor of the title of the pipeline
    /// immediately.
    GetPipelineTitle(PipelineId),
    InitLoadUrl(Url),
    /// Query the constellation to see if the current compositor output is stable
    IsReadyToSaveImage(HashMap<PipelineId, Epoch>),
    KeyEvent(Key, KeyState, KeyModifiers),
    LoadUrl(PipelineId, LoadData),
    Navigate(NavigationDirection),
    WindowSize(WindowSizeData, WindowSizeType),
    /// Requests that the constellation instruct layout to begin a new tick of the animation.
    TickAnimation(PipelineId, AnimationTickType),
    /// Dispatch a webdriver command
    WebDriverCommand(WebDriverCommandMsg),
}

pub struct SendableFrameTree {
    pub pipeline: CompositionPipeline,
    pub size: Option<TypedSize2D<PagePx, f32>>,
    pub children: Vec<SendableFrameTree>,
}

/// The subset of the pipeline that is needed for layer composition.
#[derive(Clone)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    pub layout_chan: LayoutControlChan,
    pub chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
}
