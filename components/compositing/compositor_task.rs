/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Communication with the compositor task.

pub use windowing;
pub use constellation::SendableFrameTree;

use compositor;
use headless;
use windowing::{WindowEvent, WindowMethods};

use geom::point::Point2D;
use geom::rect::Rect;
use layers::platform::surface::{NativeCompositingGraphicsContext, NativeGraphicsMetadata};
use layers::layers::LayerBufferSet;
use msg::compositor_msg::{Epoch, LayerId, LayerProperties, FrameTreeId};
use msg::compositor_msg::{PaintListener, ScriptListener};
use msg::constellation_msg::{AnimationState, ConstellationChan, PipelineId};
use msg::constellation_msg::{Key, KeyState, KeyModifiers};
use profile_traits::mem;
use profile_traits::time;
use png;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::fmt::{Error, Formatter, Debug};
use std::rc::Rc;
use style::viewport::ViewportConstraints;
use url::Url;
use util::cursor::Cursor;

/// Sends messages to the compositor. This is a trait supplied by the port because the method used
/// to communicate with the compositor may have to kick OS event loops awake, communicate cross-
/// process, and so forth.
pub trait CompositorProxy : 'static + Send {
    /// Sends a message to the compositor.
    fn send(&mut self, msg: Msg);
    /// Clones the compositor proxy.
    fn clone_compositor_proxy(&self) -> Box<CompositorProxy+'static+Send>;
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
        match self.try_recv() {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }
    fn recv_compositor_msg(&mut self) -> Msg {
        self.recv().unwrap()
    }
}

/// Implementation of the abstract `ScriptListener` interface.
impl ScriptListener for Box<CompositorProxy+'static+Send> {
    fn scroll_fragment_point(&mut self,
                             pipeline_id: PipelineId,
                             layer_id: LayerId,
                             point: Point2D<f32>) {
        self.send(Msg::ScrollFragmentPoint(pipeline_id, layer_id, point));
    }

    fn close(&mut self) {
        let (chan, port) = channel();
        self.send(Msg::Exit(chan));
        port.recv().unwrap();
    }

    fn dup(&mut self) -> Box<ScriptListener+'static> {
        box self.clone_compositor_proxy() as Box<ScriptListener+'static>
    }

    fn set_title(&mut self, pipeline_id: PipelineId, title: Option<String>) {
        self.send(Msg::ChangePageTitle(pipeline_id, title))
    }

    fn send_key_event(&mut self, key: Key, state: KeyState, modifiers: KeyModifiers) {
        self.send(Msg::KeyEvent(key, state, modifiers));
    }
}

/// Implementation of the abstract `PaintListener` interface.
impl PaintListener for Box<CompositorProxy+'static+Send> {
    fn graphics_metadata(&mut self) -> Option<NativeGraphicsMetadata> {
        let (chan, port) = channel();
        self.send(Msg::GetGraphicsMetadata(chan));
        // If the compositor is shutting down when a paint task
        // is being created, the compositor won't respond to
        // this message, resulting in an eventual panic. Instead,
        // just return None in this case, since the paint task
        // will exit shortly and never actually be requested
        // to paint buffers by the compositor.
        port.recv().unwrap_or(None)
    }

    fn assign_painted_buffers(&mut self,
                              pipeline_id: PipelineId,
                              epoch: Epoch,
                              replies: Vec<(LayerId, Box<LayerBufferSet>)>,
                              frame_tree_id: FrameTreeId) {
        self.send(Msg::AssignPaintedBuffers(pipeline_id, epoch, replies, frame_tree_id));
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

    fn notify_paint_task_exiting(&mut self, pipeline_id: PipelineId) {
        self.send(Msg::PaintTaskExited(pipeline_id))
    }
}

/// Messages from the painting task and the constellation task to the compositor task.
pub enum Msg {
    /// Requests that the compositor shut down.
    Exit(Sender<()>),

    /// Informs the compositor that the constellation has completed shutdown.
    /// Required because the constellation can have pending calls to make
    /// (e.g. SetFrameTree) at the time that we send it an ExitMsg.
    ShutdownComplete,

    /// Requests the compositor's graphics metadata. Graphics metadata is what the painter needs
    /// to create surfaces that the compositor can see. On Linux this is the X display; on Mac this
    /// is the pixel format.
    ///
    /// The headless compositor returns `None`.
    GetGraphicsMetadata(Sender<Option<NativeGraphicsMetadata>>),

