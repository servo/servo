/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Communication with the compositor task.

pub use windowing;
pub use constellation::{SendableFrameTree, FrameTreeDiff};

use compositor;
use headless;
use windowing::{WindowEvent, WindowMethods};

use azure::azure_hl::{SourceSurfaceMethods, Color};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use layers::platform::surface::{NativeCompositingGraphicsContext, NativeGraphicsMetadata};
use layers::layers::LayerBufferSet;
use servo_msg::compositor_msg::{Epoch, LayerId, LayerMetadata, ReadyState};
use servo_msg::compositor_msg::{RenderListener, RenderState, ScriptListener, ScrollPolicy};
use servo_msg::constellation_msg::{ConstellationChan, PipelineId};
use servo_util::memory::MemoryProfilerChan;
use servo_util::time::TimeProfilerChan;
use std::comm::{channel, Sender, Receiver};
use std::fmt::{FormatError, Formatter, Show};
use std::rc::Rc;

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
pub trait CompositorReceiver for Sized? : 'static {
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
        self.recv()
    }
}

/// Implementation of the abstract `ScriptListener` interface.
impl ScriptListener for Box<CompositorProxy+'static+Send> {
    fn set_ready_state(&mut self, pipeline_id: PipelineId, ready_state: ReadyState) {
        let msg = ChangeReadyState(pipeline_id, ready_state);
        self.send(msg);
    }

    fn scroll_fragment_point(&mut self,
                             pipeline_id: PipelineId,
                             layer_id: LayerId,
                             point: Point2D<f32>) {
        self.send(ScrollFragmentPoint(pipeline_id, layer_id, point));
    }

    fn close(&mut self) {
        let (chan, port) = channel();
        self.send(Exit(chan));
        port.recv();
    }

    fn dup(&mut self) -> Box<ScriptListener+'static> {
        box self.clone_compositor_proxy() as Box<ScriptListener+'static>
    }
}

pub struct LayerProperties {
    pub pipeline_id: PipelineId,
    pub epoch: Epoch,
    pub id: LayerId,
    pub rect: Rect<f32>,
    pub background_color: Color,
    pub scroll_policy: ScrollPolicy,
}

impl LayerProperties {
    fn new(pipeline_id: PipelineId, epoch: Epoch, metadata: &LayerMetadata) -> LayerProperties {
        LayerProperties {
            pipeline_id: pipeline_id,
            epoch: epoch,
            id: metadata.id,
            rect: Rect(Point2D(metadata.position.origin.x as f32,
                               metadata.position.origin.y as f32),
                       Size2D(metadata.position.size.width as f32,
                              metadata.position.size.height as f32)),
            background_color: metadata.background_color,
            scroll_policy: metadata.scroll_policy,
        }
    }
}

/// Implementation of the abstract `RenderListener` interface.
impl RenderListener for Box<CompositorProxy+'static+Send> {
    fn get_graphics_metadata(&mut self) -> Option<NativeGraphicsMetadata> {
        let (chan, port) = channel();
        self.send(GetGraphicsMetadata(chan));
        port.recv()
    }

    fn paint(&mut self,
             pipeline_id: PipelineId,
             epoch: Epoch,
             replies: Vec<(LayerId, Box<LayerBufferSet>)>) {
        self.send(Paint(pipeline_id, epoch, replies));
    }

    fn initialize_layers_for_pipeline(&mut self,
                                      pipeline_id: PipelineId,
                                      metadata: Vec<LayerMetadata>,
                                      epoch: Epoch) {
        // FIXME(#2004, pcwalton): This assumes that the first layer determines the page size, and
        // that all other layers are immediate children of it. This is sufficient to handle
        // `position: fixed` but will not be sufficient to handle `overflow: scroll` or transforms.
        let mut first = true;
        for metadata in metadata.iter() {
            let layer_properties = LayerProperties::new(pipeline_id, epoch, metadata);
            if first {
                self.send(CreateOrUpdateRootLayer(layer_properties));
                first = false
            } else {
                self.send(CreateOrUpdateDescendantLayer(layer_properties));
            }
        }
    }

    fn render_msg_discarded(&mut self) {
        self.send(RenderMsgDiscarded);
    }

    fn set_render_state(&mut self, pipeline_id: PipelineId, render_state: RenderState) {
        self.send(ChangeRenderState(pipeline_id, render_state))
    }
}

/// Messages from the painting task and the constellation task to the compositor task.
pub enum Msg {
    /// Requests that the compositor shut down.
    Exit(Sender<()>),

