/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities for querying the layout, as needed by the layout thread.

use crate::context::LayoutContext;
use crate::flow::FragmentTreeRoot;
use app_units::Au;
use euclid::default::{Point2D, Rect};
use euclid::Size2D;
use euclid::Vector2D;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use script_layout_interface::rpc::TextIndexResponse;
use script_layout_interface::rpc::{ContentBoxResponse, ContentBoxesResponse, LayoutRPC};
use script_layout_interface::rpc::{NodeGeometryResponse, NodeScrollIdResponse};
use script_layout_interface::rpc::{OffsetParentResponse, ResolvedStyleResponse, StyleResponse};
use script_layout_interface::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use script_traits::LayoutMsg as ConstellationMsg;
use script_traits::UntrustedNodeAddress;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use style::dom::OpaqueNode;
use style::properties::PropertyId;
use style::selector_parser::PseudoElement;
use style_traits::CSSPixel;
use webrender_api::units::LayoutPixel;
use webrender_api::ExternalScrollId;

/// Mutable data belonging to the LayoutThread.
///
/// This needs to be protected by a mutex so we can do fast RPCs.
pub struct LayoutThreadData {
    /// The channel on which messages can be sent to the constellation.
    pub constellation_chan: IpcSender<ConstellationMsg>,

    /// The root stacking context.
    pub display_list: Option<webrender_api::DisplayListBuilder>,

    /// A queued response for the union of the content boxes of a node.
    pub content_box_response: Option<Rect<Au>>,

    /// A queued response for the content boxes of a node.
    pub content_boxes_response: Vec<Rect<Au>>,

    /// A queued response for the client {top, left, width, height} of a node in pixels.
    pub client_rect_response: Rect<i32>,

    /// A queued response for the scroll id for a given node.
    pub scroll_id_response: Option<ExternalScrollId>,

    /// A queued response for the scroll {top, left, width, height} of a node in pixels.
    pub scroll_area_response: Rect<i32>,

    /// A queued response for the resolved style property of an element.
    pub resolved_style_response: String,

    /// A queued response for the offset parent/rect of a node.
    pub offset_parent_response: OffsetParentResponse,

    /// A queued response for the style of a node.
    pub style_response: StyleResponse,

    /// Scroll offsets of scrolling regions.
    pub scroll_offsets: HashMap<ExternalScrollId, Vector2D<f32, LayoutPixel>>,

    /// Index in a text fragment. We need this do determine the insertion point.
    pub text_index_response: TextIndexResponse,

    /// A queued response for the list of nodes at a given point.
    pub nodes_from_point_response: Vec<UntrustedNodeAddress>,

    /// A queued response for the inner text of a given element.
    pub element_inner_text_response: String,

    /// A queued response for the viewport dimensions for a given browsing context.
    pub inner_window_dimensions_response: Option<Size2D<f32, CSSPixel>>,
}

pub struct LayoutRPCImpl(pub Arc<Mutex<LayoutThreadData>>);

impl LayoutRPC for LayoutRPCImpl {
    // The neat thing here is that in order to answer the following two queries we only
    // need to compare nodes for equality. Thus we can safely work only with `OpaqueNode`.
    fn content_box(&self) -> ContentBoxResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        ContentBoxResponse(rw_data.content_box_response)
    }

    /// Requests the dimensions of all the content boxes, as in the `getClientRects()` call.
    fn content_boxes(&self) -> ContentBoxesResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        ContentBoxesResponse(rw_data.content_boxes_response.clone())
    }

    fn nodes_from_point_response(&self) -> Vec<UntrustedNodeAddress> {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        rw_data.nodes_from_point_response.clone()
    }

    fn node_geometry(&self) -> NodeGeometryResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        NodeGeometryResponse {
            client_rect: rw_data.client_rect_response,
        }
    }

    fn node_scroll_area(&self) -> NodeGeometryResponse {
        NodeGeometryResponse {
            client_rect: self.0.lock().unwrap().scroll_area_response,
        }
    }

    fn node_scroll_id(&self) -> NodeScrollIdResponse {
        NodeScrollIdResponse(
            self.0
                .lock()
                .unwrap()
                .scroll_id_response
                .expect("scroll id is not correctly fetched"),
        )
    }

    /// Retrieves the resolved value for a CSS style property.
    fn resolved_style(&self) -> ResolvedStyleResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        ResolvedStyleResponse(rw_data.resolved_style_response.clone())
    }

    fn offset_parent(&self) -> OffsetParentResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        rw_data.offset_parent_response.clone()
    }

    fn style(&self) -> StyleResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        rw_data.style_response.clone()
    }

    fn text_index(&self) -> TextIndexResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        rw_data.text_index_response.clone()
    }

    fn element_inner_text(&self) -> String {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        rw_data.element_inner_text_response.clone()
    }

    fn inner_window_dimensions(&self) -> Option<Size2D<f32, CSSPixel>> {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        rw_data.inner_window_dimensions_response.clone()
    }
}

pub fn process_content_box_request(
    requested_node: OpaqueNode,
    fragment_tree_root: Option<&FragmentTreeRoot>,
) -> Option<Rect<Au>> {
    let fragment_tree_root = match fragment_tree_root {
        Some(fragment_tree_root) => fragment_tree_root,
        None => return None,
    };

    Some(fragment_tree_root.get_content_box_for_node(requested_node))
}

pub fn process_content_boxes_request(_requested_node: OpaqueNode) -> Vec<Rect<Au>> {
    vec![]
}

pub fn process_node_geometry_request(
    requested_node: OpaqueNode,
    fragment_tree_root: Option<&FragmentTreeRoot>,
) -> Rect<i32> {
    let fragment_tree_root = match fragment_tree_root {
        Some(fragment_tree_root) => fragment_tree_root,
        None => return Rect::zero(),
    };

    fragment_tree_root.get_border_dimensions_for_node(requested_node)
}

pub fn process_node_scroll_id_request<N: LayoutNode>(
    id: PipelineId,
    requested_node: N,
) -> ExternalScrollId {
    let layout_node = requested_node.to_threadsafe();
    layout_node.generate_scroll_id(id)
}

/// https://drafts.csswg.org/cssom-view/#scrolling-area
pub fn process_node_scroll_area_request(_requested_node: OpaqueNode) -> Rect<i32> {
    Rect::zero()
}

/// Return the resolved value of property for a given (pseudo)element.
/// <https://drafts.csswg.org/cssom/#resolved-value>
pub fn process_resolved_style_request<'a, N>(
    _context: &LayoutContext,
    _node: N,
    _pseudo: &Option<PseudoElement>,
    _property: &PropertyId,
) -> String
where
    N: LayoutNode,
{
    "".to_owned()
}

pub fn process_offset_parent_query(_requested_node: OpaqueNode) -> OffsetParentResponse {
    OffsetParentResponse::empty()
}

pub fn process_style_query<N: LayoutNode>(_requested_node: N) -> StyleResponse {
    StyleResponse(None)
}

// https://html.spec.whatwg.org/multipage/#the-innertext-idl-attribute
pub fn process_element_inner_text_query<N: LayoutNode>(_node: N) -> String {
    "".to_owned()
}

pub fn process_text_index_request(_node: OpaqueNode, _point: Point2D<Au>) -> TextIndexResponse {
    TextIndexResponse(None)
}
