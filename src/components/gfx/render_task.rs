/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The task that handles all rendering/painting.

use azure::{AzFloat, AzGLContext};
use azure::azure_hl::{B8G8R8A8, DrawTarget};
use display_list::DisplayList;
use servo_msg::compositor_msg::{RenderListener, IdleRenderState, RenderingRenderState, LayerBuffer};
use servo_msg::compositor_msg::{LayerBufferSet};
use servo_msg::constellation_msg::PipelineId;
use font_context::FontContext;
use geom::matrix2d::Matrix2D;
use geom::size::Size2D;
use geom::rect::Rect;
use opts::Opts;
use render_context::RenderContext;

use std::cell::Cell;
use std::comm::{Chan, Port, SharedChan};

use servo_util::time::{ProfilerChan, profile};
use servo_util::time;

use extra::arc;

pub struct RenderLayer {
    display_list: DisplayList<()>,
    size: Size2D<uint>
}

pub enum Msg {
    RenderMsg(RenderLayer),
    ReRenderMsg(~[BufferRequest], f32, PipelineId),
    PaintPermissionGranted,
    PaintPermissionRevoked,
    ExitMsg(Chan<()>),
}

/// A request from the compositor to the renderer for tiles that need to be (re)displayed.
pub struct BufferRequest {
    // The rect in pixels that will be drawn to the screen
    screen_rect: Rect<uint>,
    
    // The rect in page coordinates that this tile represents
    page_rect: Rect<f32>,
}

pub fn BufferRequest(screen_rect: Rect<uint>, page_rect: Rect<f32>) -> BufferRequest {
    BufferRequest {
        screen_rect: screen_rect,
        page_rect: page_rect,
    }
}

#[deriving(Clone)]
pub struct RenderChan {
    chan: SharedChan<Msg>,
}

impl RenderChan {
    pub fn new(chan: Chan<Msg>) -> RenderChan {
        RenderChan {
            chan: SharedChan::new(chan),
        }
    }
    pub fn send(&self, msg: Msg) {
        self.chan.send(msg);
    }
}

priv struct RenderTask<C> {
    id: PipelineId,
    port: Port<Msg>,
    compositor: C,
    font_ctx: @mut FontContext,
    opts: Opts,

    /// A channel to the profiler.
    profiler_chan: ProfilerChan,

    share_gl_context: AzGLContext,

    /// The layer to be rendered
    render_layer: Option<RenderLayer>,
    /// Permission to send paint messages to the compositor
    paint_permission: bool,
    /// Cached copy of last layers rendered
    last_paint_msg: Option<(arc::ARC<LayerBufferSet>, Size2D<uint>)>,
}

impl<C: RenderListener + Send> RenderTask<C> {
    pub fn create(id: PipelineId,
                  port: Port<Msg>,
                  compositor: C,
                  opts: Opts,
                  profiler_chan: ProfilerChan) {
        let compositor = Cell::new(compositor);
        let opts = Cell::new(opts);
        let port = Cell::new(port);
        let profiler_chan = Cell::new(profiler_chan);

        do spawn {
            let compositor = compositor.take();
            let share_gl_context = compositor.get_gl_context();
            let opts = opts.take();
            let profiler_chan = profiler_chan.take();

            // FIXME: rust/#5967
            let mut render_task = RenderTask {
                id: id,
                port: port.take(),
                compositor: compositor,
                font_ctx: @mut FontContext::new(copy opts.render_backend,
                                                false,
                                                profiler_chan.clone()),
                opts: opts,
                profiler_chan: profiler_chan,
                share_gl_context: share_gl_context,
                render_layer: None,

                paint_permission: false,
                last_paint_msg: None,
            };

            render_task.start();
        }
    }

    fn start(&mut self) {
        debug!("render_task: beginning rendering loop");

        loop {
            match self.port.recv() {
                RenderMsg(render_layer) => {
                    if self.paint_permission {
                        self.compositor.new_layer(self.id, render_layer.size);
                    }
                    self.render_layer = Some(render_layer);
                }
                ReRenderMsg(tiles, scale, id) => {
                    self.render(tiles, scale, id);
                }
                PaintPermissionGranted => {
                    self.paint_permission = true;
                    match self.render_layer {
                        Some(ref render_layer) => {
                            self.compositor.new_layer(self.id, render_layer.size);
                        }
                        None => {}
                    }
                }
                PaintPermissionRevoked => {
                    self.paint_permission = false;
                }
                ExitMsg(response_ch) => {
                    response_ch.send(());
                    break;
                }
            }
        }
    }

    fn render(&mut self, tiles: ~[BufferRequest], scale: f32, id: PipelineId) {
        let render_layer;
        match self.render_layer {
            Some(ref r_layer) => {
                render_layer = r_layer;
            }
            _ => return, // nothing to do
        }

        self.compositor.set_render_state(RenderingRenderState);
        do time::profile(time::RenderingCategory, self.profiler_chan.clone()) {

            // FIXME: Try not to create a new array here.
            let mut new_buffers = ~[];

            // Divide up the layer into tiles.
            do time::profile(time::RenderingPrepBuffCategory, self.profiler_chan.clone()) {
                for tiles.iter().advance |tile| {
                    let width = tile.screen_rect.size.width;
                    let height = tile.screen_rect.size.height;
                    
                    let buffer = LayerBuffer {
                        draw_target: DrawTarget::new_with_fbo(self.opts.render_backend,
                                                              self.share_gl_context,
                                                              Size2D(width as i32, height as i32),
                                                              B8G8R8A8),
                        rect: tile.page_rect,
                        screen_pos: tile.screen_rect,
                        resolution: scale,
                        stride: (width * 4) as uint
                    };
                    
                    
                    {
                        // Build the render context.
                        let ctx = RenderContext {
                            canvas: &buffer,
                            font_ctx: self.font_ctx,
                            opts: &self.opts
                        };

                        // Apply the translation to render the tile we want.
                        let matrix: Matrix2D<AzFloat> = Matrix2D::identity();
                        let matrix = matrix.scale(scale as AzFloat, scale as AzFloat);
                        let matrix = matrix.translate(-(buffer.rect.origin.x) as AzFloat,
                                                      -(buffer.rect.origin.y) as AzFloat);
                        
                        ctx.canvas.draw_target.set_transform(&matrix);
                        
                        // Clear the buffer.
                        ctx.clear();
                        
                        // Draw the display list.
                        do profile(time::RenderingDrawingCategory, self.profiler_chan.clone()) {
                            render_layer.display_list.draw_into_context(&ctx);
                            ctx.canvas.draw_target.flush();
                        }
                    }
                    
                    new_buffers.push(buffer);
                    
                }

            }

            let layer_buffer_set = LayerBufferSet {
                buffers: new_buffers,
            };
            let layer_buffer_set = arc::ARC(layer_buffer_set);

            debug!("render_task: returning surface");
            if self.paint_permission {
                self.compositor.paint(id, layer_buffer_set.clone());
            }
            debug!("caching paint msg");
            self.last_paint_msg = Some((layer_buffer_set, render_layer.size));
            self.compositor.set_render_state(IdleRenderState);
        }
    }
}

