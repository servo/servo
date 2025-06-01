/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use compositing_traits::rendering_context::RenderingContext;
use compositing_traits::{CompositorMsg, CompositorProxy};
use constellation_traits::EmbedderToConstellationMessage;
use crossbeam_channel::{Receiver, Sender};
use embedder_traits::{EventLoopWaker, ShutdownState};
use profile_traits::{mem, time};
use webrender::RenderApi;
use webrender_api::DocumentId;

pub use crate::compositor::{IOCompositor, WebRenderDebugOption};

#[macro_use]
mod tracing;

mod compositor;
mod refresh_driver;
mod touch;
mod webview_manager;
mod webview_renderer;

/// Data used to construct a compositor.
pub struct InitialCompositorState {
    /// A channel to the compositor.
    pub sender: CompositorProxy,
    /// A port on which messages inbound to the compositor can be received.
    pub receiver: Receiver<CompositorMsg>,
    /// A channel to the constellation.
    pub constellation_chan: Sender<EmbedderToConstellationMessage>,
    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,
    /// A shared state which tracks whether Servo has started or has finished
    /// shutting down.
    pub shutdown_state: Rc<Cell<ShutdownState>>,
    /// Instance of webrender API
    pub webrender: webrender::Renderer,
    pub webrender_document: DocumentId,
    pub webrender_api: RenderApi,
    pub rendering_context: Rc<dyn RenderingContext>,
    pub webrender_gl: Rc<dyn gleam::gl::Gl>,
    #[cfg(feature = "webxr")]
    pub webxr_main_thread: webxr::MainThreadRegistry,
    /// An [`EventLoopWaker`] used in order to wake up the embedder when it is
    /// time to paint.
    pub event_loop_waker: Box<dyn EventLoopWaker>,
}
