/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use windowing;

use compositor;
use headless;
pub use constellation::SendableFrameTree;
use windowing::WindowMethods;

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
use servo_util::opts::Opts;
use servo_util::time::TimeProfilerChan;
use std::comm::{channel, Sender, Receiver};
use std::rc::Rc;

use url::Url;

#[cfg(target_os="linux")]
use azure::azure_hl;

/// The implementation of the layers-based compositor.
#[deriving(Clone)]
pub struct CompositorChan {
    /// A channel on which messages can be sent to the compositor.
    pub chan: Sender<Msg>,
}

/// Implementation of the abstract `ScriptListener` interface.
impl ScriptListener for CompositorChan {
    fn set_ready_state(&self, pipeline_id: PipelineId, ready_state: ReadyState) {
        let msg = ChangeReadyState(pipeline_id, ready_state);
        self.chan.send(msg);
    }

    fn scroll_fragment_point(&self,
                             pipeline_id: PipelineId,
                             layer_id: LayerId,
                             point: Point2D<f32>) {
        self.chan.send(ScrollFragmentPoint(pipeline_id, layer_id, point));
    }

    fn close(&self) {
        let (chan, port) = channel();
        self.chan.send(Exit(chan));
        port.recv();
    }

    fn dup(&self) -> Box<ScriptListener+'static> {
        box self.clone() as Box<ScriptListener+'static>
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
impl RenderListener for CompositorChan {
    fn get_graphics_metadata(&self) -> Option<NativeGraphicsMetadata> {
        let (chan, port) = channel();
        self.chan.send(GetGraphicsMetadata(chan));
        port.recv()
    }

    fn paint(&self,
             pipeline_id: PipelineId,
             epoch: Epoch,
             replies: Vec<(LayerId, Box<LayerBufferSet>)>) {
        self.chan.send(Paint(pipeline_id, epoch, replies));
    }

    fn initialize_layers_for_pipeline(&self,
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
                self.chan.send(CreateOrUpdateRootLayer(layer_properties));
                first = false
            } else {
                self.chan.send(CreateOrUpdateDescendantLayer(layer_properties));
            }
        }
    }

    fn render_msg_discarded(&self) {
        self.chan.send(RenderMsgDiscarded);
    }

    fn set_render_state(&self, pipeline_id: PipelineId, render_state: RenderState) {
        self.chan.send(ChangeRenderState(pipeline_id, render_state))
    }
}

impl CompositorChan {
    pub fn new() -> (Receiver<Msg>, CompositorChan) {
        let (chan, port) = channel();
        let compositor_chan = CompositorChan {
            chan: chan,
        };
        (port, compositor_chan)
    }

    pub fn send(&self, msg: Msg) {
        self.chan.send(msg);
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
    /// The load of a page for a given URL has completed.
    LoadComplete(PipelineId, Url),
}

pub struct CompositorTask;

impl CompositorTask {
    /// Creates a graphics context. Platform-specific.
    ///
    /// FIXME(pcwalton): Probably could be less platform-specific, using the metadata abstraction.
    #[cfg(target_os="linux")]
    pub fn create_graphics_context() -> NativeCompositingGraphicsContext {
        NativeCompositingGraphicsContext::from_display(azure_hl::current_display())
    }
    #[cfg(not(target_os="linux"))]
    pub fn create_graphics_context() -> NativeCompositingGraphicsContext {
        NativeCompositingGraphicsContext::new()
    }

    pub fn create<Window: WindowMethods>(
                  window: Option<Rc<Window>>,
                  opts: Opts,
                  port: Receiver<Msg>,
                  constellation_chan: ConstellationChan,
                  time_profiler_chan: TimeProfilerChan,
                  memory_profiler_chan: MemoryProfilerChan) {

        match window {
            Some(window) => {
                compositor::IOCompositor::create(window,
                                                 opts,
                                                 port,
                                                 constellation_chan.clone(),
                                                 time_profiler_chan,
                                                 memory_profiler_chan)
            }
            None => {
                headless::NullCompositor::create(port,
                                                 constellation_chan.clone(),
                                                 time_profiler_chan,
                                                 memory_profiler_chan)
            }
        };
    }
}
