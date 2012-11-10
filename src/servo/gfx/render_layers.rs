use gfx::display_list::DisplayList;
use gfx::compositor::{LayerBuffer, LayerBufferSet};
use opts::Opts;

use azure::AzFloat;
use azure::azure_hl::{B8G8R8A8, DrawTarget};
use core::libc::c_int;
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

const TILE_SIZE: uint = 512;

pub struct RenderLayer {
    display_list: DisplayList,
    size: Size2D<uint>
}

/// Given a layer and a buffer, either reuses the buffer (if it's of the right size and format)
/// or creates a new buffer (if it's not of the appropriate size and format) and invokes the
/// given callback with the render layer and the buffer. Returns the resulting layer buffer (which
/// might be the old layer buffer if it had the appropriate size and format).
pub fn render_layers(layer: &RenderLayer,
                     buffer_set: LayerBufferSet,
                     opts: &Opts,
                     f: &fn(layer: &RenderLayer, buffer: &LayerBuffer) -> bool) -> LayerBufferSet {
    let mut buffers = match move buffer_set { LayerBufferSet { buffers: move b } => move b };

    // FIXME: Try not to create a new array here.
    let new_buffers = dvec::DVec();

    // Divide up the layer into tiles.
    let mut y = 0;
    while y < layer.size.height {
        let mut x = 0;
        while x < layer.size.width {
            // Figure out the dimension of this tile.
            let right = uint::min(x + TILE_SIZE, layer.size.width);
            let bottom = uint::min(y + TILE_SIZE, layer.size.height);
            let width = right - x;
            let height = bottom - y;

            // Round the width up the nearest 32 pixels for DMA on the Mac.
            let mut stride = width;
            if stride % 32 != 0 {
                stride = (stride & !(32 - 1)) + 32;
            }
            assert stride % 32 == 0;
            assert stride >= width;

            debug!("tile stride %u", stride);

            let tile_rect = Rect(Point2D(x, y), Size2D(width, height));

            let buffer;
            // FIXME: Try harder to search for a matching tile.
            // FIXME: Don't use shift; it's bad for perf. Maybe reverse and pop.
            /*if buffers.len() != 0 && buffers[0].rect == tile_rect {
                debug!("reusing tile, (%u, %u)", x, y);
                buffer = buffers.shift();
            } else {*/
                // Create a new buffer.
                debug!("creating tile, (%u, %u)", x, y);

                let size = Size2D(stride as i32, height as i32);

                let mut data: ~[u8] = ~[0];
                let offset;
                unsafe {
                    // FIXME: Evil black magic to ensure that we don't perform a slow memzero
                    // of this buffer. This should be made safe.

                    let align = 256;

                    let len = ((size.width * size.height * 4) as uint) + align;
                    vec::reserve(&mut data, len);
                    vec::raw::set_len(&mut data, len);

                    // Round up to the nearest 32-byte-aligned address for DMA on the Mac.
                    let addr: uint = cast::transmute(ptr::to_unsafe_ptr(&data[0]));
                    if addr % align == 0 {
                        offset = 0;
                    } else {
                        offset = align - addr % align;
                    }

                    debug!("tile offset is %u, expected addr is %x", offset, addr + offset);
                }

                buffer = LayerBuffer {
                    draw_target: DrawTarget::new_with_data(opts.render_backend,
                                                           move data,
                                                           offset,
                                                           size,
                                                           size.width * 4,
                                                           B8G8R8A8),
                    rect: tile_rect,
                    stride: stride
                };
            //}

            let _ = f(layer, &buffer);
            new_buffers.push(move buffer);

            x += TILE_SIZE;
        }
        y += TILE_SIZE;
    }

    return LayerBufferSet { buffers: move dvec::unwrap(move new_buffers) };
}

