/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(custom_derive)]
#![feature(plugin)]
#![feature(mpsc_select)]
#![feature(plugin)]
#![plugin(plugins)]

#![deny(unsafe_code)]
#![plugin(serde_macros)]

extern crate app_units;

extern crate azure;
extern crate canvas;
extern crate canvas_traits;
extern crate clipboard;
#[cfg(target_os = "macos")]
extern crate core_graphics;
#[cfg(target_os = "macos")]
extern crate core_text;
extern crate devtools_traits;
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
extern crate num;
extern crate offscreen_gl_context;
#[macro_use]
extern crate profile_traits;
extern crate rand;
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
pub use constellation::Constellation;
use euclid::size::{Size2D};
use gfx_traits::Epoch;
use ipc_channel::ipc::{IpcSender};
use msg::constellation_msg::{FrameId, Key, KeyState, KeyModifiers, LoadData};
use msg::constellation_msg::{NavigationDirection, PipelineId, SubpageId};
use msg::constellation_msg::{WebDriverCommandMsg, WindowSizeData};
use std::collections::HashMap;
use url::Url;

mod compositor;
mod compositor_layer;
pub mod compositor_thread;
pub mod constellation;
mod delayed_composition;
pub mod pipeline;
#[cfg(not(target_os = "windows"))]
pub mod sandboxing;
mod surface_map;
mod timer_scheduler;
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
    /// id, or for the root frame if this is None, over a provided channel
    GetPipeline(Option<FrameId>, IpcSender<Option<PipelineId>>),
    /// Requests that the constellation inform the compositor of the title of the pipeline
    /// immediately.
    GetPipelineTitle(PipelineId),
    InitLoadUrl(Url),
    /// Query the constellation to see if the current compositor output is stable
    IsReadyToSaveImage(HashMap<PipelineId, Epoch>),
    KeyEvent(Key, KeyState, KeyModifiers),
    LoadUrl(PipelineId, LoadData),
    Navigate(Option<(PipelineId, SubpageId)>, NavigationDirection),
    ResizedWindow(WindowSizeData),
    /// Requests that the constellation instruct layout to begin a new tick of the animation.
    TickAnimation(PipelineId, AnimationTickType),
    /// Dispatch a webdriver command
    WebDriverCommand(WebDriverCommandMsg),
}
