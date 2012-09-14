#[doc = "
A graphics compositor that renders to PNG format buffers

Each time the renderer renders a frame the compositor will output a
`~[u8]` containing the frame in PNG format.
"];

use libc::{c_int, c_uint, c_void, c_uchar};
use azure_bg = azure::bindgen;
use azure_bg::{AzCreateDrawTargetForCairoSurface, AzReleaseDrawTarget};
use azure::cairo;
use azure::azure_hl::DrawTarget;
use azure::cairo_hl::ImageSurface;
use cairo::{CAIRO_FORMAT_ARGB32, cairo_surface_t, cairo_status_t, CAIRO_STATUS_SUCCESS};
use cairo_bg = cairo::bindgen;
use cairo_bg::{cairo_image_surface_create, cairo_surface_destroy,
                  cairo_surface_write_to_png_stream};
use compositor::Compositor;
use render_task::{RenderTask, RenderMsg};
use task::spawn_listener;
use comm::{Chan, Port};
use unsafe::reinterpret_cast;
use vec_from_buf = vec::raw::from_buf;
use ptr::addr_of;
use dom::event::Event;
use dvec::DVec;
use display_list::DisplayList;
use std::cell::Cell;

pub type PngCompositor = Chan<Msg>;

pub enum Msg {
    BeginDrawing(pipes::Chan<DrawTarget>),
    Draw(pipes::Chan<DrawTarget>, DrawTarget),
    Exit
}

impl Chan<Msg> : Compositor {
    fn begin_drawing(+next_dt: pipes::Chan<DrawTarget>) {
        self.send(BeginDrawing(next_dt))
    }
    fn draw(+next_dt: pipes::Chan<DrawTarget>, +draw_me: DrawTarget) {
        self.send(Draw(next_dt, draw_me))
    }
    fn add_event_listener(_listener: Chan<Event>) {
        // No events in this compositor.
    }
}

pub fn PngCompositor(output: Chan<~[u8]>) -> PngCompositor {
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

fn do_draw(sender: pipes::Chan<DrawTarget>,
           +dt: DrawTarget,
           output: Chan<~[u8]>,
           cairo_surface: ImageSurface) {
    let buffer = io::mem_buffer();
    cairo_surface.write_to_png_stream(&buffer);
    let @{ buf: buffer, pos: _ } <- buffer;
    output.send(dvec::unwrap(move buffer));

    // Send the next draw target to the renderer
    sender.send(move dt);
}

#[test]
fn sanity_check() {
    do listen |self_channel| {
        let compositor = PngCompositor(self_channel);
        let renderer = RenderTask(compositor);

        let dlist : DisplayList = DVec();
        renderer.send(RenderMsg(dlist));
        let (exit_chan, exit_response_from_engine) = pipes::stream();
        renderer.send(render_task::ExitMsg(exit_chan));
        exit_response_from_engine.recv();

        compositor.send(Exit)
    }
}
