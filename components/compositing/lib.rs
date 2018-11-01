/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate log;

pub use crate::compositor::IOCompositor;
pub use crate::compositor::RenderNotifier;
pub use crate::compositor::ShutdownState;
pub use crate::compositor_thread::CompositorProxy;
use euclid::TypedSize2D;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use msg::constellation_msg::TopLevelBrowsingContextId;
use script_traits::{ConstellationControlMsg, LayoutControlMsg};
use style_traits::CSSPixel;

mod compositor;
pub mod compositor_thread;
#[cfg(feature = "gleam")]
mod gl;
mod touch;
pub mod windowing;

pub struct SendableFrameTree {
    pub pipeline: CompositionPipeline,
    pub size: Option<TypedSize2D<f32, CSSPixel>>,
    pub children: Vec<SendableFrameTree>,
}

/// The subset of the pipeline that is needed for layer composition.
#[derive(Clone)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub top_level_browsing_context_id: TopLevelBrowsingContextId,
    pub script_chan: IpcSender<ConstellationControlMsg>,
    pub layout_chan: IpcSender<LayoutControlMsg>,
}
