/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::rc::Rc;

use compositing_traits::{CompositorProxy, CompositorReceiver, ConstellationMsg};
use crossbeam_channel::Sender;
use profile_traits::{mem, time};
use webrender::RenderApi;
use webrender_api::DocumentId;
use webrender_traits::rendering_context::RenderingContext;

pub use crate::compositor::{CompositeTarget, IOCompositor, ShutdownState};

#[macro_use]
mod tracing;

mod compositor;
mod touch;
pub mod webview;
pub mod windowing;

/// Data used to construct a compositor.
pub struct InitialCompositorState {
    /// A channel to the compositor.
    pub sender: CompositorProxy,
    /// A port on which messages inbound to the compositor can be received.
    pub receiver: CompositorReceiver,
    /// A channel to the constellation.
    pub constellation_chan: Sender<ConstellationMsg>,
    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,
    /// Instance of webrender API
    pub webrender: webrender::Renderer,
    pub webrender_document: DocumentId,
    pub webrender_api: RenderApi,
    pub rendering_context: Rc<dyn RenderingContext>,
    pub webrender_gl: Rc<dyn gleam::gl::Gl>,
    #[cfg(feature = "webxr")]
    pub webxr_main_thread: webxr::MainThreadRegistry,
}
