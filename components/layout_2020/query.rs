/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities for querying the layout, as needed by the layout thread.
use crate::context::LayoutContext;
use crate::flow::FragmentTree;
use crate::fragments::Fragment;
use app_units::Au;
use euclid::default::{Point2D, Rect};
use euclid::Size2D;
use euclid::Vector2D;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use script_layout_interface::rpc::TextIndexResponse;
use script_layout_interface::rpc::{ContentBoxResponse, ContentBoxesResponse, LayoutRPC};
use script_layout_interface::rpc::{NodeGeometryResponse, NodeScrollIdResponse};
use script_layout_interface::rpc::{OffsetParentResponse, ResolvedStyleResponse};
use script_layout_interface::wrapper_traits::{
    LayoutNode, ThreadSafeLayoutElement, ThreadSafeLayoutNode,
};
use script_traits::LayoutMsg as ConstellationMsg;
use script_traits::UntrustedNodeAddress;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use style::computed_values::position::T as Position;
use style::context::{StyleContext, ThreadLocalStyleContext};
use style::dom::OpaqueNode;
use style::dom::TElement;
use style::properties::{LonghandId, PropertyDeclarationId, PropertyId};
use style::selector_parser::PseudoElement;
use style::stylist::RuleInclusion;
use style::traversal::resolve_style;
use style::values::generics::text::LineHeight;
use style_traits::CSSPixel;
use style_traits::ToCss;
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
    fragment_tree: Option<Arc<FragmentTree>>,
) -> Option<Rect<Au>> {
    Some(fragment_tree?.get_content_box_for_node(requested_node))
}

pub fn process_content_boxes_request(_requested_node: OpaqueNode) -> Vec<Rect<Au>> {
    vec![]
}

pub fn process_node_geometry_request(
    requested_node: OpaqueNode,
    fragment_tree: Option<Arc<FragmentTree>>,
) -> Rect<i32> {
    if let Some(fragment_tree) = fragment_tree {
        fragment_tree.get_border_dimensions_for_node(requested_node)
    } else {
        Rect::zero()
    }
}

