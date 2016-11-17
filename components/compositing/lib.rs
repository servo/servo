/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(box_syntax)]
#![feature(plugin)]
#![feature(proc_macro)]
#![plugin(plugins)]

#![deny(unsafe_code)]

extern crate euclid;
extern crate gfx_traits;
extern crate gleam;
extern crate image;
extern crate ipc_channel;
#[macro_use]
extern crate log;
extern crate msg;
extern crate net_traits;
#[macro_use]
extern crate profile_traits;
extern crate script_traits;
#[macro_use]
extern crate serde_derive;
extern crate servo_url;
extern crate style_traits;
extern crate time;
#[macro_use]
extern crate util;
extern crate webrender;
extern crate webrender_traits;

pub use compositor_thread::CompositorProxy;
pub use compositor::IOCompositor;
use euclid::size::TypedSize2D;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use script_traits::{ConstellationControlMsg, LayoutControlMsg};
use style_traits::PagePx;

mod compositor;
pub mod compositor_thread;
mod delayed_composition;
mod touch;
pub mod windowing;

pub struct SendableFrameTree {
    pub pipeline: CompositionPipeline,
    pub size: Option<TypedSize2D<f32, PagePx>>,
    pub children: Vec<SendableFrameTree>,
}

/// The subset of the pipeline that is needed for layer composition.
#[derive(Clone)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    pub layout_chan: IpcSender<LayoutControlMsg>,
}
