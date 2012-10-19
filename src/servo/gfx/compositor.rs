use dom::event::Event;
use azure::cairo_hl::ImageSurface;
use azure::azure_hl::{DrawTarget};
use geom::size::Size2D;

struct LayerBuffer {
    // TODO: We should not be coupled to Cairo this tightly. Instead we should pull the buffer out
    // of the draw target with the Azure API.
    cairo_surface: ImageSurface,

    draw_target: DrawTarget,

    size: Size2D<uint>,

    // NB: stride is in pixels, like OpenGL GL_UNPACK_ROW_LENGTH.
    stride: uint
}

/**
The interface used to by the renderer to aquire draw targets for
each rendered frame and submit them to be drawn to the display
*/
trait Compositor {
    fn begin_drawing(next_dt: pipes::Chan<LayerBuffer>);
    fn draw(next_dt: pipes::Chan<LayerBuffer>, +draw_me: LayerBuffer);
}

