/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The task that handles all rendering/painting.

use azure::AzFloat;
use compositor::{Compositor, LayerBufferSet};
use font_context::FontContext;
use geom::matrix2d::Matrix2D;
use opts::Opts;
use render_context::RenderContext;
use render_layers::{RenderLayer, render_layers};

use core::cell::Cell;
use core::comm::{Port, SharedChan};
use core::task::SingleThreaded;
use std::task_pool::TaskPool;
use servo_net::util::spawn_listener;

use servo_util::time;
use servo_util::time::time;
use servo_util::time::profile;
use servo_util::time::ProfilerChan;

pub enum Msg {
    RenderMsg(RenderLayer),
    ExitMsg(comm::Chan<()>)
}

pub type RenderTask = SharedChan<Msg>;

pub fn RenderTask<C:Compositor + Owned>(compositor: C,
                                        opts: Opts,
                                        prof_chan: ProfilerChan) -> RenderTask {
    let compositor_cell = Cell(compositor);
    let opts_cell = Cell(opts);
    let render_task = do spawn_listener |po: Port<Msg>| {
        let (layer_buffer_set_port, layer_buffer_channel) = comm::stream();

        let compositor = compositor_cell.take();
        compositor.begin_drawing(layer_buffer_channel);

        // FIXME: Annoying three-cell dance here. We need one-shot closures.
        let opts = opts_cell.with_ref(|o| copy *o);
        let n_threads = opts.n_render_threads;
        let new_opts_cell = Cell(opts);
        let prof_chan2 = prof_chan.clone();

        let thread_pool = do TaskPool::new(n_threads, Some(SingleThreaded)) {
            let opts_cell = Cell(new_opts_cell.with_ref(|o| copy *o));
            let prof_chan = prof_chan2.clone();
            let f: ~fn(uint) -> ThreadRenderContext = |thread_index| {
                ThreadRenderContext {
                    thread_index: thread_index,
                    font_ctx: @mut FontContext::new(opts_cell.with_ref(|o| o.render_backend),
                                                    false,
                                                    prof_chan.clone()),
                    opts: opts_cell.with_ref(|o| copy *o),
                }
            };
            f
        };

        // FIXME: rust/#5967
        let mut r = Renderer {
            port: po,
            compositor: compositor,
            layer_buffer_set_port: Cell(layer_buffer_set_port),
            thread_pool: thread_pool,
            opts: opts_cell.take(),
            prof_chan: prof_chan.clone()
        };
        r.start();
    };
    SharedChan::new(render_task)
}

/// Data that needs to be kept around for each render thread.
priv struct ThreadRenderContext {
    thread_index: uint,
    font_ctx: @mut FontContext,
    opts: Opts,
}

priv struct Renderer<C> {
    port: Port<Msg>,
    compositor: C,
    layer_buffer_set_port: Cell<comm::Port<LayerBufferSet>>,
    thread_pool: TaskPool<ThreadRenderContext>,
    opts: Opts,
    prof_chan: ProfilerChan,
}

impl<C: Compositor + Owned> Renderer<C> {
    fn start(&mut self) {
        debug!("renderer: beginning rendering loop");

        loop {
            match self.port.recv() {
                RenderMsg(render_layer) => self.render(render_layer),
                ExitMsg(response_ch) => {
                    response_ch.send(());
                    break;
                }
            }
        }
    }

    fn render(&mut self, render_layer: RenderLayer) {
        debug!("renderer: got render request");

        let layer_buffer_set_port = self.layer_buffer_set_port.take();

        if !layer_buffer_set_port.peek() {
            warn!("renderer: waiting on layer buffer");
        }

        let (new_layer_buffer_set_port, layer_buffer_set_channel) = comm::stream();
        self.layer_buffer_set_port.put_back(new_layer_buffer_set_port);

        let layer_buffer_set_channel_cell = Cell(layer_buffer_set_channel);

        debug!("renderer: rendering");

        do profile(time::RenderingCategory, self.prof_chan.clone()) {
            let layer_buffer_set_channel = layer_buffer_set_channel_cell.take();

            let layer_buffer_set = do render_layers(&render_layer, &self.opts, self.prof_chan.clone())
                    |render_layer_ref, layer_buffer, buffer_chan| {
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
                        let matrix = matrix.translate(&-(layer_buffer.rect.origin.x as AzFloat),
                                                      &-(layer_buffer.rect.origin.y as AzFloat));
                        layer_buffer.draw_target.set_transform(&matrix);

                        // Clear the buffer.
                        ctx.clear();

                        // Draw the display list.
                        let render_layer: &RenderLayer = unsafe {
                            cast::transmute(render_layer_ref)
                        };
                        render_layer.display_list.draw_into_context(&ctx);
                    }

                    // Send back the buffer.
                    buffer_chan.send(layer_buffer_cell.take());
                }
            };

            debug!("renderer: returning surface");
            self.compositor.draw(layer_buffer_set_channel, layer_buffer_set);
        }
    }
}
