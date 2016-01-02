/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Communication with the compositor thread.

use CompositorMsg as ConstellationMsg;
use compositor::{self, CompositingReason};
use euclid::point::Point2D;
use euclid::size::Size2D;
use gfx_traits::{Epoch, FrameTreeId, LayerId, LayerProperties, PaintListener};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use layers::layers::{BufferRequest, LayerBufferSet};
use layers::platform::surface::{NativeDisplay, NativeSurface};
use msg::constellation_msg::{Image, Key, KeyModifiers, KeyState, PipelineId};
use profile_traits::mem;
use profile_traits::time;
use script_traits::{AnimationState, EventResult, ScriptToCompositorMsg};
use std::fmt::{Debug, Error, Formatter};
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use style_traits::cursor::Cursor;
use style_traits::viewport::ViewportConstraints;
use url::Url;
use windowing::{WindowEvent, WindowMethods};
pub use constellation::SendableFrameTree;
pub use windowing;
use webrender;
use webrender_traits;

/// Sends messages to the compositor. This is a trait supplied by the port because the method used
/// to communicate with the compositor may have to kick OS event loops awake, communicate cross-
/// process, and so forth.
pub trait CompositorProxy : 'static + Send {
    /// Sends a message to the compositor.
    fn send(&self, msg: Msg);
    /// Clones the compositor proxy.
    fn clone_compositor_proxy(&self) -> Box<CompositorProxy + 'static + Send>;
}

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

