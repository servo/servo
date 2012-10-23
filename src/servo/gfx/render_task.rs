use au = gfx::geometry;
use au::Au;
use comm::*;
use compositor::{Compositor, LayerBuffer};
use dl = display_list;
use mod gfx::render_layers;
use render_layers::render_layers;
use gfx::render_layers::RenderLayer;
use libc::size_t;
use libc::types::common::c99::uint16_t;
use pipes::{Port, Chan};
use platform::osmain;
use std::cell::Cell;
use text::font_cache::FontCache;
use render_context::RenderContext;

pub enum Msg {
    RenderMsg(RenderLayer),
    ExitMsg(pipes::Chan<()>)
}

pub type RenderTask = comm::Chan<Msg>;

pub fn RenderTask<C: Compositor Send>(compositor: C) -> RenderTask {
    let compositor_cell = Cell(move compositor);
    do task::spawn_listener |po: comm::Port<Msg>, move compositor_cell| {
        let (layer_buffer_channel, layer_buffer_port) = pipes::stream();

        let compositor = compositor_cell.take();
        compositor.begin_drawing(move layer_buffer_channel);

        Renderer {
            port: po,
            compositor: move compositor,
            mut layer_buffer_port: Cell(move layer_buffer_port),
            font_cache: FontCache(),
        }.start();
    }
}

priv struct Renderer<C: Compositor Send> {
    port: comm::Port<Msg>,
    compositor: C,
    layer_buffer_port: Cell<pipes::Port<LayerBuffer>>,
    font_cache: @FontCache
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

        let layer_buffer_port = self.layer_buffer_port.take();

        if !layer_buffer_port.peek() {
            warn!("renderer: waiting on layer buffer");
        }

        let layer_buffer = layer_buffer_port.recv();
        let (layer_buffer_channel, new_layer_buffer_port) = pipes::stream();
        self.layer_buffer_port.put_back(move new_layer_buffer_port);

        let render_layer_cell = Cell(move render_layer);
        let layer_buffer_cell = Cell(move layer_buffer);
        let layer_buffer_channel_cell = Cell(move layer_buffer_channel);

        #debug("renderer: rendering");

        do util::time::time(~"rendering") {
            let render_layer = render_layer_cell.take();
            let layer_buffer = layer_buffer_cell.take();
            let layer_buffer_channel = layer_buffer_channel_cell.take();

            let layer_buffer = for render_layers(&render_layer, move layer_buffer)
                    |render_layer, layer_buffer| {
                let ctx = RenderContext {
                    canvas: layer_buffer,
                    font_cache: self.font_cache
                };

                ctx.clear();
                render_layer.display_list.draw_into_context(&ctx);
            };

            #debug("renderer: returning surface");
            self.compositor.draw(move layer_buffer_channel, move layer_buffer);
        }
    }
}
