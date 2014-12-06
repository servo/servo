/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Communication with the compositor task.

pub use constellation::{SendableFrameTree, FrameTreeDiff};
pub use windowing;

use compositor;
use headless;
use main_thread::MainThreadProxy;
use windowing::{CompositorSupport, MouseWindowEvent, WindowMethods};

use azure::azure_hl::{SourceSurfaceMethods, Color};
use geom::point::{Point2D, TypedPoint2D};
use geom::rect::Rect;
use geom::scale_factor::ScaleFactor;
use geom::size::{Size2D, TypedSize2D};
use layers::geometry::DevicePixel;
use layers::layers::LayerBufferSet;
use layers::platform::surface::{NativeCompositingGraphicsContext, NativeGraphicsMetadata};
use servo_msg::compositor_msg::{Epoch, LayerId, LayerMetadata, PaintListener, PaintState};
use servo_msg::compositor_msg::{ReadyState, ScriptToCompositorThreadProxy, ScrollPolicy};
use servo_msg::constellation_msg::{ConstellationChan, PipelineId};
use servo_util::geometry::ScreenPx;
use servo_util::memory::MemoryProfilerChan;
use servo_util::time::TimeProfilerChan;
use std::comm::{mod, Receiver, Sender};
use std::fmt::{FormatError, Formatter, Show};

/// Sends messages to the compositor.
#[deriving(Clone)]
pub struct CompositorProxy {
    sender: Sender<Msg>,
}

impl CompositorProxy {
    pub fn send(&mut self, msg: Msg) {
        self.sender.send(msg)
    }
}

/// The port that the compositor receives messages on.
pub type CompositorReceiver = Receiver<Msg>;

/// Creates the channel to the compositor and returns both ends.
pub fn create_channel() -> (CompositorProxy, CompositorReceiver) {
    let (sender, receiver) = comm::channel();
    (CompositorProxy {
        sender: sender,
    }, receiver)
}

/// Implementation of the abstract `ScriptListener` interface.
impl ScriptToCompositorThreadProxy for CompositorProxy {
    fn set_ready_state(&mut self, pipeline_id: PipelineId, ready_state: ReadyState) {
        self.send(ChangeReadyState(pipeline_id, ready_state))
    }

    fn scroll_fragment_point(&mut self,
                             pipeline_id: PipelineId,
                             layer_id: LayerId,
                             point: Point2D<f32>) {
        self.send(ScrollFragmentPoint(pipeline_id, layer_id, point))
    }