pub fn process_node_scroll_id_request<'dom>(
    id: PipelineId,
    requested_node: impl LayoutNode<'dom>,
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
pub fn process_resolved_style_request<'dom>(
    context: &LayoutContext,
    node: impl LayoutNode<'dom>,
    pseudo: &Option<PseudoElement>,
    property: &PropertyId,
    fragment_tree: Option<Arc<FragmentTree>>,
) -> String {
    if !node.as_element().unwrap().has_data() {
        return process_resolved_style_request_for_unstyled_node(context, node, pseudo, property);
    }

    // We call process_resolved_style_request after performing a whole-document
    // traversal, so in the common case, the element is styled.
    let layout_element = node.to_threadsafe().as_element().unwrap();
    let layout_element = match *pseudo {
        None => Some(layout_element),
        Some(PseudoElement::Before) => layout_element.get_before_pseudo(),
        Some(PseudoElement::After) => layout_element.get_after_pseudo(),
        Some(_) => {
            warn!("Got unexpected pseudo element type!");
            None
        },
    };

    let layout_element = match layout_element {
        None => {
            // The pseudo doesn't exist, return nothing.  Chrome seems to query
            // the element itself in this case, Firefox uses the resolved value.
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=29006
            return String::new();
        },
        Some(layout_element) => layout_element,
    };

    let style = &*layout_element.resolved_style();
    let longhand_id = match *property {
        PropertyId::LonghandAlias(id, _) | PropertyId::Longhand(id) => id,
        // Firefox returns blank strings for the computed value of shorthands,
        // so this should be web-compatible.
        PropertyId::ShorthandAlias(..) | PropertyId::Shorthand(_) => return String::new(),
        PropertyId::Custom(ref name) => {
            return style.computed_value_to_string(PropertyDeclarationId::Custom(name));
        },
    }
    .to_physical(style.writing_mode);

    let computed_style =
        || style.computed_value_to_string(PropertyDeclarationId::Longhand(longhand_id));

    // We do not yet support pseudo content.
    if pseudo.is_some() {
        return computed_style();
    }

    // https://drafts.csswg.org/cssom/#dom-window-getcomputedstyle
    // Here we are trying to conform to the specification that says that getComputedStyle
    // should return the used values in certain circumstances. For size and positional
    // properties we might need to walk the Fragment tree to figure those out. We always
    // fall back to returning the computed value.

    // For line height, the resolved value is the computed value if it
    // is "normal" and the used value otherwise.
    if longhand_id == LonghandId::LineHeight {
        let font_size = style.get_font().font_size.size.0;
        return match style.get_inherited_text().line_height {
            LineHeight::Normal => computed_style(),
            LineHeight::Number(value) => (font_size * value.0).to_css_string(),
            LineHeight::Length(value) => value.0.to_css_string(),
        };
    }

    // https://drafts.csswg.org/cssom/#dom-window-getcomputedstyle
    // The properties that we calculate below all resolve to the computed value
    // when the element is display:none or display:contents.
    let display = style.get_box().display;
    if display.is_none() || display.is_contents() {
        return computed_style();
    }

    let fragment_tree = match fragment_tree {
        Some(fragment_tree) => fragment_tree,
        None => return computed_style(),
    };
    fragment_tree
        .find(|fragment, containing_block| {
            let box_fragment = match fragment {
                Fragment::Box(ref box_fragment) if box_fragment.tag == node.opaque() => {
                    box_fragment
                },
                _ => return None,
            };

            let positioned = style.get_box().position != Position::Static;
            let content_rect = box_fragment
                .content_rect
                .to_physical(box_fragment.style.writing_mode, &containing_block);
            let margins = box_fragment
                .margin
                .to_physical(box_fragment.style.writing_mode);
            let padding = box_fragment
                .padding
                .to_physical(box_fragment.style.writing_mode);
            match longhand_id {
                LonghandId::Width => Some(content_rect.size.width),
                LonghandId::Height => Some(content_rect.size.height),
                LonghandId::MarginBottom => Some(margins.bottom),
                LonghandId::MarginTop => Some(margins.top),
                LonghandId::MarginLeft => Some(margins.left),
                LonghandId::MarginRight => Some(margins.right),
                LonghandId::PaddingBottom => Some(padding.bottom),
                LonghandId::PaddingTop => Some(padding.top),
                LonghandId::PaddingLeft => Some(padding.left),
                LonghandId::PaddingRight => Some(padding.right),
                // TODO(mrobinson): These following values are often wrong, because these are not
                // exactly the "used value" for the positional properties. The real used values are
                // lost by the time the Fragment tree is constructed, so we may need to record them in
                // the tree to properly answer this query. That said, we can return an okayish value
                // sometimes simply by using the calculated position in the containing block.
                LonghandId::Top if positioned => Some(content_rect.origin.y),
                LonghandId::Left if positioned => Some(content_rect.origin.x),
                _ => None,
            }
            .map(|value| value.to_css_string())
        })
        .unwrap_or_else(computed_style)
}

pub fn process_resolved_style_request_for_unstyled_node<'dom>(
    context: &LayoutContext,
    node: impl LayoutNode<'dom>,
    pseudo: &Option<PseudoElement>,
    property: &PropertyId,
) -> String {
    // In a display: none subtree. No pseudo-element exists.
    if pseudo.is_some() {
        return String::new();
    }

    let mut tlc = ThreadLocalStyleContext::new(&context.style_context);
    let mut context = StyleContext {
        shared: &context.style_context,
        thread_local: &mut tlc,
    };

    let element = node.as_element().unwrap();
    let styles = resolve_style(&mut context, element, RuleInclusion::All, pseudo.as_ref());
    let style = styles.primary();
    let longhand_id = match *property {
        PropertyId::LonghandAlias(id, _) | PropertyId::Longhand(id) => id,
        // Firefox returns blank strings for the computed value of shorthands,
        // so this should be web-compatible.
        PropertyId::ShorthandAlias(..) | PropertyId::Shorthand(_) => return String::new(),
        PropertyId::Custom(ref name) => {
            return style.computed_value_to_string(PropertyDeclarationId::Custom(name));
        },
    };

    // No need to care about used values here, since we're on a display: none
    // subtree, use the resolved value.
    style.computed_value_to_string(PropertyDeclarationId::Longhand(longhand_id))
}

pub fn process_offset_parent_query(_requested_node: OpaqueNode) -> OffsetParentResponse {
    OffsetParentResponse::empty()
}

// https://html.spec.whatwg.org/multipage/#the-innertext-idl-attribute
pub fn process_element_inner_text_query<'dom>(_node: impl LayoutNode<'dom>) -> String {
    "".to_owned()
}

pub fn process_text_index_request(_node: OpaqueNode, _point: Point2D<Au>) -> TextIndexResponse {
    TextIndexResponse(None)
}
