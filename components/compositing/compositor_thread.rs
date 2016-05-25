/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Communication with the compositor thread.

use compositing_traits::{CompositingReason, CompositorProxy, Msg};
use compositor;
use profile_traits::mem;
use profile_traits::time;
use script_traits::ConstellationMsg;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use webrender;
use webrender_traits;
use windowing::{WindowEvent, WindowMethods};

/// The port that the compositor receives messages on. As above, this is a trait supplied by the
/// Servo port.
pub trait CompositorReceiver : 'static {
    /// Receives the next message inbound for the compositor. This must not block.
    fn try_recv_compositor_msg(&mut self) -> Option<Msg>;
    /// Synchronously waits for, and returns, the next message inbound for the compositor.
    fn recv_compositor_msg(&mut self) -> Msg;
}

/// A convenience implementation of `CompositorReceiver` for a plain old Rust `Receiver`.
impl CompositorReceiver for Receiver<Msg> {
    fn try_recv_compositor_msg(&mut self) -> Option<Msg> {
        self.try_recv().ok()
    }
    fn recv_compositor_msg(&mut self) -> Msg {
        self.recv().unwrap()
    }
}

pub trait RenderListener {
    fn recomposite(&mut self, reason: CompositingReason);
}

impl RenderListener for Box<CompositorProxy + 'static> {
    fn recomposite(&mut self, reason: CompositingReason) {
        self.send(Msg::Recomposite(reason));
    }
}

pub struct CompositorThread;

impl CompositorThread {
    pub fn create<Window>(window: Rc<Window>,
                          state: InitialCompositorState)
                          -> Box<CompositorEventListener + 'static>
                          where Window: WindowMethods + 'static {
        box compositor::IOCompositor::create(window, state)
            as Box<CompositorEventListener>
    }
}

pub trait CompositorEventListener {
    fn handle_events(&mut self, events: Vec<WindowEvent>) -> bool;
    fn repaint_synchronously(&mut self);
    fn pinch_zoom_level(&self) -> f32;
    /// Requests that the compositor send the title for the main frame as soon as possible.
    fn title_for_main_frame(&self);
}

/// Data used to construct a compositor.
pub struct InitialCompositorState {
    /// A channel to the compositor.
    pub sender: Box<CompositorProxy + Send>,
    /// A port on which messages inbound to the compositor can be received.
    pub receiver: Box<CompositorReceiver>,
    /// A channel to the constellation.
    pub constellation_chan: Sender<ConstellationMsg>,
    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,
    /// Instance of webrender API if enabled
    pub webrender: Option<webrender::Renderer>,
    pub webrender_api_sender: Option<webrender_traits::RenderApiSender>,
}
