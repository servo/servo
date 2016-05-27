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
extern crate style_traits;
extern crate time;
extern crate url;
#[macro_use]
extern crate util;
extern crate webrender;
extern crate webrender_traits;

pub use compositor_thread::{CompositorEventListener, CompositorProxy, CompositorThread};
use euclid::size::TypedSize2D;
use gfx::paint_thread::ChromeToPaintMsg;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use script_traits::{ConstellationControlMsg, LayoutControlMsg};
use std::sync::mpsc::Sender;
use util::geometry::PagePx;

mod compositor;
mod compositor_layer;
pub mod compositor_thread;
mod delayed_composition;
mod surface_map;
mod touch;
pub mod windowing;

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
    pub layout_chan: IpcSender<LayoutControlMsg>,
    pub chrome_to_paint_chan: Sender<ChromeToPaintMsg>,
}