    fn dup(&mut self) -> Box<ScriptToCompositorThreadProxy + Send> {
        box self.clone() as Box<ScriptToCompositorThreadProxy + Send>
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

/// Implementation of the abstract `PaintListener` interface.
impl PaintListener for CompositorProxy {
    fn get_graphics_metadata(&mut self) -> Option<NativeGraphicsMetadata> {
        let (chan, port) = channel();
        self.send(GetGraphicsMetadata(chan));
        port.recv()
    }

    fn paint(&mut self,
             pipeline_id: PipelineId,
             epoch: Epoch,
             replies: Vec<(LayerId, Box<LayerBufferSet>)>) {
        self.send(Paint(pipeline_id, epoch, replies))
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

    fn paint_msg_discarded(&mut self) {
        self.send(PaintMsgDiscarded);
    }

    fn set_paint_state(&mut self, pipeline_id: PipelineId, paint_state: PaintState) {
        self.send(ChangePaintState(pipeline_id, paint_state))
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

    /// Requests the compositor's graphics metadata. Graphics metadata is what the painter needs
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
    /// Alerts the compositor to the current status of painting.
    ChangePaintState(PipelineId, PaintState),
    /// Alerts the compositor that the PaintMsg has been discarded.
    PaintMsgDiscarded,
    /// Sets the channel to the current layout and paint tasks, along with their id
    SetIds(SendableFrameTree, Sender<()>, ConstellationChan),
    /// Sends an updated version of the frame tree.
    FrameTreeUpdateMsg(FrameTreeDiff, Sender<()>),
    /// The load of a page has completed.
    LoadComplete,
    /// Indicates that the scrolling timeout with the given starting timestamp has happened and a
    /// composite should happen. (See the `scrolling` module.)
    ScrollTimeout(u64),
    /// Requests that a new composite occur.
    Refresh,
    /// Alerts the compositor that the window has been resized.
    Resize(TypedSize2D<DevicePixel,uint>, ScaleFactor<ScreenPx,DevicePixel,f32>),
    /// Tells the compositor to scroll. The first element is the delta and the
    /// second element is the cursor.
    Scroll(TypedPoint2D<DevicePixel,f32>, TypedPoint2D<DevicePixel,i32>),
    /// Sends a mouse event.
    SendMouseEvent(MouseWindowEvent),
    /// Sends a mouse move event at the given point.
    SendMouseMoveEvent(TypedPoint2D<DevicePixel,f32>),
    /// Tells the compositor to pinch zoom.
    ///
    /// TODO(pcwalton): This should have an origin as well.
    PinchZoom(f32),
    /// Tells the compositor to zoom.
    Zoom(f32),
    /// Requests that a composite occur after the next paint. You must be careful when blocking on
    /// the resulting channel, as if a paint is not scheduled then you will hang forever.
    SynchronousRefresh(Sender<()>),
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
            ChangePaintState(..) => write!(f, "ChangePaintState"),
            PaintMsgDiscarded(..) => write!(f, "PaintMsgDiscarded"),
            SetIds(..) => write!(f, "SetIds"),
            FrameTreeUpdateMsg(..) => write!(f, "FrameTreeUpdateMsg"),
            LoadComplete => write!(f, "LoadComplete"),
            ScrollTimeout(..) => write!(f, "ScrollTimeout"),
            Refresh => write!(f, "Refresh"),
            Resize(..) => write!(f, "Resize"),
            Scroll(..) => write!(f, "Scroll"),
            SendMouseEvent(..) => write!(f, "SendMouseEvent"),
            SendMouseMoveEvent(..) => write!(f, "SendMouseMoveEvent"),
            PinchZoom(..) => write!(f, "PinchZoom"),
            Zoom(..) => write!(f, "Zoom"),
            SynchronousRefresh(..) => write!(f, "SynchronousRefresh"),
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

    pub fn create(state: InitialCompositorState) -> Box<CompositorEventListener + 'static> {
        if state.native_graphics_metadata.is_some() {
            box compositor::IOCompositor::create(state) as Box<CompositorEventListener>
        } else {
            box headless::NullCompositor::create(state) as Box<CompositorEventListener>
        }
    }
}

pub trait CompositorEventListener {
    fn handle_events(&mut self) -> bool;
    fn shutdown(&mut self);
}

/// Data used to construct a compositor.
pub struct InitialCompositorState {
    /// A channel to the main thread.
    pub main_thread_proxy: Box<MainThreadProxy + Send>,
    /// A channel to the compositor.
    pub sender: CompositorProxy,
    /// A port on which messages inbound to the compositor can be received.
    pub receiver: CompositorReceiver,
    /// A channel to the constellation.
    pub constellation_sender: ConstellationChan,
    /// A channel to the time profiler thread.
    pub time_profiler_sender: TimeProfilerChan,
    /// A channel to the memory profiler thread.
    pub memory_profiler_sender: MemoryProfilerChan,
    /// The initial framebuffer size of the window.
    pub window_framebuffer_size: TypedSize2D<DevicePixel,uint>,
    /// The initial device pixel ratio for the window.
    pub hidpi_factor: ScaleFactor<ScreenPx,DevicePixel,f32>,
    /// Native graphics metadata needed to create a graphics context. If `None`, this is a headless
    /// compositor.
    pub native_graphics_metadata: Option<NativeGraphicsMetadata>,
    /// The compositor support object, which is used to create off-thread compositors.
    pub compositor_support: Box<CompositorSupport + Send>,
}

