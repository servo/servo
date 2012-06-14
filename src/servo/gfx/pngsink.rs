#[doc = "
A graphics sink that renders to PNG format buffers

Each time the renderer renders a frame the bufsink will output a
`[u8]` containing the frame in PNG format.
"];

export msg, pngsink;

import azure::cairo;
import azure_bg = azure::bindgen;
import azure::AzDrawTargetRef;
import cairo_bg = cairo::bindgen;

enum msg {
    begin_drawing(chan<AzDrawTargetRef>),
    draw(chan<AzDrawTargetRef>, AzDrawTargetRef),
    exit
}

impl pngsink of renderer::sink for chan<msg> {
    fn begin_drawing(next_dt: chan<AzDrawTargetRef>) {
        self.send(begin_drawing(next_dt))
    }
    fn draw(next_dt: chan<AzDrawTargetRef>, draw_me: AzDrawTargetRef) {
        self.send(draw(next_dt, draw_me))
    }
}

fn pngsink(output: chan<[u8]>) -> chan<msg> {
    task::spawn_listener::<msg> { |po|

        let cairo_surf = cairo_bg::cairo_image_surface_create(
            cairo::CAIRO_FORMAT_ARGB32, 800 as libc::c_int, 600 as libc::c_int
            );
        assert cairo_surf.is_not_null();

        let draw_target = azure_bg::AzCreateDrawTargetForCairoSurface(cairo_surf);
        assert draw_target.is_not_null();

        loop {
            alt po.recv() {
              begin_drawing(sender) {
                #debug("pngsink: begin_drawing");
                sender.send(draw_target);
              }
              draw(sender, dt) {
                #debug("pngsink: draw");
                do_draw(sender, dt, output, cairo_surf);
              }
              exit { break }
            }
        }

        azure_bg::AzReleaseDrawTarget(draw_target);
        cairo_bg::cairo_surface_destroy(cairo_surf);
    }
}

fn do_draw(sender: chan<AzDrawTargetRef>,
           dt: AzDrawTargetRef,
           output: comm::chan<[u8]>,
           cairo_surf: *cairo::cairo_surface_t) {

    import libc::*;

    listen {|data_ch|

        crust fn write_fn(closure: *c_void,
                          data: *c_uchar,
                          len: c_uint)

            -> cairo::cairo_status_t unsafe {

            let p: *chan<[u8]> = unsafe::reinterpret_cast(closure);
            let data_ch = *p;

            // Convert from *c_uchar to *u8
            let data = unsafe::reinterpret_cast(data);
            let len = len as uint;
            // Copy to a vector
            let data = vec::unsafe::from_buf(data, len);
            data_ch.send(data);

            ret cairo::CAIRO_STATUS_SUCCESS;
        }

        let closure = ptr::addr_of(data_ch);

        unsafe {
            cairo_bg::cairo_surface_write_to_png_stream(
                cairo_surf, write_fn, unsafe::reinterpret_cast(closure));
        }

        // Collect the entire image into a single vector
        let mut result = [];
        while data_ch.peek() {
            result += data_ch.recv();
        }

        // Send the PNG image away
        output.send(result);
    }
    // Send the next draw target to the renderer
    sender.send(dt);
}

#[test]
fn sanity_check() {
    listen {
        |self_channel|

        let sink = pngsink(self_channel);
        let renderer = renderer::renderer(sink);

        let dlist = [];
        renderer.send(renderer::RenderMsg(dlist));
        listen {
            |from_renderer|
            renderer.send(renderer::ExitMsg(from_renderer));
            from_renderer.recv();
        }

        sink.send(exit)
    }
}
