/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure_hl::Color;
use geom::point::Point2D;
use geom::rect::Rect;
use layers::platform::surface::NativeGraphicsMetadata;
use layers::layers::LayerBufferSet;
use serialize::{Encoder, Encodable};
use std::fmt::{Formatter, Show};
use std::fmt;

use constellation_msg::PipelineId;

/// The status of the renderer.
#[deriving(Eq, Ord, PartialOrd, PartialEq, Clone)]
pub enum RenderState {
    RenderingRenderState,
    IdleRenderState,
}

#[deriving(Eq, Ord, PartialOrd, PartialEq, Clone)]
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
#[deriving(PartialEq, PartialOrd)]
pub struct Epoch(pub uint);

impl Epoch {
    pub fn next(&mut self) {
        let Epoch(ref mut u) = *self;
        *u += 1;
    }
}

#[deriving(Clone, PartialEq)]
pub struct LayerId(pub uint, pub uint);

impl Show for LayerId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LayerId(a, b) = *self;
        write!(f, "Layer({}, {})", a, b)
    }
}

impl LayerId {
    /// FIXME(#2011, pcwalton): This is unfortunate. Maybe remove this in the future.
    pub fn null() -> LayerId {
        LayerId(0, 0)
    }
}

/// The scrolling policy of a layer.
#[deriving(PartialEq)]
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
                                      metadata: Vec<LayerMetadata>,
                                      epoch: Epoch);

    fn set_layer_clip_rect(&self,
                           pipeline_id: PipelineId,
                           layer_id: LayerId,
                           new_rect: Rect<uint>);

    /// Sends new tiles for the given layer to the compositor.
    fn paint(&self,
             pipeline_id: PipelineId,
             epoch: Epoch,
             replies: Vec<(LayerId, Box<LayerBufferSet>)>);

    fn set_render_state(&self, render_state: RenderState, pipeline_id: PipelineId);
}

/// The interface used by the script task to tell the compositor to update its ready state,
/// which is used in displaying the appropriate message in the window's title.
pub trait ScriptListener : Clone {
    fn set_ready_state(&self, pipeline_id: PipelineId, ReadyState);
    fn scroll_fragment_point(&self,
                             pipeline_id: PipelineId,
                             layer_id: LayerId,
                             point: Point2D<f32>);
    fn close(&self);
    fn dup(&self) -> Box<ScriptListener>;
}

impl<E, S: Encoder<E>> Encodable<S, E> for Box<ScriptListener> {
    fn encode(&self, _s: &mut S) -> Result<(), E> {
        Ok(())
    }
}
