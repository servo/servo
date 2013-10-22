/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use windowing;

use servo_msg::compositor_msg::{RenderListener, LayerBufferSet, RenderState};
use servo_msg::compositor_msg::{ReadyState, ScriptListener, Epoch};
use servo_msg::constellation_msg::{ConstellationChan, PipelineId};
use gfx::opts::Opts;

use azure::azure::AzGLContext;
use std::comm;
use std::comm::{Chan, SharedChan, Port};
use std::num::Orderable;
use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use servo_util::time::ProfilerChan;

use constellation::SendableFrameTree;

mod quadtree;
mod compositor_layer;

mod run;
mod run_headless;


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

    fn close(&self) {
        self.chan.send(Exit);
    }

}

/// Implementation of the abstract `RenderListener` interface.
impl RenderListener for CompositorChan {

    fn get_gl_context(&self) -> AzGLContext {
        let (port, chan) = comm::stream();
        self.chan.send(GetGLContext(chan));
        port.recv()
    }

    fn paint(&self, id: PipelineId, layer_buffer_set: ~LayerBufferSet, epoch: Epoch) {
        self.chan.send(Paint(id, layer_buffer_set, epoch))
    }

    fn new_layer(&self, id: PipelineId, page_size: Size2D<uint>) {
        let Size2D { width, height } = page_size;
        self.chan.send(NewLayer(id, Size2D(width as f32, height as f32)))
    }
    fn set_layer_page_size(&self, id: PipelineId, page_size: Size2D<uint>, epoch: Epoch) {
        let Size2D { width, height } = page_size;
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

    pub fn new(chan: Chan<Msg>) -> CompositorChan {
        CompositorChan {
            chan: SharedChan::new(chan),
        }
    }

    pub fn send(&self, msg: Msg) {
        self.chan.send(msg);
    }

    pub fn get_size(&self) -> Size2D<int> {
        let (port, chan) = comm::stream();
        self.chan.send(GetSize(chan));
        port.recv()
    }
}

/// Messages to the compositor.
pub enum Msg {
    /// Requests that the compositor shut down.
    Exit,
    /// Requests the window size
    GetSize(Chan<Size2D<int>>),
    /// Requests the compositors GL context.
    GetGLContext(Chan<AzGLContext>),

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

    /// Requests that the compositor paint the given layer buffer set for the given page size.
    Paint(PipelineId, ~LayerBufferSet, Epoch),
    /// Alerts the compositor to the current status of page loading.
    ChangeReadyState(ReadyState),
    /// Alerts the compositor to the current status of rendering.
    ChangeRenderState(RenderState),
    /// Sets the channel to the current layout and render tasks, along with their id
    SetIds(SendableFrameTree, Chan<()>, ConstellationChan),
}

pub struct CompositorTask {
    opts: Opts,
    port: Port<Msg>,
    profiler_chan: ProfilerChan,
    shutdown_chan: SharedChan<()>,
}

impl CompositorTask {
    pub fn new(opts: Opts,
               port: Port<Msg>,
               profiler_chan: ProfilerChan,
               shutdown_chan: Chan<()>)
               -> CompositorTask {
        CompositorTask {
            opts: opts,
            port: port,
            profiler_chan: profiler_chan,
            shutdown_chan: SharedChan::new(shutdown_chan),
        }
    }

    pub fn run(&self) {
        if self.opts.headless {
            run_headless::run_compositor(self);
        } else {
            run::run_compositor(self);
        }
    }
}
