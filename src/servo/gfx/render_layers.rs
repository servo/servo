use gfx::display_list::DisplayList;
use gfx::compositor::LayerBuffer;

use azure::azure_hl::DrawTarget;
use azure::cairo::CAIRO_FORMAT_RGB24;
use azure::cairo_hl::ImageSurface;
use core::libc::c_int;
use geom::size::Size2D;

pub struct RenderLayer {
    display_list: DisplayList,
    size: Size2D<uint>
}

/// Given a layer and a buffer, either reuses the buffer (if it's of the right size and format)
/// or creates a new buffer (if it's not of the appropriate size and format) and invokes the
/// given callback with the render layer and the buffer. Returns the resulting layer buffer (which
/// might be the old layer buffer if it had the appropriate size and format).
pub fn render_layers(layer: &RenderLayer,
                     buffer: LayerBuffer,
                     f: &fn(layer: &RenderLayer, buffer: &LayerBuffer) -> bool) -> LayerBuffer {
    let mut buffer = move buffer;
    if buffer.size != layer.size {
        // Create a new buffer.

        // Round the width up the nearest 32 pixels for DMA on the Mac.
        let mut stride = layer.size.width;
        if stride % 32 != 0 {
            stride = (stride & !(32 - 1)) + 32;
        }
        assert stride % 32 == 0;
        assert stride >= layer.size.width;

        let cairo_surface = ImageSurface(CAIRO_FORMAT_RGB24,
                                         stride as c_int,
                                         layer.size.height as c_int);
        let draw_target = DrawTarget(&cairo_surface);
        buffer = LayerBuffer {
            cairo_surface: move cairo_surface,
            draw_target: move draw_target,
            size: copy layer.size,
            stride: stride
        };
    }

    let _ = f(layer, &buffer);
    return move buffer;
}

