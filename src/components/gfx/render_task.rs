/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The task that handles all rendering/painting.

use azure::{AzFloat, AzGLContext};
use azure::azure_hl::{B8G8R8A8, DrawTarget};
use display_list::DisplayList;
use servo_msg::compositor::{RenderListener, IdleRenderState, RenderingRenderState, LayerBuffer};
use servo_msg::compositor::LayerBufferSet;
use font_context::FontContext;
use geom::matrix2d::Matrix2D;
use geom::size::Size2D;
use geom::rect::Rect;
use opts::Opts;
use render_context::RenderContext;

use core::cell::Cell;
use core::comm::{Chan, Port, SharedChan};

use servo_util::time::{ProfilerChan, profile};
use servo_util::time;


pub struct RenderLayer {
    display_list: DisplayList<()>,
    size: Size2D<uint>
}

pub enum Msg<C> {
    AttachCompositorMsg(C),
    RenderMsg(RenderLayer),
    ReRenderMsg(~[(Rect<uint>, Rect<f32>)], f32),
    ExitMsg(Chan<()>),
}

pub struct RenderChan<C> {
    chan: SharedChan<Msg<C>>,
}

impl<C: RenderListener + Owned> Clone for RenderChan<C> {
    pub fn clone(&self) -> RenderChan<C> {
        RenderChan {
            chan: self.chan.clone(),
        }
    }
}

impl<C: RenderListener + Owned> RenderChan<C> {
    pub fn new(chan: Chan<Msg<C>>) -> RenderChan<C> {
        RenderChan {
            chan: SharedChan::new(chan),
        }
    }
    pub fn send(&self, msg: Msg<C>) {
        self.chan.send(msg);
    }
}

pub fn create_render_task<C: RenderListener + Owned>(port: Port<Msg<C>>,
                                                     compositor: C,
                                                     opts: Opts,
                                                     profiler_chan: ProfilerChan) {
    let compositor_cell = Cell(compositor);
    let opts_cell = Cell(opts);
    let port = Cell(port);

    do spawn {
        let compositor = compositor_cell.take();
        let share_gl_context = compositor.get_gl_context();
        let opts = opts_cell.with_ref(|o| copy *o);
        let profiler_chan = profiler_chan.clone();
        let profiler_chan_copy = profiler_chan.clone();

        // FIXME: rust/#5967
        let mut renderer = Renderer {
            port: port.take(),
            compositor: compositor,
            font_ctx: @mut FontContext::new(opts.render_backend,
                                            false,
                                            profiler_chan),
            opts: opts_cell.take(),
            profiler_chan: profiler_chan_copy,
            share_gl_context: share_gl_context,
            render_layer: None,
        };

        renderer.start();
    }
}

priv struct Renderer<C> {
    port: Port<Msg<C>>,
    compositor: C,
    font_ctx: @mut FontContext,
    opts: Opts,

    /// A channel to the profiler.
    profiler_chan: ProfilerChan,

    share_gl_context: AzGLContext,

    /// The layer to be rendered
    render_layer: Option<RenderLayer>,
}

impl<C: RenderListener + Owned> Renderer<C> {
    fn start(&mut self) {
        debug!("renderer: beginning rendering loop");

        loop {
            match self.port.recv() {
                AttachCompositorMsg(compositor) => self.compositor = compositor,
                RenderMsg(render_layer) => {
                    self.compositor.new_layer(render_layer.size, self.opts.tile_size);
                    self.render_layer = Some(render_layer);
                }
                ReRenderMsg(tiles, scale) => {
                    self.render(tiles, scale);
                }
                ExitMsg(response_ch) => {
                    response_ch.send(());
                    break;
                }
            }
        }
    }

    fn render(&mut self, tiles: ~[(Rect<uint>, Rect<f32>)], scale: f32) {
        debug!("renderer: rendering");
        
        let render_layer;
        match self.render_layer {
            Some(ref r_layer) => {
                render_layer = r_layer;
            }
            _ => return, // nothing to do
        }

        self.compositor.set_render_state(RenderingRenderState);
        do profile(time::RenderingCategory, self.profiler_chan.clone()) {

            // FIXME: Try not to create a new array here.
            let mut new_buffers = ~[];

            // Divide up the layer into tiles.
            do time::profile(time::RenderingPrepBuffCategory, self.profiler_chan.clone()) {
                for tiles.each |tile_rects| {
                    let (screen_rect, page_rect) = *tile_rects;
                    let width = screen_rect.size.width;
                    let height = screen_rect.size.height;
                    
                    let buffer = LayerBuffer {
                        draw_target: DrawTarget::new_with_fbo(self.opts.render_backend,
                                                              self.share_gl_context,
                                                              Size2D(width as i32, height as i32),
                                                              B8G8R8A8),
                        rect: page_rect,
                        screen_pos: screen_rect,
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

            debug!("renderer: returning surface");
            self.compositor.paint(layer_buffer_set, render_layer.size);
            self.compositor.set_render_state(IdleRenderState);
        }
    }
}