    /// Tells the compositor to create or update the layers for a pipeline if necessary
    /// (i.e. if no layer with that ID exists).
    InitializeLayersForPipeline(PipelineId, Epoch, Vec<LayerProperties>),
    /// Alerts the compositor that the specified layer's rect has changed.
    SetLayerRect(PipelineId, LayerId, Rect<f32>),
    /// Scroll a page in a window
    ScrollFragmentPoint(PipelineId, LayerId, Point2D<f32>),
    /// Requests that the compositor assign the painted buffers to the given layers.
    AssignPaintedBuffers(PipelineId, Epoch, Vec<(LayerId, Box<LayerBufferSet>)>, FrameTreeId),
    /// Alerts the compositor that the current page has changed its title.
    ChangePageTitle(PipelineId, Option<String>),
    /// Alerts the compositor that the current page has changed its URL.
    ChangePageUrl(PipelineId, Url),
    /// Alerts the compositor that the given pipeline has changed whether it is running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
    /// Replaces the current frame tree, typically called during main frame navigation.
    SetFrameTree(SendableFrameTree, Sender<()>, ConstellationChan),
    /// The load of a page has begun: (can go back, can go forward).
    LoadStart(bool, bool),
    /// The load of a page has completed: (can go back, can go forward).
    LoadComplete(bool, bool),
    /// Indicates that the scrolling timeout with the given starting timestamp has happened and a
    /// composite should happen. (See the `scrolling` module.)
    ScrollTimeout(u64),
    RecompositeAfterScroll,
    /// Sends an unconsumed key event back to the compositor.
    KeyEvent(Key, KeyState, KeyModifiers),
    /// Changes the cursor.
    SetCursor(Cursor),
    /// Composite to a PNG file and return the Image over a passed channel.
    CreatePng(Sender<Option<png::Image>>),
    /// Informs the compositor that the paint task for the given pipeline has exited.
    PaintTaskExited(PipelineId),
    /// Alerts the compositor that the viewport has been constrained in some manner
    ViewportConstrained(PipelineId, ViewportConstraints),
    /// A reply to the compositor asking if the output image is stable.
    IsReadyToSaveImageReply(bool),
    /// A favicon was detected
    NewFavicon(Url),
    /// <head> tag finished parsing
    HeadParsed,
}

impl Debug for Msg {
    fn fmt(&self, f: &mut Formatter) -> Result<(),Error> {
        match *self {
            Msg::Exit(..) => write!(f, "Exit"),
            Msg::ShutdownComplete(..) => write!(f, "ShutdownComplete"),
            Msg::GetGraphicsMetadata(..) => write!(f, "GetGraphicsMetadata"),
            Msg::InitializeLayersForPipeline(..) => write!(f, "InitializeLayersForPipeline"),
            Msg::SetLayerRect(..) => write!(f, "SetLayerRect"),
            Msg::ScrollFragmentPoint(..) => write!(f, "ScrollFragmentPoint"),
            Msg::AssignPaintedBuffers(..) => write!(f, "AssignPaintedBuffers"),
            Msg::ChangeRunningAnimationsState(..) => write!(f, "ChangeRunningAnimationsState"),
            Msg::ChangePageTitle(..) => write!(f, "ChangePageTitle"),
            Msg::ChangePageUrl(..) => write!(f, "ChangePageUrl"),
            Msg::SetFrameTree(..) => write!(f, "SetFrameTree"),
            Msg::LoadComplete(..) => write!(f, "LoadComplete"),
            Msg::LoadStart(..) => write!(f, "LoadStart"),
            Msg::ScrollTimeout(..) => write!(f, "ScrollTimeout"),
            Msg::RecompositeAfterScroll => write!(f, "RecompositeAfterScroll"),
            Msg::KeyEvent(..) => write!(f, "KeyEvent"),
            Msg::SetCursor(..) => write!(f, "SetCursor"),
            Msg::CreatePng(..) => write!(f, "CreatePng"),
            Msg::PaintTaskExited(..) => write!(f, "PaintTaskExited"),
            Msg::ViewportConstrained(..) => write!(f, "ViewportConstrained"),
            Msg::IsReadyToSaveImageReply(..) => write!(f, "IsReadyToSaveImageReply"),
            Msg::NewFavicon(..) => write!(f, "NewFavicon"),
            Msg::HeadParsed => write!(f, "HeadParsed"),
        }
    }
}

pub struct CompositorTask;

impl CompositorTask {
    /// Creates a graphics context. Platform-specific.
    ///
    /// FIXME(pcwalton): Probably could be less platform-specific, using the metadata abstraction.
    #[cfg(target_os="linux")]
    pub fn create_graphics_context(native_metadata: &NativeGraphicsMetadata)
                                    -> NativeCompositingGraphicsContext {
        NativeCompositingGraphicsContext::from_display(native_metadata.display)
    }
    #[cfg(not(target_os="linux"))]
    pub fn create_graphics_context(_: &NativeGraphicsMetadata)
                                    -> NativeCompositingGraphicsContext {
        NativeCompositingGraphicsContext::new()
    }

    pub fn create<Window>(window: Option<Rc<Window>>,
                          sender: Box<CompositorProxy+Send>,
                          receiver: Box<CompositorReceiver>,
                          constellation_chan: ConstellationChan,
                          time_profiler_chan: time::ProfilerChan,
                          mem_profiler_chan: mem::ProfilerChan)
                          -> Box<CompositorEventListener + 'static>
                          where Window: WindowMethods + 'static {
        match window {
            Some(window) => {
                box compositor::IOCompositor::create(window,
                                                     sender,
                                                     receiver,
                                                     constellation_chan,
                                                     time_profiler_chan,
                                                     mem_profiler_chan)
                    as Box<CompositorEventListener>
            }
            None => {
                box headless::NullCompositor::create(receiver,
                                                     constellation_chan,
                                                     time_profiler_chan,
                                                     mem_profiler_chan)
                    as Box<CompositorEventListener>
            }
        }
    }
}

pub trait CompositorEventListener {
    fn handle_events(&mut self, events: Vec<WindowEvent>) -> bool;
    fn repaint_synchronously(&mut self);
    fn shutdown(&mut self);
    fn pinch_zoom_level(&self) -> f32;
    /// Requests that the compositor send the title for the main frame as soon as possible.
    fn get_title_for_main_frame(&self);
}
