/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure_hl::Color;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use layers::platform::surface::{NativeGraphicsMetadata, NativePaintingGraphicsContext};
use layers::platform::surface::{NativeSurface, NativeSurfaceMethods};
use serialize::{Encoder, Encodable};
use std::fmt::{Formatter, Show};
use std::fmt;

use constellation_msg::PipelineId;

pub struct LayerBuffer {
    /// The native surface which can be shared between threads or processes. On Mac this is an
    /// `IOSurface`; on Linux this is an X Pixmap; on Android this is an `EGLImageKHR`.
    pub native_surface: NativeSurface,

    /// The rect in the containing RenderLayer that this represents.
    pub rect: Rect<f32>,

    /// The rect in pixels that will be drawn to the screen.
    pub screen_pos: Rect<uint>,

    /// The scale at which this tile is rendered
    pub resolution: f32,

    /// NB: stride is in pixels, like OpenGL GL_UNPACK_ROW_LENGTH.
    pub stride: uint,
}

/// A set of layer buffers. This is an atomic unit used to switch between the front and back
/// buffers.
pub struct LayerBufferSet {
    pub buffers: Vec<~LayerBuffer>
}

impl LayerBufferSet {
    /// Notes all buffer surfaces will leak if not destroyed via a call to `destroy`.
    pub fn mark_will_leak(&mut self) {
        for buffer in self.buffers.mut_iter() {
            buffer.native_surface.mark_will_leak()
        }
    }
}

/// The status of the renderer.
#[deriving(Eq, Clone)]
pub enum RenderState {
    IdleRenderState,
    RenderingRenderState,
}

#[deriving(Eq, Clone)]
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
pub struct Epoch(pub uint);

impl Epoch {
    pub fn next(&mut self) {
        let Epoch(ref mut u) = *self;
        *u += 1;
    }
}

#[deriving(Clone, Eq)]
pub struct LayerId(pub uint, pub uint);

impl Show for LayerId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LayerId(a, b) = *self;
        write!(f.buf, "Layer({}, {})", a, b);
        Ok(())
    }
}

impl LayerId {
    /// FIXME(#2011, pcwalton): This is unfortunate. Maybe remove this in the future.
    pub fn null() -> LayerId {
        LayerId(0, 0)
    }
}

/// The scrolling policy of a layer.
#[deriving(Eq)]
pub enum ScrollPolicy {
    /// These layers scroll when the parent receives a scrolling message.
    Scrollable,
    /// These layers do not scroll when the parent receives a scrolling message.
    FixedPosition,
}

/// All layer-specific information that the painting task sends to the compositor other than the
/// buffer contents of the layer itself.
pub struct LayerMetadata {
    /// An opaque ID. This is usually the address of the flow and index of the box within it.
    pub id: LayerId,
    /// The position and size of the layer in pixels.
    pub position: Rect<uint>,
    /// The background color of the layer.
    pub background_color: Color,
    /// The scrolling policy of this layer.
    pub scroll_policy: ScrollPolicy,
}

/// The interface used by the renderer to acquire draw targets for each render frame and
/// submit them to be drawn to the display.
pub trait RenderListener {
    fn get_graphics_metadata(&self) -> Option<NativeGraphicsMetadata>;

    /// Informs the compositor of the layers for the given pipeline. The compositor responds by
    /// creating and/or destroying render layers as necessary.
    fn initialize_layers_for_pipeline(&self,
                                      pipeline_id: PipelineId,
                                      metadata: ~[LayerMetadata],
                                      epoch: Epoch);

    fn set_layer_clip_rect(&self,
                           pipeline_id: PipelineId,
                           layer_id: LayerId,
                           new_rect: Rect<uint>);

    fn delete_layer_group(&self, PipelineId);

    /// Sends new tiles for the given layer to the compositor.
    fn paint(&self,
             pipeline_id: PipelineId,
             layer_id: LayerId,
             layer_buffer_set: ~LayerBufferSet,
             epoch: Epoch);

    fn set_render_state(&self, render_state: RenderState);
}

/// The interface used by the script task to tell the compositor to update its ready state,
/// which is used in displaying the appropriate message in the window's title.
pub trait ScriptListener : Clone {
    fn set_ready_state(&self, ReadyState);
    fn scroll_fragment_point(&self,
                             pipeline_id: PipelineId,
                             layer_id: LayerId,
                             point: Point2D<f32>);
    fn close(&self);
    fn dup(&self) -> ~ScriptListener;
}

impl<E, S: Encoder<E>> Encodable<S, E> for ~ScriptListener {
    fn encode(&self, _s: &mut S) -> Result<(), E> {
        Ok(())
    }
}

/// The interface used by the quadtree and buffer map to get info about layer buffers.
pub trait Tile {
    /// Returns the amount of memory used by the tile
    fn get_mem(&self) -> uint;
    /// Returns true if the tile is displayable at the given scale
    fn is_valid(&self, f32) -> bool;
    /// Returns the Size2D of the tile
    fn get_size_2d(&self) -> Size2D<uint>;

    /// Marks the layer buffer as not leaking. See comments on
    /// `NativeSurfaceMethods::mark_wont_leak` for how this is used.
    fn mark_wont_leak(&mut self);

    /// Destroys the layer buffer. Painting task only.
    fn destroy(self, graphics_context: &NativePaintingGraphicsContext);
}

impl Tile for ~LayerBuffer {
    fn get_mem(&self) -> uint {
        // This works for now, but in the future we may want a better heuristic
        self.screen_pos.size.width * self.screen_pos.size.height
    }
    fn is_valid(&self, scale: f32) -> bool {
        (self.resolution - scale).abs() < 1.0e-6
    }
    fn get_size_2d(&self) -> Size2D<uint> {
        self.screen_pos.size
    }
    fn mark_wont_leak(&mut self) {
        self.native_surface.mark_wont_leak()
    }
    fn destroy(self, graphics_context: &NativePaintingGraphicsContext) {
        let mut this = self;
        this.native_surface.destroy(graphics_context)
    }
}

