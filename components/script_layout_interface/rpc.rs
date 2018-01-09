/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::{Point2D, Rect};
use script_traits::UntrustedNodeAddress;
use servo_arc::Arc;
use style::properties::ComputedValues;
use style::properties::longhands::overflow_x;
use webrender_api::ExternalScrollId;

/// Synchronous messages that script can send to layout.
///
/// In general, you should use messages to talk to Layout. Use the RPC interface
/// if and only if the work is
///
///   1) read-only with respect to LayoutThreadData,
///   2) small,
///   3) and really needs to be fast.
pub trait LayoutRPC {
    /// Requests the dimensions of the content box, as in the `getBoundingClientRect()` call.
    fn content_box(&self) -> ContentBoxResponse;
    /// Requests the dimensions of all the content boxes, as in the `getClientRects()` call.
    fn content_boxes(&self) -> ContentBoxesResponse;
    /// Requests the geometry of this node. Used by APIs such as `clientTop`.
    fn node_geometry(&self) -> NodeGeometryResponse;
    /// Requests the scroll geometry of this node. Used by APIs such as `scrollTop`.
    fn node_scroll_area(&self) -> NodeGeometryResponse;
    /// Requests the scroll id of this node. Used by APIs such as `scrollTop`
    fn node_scroll_id(&self) -> NodeScrollIdResponse;
    /// Query layout for the resolved value of a given CSS property
    fn resolved_style(&self) -> ResolvedStyleResponse;
    fn offset_parent(&self) -> OffsetParentResponse;
    /// Requests the styles for an element. Contains a `None` value if the element is in a `display:
    /// none` subtree.
    fn style(&self) -> StyleResponse;
    fn text_index(&self) -> TextIndexResponse;
    /// Requests the list of nodes from the given point.
    fn nodes_from_point_response(&self) -> Vec<UntrustedNodeAddress>;
    /// Query layout to get the inner text for a given element.
    fn element_inner_text(&self) -> String;
}

pub struct ContentBoxResponse(pub Option<Rect<Au>>);

pub struct ContentBoxesResponse(pub Vec<Rect<Au>>);

pub struct NodeGeometryResponse {
    pub client_rect: Rect<i32>,
}

pub struct NodeOverflowResponse(pub Option<Point2D<overflow_x::computed_value::T>>);

pub struct NodeScrollIdResponse(pub ExternalScrollId);

pub struct ResolvedStyleResponse(pub String);

#[derive(Clone)]
pub struct OffsetParentResponse {
    pub node_address: Option<UntrustedNodeAddress>,
    pub rect: Rect<Au>,
}

impl OffsetParentResponse {
    pub fn empty() -> OffsetParentResponse {
        OffsetParentResponse {
            node_address: None,
            rect: Rect::zero(),
        }
    }
}

#[derive(Clone)]
pub struct StyleResponse(pub Option<Arc<ComputedValues>>);

#[derive(Clone)]
pub struct TextIndexResponse(pub Option<usize>);
