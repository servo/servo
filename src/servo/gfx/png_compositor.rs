/*!
A graphics compositor that renders to PNG format buffers

Each time the renderer renders a frame the compositor will output a
`~[u8]` containing the frame in PNG format.
*/

use libc::{c_int, c_uint, c_void, c_uchar};
use azure_bg = azure::bindgen;
use azure_bg::{AzCreateDrawTargetForCairoSurface, AzReleaseDrawTarget};
use azure::azure_hl::DrawTarget;
use cairo::cairo_hl::ImageSurface;
use cairo::{CAIRO_FORMAT_ARGB32, cairo_surface_t, cairo_status_t, CAIRO_STATUS_SUCCESS};
use cairo_bg = cairo::bindgen;
use cairo_bg::{cairo_image_surface_create, cairo_surface_destroy,
                  cairo_surface_write_to_png_stream};
use compositor::Compositor;
use render_task::{RenderTask, RenderMsg};
use task::spawn_listener;
use comm::{Chan, Port};
use cast::reinterpret_cast;
use ptr::addr_of;
use dom::event::Event;
use dvec::DVec;
use display_list::DisplayList;
use std::cell::Cell;
use core::io::BytesWriter;
use gfx::compositor::LayerBuffer;
use geom::size::Size2D;
use gfx::render_layers::RenderLayer;

pub type PngCompositor = Chan<Msg>;

pub enum Msg {
    BeginDrawing(pipes::Chan<LayerBuffer>),
    Draw(pipes::Chan<LayerBuffer>, LayerBuffer),
    Exit
}

impl Chan<Msg> : Compositor {
    fn begin_drawing(next_dt: pipes::Chan<LayerBuffer>) {
        self.send(BeginDrawing(move next_dt))
    }
    fn draw(next_dt: pipes::Chan<LayerBuffer>, draw_me: LayerBuffer) {
        self.send(Draw(move next_dt, move draw_me))
    }
}

pub fn PngCompositor(output: Chan<~[u8]>) -> PngCompositor {
    do spawn_listener |po: Port<Msg>| {
        let cairo_surface = ImageSurface(CAIRO_FORMAT_ARGB32, 800, 600);
        let draw_target = DrawTarget(&cairo_surface);
        let layer_buffer = LayerBuffer {
            cairo_surface: cairo_surface.clone(),
            draw_target: move draw_target,
            size: Size2D(800u, 600u),
            stride: 800
        };
        let layer_buffer = Cell(move layer_buffer);

        loop {
            match po.recv() {
                BeginDrawing(sender) => {
                    debug!("png_compositor: begin_drawing");
                    sender.send(layer_buffer.take());
                }
                Draw(move sender, move layer_buffer) => {
                    debug!("png_compositor: draw");
                    do_draw(move sender, move layer_buffer, output, &cairo_surface);
                }
                Exit => break
            }
        }
    }
}

fn do_draw(sender: pipes::Chan<LayerBuffer>,
           layer_buffer: LayerBuffer,
           output: Chan<~[u8]>,
           cairo_surface: &ImageSurface) {
    let buffer = BytesWriter();
    cairo_surface.write_to_png_stream(&buffer);
    output.send(buffer.bytes.get());

    // Send the next draw target to the renderer
    sender.send(move layer_buffer);
}

#[test]
fn sanity_check() {
    do listen |self_channel| {
        let compositor = PngCompositor(self_channel);
        let renderer = RenderTask(compositor);

        let dlist : DisplayList = DVec();
        let render_layer = RenderLayer { display_list: move dlist, size: Size2D(800u, 600u) };
        renderer.send(RenderMsg(move render_layer));
        let (exit_chan, exit_response_from_engine) = pipes::stream();
        renderer.send(render_task::ExitMsg(move exit_chan));
        exit_response_from_engine.recv();

        compositor.send(Exit)
    }
}
