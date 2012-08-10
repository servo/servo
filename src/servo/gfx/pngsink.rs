#[doc = "
A graphics sink that renders to PNG format buffers

Each time the renderer renders a frame the bufsink will output a
`~[u8]` containing the frame in PNG format.
"];

export PngSink, Msg, Exit;

import libc::{c_int, c_uint, c_void, c_uchar};
import azure::AzDrawTargetRef;
import azure_bg = azure::bindgen;
import azure_bg::{AzCreateDrawTargetForCairoSurface, AzReleaseDrawTarget};
import azure::cairo;
import azure::azure_hl::DrawTarget;
import azure::cairo_hl::ImageSurface;
import cairo::{CAIRO_FORMAT_ARGB32, cairo_surface_t, cairo_status_t, CAIRO_STATUS_SUCCESS};
import cairo_bg = cairo::bindgen;
import cairo_bg::{cairo_image_surface_create, cairo_surface_destroy,
                  cairo_surface_write_to_png_stream};
import renderer::{Renderer, Sink, RenderMsg};
import task::spawn_listener;
import comm::{chan, port};
import unsafe::reinterpret_cast;
import vec_from_buf = vec::unsafe::from_buf;
import ptr::addr_of;
import dom::event::Event;
import dvec::dvec;
import layout::display_list::display_list;

type PngSink = chan<Msg>;

enum Msg {
    BeginDrawing(pipes::chan<AzDrawTargetRef>),
    Draw(pipes::chan<AzDrawTargetRef>, AzDrawTargetRef),
    Exit
}

impl chan<Msg> : Sink {
    fn begin_drawing(+next_dt: pipes::chan<AzDrawTargetRef>) {
        self.send(BeginDrawing(next_dt))
    }
    fn draw(+next_dt: pipes::chan<AzDrawTargetRef>, draw_me: AzDrawTargetRef) {
        self.send(Draw(next_dt, draw_me))
    }
    fn add_event_listener(_listener: chan<Event>) {
        // No events in this sink.
    }
}

fn PngSink(output: chan<~[u8]>) -> PngSink {
    do spawn_listener |po: port<Msg>| {
        let cairo_surface = ImageSurface(CAIRO_FORMAT_ARGB32, 800, 600);
        let draw_target = DrawTarget(cairo_surface);

        loop {
            match po.recv() {
                BeginDrawing(sender) => {
                    debug!("pngsink: begin_drawing");
                    sender.send(draw_target.azure_draw_target);
                }
                Draw(sender, dt) => {
                    debug!("pngsink: draw");
                    do_draw(sender, dt, output, cairo_surface);
                }
                Exit => break
            }
        }
    }
}

fn do_draw(sender: pipes::chan<AzDrawTargetRef>, dt: AzDrawTargetRef, output: chan<~[u8]>,
           cairo_surface: ImageSurface) {
    let buffer = io::mem_buffer();
    cairo_surface.write_to_png_stream(&buffer);
    let @{ buf: buffer, pos: _ } <- buffer;
    output.send(vec::from_mut(dvec::unwrap(move buffer)));

    // Send the next draw target to the renderer
    sender.send(dt);
}

#[test]
fn sanity_check() {
    do listen |self_channel| {
        let sink = PngSink(self_channel);
        let renderer = Renderer(sink);

        let dlist : display_list = dvec();
        renderer.send(RenderMsg(dlist));
        let (exit_chan, exit_response_from_engine) = pipes::stream();
        renderer.send(renderer::ExitMsg(exit_chan));
        exit_response_from_engine.recv();

        sink.send(Exit)
    }
}
