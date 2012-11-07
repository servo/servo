use comm::*;
use libc::size_t;
use libc::types::common::c99::uint16_t;
use pipes::{Port, Chan};
use std::cell::Cell;

use azure::AzFloat;
use geom::matrix2d::Matrix2D;

use dl = display_list;
use gfx::{
    FontContext,
    RenderContext,
    RenderLayer,
};
use gfx::compositor::{
    Compositor,
    LayerBufferSet,
};
use mod gfx::render_layers;
use platform::osmain;
use render_layers::render_layers;


pub enum Msg {
    RenderMsg(RenderLayer),
    ExitMsg(pipes::Chan<()>)
}

pub type RenderTask = comm::Chan<Msg>;

pub fn RenderTask<C: Compositor Send>(compositor: C) -> RenderTask {
    let compositor_cell = Cell(move compositor);
    do task::spawn_listener |po: comm::Port<Msg>, move compositor_cell| {
        let (layer_buffer_channel, layer_buffer_set_port) = pipes::stream();

        let compositor = compositor_cell.take();
        compositor.begin_drawing(move layer_buffer_channel);

        Renderer {
            port: po,
            compositor: move compositor,
            mut layer_buffer_set_port: Cell(move layer_buffer_set_port),
            font_ctx: @FontContext::new(),
        }.start();
    }
}

priv struct Renderer<C: Compositor Send> {
    port: comm::Port<Msg>,
    compositor: C,
    layer_buffer_set_port: Cell<pipes::Port<LayerBufferSet>>,
    font_ctx: @FontContext
}

impl<C: Compositor Send> Renderer<C> {
    fn start() {
        debug!("renderer: beginning rendering loop");

        loop {
            match self.port.recv() {
                RenderMsg(move render_layer) => self.render(move render_layer),
                ExitMsg(response_ch) => {
                    response_ch.send(());
                    break;
                }
            }
        }
    }

    fn render(render_layer: RenderLayer) {
        debug!("renderer: got render request");

        let layer_buffer_set_port = self.layer_buffer_set_port.take();

        if !layer_buffer_set_port.peek() {
            warn!("renderer: waiting on layer buffer");
        }

        let layer_buffer_set = layer_buffer_set_port.recv();
        let (layer_buffer_set_channel, new_layer_buffer_set_port) = pipes::stream();
        self.layer_buffer_set_port.put_back(move new_layer_buffer_set_port);

        let render_layer_cell = Cell(move render_layer);
        let layer_buffer_set_cell = Cell(move layer_buffer_set);
        let layer_buffer_set_channel_cell = Cell(move layer_buffer_set_channel);

        #debug("renderer: rendering");

        do util::time::time(~"rendering") {
            let render_layer = render_layer_cell.take();
            let layer_buffer_set = layer_buffer_set_cell.take();
            let layer_buffer_set_channel = layer_buffer_set_channel_cell.take();

            let layer_buffer_set = for render_layers(&render_layer, move layer_buffer_set)
                    |render_layer, layer_buffer| {
                let ctx = RenderContext {
                    canvas: layer_buffer,
                    font_ctx: self.font_ctx
                };

                // Apply the translation to render the tile we want.
                let matrix: Matrix2D<AzFloat> = Matrix2D::identity();
                let matrix = matrix.translate(&-(layer_buffer.rect.origin.x as AzFloat),
                                              &-(layer_buffer.rect.origin.y as AzFloat));
                layer_buffer.draw_target.set_transform(&matrix);

                ctx.clear();

                render_layer.display_list.draw_into_context(&ctx);
            };

            #debug("renderer: returning surface");
            self.compositor.draw(move layer_buffer_set_channel, move layer_buffer_set);
        }
    }
}
