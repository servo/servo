/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use windowing;

use compositor;
use headless;
pub use constellation::SendableFrameTree;
use windowing::{ApplicationMethods, WindowMethods};
use platform::Application;

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
        let msg = ChangeReadyState(ready_state, pipeline_id);
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

    fn dup(&self) -> Box<ScriptListener> {
        box self.clone() as Box<ScriptListener>
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

    fn set_layer_clip_rect(&self,
                           pipeline_id: PipelineId,
                           layer_id: LayerId,
                           new_rect: Rect<uint>) {
        let new_rect = Rect(Point2D(new_rect.origin.x as f32,
                                    new_rect.origin.y as f32),
                            Size2D(new_rect.size.width as f32,
                                   new_rect.size.height as f32));
        self.chan.send(SetLayerClipRect(pipeline_id, layer_id, new_rect))
    }

    fn set_render_state(&self,
                        render_state: RenderState,
                        pipeline_id: PipelineId) {
        self.chan.send(ChangeRenderState(render_state, pipeline_id))
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
    /// Alerts the compositor that the specified layer's clipping rect has changed.
    /// If LayerId is LayerId::null() then set the pipeline root layer clipping rect.
    SetLayerClipRect(PipelineId, LayerId, Rect<f32>),
    /// Scroll a page in a window
    ScrollFragmentPoint(PipelineId, LayerId, Point2D<f32>),
    /// Requests that the compositor paint the given layer buffer set for the given page size.
    Paint(PipelineId, Epoch, Vec<(LayerId, Box<LayerBufferSet>)>),
    /// Alerts the compositor to the current status of page loading.
    ChangeReadyState(ReadyState, PipelineId),
    /// Alerts the compositor to the current status of rendering.
    ChangeRenderState(RenderState, PipelineId),
    /// Sets the channel to the current layout and render tasks, along with their id
    SetIds(SendableFrameTree, Sender<()>, ConstellationChan),
    /// The load of a page for a given URL has completed.
    LoadComplete(PipelineId, Url),
}

pub enum CompositorMode {
    Windowed(Application),
    Headless
}

pub struct CompositorTask {
    pub mode: CompositorMode,
}

impl CompositorTask {
    fn new(is_headless: bool) -> CompositorTask {
        let mode: CompositorMode = if is_headless {
            Headless
        } else {
            Windowed(ApplicationMethods::new())
        };

        CompositorTask {
            mode: mode
        }
    }

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

    pub fn create(opts: Opts,
                  port: Receiver<Msg>,
                  constellation_chan: ConstellationChan,
                  time_profiler_chan: TimeProfilerChan,
                  memory_profiler_chan: MemoryProfilerChan) {

        let compositor = CompositorTask::new(opts.headless);

        match compositor.mode {
            Windowed(ref app) => {
                compositor::IOCompositor::create(app,
                                                 opts,
                                                 port,
                                                 constellation_chan.clone(),
                                                 time_profiler_chan,
                                                 memory_profiler_chan)
            }
            Headless => {
                headless::NullCompositor::create(port,
                                                 constellation_chan.clone(),
                                                 time_profiler_chan,
                                                 memory_profiler_chan)
            }
        };
    }
}
