/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure_hl::DrawTarget;
use azure::azure::AzGLContext;
use geom::rect::Rect;
use geom::size::Size2D;

use constellation_msg::PipelineId;

#[deriving(Clone)]
pub struct LayerBuffer {
    draw_target: DrawTarget,

    // The rect in the containing RenderLayer that this represents.
    rect: Rect<f32>,

    // The rect in pixels that will be drawn to the screen.
    screen_pos: Rect<uint>,

    // The scale at which this tile is rendered
    resolution: f32,

    // NB: stride is in pixels, like OpenGL GL_UNPACK_ROW_LENGTH.
    stride: uint,
        
}

/// A set of layer buffers. This is an atomic unit used to switch between the front and back
/// buffers.
#[deriving(Clone)]
pub struct LayerBufferSet {
    buffers: ~[~LayerBuffer]
}

/// The status of the renderer.
#[deriving(Eq)]
pub enum RenderState {
    IdleRenderState,
    RenderingRenderState,
}

#[deriving(Eq)]
pub enum ReadyState {
    /// Informs the compositor that nothing has been done yet. Used for setting status
    Blank,
    /// Informs the compositor that a page is loading. Used for setting status
    Loading,
    /// Informs the compositor that a page is performing layout. Used for setting status
    PerformingLayout,
    /// Informs the compositor that a page is finished loading. Used for setting status
    FinishedLoading,
}

/// A newtype struct for denoting the age of messages; prevents race conditions.
#[deriving(Eq)]
pub struct Epoch(uint);

impl Epoch {
    pub fn next(&mut self) {
        **self += 1;
    }
}

/// The interface used by the renderer to acquire draw targets for each render frame and
/// submit them to be drawn to the display.
pub trait RenderListener {
    fn get_gl_context(&self) -> AzGLContext;
    fn new_layer(&self, PipelineId, Size2D<uint>);
    fn set_layer_page_size(&self, PipelineId, Size2D<uint>, Epoch);
    fn set_layer_clip_rect(&self, PipelineId, Rect<uint>);
    fn delete_layer(&self, PipelineId);
    fn paint(&self, id: PipelineId, layer_buffer_set: ~LayerBufferSet, Epoch);
    fn set_render_state(&self, render_state: RenderState);
}

/// The interface used by the script task to tell the compositor to update its ready state,
/// which is used in displaying the appropriate message in the window's title.
pub trait ScriptListener : Clone {
    fn set_ready_state(&self, ReadyState);
    fn invalidate_rect(&self, PipelineId, Rect<uint>);
    fn close(&self);
}

/// The interface used by the quadtree to get info about LayerBuffers
pub trait Tile {
    /// Returns the amount of memory used by the tile
    fn get_mem(&self) -> uint;
    /// Returns true if the tile is displayable at the given scale
    fn is_valid(&self, f32) -> bool;
    /// Returns the Size2D of the tile
    fn get_size_2d(&self) -> Size2D<uint>;
}

impl Tile for ~LayerBuffer {
    fn get_mem(&self) -> uint {
        // This works for now, but in the future we may want a better heuristic
        self.screen_pos.size.width * self.screen_pos.size.height
    }
    fn is_valid(&self, scale: f32) -> bool {
        self.resolution.approx_eq(&scale)
    }
    fn get_size_2d(&self) -> Size2D<uint> {
        self.screen_pos.size
    }
}
