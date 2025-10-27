/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use base::generic_channel::RoutedReceiver;
use compositing_traits::rendering_context::RenderingContext;
use compositing_traits::{CompositorMsg, CompositorProxy};
use constellation_traits::EmbedderToConstellationMessage;
use crossbeam_channel::Sender;
use embedder_traits::{EventLoopWaker, RefreshDriver, ShutdownState};
use profile_traits::{mem, time};
#[cfg(feature = "webxr")]
use webxr::WebXrRegistry;

pub use crate::compositor::{IOCompositor, WebRenderDebugOption};

#[macro_use]
mod tracing;

mod compositor;
mod painter;
mod pinch_zoom;
mod pipeline_details;
mod refresh_driver;
mod render_notifier;
mod screenshot;
mod touch;
mod webview_manager;
mod webview_renderer;

/// Data used to construct a compositor.
pub struct InitialCompositorState {
    /// A channel to the compositor.
    pub compositor_proxy: CompositorProxy,
    /// A port on which messages inbound to the compositor can be received.
    pub receiver: RoutedReceiver<CompositorMsg>,
    /// A channel to the constellation.
    pub embedder_to_constellation_sender: Sender<EmbedderToConstellationMessage>,
    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,
    /// A shared state which tracks whether Servo has started or has finished
    /// shutting down.
    pub shutdown_state: Rc<Cell<ShutdownState>>,
    /// The target [`RenderingContext`] of this renderer.
    pub rendering_context: Rc<dyn RenderingContext>,
    /// An [`EventLoopWaker`] used in order to wake up the embedder when it is
    /// time to paint.
    pub event_loop_waker: Box<dyn EventLoopWaker>,
    /// An optional [`RefreshDriver`] provided by the embedder.
    pub refresh_driver: Option<Rc<dyn RefreshDriver>>,
    /// A [`PathBuf`] which can be used to override WebRender shaders.
    pub shaders_path: Option<PathBuf>,
    /// If WebXR is enabled, a [`WebXrRegistry`] to register WebXR threads.
    #[cfg(feature = "webxr")]
    pub webxr_registry: Box<dyn WebXrRegistry>,
}