pub fn run_script_listener_thread(compositor_proxy: Box<CompositorProxy + 'static + Send>,
                                  receiver: IpcReceiver<ScriptToCompositorMsg>) {
    while let Ok(msg) = receiver.recv() {
        match msg {
            ScriptToCompositorMsg::ScrollFragmentPoint(pipeline_id, layer_id, point, smooth) => {
                compositor_proxy.send(Msg::ScrollFragmentPoint(pipeline_id,
                                                               layer_id,
                                                               point,
                                                               smooth));
            }

            ScriptToCompositorMsg::GetClientWindow(send) => {
                compositor_proxy.send(Msg::GetClientWindow(send));
            }

            ScriptToCompositorMsg::MoveTo(point) => {
                compositor_proxy.send(Msg::MoveTo(point));
            }

            ScriptToCompositorMsg::ResizeTo(size) => {
                compositor_proxy.send(Msg::ResizeTo(size));
            }

            ScriptToCompositorMsg::Exit => {
                let (chan, port) = ipc::channel().unwrap();
                compositor_proxy.send(Msg::Exit(chan));
                port.recv().unwrap();
            }

            ScriptToCompositorMsg::SetTitle(pipeline_id, title) => {
                compositor_proxy.send(Msg::ChangePageTitle(pipeline_id, title))
            }

            ScriptToCompositorMsg::SendKeyEvent(key, key_state, key_modifiers) => {
                compositor_proxy.send(Msg::KeyEvent(key, key_state, key_modifiers))
            }

            ScriptToCompositorMsg::TouchEventProcessed(result) => {
                compositor_proxy.send(Msg::TouchEventProcessed(result))
            }

            ScriptToCompositorMsg::Exited => break,
        }
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

/// Implementation of the abstract `PaintListener` interface.
impl PaintListener for Box<CompositorProxy + 'static + Send> {
    fn native_display(&mut self) -> Option<NativeDisplay> {
        let (chan, port) = channel();
        self.send(Msg::GetNativeDisplay(chan));
        // If the compositor is shutting down when a paint thread
        // is being created, the compositor won't respond to
        // this message, resulting in an eventual panic. Instead,
        // just return None in this case, since the paint thread
        // will exit shortly and never actually be requested
        // to paint buffers by the compositor.
        port.recv().ok()
    }

    fn assign_painted_buffers(&mut self,
                              pipeline_id: PipelineId,
                              epoch: Epoch,
                              replies: Vec<(LayerId, Box<LayerBufferSet>)>,
                              frame_tree_id: FrameTreeId) {
        self.send(Msg::AssignPaintedBuffers(pipeline_id, epoch, replies, frame_tree_id));
    }

    fn ignore_buffer_requests(&mut self, buffer_requests: Vec<BufferRequest>) {
        let mut native_surfaces = Vec::new();
        for request in buffer_requests.into_iter() {
            if let Some(native_surface) = request.native_surface {
                native_surfaces.push(native_surface);
            }
        }
        if !native_surfaces.is_empty() {
            self.send(Msg::ReturnUnusedNativeSurfaces(native_surfaces));
        }
    }

    fn initialize_layers_for_pipeline(&mut self,
                                      pipeline_id: PipelineId,
                                      properties: Vec<LayerProperties>,
                                      epoch: Epoch) {
        // FIXME(#2004, pcwalton): This assumes that the first layer determines the page size, and
        // that all other layers are immediate children of it. This is sufficient to handle
        // `position: fixed` but will not be sufficient to handle `overflow: scroll` or transforms.
        self.send(Msg::InitializeLayersForPipeline(pipeline_id, epoch, properties));
    }

    fn notify_paint_thread_exiting(&mut self, pipeline_id: PipelineId) {
        self.send(Msg::PaintThreadExited(pipeline_id))
    }
}

/// Messages from the painting thread and the constellation thread to the compositor thread.
pub enum Msg {
    /// Requests that the compositor shut down.
    Exit(IpcSender<()>),

    /// Informs the compositor that the constellation has completed shutdown.
    /// Required because the constellation can have pending calls to make
    /// (e.g. SetFrameTree) at the time that we send it an ExitMsg.
    ShutdownComplete,

    /// Requests the compositor's graphics metadata. Graphics metadata is what the painter needs
    /// to create surfaces that the compositor can see. On Linux this is the X display; on Mac this
    /// is the pixel format.
    GetNativeDisplay(Sender<NativeDisplay>),

    /// Tells the compositor to create or update the layers for a pipeline if necessary
    /// (i.e. if no layer with that ID exists).
    InitializeLayersForPipeline(PipelineId, Epoch, Vec<LayerProperties>),
    /// Scroll a page in a window
    ScrollFragmentPoint(PipelineId, LayerId, Point2D<f32>, bool),
    /// Requests that the compositor assign the painted buffers to the given layers.
    AssignPaintedBuffers(PipelineId, Epoch, Vec<(LayerId, Box<LayerBufferSet>)>, FrameTreeId),
    /// Alerts the compositor that the current page has changed its title.
    ChangePageTitle(PipelineId, Option<String>),
    /// Alerts the compositor that the current page has changed its URL.
    ChangePageUrl(PipelineId, Url),
    /// Alerts the compositor that the given pipeline has changed whether it is running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
    /// Replaces the current frame tree, typically called during main frame navigation.
    SetFrameTree(SendableFrameTree, IpcSender<()>, Sender<ConstellationMsg>),
    /// The load of a page has begun: (can go back, can go forward).
    LoadStart(bool, bool),
    /// The load of a page has completed: (can go back, can go forward, is root frame).
    LoadComplete(bool, bool, bool),
    /// We hit the delayed composition timeout. (See `delayed_composition.rs`.)
    DelayedCompositionTimeout(u64),
    /// Composite.
    Recomposite(CompositingReason),
    /// Sends an unconsumed key event back to the compositor.
    KeyEvent(Key, KeyState, KeyModifiers),
    /// Script has handled a touch event, and either prevented or allowed default actions.
    TouchEventProcessed(EventResult),
    /// Changes the cursor.
    SetCursor(Cursor),
    /// Composite to a PNG file and return the Image over a passed channel.
    CreatePng(IpcSender<Option<Image>>),
    /// Informs the compositor that the paint thread for the given pipeline has exited.
    PaintThreadExited(PipelineId),
    /// Alerts the compositor that the viewport has been constrained in some manner
    ViewportConstrained(PipelineId, ViewportConstraints),
    /// A reply to the compositor asking if the output image is stable.
    IsReadyToSaveImageReply(bool),
    /// A favicon was detected
    NewFavicon(Url),
    /// <head> tag finished parsing
    HeadParsed,
    /// Signal that the paint thread ignored the paint requests that carried
    /// these native surfaces, so that they can be re-added to the surface cache.
    ReturnUnusedNativeSurfaces(Vec<NativeSurface>),
    /// Collect memory reports and send them back to the given mem::ReportsChan.
    CollectMemoryReports(mem::ReportsChan),
    /// A status message to be displayed by the browser chrome.
    Status(Option<String>),
    /// Get Window Informations size and position
    GetClientWindow(IpcSender<(Size2D<u32>, Point2D<i32>)>),
    /// Move the window to a point
    MoveTo(Point2D<i32>),
    /// Resize the window to size
    ResizeTo(Size2D<u32>),
    /// A pipeline was shut down.
    // This message acts as a synchronization point between the constellation,
    // when it shuts down a pipeline, to the compositor; when the compositor
    // sends a reply on the IpcSender, the constellation knows it's safe to
    // tear down the other threads associated with this pipeline.
    PipelineExited(PipelineId, IpcSender<()>),
}

impl Debug for Msg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Msg::Exit(..) => write!(f, "Exit"),
            Msg::ShutdownComplete => write!(f, "ShutdownComplete"),
            Msg::GetNativeDisplay(..) => write!(f, "GetNativeDisplay"),
            Msg::InitializeLayersForPipeline(..) => write!(f, "InitializeLayersForPipeline"),
            Msg::ScrollFragmentPoint(..) => write!(f, "ScrollFragmentPoint"),
            Msg::AssignPaintedBuffers(..) => write!(f, "AssignPaintedBuffers"),
            Msg::ChangeRunningAnimationsState(..) => write!(f, "ChangeRunningAnimationsState"),
            Msg::ChangePageTitle(..) => write!(f, "ChangePageTitle"),
            Msg::ChangePageUrl(..) => write!(f, "ChangePageUrl"),
            Msg::SetFrameTree(..) => write!(f, "SetFrameTree"),
            Msg::LoadComplete(..) => write!(f, "LoadComplete"),
            Msg::LoadStart(..) => write!(f, "LoadStart"),
            Msg::DelayedCompositionTimeout(..) => write!(f, "DelayedCompositionTimeout"),
            Msg::Recomposite(..) => write!(f, "Recomposite"),
            Msg::KeyEvent(..) => write!(f, "KeyEvent"),
            Msg::TouchEventProcessed(..) => write!(f, "TouchEventProcessed"),
            Msg::SetCursor(..) => write!(f, "SetCursor"),
            Msg::CreatePng(..) => write!(f, "CreatePng"),
            Msg::PaintThreadExited(..) => write!(f, "PaintThreadExited"),
            Msg::ViewportConstrained(..) => write!(f, "ViewportConstrained"),
            Msg::IsReadyToSaveImageReply(..) => write!(f, "IsReadyToSaveImageReply"),
            Msg::NewFavicon(..) => write!(f, "NewFavicon"),
            Msg::HeadParsed => write!(f, "HeadParsed"),
            Msg::ReturnUnusedNativeSurfaces(..) => write!(f, "ReturnUnusedNativeSurfaces"),
            Msg::CollectMemoryReports(..) => write!(f, "CollectMemoryReports"),
            Msg::Status(..) => write!(f, "Status"),
            Msg::GetClientWindow(..) => write!(f, "GetClientWindow"),
            Msg::MoveTo(..) => write!(f, "MoveTo"),
            Msg::ResizeTo(..) => write!(f, "ResizeTo"),
            Msg::PipelineExited(..) => write!(f, "PipelineExited"),
        }
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
