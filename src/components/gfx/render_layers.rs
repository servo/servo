/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositor::{LayerBuffer, LayerBufferSet};
use display_list::DisplayList;
use opts::Opts;
use servo_util::time;
use servo_util::time::ProfilerChan;

use azure::azure_hl::{B8G8R8A8, DrawTarget};
use core::comm::Chan;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

/// The type representing the lack of extra display list data. This is used when sending display
/// list data off to be rendered.
pub type Nothing = ();

pub struct RenderLayer {
    display_list: DisplayList<Nothing>,
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
                     opts: &Opts,
                     prof_chan: ProfilerChan,
                     f: RenderFn)
                     -> LayerBufferSet {
    let tile_size = opts.tile_size;
    let scale = opts.zoom;

    // FIXME: Try not to create a new array here.
    let mut new_buffer_ports = ~[];

    // Divide up the layer into tiles.
    do time::profile(time::RenderingPrepBuffCategory, prof_chan.clone()) {
        let layer: &RenderLayer = unsafe { cast::transmute(layer_ref) };
        let mut y = 0;
        while y < layer.size.height * scale {
            let mut x = 0;
            while x < layer.size.width * scale {
                // Figure out the dimension of this tile.
                let right = uint::min(x + tile_size, layer.size.width * scale);
                let bottom = uint::min(y + tile_size, layer.size.height * scale);
                let width = right - x;
                let height = bottom - y;

                // Round the width up the nearest 32 pixels for DMA on the Mac.
                let aligned_width = if width % 32 == 0 {
                    width
                } else {
                    (width & !(32 - 1)) + 32
                };
                assert!(aligned_width % 32 == 0);
                assert!(aligned_width >= width);

                debug!("tile aligned_width %u", aligned_width);

                let tile_rect = Rect(Point2D(x / scale, y / scale), Size2D(aligned_width, height)); //change this
                let screen_rect = Rect(Point2D(x, y), Size2D(aligned_width, height)); //change this

                let buffer;
                // FIXME: Try harder to search for a matching tile.
                // FIXME: Don't use shift; it's bad for perf. Maybe reverse and pop.
                /*if buffers.len() != 0 && buffers[0].rect == tile_rect {
                    debug!("reusing tile, (%u, %u)", x, y);
                    buffer = buffers.shift();
                } else {*/
                    // Create a new buffer.
                    debug!("creating tile, (%u, %u)", x, y);

                    let size = Size2D(aligned_width as i32, height as i32);
                    // FIXME: This may not be always true.
                    let stride = (aligned_width as i32) * 4;

                    let mut data: ~[u8] = ~[0];
                    let offset;
                    unsafe {
                        // FIXME: Evil black magic to ensure that we don't perform a slow memzero
                        // of this buffer. This should be made safe.

                        let align = 256;

                        let len = ((stride * size.height) as uint) + align;
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
                                                               stride,
                                                               B8G8R8A8),
                        rect: tile_rect,
                        screen_pos: screen_rect,
                        stride: stride as uint
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
    do time::profile(time::RenderingWaitSubtasksCategory, prof_chan.clone()) {
        for new_buffer_ports.each |new_buffer_port| {
            new_buffers.push(new_buffer_port.recv());
        }
    }

    LayerBufferSet {
        buffers: new_buffers,
    }
}

