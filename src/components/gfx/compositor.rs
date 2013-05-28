/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure_hl::{DrawTarget};
use geom::rect::Rect;

pub struct LayerBuffer {
    draw_target: DrawTarget,

    // The rect in the containing RenderLayer that this represents.
    rect: Rect<uint>,

    // NB: stride is in pixels, like OpenGL GL_UNPACK_ROW_LENGTH.
    stride: uint
}

/// A set of layer buffers. This is an atomic unit used to switch between the front and back
/// buffers.
pub struct LayerBufferSet {
    buffers: ~[LayerBuffer]
}

/// The interface used to by the renderer to acquire draw targets for each rendered frame and
/// submit them to be drawn to the display.
pub trait Compositor {
    fn paint(&self, layer_buffer_set: LayerBufferSet);
}

