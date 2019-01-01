/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate log;

pub use crate::compositor::IOCompositor;
pub use crate::compositor::RenderNotifier;
pub use crate::compositor::ShutdownState;
pub use crate::compositor_thread::CompositorProxy;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use msg::constellation_msg::TopLevelBrowsingContextId;
use script_traits::{ConstellationControlMsg, LayoutControlMsg};

mod compositor;
pub mod compositor_thread;
#[cfg(feature = "gl")]
mod gl;
mod touch;
pub mod windowing;

pub struct SendableFrameTree {
    pub pipeline: CompositionPipeline,
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
