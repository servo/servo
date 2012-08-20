#[doc = "
A graphics compositor that renders to PNG format buffers

Each time the renderer renders a frame the compositor will output a
`~[u8]` containing the frame in PNG format.
"];

export PngCompositor, Msg, Exit;

import libc::{c_int, c_uint, c_void, c_uchar};
import azure_bg = azure::bindgen;
import azure_bg::{AzCreateDrawTargetForCairoSurface, AzReleaseDrawTarget};
import azure::cairo;
import azure::azure_hl::DrawTarget;
import azure::cairo_hl::ImageSurface;
import cairo::{CAIRO_FORMAT_ARGB32, cairo_surface_t, cairo_status_t, CAIRO_STATUS_SUCCESS};
import cairo_bg = cairo::bindgen;
import cairo_bg::{cairo_image_surface_create, cairo_surface_destroy,
                  cairo_surface_write_to_png_stream};
import compositor::Compositor;
import render_task::{RenderTask, RenderMsg};
import task::spawn_listener;
import comm::{Chan, Port, chan, port};
import unsafe::reinterpret_cast;
import vec_from_buf = vec::unsafe::from_buf;
import ptr::addr_of;
import dom::event::Event;
import dvec::dvec;
import layout::display_list::display_list;
import std::cell::Cell;

type PngCompositor = Chan<Msg>;

enum Msg {
    BeginDrawing(pipes::chan<DrawTarget>),
    Draw(pipes::chan<DrawTarget>, DrawTarget),
    Exit
}

impl Chan<Msg> : Compositor {
    fn begin_drawing(+next_dt: pipes::chan<DrawTarget>) {
        self.send(BeginDrawing(next_dt))
    }
    fn draw(+next_dt: pipes::chan<DrawTarget>, +draw_me: DrawTarget) {
        self.send(Draw(next_dt, draw_me))
    }
    fn add_event_listener(_listener: Chan<Event>) {
        // No events in this compositor.
    }
}

fn PngCompositor(output: Chan<~[u8]>) -> PngCompositor {
    do spawn_listener |po: Port<Msg>| {
        let cairo_surface = ImageSurface(CAIRO_FORMAT_ARGB32, 800, 600);
        let draw_target = Cell(DrawTarget(cairo_surface));

        loop {
            match po.recv() {
                BeginDrawing(sender) => {
                    debug!("png_compositor: begin_drawing");
                    sender.send(draw_target.take());
                }
                Draw(sender, dt) => {
                    debug!("png_compositor: draw");
                    do_draw(sender, dt.clone(), output, cairo_surface);
                }
                Exit => break
            }
        }
    }
}

fn do_draw(sender: pipes::chan<DrawTarget>,
           +dt: DrawTarget,
           output: Chan<~[u8]>,
           cairo_surface: ImageSurface) {
    let buffer = io::mem_buffer();
    cairo_surface.write_to_png_stream(&buffer);
    let @{ buf: buffer, pos: _ } <- buffer;
    output.send(vec::from_mut(dvec::unwrap(move buffer)));

    // Send the next draw target to the renderer
    sender.send(move dt);
}

#[test]
fn sanity_check() {
    do listen |self_channel| {
        let compositor = PngCompositor(self_channel);
        let renderer = RenderTask(compositor);

        let dlist : display_list = dvec();
        renderer.send(RenderMsg(dlist));
        let (exit_chan, exit_response_from_engine) = pipes::stream();
        renderer.send(render_task::ExitMsg(exit_chan));
        exit_response_from_engine.recv();

        compositor.send(Exit)
    }
}
