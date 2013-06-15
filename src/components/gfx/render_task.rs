/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The task that handles all rendering/painting.

use azure::{AzFloat, AzGLContext};
use compositor::{RenderListener, IdleRenderState, RenderingRenderState};
use font_context::FontContext;
use geom::matrix2d::Matrix2D;
use opts::Opts;
use render_context::RenderContext;
use render_layers::{RenderLayer, render_layers};

use core::cell::Cell;
use core::comm::{Chan, Port, SharedChan};
use core::task::SingleThreaded;
use std::task_pool::TaskPool;

use servo_net::util::spawn_listener;

use servo_util::time::{ProfilerChan, profile};
use servo_util::time;

pub enum Msg<C> {
    AttachCompositorMsg(C),
    RenderMsg(RenderLayer),
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

        // FIXME: Annoying three-cell dance here. We need one-shot closures.
        let opts = opts_cell.with_ref(|o| copy *o);
        let n_threads = opts.n_render_threads;
        let new_opts_cell = Cell(opts);

        let profiler_chan = profiler_chan.clone();
        let profiler_chan_copy = profiler_chan.clone();

        let thread_pool = do TaskPool::new(n_threads, Some(SingleThreaded)) {
            let opts_cell = Cell(new_opts_cell.with_ref(|o| copy *o));
            let profiler_chan = Cell(profiler_chan.clone());

            let f: ~fn(uint) -> ThreadRenderContext = |thread_index| {
                let opts = opts_cell.with_ref(|opts| copy *opts);

                ThreadRenderContext {
                    thread_index: thread_index,
                    font_ctx: @mut FontContext::new(opts.render_backend,
                                                    false,
                                                    profiler_chan.take()),
                    opts: opts,
                }
            };
            f
        };

        // FIXME: rust/#5967
        let mut renderer = Renderer {
            port: port.take(),
            compositor: compositor,
            thread_pool: thread_pool,
            opts: opts_cell.take(),
            profiler_chan: profiler_chan_copy,
            share_gl_context: share_gl_context,
        };

        renderer.start();
    }
}

/// Data that needs to be kept around for each render thread.
priv struct ThreadRenderContext {
    thread_index: uint,
    font_ctx: @mut FontContext,
    opts: Opts,
}

priv struct Renderer<C> {
    port: Port<Msg<C>>,
    compositor: C,
    thread_pool: TaskPool<ThreadRenderContext>,
    opts: Opts,

    /// A channel to the profiler.
    profiler_chan: ProfilerChan,

    share_gl_context: AzGLContext,
}

impl<C: RenderListener + Owned> Renderer<C> {
    fn start(&mut self) {
        debug!("renderer: beginning rendering loop");

        loop {
            match self.port.recv() {
                AttachCompositorMsg(compositor) => self.compositor = compositor,
                RenderMsg(render_layer) => self.render(render_layer),
                ExitMsg(response_ch) => {
                    response_ch.send(());
                    break;
                }
            }
        }
    }

    fn render(&mut self, render_layer: RenderLayer) {
        debug!("renderer: rendering");
        self.compositor.set_render_state(RenderingRenderState);
        do profile(time::RenderingCategory, self.profiler_chan.clone()) {
            let layer_buffer_set = do render_layers(&render_layer,
                                                    &self.opts,
                                                    self.profiler_chan.clone(),
                                                    self.share_gl_context) |render_layer_ref,
                                                                                 layer_buffer,
                                                                                 buffer_chan| {
                let layer_buffer_cell = Cell(layer_buffer);
                do self.thread_pool.execute |thread_render_context| {
                    do layer_buffer_cell.with_ref |layer_buffer| {
                        // Build the render context.
                        let ctx = RenderContext {
                            canvas: layer_buffer,
                            font_ctx: thread_render_context.font_ctx,
                            opts: &thread_render_context.opts
                        };

                        // Apply the translation to render the tile we want.
                        let matrix: Matrix2D<AzFloat> = Matrix2D::identity();
                        let scale = thread_render_context.opts.zoom as f32;

                        let matrix = matrix.scale(scale as AzFloat, scale as AzFloat);
                        let matrix = matrix.translate(-(layer_buffer.rect.origin.x as f32) as AzFloat,
                                                      -(layer_buffer.rect.origin.y as f32) as AzFloat);

                        layer_buffer.draw_target.set_transform(&matrix);

                        // Clear the buffer.
                        ctx.clear();
                        

                        // Draw the display list.
                        let render_layer: &RenderLayer = unsafe {
                            cast::transmute(render_layer_ref)
                        };
                        
                        render_layer.display_list.draw_into_context(&ctx);
                        ctx.canvas.draw_target.flush();
                    }

                    // Send back the buffer.
                    buffer_chan.send(layer_buffer_cell.take());
                }
            };

            debug!("renderer: returning surface");
            self.compositor.paint(layer_buffer_set, render_layer.size);
            self.compositor.set_render_state(IdleRenderState);
        }
    }
}

