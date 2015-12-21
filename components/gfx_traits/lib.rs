/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(custom_derive, plugin)]
#![plugin(plugins, serde_macros)]

#![crate_name = "gfx_traits"]
#![crate_type = "rlib"]

extern crate azure;
extern crate euclid;
extern crate layers;
extern crate msg;
extern crate serde;
extern crate util;

pub mod color;
mod paint_listener;

pub use paint_listener::PaintListener;
use azure::azure_hl::Color;
use euclid::matrix::Matrix4;
use euclid::rect::Rect;
use msg::compositor_msg::LayerType;
use msg::constellation_msg::{Failure, PipelineId};
use std::fmt::{self, Debug, Formatter};

/// Messages from the paint task to the constellation.
#[derive(Deserialize, Serialize)]
pub enum PaintMsg {
    Failure(Failure),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayerKind {
    NoTransform,
    HasTransform,
}

/// The scrolling policy of a layer.
#[derive(Clone, PartialEq, Eq, Copy, Deserialize, Serialize, Debug, HeapSizeOf)]
pub enum ScrollPolicy {
    /// These layers scroll when the parent receives a scrolling message.
    Scrollable,
    /// These layers do not scroll when the parent receives a scrolling message.
    FixedPosition,
}

#[derive(Clone, PartialEq, Eq, Copy, Hash, Deserialize, Serialize, HeapSizeOf)]
pub struct LayerId(
    /// The type of the layer. This serves to differentiate layers that share fragments.
    LayerType,
    /// The identifier for this layer's fragment, derived from the fragment memory address.
    usize,
    /// An index for identifying companion layers, synthesized to ensure that
    /// content on top of this layer's fragment has the proper rendering order.
    usize
);

impl Debug for LayerId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LayerId(layer_type, id, companion) = *self;
        let type_string = match layer_type {
            LayerType::FragmentBody => "-FragmentBody",
            LayerType::OverflowScroll => "-OverflowScroll",
            LayerType::BeforePseudoContent => "-BeforePseudoContent",
            LayerType::AfterPseudoContent => "-AfterPseudoContent",
        };

        write!(f, "{}{}-{}", id, type_string, companion)
    }
}

impl LayerId {
    /// FIXME(#2011, pcwalton): This is unfortunate. Maybe remove this in the future.
    pub fn null() -> LayerId {
        LayerId(LayerType::FragmentBody, 0, 0)
    }

    pub fn new_of_type(layer_type: LayerType, fragment_id: usize) -> LayerId {
        LayerId(layer_type, fragment_id, 0)
    }

    pub fn companion_layer_id(&self) -> LayerId {
        let LayerId(layer_type, id, companion) = *self;
        LayerId(layer_type, id, companion + 1)
    }
}

/// All layer-specific information that the painting task sends to the compositor other than the
/// buffer contents of the layer itself.
#[derive(Copy, Clone)]
pub struct LayerProperties {
    /// An opaque ID. This is usually the address of the flow and index of the box within it.
    pub id: LayerId,
    /// The id of the parent layer.
    pub parent_id: Option<LayerId>,
    /// The position and size of the layer in pixels.
    pub rect: Rect<f32>,
    /// The background color of the layer.
    pub background_color: Color,
    /// The scrolling policy of this layer.
    pub scroll_policy: ScrollPolicy,
    /// The transform for this layer
    pub transform: Matrix4,
    /// The perspective transform for this layer
    pub perspective: Matrix4,
    /// The subpage that this layer represents. If this is `Some`, this layer represents an
    /// iframe.
    pub subpage_pipeline_id: Option<PipelineId>,
    /// Whether this layer establishes a new 3d rendering context.
    pub establishes_3d_context: bool,
    /// Whether this layer scrolls its overflow area.
    pub scrolls_overflow_area: bool,
}
