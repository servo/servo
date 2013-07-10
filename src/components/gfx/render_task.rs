/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The task that handles all rendering/painting.

use azure::{AzFloat, AzGLContext};
use azure::azure_hl::{B8G8R8A8, DrawTarget};
use display_list::DisplayList;
use servo_msg::compositor_msg::{RenderListener, IdleRenderState, RenderingRenderState, LayerBuffer};
use servo_msg::compositor_msg::{LayerBufferSet};
use font_context::FontContext;
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use opts::Opts;
use render_context::RenderContext;

use std::cell::Cell;
use std::comm::{Chan, Port, SharedChan};
use std::uint;

use servo_util::time::{ProfilerChan, profile};
use servo_util::time;

use extra::arc;

pub struct RenderLayer {
    display_list: DisplayList<()>,
    size: Size2D<uint>
}

pub enum Msg {
    RenderMsg(RenderLayer),
    ReRenderMsg(f32),
    PaintPermissionGranted,
    PaintPermissionRevoked,
    ExitMsg(Chan<()>),
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
    id: uint,
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
    pub fn create(id: uint,
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
                    self.render_layer = Some(render_layer);
                    self.render(1.0);
                }
                ReRenderMsg(scale) => {
                    self.render(scale);
                }
                PaintPermissionGranted => {
                    self.paint_permission = true;
                    match self.last_paint_msg {
                        Some((ref layer_buffer_set, ref layer_size)) => {
                            self.compositor.paint(self.id, layer_buffer_set.clone(), *layer_size);
                            self.compositor.set_render_state(IdleRenderState);
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

    fn render(&mut self, scale: f32) {
        debug!("render_task: rendering");
        
        let render_layer;
        match (self.render_layer) {
            None => return,
            Some(ref r_layer) => {
                render_layer = r_layer;
            }
        }

        self.compositor.set_render_state(RenderingRenderState);
        do time::profile(time::RenderingCategory, self.profiler_chan.clone()) {
            let tile_size = self.opts.tile_size;

            // FIXME: Try not to create a new array here.
            let mut new_buffers = ~[];

            // Divide up the layer into tiles.
            do time::profile(time::RenderingPrepBuffCategory, self.profiler_chan.clone()) {
                let mut y = 0;
                while y < (render_layer.size.height as f32 * scale).ceil() as uint {
                    let mut x = 0;
                    while x < (render_layer.size.width as f32 * scale).ceil() as uint {
                        // Figure out the dimension of this tile.
                        let right = uint::min(x + tile_size, (render_layer.size.width as f32 * scale).ceil() as uint);
                        let bottom = uint::min(y + tile_size, (render_layer.size.height as f32 * scale).ceil() as uint);
                        let width = right - x;
                        let height = bottom - y;

                        let tile_rect = Rect(Point2D(x as f32 / scale, y as f32 / scale), Size2D(width as f32, height as f32));
                        let screen_rect = Rect(Point2D(x, y), Size2D(width, height));

                        let buffer = LayerBuffer {
                            draw_target: DrawTarget::new_with_fbo(self.opts.render_backend,
                                                                  self.share_gl_context,
                                                                  Size2D(width as i32,
                                                                         height as i32),
                                                                  B8G8R8A8),
                            rect: tile_rect,
                            screen_pos: screen_rect,
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

                        x += tile_size;
                    }

                    y += tile_size;
                }
            }

            let layer_buffer_set = LayerBufferSet {
                buffers: new_buffers,
            };
            let layer_buffer_set = arc::ARC(layer_buffer_set);

            debug!("render_task: returning surface");
            if self.paint_permission {
                self.compositor.paint(self.id, layer_buffer_set.clone(), render_layer.size);
            }
            debug!("caching paint msg");
            self.last_paint_msg = Some((layer_buffer_set, render_layer.size));
            self.compositor.set_render_state(IdleRenderState);
        }
    }
}

