/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::default::Rect;
use euclid::Size2D;
use script_traits::UntrustedNodeAddress;
use servo_arc::Arc;
use style::properties::style_structs::Font;
use style_traits::CSSPixel;
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
    /// Query layout to get the resolved font style for canvas.
    fn resolved_font_style(&self) -> Option<Arc<Font>>;
    fn offset_parent(&self) -> OffsetParentResponse;
    fn text_index(&self) -> TextIndexResponse;
    /// Requests the list of nodes from the given point.
    fn nodes_from_point_response(&self) -> Vec<UntrustedNodeAddress>;
    /// Query layout to get the inner text for a given element.
    fn element_inner_text(&self) -> String;
    /// Get the dimensions of an iframe's inner window.
    fn inner_window_dimensions(&self) -> Option<Size2D<f32, CSSPixel>>;
}

pub struct ContentBoxResponse(pub Option<Rect<Au>>);

pub struct ContentBoxesResponse(pub Vec<Rect<Au>>);

pub struct NodeGeometryResponse {
    pub client_rect: Rect<i32>,
}

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
pub struct TextIndexResponse(pub Option<usize>);
