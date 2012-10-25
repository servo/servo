use azure::azure_hl::{DrawTarget};
use cairo::cairo_hl::ImageSurface;
use dom::event::Event;
use geom::rect::Rect;

struct LayerBuffer {
    // TODO: We should not be coupled to Cairo this tightly. Instead we should pull the buffer out
    // of the draw target with the Azure API.
    cairo_surface: ImageSurface,

    draw_target: DrawTarget,

    // The rect in the containing RenderLayer that this represents.
    rect: Rect<uint>,

    // NB: stride is in pixels, like OpenGL GL_UNPACK_ROW_LENGTH.
    stride: uint
}

/// A set of layer buffers. This is an atomic unit used to switch between the front and back
/// buffers.
struct LayerBufferSet {
    buffers: ~[LayerBuffer]
}

/**
The interface used to by the renderer to aquire draw targets for
each rendered frame and submit them to be drawn to the display
*/
trait Compositor {
    fn begin_drawing(next_dt: pipes::Chan<LayerBufferSet>);
    fn draw(next_dt: pipes::Chan<LayerBufferSet>, +draw_me: LayerBufferSet);
}

