/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use base::generic_channel::RoutedReceiver;
use compositing_traits::{PaintMessage, PaintProxy};
use constellation_traits::EmbedderToConstellationMessage;
use crossbeam_channel::Sender;
use embedder_traits::{EventLoopWaker, ShutdownState};
use profile_traits::{mem, time};
#[cfg(feature = "webxr")]
use webxr::WebXrRegistry;

pub use crate::paint::{Paint, WebRenderDebugOption};

#[macro_use]
mod tracing;

mod largest_contentful_paint_calculator;
mod paint;
mod painter;
mod pinch_zoom;
mod pipeline_details;
mod refresh_driver;
mod render_notifier;
mod screenshot;
mod touch;
mod webrender_external_images;
mod webview_renderer;
#[cfg(feature = "wheel_fling")]
mod wheel_fling;

/// Data used to initialize the `Paint` subsystem.
pub struct InitialPaintState {
    /// A channel to `Paint`.
    pub paint_proxy: PaintProxy,
    /// A port on which messages inbound to `Paint` can be received.
    pub receiver: RoutedReceiver<PaintMessage>,
    /// A channel to the constellation.
    pub embedder_to_constellation_sender: Sender<EmbedderToConstellationMessage>,
    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,
    /// A shared state which tracks whether Servo has started or has finished
    /// shutting down.
    pub shutdown_state: Rc<Cell<ShutdownState>>,
    /// An [`EventLoopWaker`] used in order to wake up the embedder when it is
    /// time to paint.
    pub event_loop_waker: Box<dyn EventLoopWaker>,
    /// If WebXR is enabled, a [`WebXrRegistry`] to register WebXR threads.
    #[cfg(feature = "webxr")]
    pub webxr_registry: Box<dyn WebXrRegistry>,
}
