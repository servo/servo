/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor::{LayerBuffer, LayerBufferSet};
use display_list::DisplayList;
use opts::Opts;
use util::time;

use azure::azure_hl::{B8G8R8A8, DrawTarget};
use core::comm::Chan;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

pub struct RenderLayer {
    display_list: DisplayList,
    size: Size2D<uint>
}

type RenderFn<'self> = &'self fn(layer: *RenderLayer,
                                 buffer: LayerBuffer,
                                 return_buffer: Chan<LayerBuffer>);

/// Given a layer and a buffer, either reuses the buffer (if it's of the right size and format)
/// or creates a new buffer (if it's not of the appropriate size and format) and invokes the
/// given callback with the render layer and the buffer. Returns the resulting layer buffer (which
/// might be the old layer buffer if it had the appropriate size and format).
pub fn render_layers(layer_ref: *RenderLayer,
                     buffer_set: LayerBufferSet,
                     opts: &Opts,
                     f: RenderFn) -> LayerBufferSet {
    let tile_size = opts.tile_size;

    let mut _buffers = match buffer_set { LayerBufferSet { buffers: b } => b };

    // FIXME: Try not to create a new array here.
    let mut new_buffer_ports = ~[];

    // Divide up the layer into tiles.
    do time::time("rendering: preparing buffers") {
        let layer: &RenderLayer = unsafe { cast::transmute(layer_ref) };
        let mut y = 0;
        while y < layer.size.height {
            let mut x = 0;
            while x < layer.size.width {
                // Figure out the dimension of this tile.
                let right = uint::min(x + tile_size, layer.size.width);
                let bottom = uint::min(y + tile_size, layer.size.height);
                let width = right - x;
                let height = bottom - y;

                // Round the width up the nearest 32 pixels for DMA on the Mac.
                let mut stride = width;
                if stride % 32 != 0 {
                    stride = (stride & !(32 - 1)) + 32;
                }
                assert!(stride % 32 == 0);
                assert!(stride >= width);

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
                                                               data,
                                                               offset,
                                                               size,
                                                               size.width * 4,
                                                               B8G8R8A8),
                        rect: tile_rect,
                        stride: stride
                    };
                //}

                // Create a port and channel pair to receive the new buffer.
                let (new_buffer_port, new_buffer_chan) = comm::stream();

                // Send the buffer to the child.
                f(layer_ref, buffer, new_buffer_chan);

                // Enqueue the port.
                new_buffer_ports.push(new_buffer_port);

                x += tile_size;
            }
            y += tile_size;
        }
    }

    let mut new_buffers = ~[];
    do time::time("rendering: waiting on subtasks") {
        for new_buffer_ports.each |new_buffer_port| {
            new_buffers.push(new_buffer_port.recv());
        }
    }

    return LayerBufferSet { buffers: new_buffers };
}

