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
import cairo::{CAIRO_FORMAT_ARGB32, cairo_surface_t, cairo_status_t,
               CAIRO_STATUS_SUCCESS};
import cairo_bg = cairo::bindgen;
import cairo_bg::{cairo_image_surface_create, cairo_surface_destroy,
                  cairo_surface_write_to_png_stream};
import renderer::{Renderer, Sink, RenderMsg};
import task::spawn_listener;
import comm::chan;
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

impl PngSink of Sink for chan<Msg> {
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
    spawn_listener::<Msg>(|po| {
        let cairo_surf = cairo_image_surface_create(
            CAIRO_FORMAT_ARGB32, 800 as c_int, 600 as c_int
            );
        assert cairo_surf.is_not_null();

        let draw_target = AzCreateDrawTargetForCairoSurface(cairo_surf);
        assert draw_target.is_not_null();

        loop {
            match po.recv() {
              BeginDrawing(sender) => {
                #debug("pngsink: begin_drawing");
                sender.send(draw_target);
              }
              Draw(sender, dt) => {
                #debug("pngsink: draw");
                do_draw(sender, dt, output, cairo_surf);
              }
              Exit => { break }
            }
        }

        AzReleaseDrawTarget(draw_target);
        cairo_surface_destroy(cairo_surf);
    })
}

fn do_draw(sender: pipes::chan<AzDrawTargetRef>,
           dt: AzDrawTargetRef,
           output: chan<~[u8]>,
           cairo_surf: *cairo_surface_t) {

    listen(|data_ch: chan<~[u8]>| {

        extern fn write_fn(closure: *c_void,
                           data: *c_uchar,
                           len: c_uint)

            -> cairo_status_t unsafe {

            let p: *chan<~[u8]> = reinterpret_cast(closure);
            let data_ch = *p;

            // Convert from *c_uchar to *u8
            let data = reinterpret_cast(data);
            let len = len as uint;
            // Copy to a vector
            let data = vec_from_buf(data, len);
            data_ch.send(data);

            return CAIRO_STATUS_SUCCESS;
        }

        let closure = addr_of(data_ch);

        unsafe {
            cairo_surface_write_to_png_stream(
                cairo_surf, write_fn, reinterpret_cast(closure));
        }

        // Collect the entire image into a single vector
        let mut result = ~[];
        while data_ch.peek() {
            result += data_ch.recv();
        }

        // Send the PNG image away
        output.send(result);
    });
    // Send the next draw target to the renderer
    sender.send(dt);
}

#[test]
fn sanity_check() {
    listen(|self_channel| {

        let sink = PngSink(self_channel);
        let renderer = Renderer(sink);

        let dlist : display_list = dvec();
        renderer.send(RenderMsg(dlist));
        let (exit_chan, exit_response_from_engine) = pipes::stream();
        renderer.send(renderer::ExitMsg(exit_chan));
        exit_response_from_engine.recv();

        sink.send(Exit)
    })
}
