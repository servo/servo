use au = gfx::geometry;
use au::au;
use comm::*;
use compositor::Compositor;
use dl = display_list;
use mod gfx::render_layers;
use gfx::render_layers::RenderLayer;
use libc::size_t;
use libc::types::common::c99::uint16_t;
use pipes::{Port, Chan};
use platform::osmain;
use std::cell::Cell;
use text::font_cache::FontCache;
use render_context::RenderContext;

pub type Renderer = comm::Chan<Msg>;

pub enum Msg {
    RenderMsg(RenderLayer),
    ExitMsg(pipes::Chan<()>)
}

pub type RenderTask = comm::Chan<Msg>;

pub fn RenderTask<C: Compositor Send>(compositor: C) -> RenderTask {
    do task::spawn_listener |po: comm::Port<Msg>| {
        let (layer_buffer_channel, layer_buffer_port) = pipes::stream();
        let mut layer_buffer_channel = layer_buffer_channel;
        let mut layer_buffer_port = layer_buffer_port;

        let font_cache = FontCache();

        debug!("renderer: beginning rendering loop");

        compositor.begin_drawing(move layer_buffer_channel);

        loop {
            match po.recv() {
                RenderMsg(move render_layer) => {
                    debug!("renderer: got render request");

                    if !layer_buffer_port.peek() {
                        warn!("renderer: waiting on layer buffer");
                    }

                    let layer_buffer_cell = Cell(layer_buffer_port.recv());

                    let (layer_buffer_channel, new_layer_buffer_port) = pipes::stream();
                    let layer_buffer_channel = Cell(move layer_buffer_channel);
                    layer_buffer_port = new_layer_buffer_port;

                    let render_layer = Cell(move render_layer);

                    #debug("renderer: rendering");

                    do util::time::time(~"rendering") {
                        let layer_buffer = layer_buffer_cell.take();
                        let render_layer = move render_layer.take();

                        let layer_buffer =
                                for render_layers::render_layers(&render_layer, move layer_buffer)
                                |render_layer, layer_buffer| {
                            let ctx = RenderContext {
                                canvas: layer_buffer,
                                font_cache: font_cache
                            };

                            ctx.clear();
                            render_layer.display_list.draw(&ctx);
                        };

                        #debug("renderer: returning surface");
                        compositor.draw(layer_buffer_channel.take(), move layer_buffer);
                    }
                }
                ExitMsg(response_ch) => {
                    response_ch.send(());
                    break;
                }
            }
        }
    }
}
