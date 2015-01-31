/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Communication with the compositor task.

pub use windowing;
pub use constellation::{FrameId, SendableFrameTree};

use compositor;
use headless;
use windowing::{WindowEvent, WindowMethods};

use azure::azure_hl::{SourceSurfaceMethods, Color};
use geom::point::Point2D;
use geom::rect::{Rect, TypedRect};
use geom::size::Size2D;
use layers::platform::surface::{NativeCompositingGraphicsContext, NativeGraphicsMetadata};
use layers::layers::LayerBufferSet;
use pipeline::CompositionPipeline;
use msg::compositor_msg::{Epoch, LayerId, LayerMetadata, ReadyState};
use msg::compositor_msg::{PaintListener, PaintState, ScriptListener, ScrollPolicy};
use msg::constellation_msg::{ConstellationChan, LoadData, PipelineId};
use msg::constellation_msg::{Key, KeyState, KeyModifiers};
use util::cursor::Cursor;
use util::geometry::PagePx;
use util::memory::MemoryProfilerChan;
use util::time::TimeProfilerChan;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::fmt::{Error, Formatter, Debug};
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
    fn set_ready_state(&mut self, pipeline_id: PipelineId, ready_state: ReadyState) {
        let msg = Msg::ChangeReadyState(pipeline_id, ready_state);
        self.send(msg);
    }

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

/// Information about each layer that the compositor keeps.
#[derive(Copy)]
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
impl PaintListener for Box<CompositorProxy+'static+Send> {
    fn get_graphics_metadata(&mut self) -> Option<NativeGraphicsMetadata> {
        let (chan, port) = channel();
        self.send(Msg::GetGraphicsMetadata(chan));
        port.recv().unwrap()
    }

    fn assign_painted_buffers(&mut self,
                              pipeline_id: PipelineId,
                              epoch: Epoch,
                              replies: Vec<(LayerId, Box<LayerBufferSet>)>) {
        self.send(Msg::AssignPaintedBuffers(pipeline_id, epoch, replies));
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
                self.send(Msg::CreateOrUpdateBaseLayer(layer_properties));
                first = false
            } else {
                self.send(Msg::CreateOrUpdateDescendantLayer(layer_properties));
            }
        }
    }

    fn paint_msg_discarded(&mut self) {
        self.send(Msg::PaintMsgDiscarded);
    }

    fn set_paint_state(&mut self, pipeline_id: PipelineId, paint_state: PaintState) {
        self.send(Msg::ChangePaintState(pipeline_id, paint_state))
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

    /// Tells the compositor to create the root layer for a pipeline if necessary (i.e. if no layer
    /// with that ID exists).
    CreateOrUpdateBaseLayer(LayerProperties),
    /// Tells the compositor to create a descendant layer for a pipeline if necessary (i.e. if no
    /// layer with that ID exists).
    CreateOrUpdateDescendantLayer(LayerProperties),
    /// Alerts the compositor that the specified layer's origin has changed.
    SetLayerOrigin(PipelineId, LayerId, Point2D<f32>),
    /// Scroll a page in a window
    ScrollFragmentPoint(PipelineId, LayerId, Point2D<f32>),
    /// Requests that the compositor assign the painted buffers to the given layers.
    AssignPaintedBuffers(PipelineId, Epoch, Vec<(LayerId, Box<LayerBufferSet>)>),
    /// Alerts the compositor to the current status of page loading.
    ChangeReadyState(PipelineId, ReadyState),
    /// Alerts the compositor to the current status of painting.
    ChangePaintState(PipelineId, PaintState),
    /// Alerts the compositor that the current page has changed its title.
    ChangePageTitle(PipelineId, Option<String>),
    /// Alerts the compositor that the current page has changed its load data (including URL).
    ChangePageLoadData(FrameId, LoadData),
    /// Alerts the compositor that a `PaintMsg` has been discarded.
    PaintMsgDiscarded,
    /// Replaces the current frame tree, typically called during main frame navigation.
    SetFrameTree(SendableFrameTree, Sender<()>, ConstellationChan),
    /// Requests the compositor to create a root layer for a new frame.
    CreateRootLayerForPipeline(CompositionPipeline, CompositionPipeline, Option<TypedRect<PagePx, f32>>, Sender<()>),
    /// Requests the compositor to change a root layer's pipeline and remove all child layers.
    ChangeLayerPipelineAndRemoveChildren(CompositionPipeline, CompositionPipeline, Sender<()>),
    /// The load of a page has completed.
    LoadComplete,
    /// Indicates that the scrolling timeout with the given starting timestamp has happened and a
    /// composite should happen. (See the `scrolling` module.)
    ScrollTimeout(u64),
    /// Sends an unconsumed key event back to the compositor.
    KeyEvent(Key, KeyState, KeyModifiers),
    /// Changes the cursor.
    SetCursor(Cursor),
    /// Informs the compositor that the paint task for the given pipeline has exited.
    PaintTaskExited(PipelineId),
}

impl Debug for Msg {
    fn fmt(&self, f: &mut Formatter) -> Result<(),Error> {
        match *self {
            Msg::Exit(..) => write!(f, "Exit"),
            Msg::ShutdownComplete(..) => write!(f, "ShutdownComplete"),
            Msg::GetGraphicsMetadata(..) => write!(f, "GetGraphicsMetadata"),
            Msg::CreateOrUpdateBaseLayer(..) => write!(f, "CreateOrUpdateBaseLayer"),
            Msg::CreateOrUpdateDescendantLayer(..) => write!(f, "CreateOrUpdateDescendantLayer"),
            Msg::SetLayerOrigin(..) => write!(f, "SetLayerOrigin"),
            Msg::ScrollFragmentPoint(..) => write!(f, "ScrollFragmentPoint"),
            Msg::AssignPaintedBuffers(..) => write!(f, "AssignPaintedBuffers"),
            Msg::ChangeReadyState(..) => write!(f, "ChangeReadyState"),
            Msg::ChangePaintState(..) => write!(f, "ChangePaintState"),
            Msg::ChangePageTitle(..) => write!(f, "ChangePageTitle"),
            Msg::ChangePageLoadData(..) => write!(f, "ChangePageLoadData"),
            Msg::PaintMsgDiscarded(..) => write!(f, "PaintMsgDiscarded"),
            Msg::SetFrameTree(..) => write!(f, "SetFrameTree"),
            Msg::CreateRootLayerForPipeline(..) => write!(f, "CreateRootLayerForPipeline"),
            Msg::ChangeLayerPipelineAndRemoveChildren(..) => write!(f, "ChangeLayerPipelineAndRemoveChildren"),
            Msg::LoadComplete => write!(f, "LoadComplete"),
            Msg::ScrollTimeout(..) => write!(f, "ScrollTimeout"),
            Msg::KeyEvent(..) => write!(f, "KeyEvent"),
            Msg::SetCursor(..) => write!(f, "SetCursor"),
            Msg::PaintTaskExited(..) => write!(f, "PaintTaskExited"),
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
    fn pinch_zoom_level(&self) -> f32;
    /// Requests that the compositor send the title for the main frame as soon as possible.
    fn get_title_for_main_frame(&self);
}