    /// Informs the compositor that the constellation has completed shutdown.
    /// Required because the constellation can have pending calls to make (e.g. SetIds)
    /// at the time that we send it an ExitMsg.
    ShutdownComplete,

    /// Requests the compositor's graphics metadata. Graphics metadata is what the renderer needs
    /// to create surfaces that the compositor can see. On Linux this is the X display; on Mac this
    /// is the pixel format.
    ///
    /// The headless compositor returns `None`.
    GetGraphicsMetadata(Sender<Option<NativeGraphicsMetadata>>),

    /// Tells the compositor to create the root layer for a pipeline if necessary (i.e. if no layer
    /// with that ID exists).
    CreateOrUpdateRootLayer(LayerProperties),
    /// Tells the compositor to create a descendant layer for a pipeline if necessary (i.e. if no
    /// layer with that ID exists).
    CreateOrUpdateDescendantLayer(LayerProperties),
    /// Alerts the compositor that the specified layer's origin has changed.
    SetLayerOrigin(PipelineId, LayerId, Point2D<f32>),
    /// Scroll a page in a window
    ScrollFragmentPoint(PipelineId, LayerId, Point2D<f32>),
    /// Requests that the compositor paint the given layer buffer set for the given page size.
    Paint(PipelineId, Epoch, Vec<(LayerId, Box<LayerBufferSet>)>),
    /// Alerts the compositor to the current status of page loading.
    ChangeReadyState(PipelineId, ReadyState),
    /// Alerts the compositor to the current status of rendering.
    ChangeRenderState(PipelineId, RenderState),
    /// Alerts the compositor that the RenderMsg has been discarded.
    RenderMsgDiscarded,
    /// Sets the channel to the current layout and render tasks, along with their id
    SetIds(SendableFrameTree, Sender<()>, ConstellationChan),
    /// Sends an updated version of the frame tree.
    FrameTreeUpdateMsg(FrameTreeDiff, Sender<()>),
    /// The load of a page has completed.
    LoadComplete,
    /// Indicates that the scrolling timeout with the given starting timestamp has happened and a
    /// composite should happen. (See the `scrolling` module.)
    ScrollTimeout(u64),
}

impl Show for Msg {
    fn fmt(&self, f: &mut Formatter) -> Result<(),FormatError> {
        match *self {
            Exit(..) => write!(f, "Exit"),
            ShutdownComplete(..) => write!(f, "ShutdownComplete"),
            GetGraphicsMetadata(..) => write!(f, "GetGraphicsMetadata"),
            CreateOrUpdateRootLayer(..) => write!(f, "CreateOrUpdateRootLayer"),
            CreateOrUpdateDescendantLayer(..) => write!(f, "CreateOrUpdateDescendantLayer"),
            SetLayerOrigin(..) => write!(f, "SetLayerOrigin"),
            ScrollFragmentPoint(..) => write!(f, "ScrollFragmentPoint"),
            Paint(..) => write!(f, "Paint"),
            ChangeReadyState(..) => write!(f, "ChangeReadyState"),
            ChangeRenderState(..) => write!(f, "ChangeRenderState"),
            RenderMsgDiscarded(..) => write!(f, "RenderMsgDiscarded"),
            SetIds(..) => write!(f, "SetIds"),
            FrameTreeUpdateMsg(..) => write!(f, "FrameTreeUpdateMsg"),
            LoadComplete => write!(f, "LoadComplete"),
            ScrollTimeout(..) => write!(f, "ScrollTimeout"),
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
                          time_profiler_chan: TimeProfilerChan,
                          memory_profiler_chan: MemoryProfilerChan)
                          -> Box<CompositorEventListener + 'static>
                          where Window: WindowMethods + 'static {
        match window {
            Some(window) => {
                box compositor::IOCompositor::create(window,
                                                     sender,
                                                     receiver,
                                                     constellation_chan.clone(),
                                                     time_profiler_chan,
                                                     memory_profiler_chan)
                    as Box<CompositorEventListener>
            }
            None => {
                box headless::NullCompositor::create(receiver,
                                                     constellation_chan.clone(),
                                                     time_profiler_chan,
                                                     memory_profiler_chan)
                    as Box<CompositorEventListener>
            }
        }
    }
}

pub trait CompositorEventListener {
    fn handle_event(&mut self, event: WindowEvent) -> bool;
    fn repaint_synchronously(&mut self);
    fn shutdown(&mut self);
}

