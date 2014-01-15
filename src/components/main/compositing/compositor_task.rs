/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use windowing;

use constellation::SendableFrameTree;
use windowing::{ApplicationMethods, WindowMethods};
use platform::Application;

use azure::azure_hl::{SourceSurfaceMethods, Color};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::opts::Opts;
use layers::platform::surface::{NativeCompositingGraphicsContext, NativeGraphicsMetadata};
use servo_msg::compositor_msg::{Epoch, RenderListener, LayerBufferSet, RenderState, ReadyState};
use servo_msg::compositor_msg::{ScriptListener, Tile};
use servo_msg::constellation_msg::{ConstellationChan, PipelineId};
use servo_util::time::ProfilerChan;
use std::comm::{Chan, SharedChan, Port};
use std::num::Orderable;

#[cfg(target_os="linux")]
use azure::azure_hl;

mod quadtree;
mod compositor_layer;

mod compositor;
mod headless;

/// The implementation of the layers-based compositor.
#[deriving(Clone)]
pub struct CompositorChan {
    /// A channel on which messages can be sent to the compositor.
    chan: SharedChan<Msg>,
}

/// Implementation of the abstract `ScriptListener` interface.
impl ScriptListener for CompositorChan {
    fn set_ready_state(&self, ready_state: ReadyState) {
        let msg = ChangeReadyState(ready_state);
        self.chan.send(msg);
    }

    fn invalidate_rect(&self, id: PipelineId, rect: Rect<uint>) {
        self.chan.send(InvalidateRect(id, rect));
    }

    fn scroll_fragment_point(&self, id: PipelineId, point: Point2D<f32>) {
	    self.chan.send(ScrollFragmentPoint(id, point));
    }

    fn close(&self) {
        let (port, chan) = Chan::new();
        self.chan.send(Exit(chan));
        port.recv();
    }

}

/// Implementation of the abstract `RenderListener` interface.
impl RenderListener for CompositorChan {
    fn get_graphics_metadata(&self) -> Option<NativeGraphicsMetadata> {
        let (port, chan) = Chan::new();
        self.chan.send(GetGraphicsMetadata(chan));
        port.recv()
    }

    fn paint(&self, id: PipelineId, layer_buffer_set: ~LayerBufferSet, epoch: Epoch) {
        self.chan.send(Paint(id, layer_buffer_set, epoch))
    }

    fn new_layer(&self, id: PipelineId, page_size: Size2D<uint>) {
        let Size2D { width, height } = page_size;
        self.chan.send(NewLayer(id, Size2D(width as f32, height as f32)))
    }
    fn set_layer_page_size_and_color(&self, id: PipelineId, page_size: Size2D<uint>, epoch: Epoch, color: Color) {
        let Size2D { width, height } = page_size;
        self.chan.send(SetUnRenderedColor(id, color));
        self.chan.send(SetLayerPageSize(id, Size2D(width as f32, height as f32), epoch))
    }

    fn set_layer_clip_rect(&self, id: PipelineId, new_rect: Rect<uint>) {
        let new_rect = Rect(Point2D(new_rect.origin.x as f32,
                                    new_rect.origin.y as f32),
                            Size2D(new_rect.size.width as f32,
                                   new_rect.size.height as f32));
        self.chan.send(SetLayerClipRect(id, new_rect))
    }

    fn delete_layer(&self, id: PipelineId) {
        self.chan.send(DeleteLayer(id))
    }

    fn set_render_state(&self, render_state: RenderState) {
        self.chan.send(ChangeRenderState(render_state))
    }
}

impl CompositorChan {
    pub fn new() -> (Port<Msg>, CompositorChan) {
        let (port, chan) = SharedChan::new();
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
    Exit(Chan<()>),

    /// Informs the compositor that the constellation has completed shutdown.
    /// Required because the constellation can have pending calls to make (e.g. SetIds)
    /// at the time that we send it an ExitMsg.
    ShutdownComplete,

    /// Requests the compositor's graphics metadata. Graphics metadata is what the renderer needs
    /// to create surfaces that the compositor can see. On Linux this is the X display; on Mac this
    /// is the pixel format.
    ///
    /// The headless compositor returns `None`.
    GetGraphicsMetadata(Chan<Option<NativeGraphicsMetadata>>),

    /// Alerts the compositor that there is a new layer to be rendered.
    NewLayer(PipelineId, Size2D<f32>),
    /// Alerts the compositor that the specified layer's page has changed size.
    SetLayerPageSize(PipelineId, Size2D<f32>, Epoch),
    /// Alerts the compositor that the specified layer's clipping rect has changed.
    SetLayerClipRect(PipelineId, Rect<f32>),
    /// Alerts the compositor that the specified layer has been deleted.
    DeleteLayer(PipelineId),
    /// Invalidate a rect for a given layer
    InvalidateRect(PipelineId, Rect<uint>),
    /// Scroll a page in a window
    ScrollFragmentPoint(PipelineId, Point2D<f32>),
    /// Requests that the compositor paint the given layer buffer set for the given page size.
    Paint(PipelineId, ~LayerBufferSet, Epoch),
    /// Alerts the compositor to the current status of page loading.
    ChangeReadyState(ReadyState),
    /// Alerts the compositor to the current status of rendering.
    ChangeRenderState(RenderState),
    /// Sets the channel to the current layout and render tasks, along with their id
    SetIds(SendableFrameTree, Chan<()>, ConstellationChan),

    SetUnRenderedColor(PipelineId, Color),
}

pub enum CompositorMode {
    Windowed(Application),
    Headless
}

pub struct CompositorTask {
    mode: CompositorMode,
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
    fn create_graphics_context() -> NativeCompositingGraphicsContext {
        NativeCompositingGraphicsContext::from_display(azure_hl::current_display())
    }
    #[cfg(not(target_os="linux"))]
    fn create_graphics_context() -> NativeCompositingGraphicsContext {
        NativeCompositingGraphicsContext::new()
    }

    pub fn create(opts: Opts,
                  port: Port<Msg>,
                  constellation_chan: ConstellationChan,
                  profiler_chan: ProfilerChan) {

        let compositor = CompositorTask::new(opts.headless);

        match compositor.mode {
            Windowed(ref app) => {
                compositor::IOCompositor::create(app,
                                                 opts,
                                                 port,
                                                 constellation_chan.clone(),
                                                 profiler_chan)
            }
            Headless => {
                headless::NullCompositor::create(port,
                                                 constellation_chan.clone())
            }
        };
    }
}
