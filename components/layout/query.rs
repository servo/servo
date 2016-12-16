/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities for querying the layout, as needed by the layout thread.

use app_units::Au;
use construct::ConstructionResult;
use context::{ScopedThreadLocalLayoutContext, SharedLayoutContext};
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use flow::{self, Flow};
use fragment::{Fragment, FragmentBorderBoxIterator, SpecificFragmentInfo};
use gfx::display_list::{DisplayItemMetadata, DisplayList, OpaqueNode, ScrollOffsetMap};
use gfx_traits::ScrollRootId;
use ipc_channel::ipc::IpcSender;
use opaque_node::OpaqueNodeMethods;
use script_layout_interface::rpc::{ContentBoxResponse, ContentBoxesResponse};
use script_layout_interface::rpc::{HitTestResponse, LayoutRPC};
use script_layout_interface::rpc::{MarginStyleResponse, NodeGeometryResponse};
use script_layout_interface::rpc::{NodeOverflowResponse, OffsetParentResponse};
use script_layout_interface::rpc::{NodeScrollRootIdResponse, ResolvedStyleResponse};
use script_layout_interface::wrapper_traits::{LayoutNode, ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use script_traits::LayoutMsg as ConstellationMsg;
use script_traits::UntrustedNodeAddress;
use sequential;
use std::cmp::{min, max};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use style::computed_values;
use style::context::StyleContext;
use style::dom::TElement;
use style::logical_geometry::{WritingMode, BlockFlowDirection, InlineBaseDirection};
use style::properties::{style_structs, PropertyId, PropertyDeclarationId, LonghandId};
use style::properties::longhands::{display, position};
use style::selector_parser::PseudoElement;
use style::stylist::Stylist;
use style_traits::ToCss;
use style_traits::cursor::Cursor;
use wrapper::{LayoutNodeHelpers, LayoutNodeLayoutData};

/// Mutable data belonging to the LayoutThread.
///
/// This needs to be protected by a mutex so we can do fast RPCs.
pub struct LayoutThreadData {
    /// The channel on which messages can be sent to the constellation.
    pub constellation_chan: IpcSender<ConstellationMsg>,

    /// The root stacking context.
    pub display_list: Option<Arc<DisplayList>>,

    /// Performs CSS selector matching and style resolution.
    pub stylist: Arc<Stylist>,

    /// A queued response for the union of the content boxes of a node.
    pub content_box_response: Rect<Au>,

    /// A queued response for the content boxes of a node.
    pub content_boxes_response: Vec<Rect<Au>>,

    /// A queued response for the client {top, left, width, height} of a node in pixels.
    pub client_rect_response: Rect<i32>,

    /// A queued response for the node at a given point
    pub hit_test_response: (Option<DisplayItemMetadata>, bool),

    /// A queued response for the scroll root id for a given node.
    pub scroll_root_id_response: Option<ScrollRootId>,

    /// A pair of overflow property in x and y
    pub overflow_response: NodeOverflowResponse,

    /// A queued response for the scroll {top, left, width, height} of a node in pixels.
    pub scroll_area_response: Rect<i32>,

    /// A queued response for the resolved style property of an element.
    pub resolved_style_response: String,

    /// A queued response for the offset parent/rect of a node.
    pub offset_parent_response: OffsetParentResponse,

    /// A queued response for the offset parent/rect of a node.
    pub margin_style_response: MarginStyleResponse,

    /// Scroll offsets of stacking contexts. This will only be populated if WebRender is in use.
    pub stacking_context_scroll_offsets: ScrollOffsetMap,
}

pub struct LayoutRPCImpl(pub Arc<Mutex<LayoutThreadData>>);

// https://drafts.csswg.org/cssom-view/#overflow-directions
fn overflow_direction(writing_mode: &WritingMode) -> OverflowDirection {
    match (writing_mode.block_flow_direction(), writing_mode.inline_base_direction()) {
        (BlockFlowDirection::TopToBottom, InlineBaseDirection::LeftToRight) |
            (BlockFlowDirection::LeftToRight, InlineBaseDirection::LeftToRight) => OverflowDirection::RightAndDown,
        (BlockFlowDirection::TopToBottom, InlineBaseDirection::RightToLeft) |
            (BlockFlowDirection::RightToLeft, InlineBaseDirection::LeftToRight) => OverflowDirection::LeftAndDown,
        (BlockFlowDirection::RightToLeft, InlineBaseDirection::RightToLeft) => OverflowDirection::LeftAndUp,
        (BlockFlowDirection::LeftToRight, InlineBaseDirection::RightToLeft) => OverflowDirection::RightAndUp
    }
}

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

    /// Requests the node containing the point of interest.
    fn hit_test(&self) -> HitTestResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        let &(ref result, update_cursor) = &rw_data.hit_test_response;
        if update_cursor {
            // Compute the new cursor.
            let cursor = match *result {
                None => Cursor::Default,
                Some(dim) => dim.pointing.unwrap(),
            };
            rw_data.constellation_chan.send(ConstellationMsg::SetCursor(cursor)).unwrap();
        }
        HitTestResponse {
            node_address: result.map(|dim| dim.node.to_untrusted_node_address()),
        }
    }

    fn nodes_from_point(&self,
                        page_point: Point2D<f32>,
                        client_point: Point2D<f32>) -> Vec<UntrustedNodeAddress> {
        let page_point = Point2D::new(Au::from_f32_px(page_point.x),
                                      Au::from_f32_px(page_point.y));
        let client_point = Point2D::new(Au::from_f32_px(client_point.x),
                                        Au::from_f32_px(client_point.y));

        let nodes_from_point_list = {
            let &LayoutRPCImpl(ref rw_data) = self;
            let rw_data = rw_data.lock().unwrap();
            let result = match rw_data.display_list {
                None => panic!("Tried to hit test without a DisplayList"),
                Some(ref display_list) => {
                    display_list.hit_test(&page_point,
                                          &client_point,
                                          &rw_data.stacking_context_scroll_offsets)
                }
            };

            result
        };

        nodes_from_point_list.iter()
           .rev()
           .map(|metadata| metadata.node.to_untrusted_node_address())
           .collect()
    }

    fn node_geometry(&self) -> NodeGeometryResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        NodeGeometryResponse {
            client_rect: rw_data.client_rect_response
        }
    }

    fn node_overflow(&self) -> NodeOverflowResponse {
        NodeOverflowResponse(self.0.lock().unwrap().overflow_response.0)
    }

    fn node_scroll_area(&self) -> NodeGeometryResponse {
        NodeGeometryResponse {
            client_rect: self.0.lock().unwrap().scroll_area_response
        }
    }

    fn node_scroll_root_id(&self) -> NodeScrollRootIdResponse {
        NodeScrollRootIdResponse(self.0.lock()
                                        .unwrap().scroll_root_id_response
                                        .expect("scroll_root_id is not correctly fetched"))
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

    fn margin_style(&self) -> MarginStyleResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        rw_data.margin_style_response.clone()
    }
}

struct UnioningFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    rect: Option<Rect<Au>>,
}

impl UnioningFragmentBorderBoxIterator {
    fn new(node_address: OpaqueNode) -> UnioningFragmentBorderBoxIterator {
        UnioningFragmentBorderBoxIterator {
            node_address: node_address,
            rect: None
        }
    }
}

impl FragmentBorderBoxIterator for UnioningFragmentBorderBoxIterator {
    fn process(&mut self, _: &Fragment, _: i32, border_box: &Rect<Au>) {
        self.rect = match self.rect {
            Some(rect) => {
                Some(rect.union(border_box))
            }
            None => {
                Some(*border_box)
            }
        };
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.contains_node(self.node_address)
    }
}

struct CollectingFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    rects: Vec<Rect<Au>>,
}

impl CollectingFragmentBorderBoxIterator {
    fn new(node_address: OpaqueNode) -> CollectingFragmentBorderBoxIterator {
        CollectingFragmentBorderBoxIterator {
            node_address: node_address,
            rects: Vec::new(),
        }
    }
}

impl FragmentBorderBoxIterator for CollectingFragmentBorderBoxIterator {
    fn process(&mut self, _: &Fragment, _: i32, border_box: &Rect<Au>) {
        self.rects.push(*border_box);
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.contains_node(self.node_address)
    }
}

enum Side {
    Left,
    Right,
    Bottom,
    Top
}

enum MarginPadding {
    Margin,
    Padding
}

enum PositionProperty {
    Left,
    Right,
    Top,
    Bottom,
    Width,
    Height,
}

#[derive(Debug)]
enum OverflowDirection {
    RightAndDown,
    LeftAndDown,
    LeftAndUp,
    RightAndUp,
}

struct PositionRetrievingFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    result: Option<Au>,
    position: Point2D<Au>,
    property: PositionProperty,
}

impl PositionRetrievingFragmentBorderBoxIterator {
    fn new(node_address: OpaqueNode,
           property: PositionProperty,
           position: Point2D<Au>) -> PositionRetrievingFragmentBorderBoxIterator {
        PositionRetrievingFragmentBorderBoxIterator {
            node_address: node_address,
            position: position,
            property: property,
            result: None,
        }
    }
}

impl FragmentBorderBoxIterator for PositionRetrievingFragmentBorderBoxIterator {
    fn process(&mut self, fragment: &Fragment, _: i32, border_box: &Rect<Au>) {
        let border_padding = fragment.border_padding.to_physical(fragment.style.writing_mode);
        self.result =
            Some(match self.property {
                     PositionProperty::Left => self.position.x,
                     PositionProperty::Top => self.position.y,
                     PositionProperty::Width => border_box.size.width - border_padding.horizontal(),
                     PositionProperty::Height => border_box.size.height - border_padding.vertical(),
                     // TODO: the following 2 calculations are completely wrong.
                     // They should return the difference between the parent's and this
                     // fragment's border boxes.
                     PositionProperty::Right => border_box.max_x() + self.position.x,
                     PositionProperty::Bottom => border_box.max_y() + self.position.y,
        });
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.contains_node(self.node_address)
    }
}

struct MarginRetrievingFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    result: Option<Au>,
    writing_mode: WritingMode,
    margin_padding: MarginPadding,
    side: Side,
}

impl MarginRetrievingFragmentBorderBoxIterator {
    fn new(node_address: OpaqueNode, side: Side, margin_padding:
    MarginPadding, writing_mode: WritingMode) -> MarginRetrievingFragmentBorderBoxIterator {
        MarginRetrievingFragmentBorderBoxIterator {
            node_address: node_address,
            side: side,
            margin_padding: margin_padding,
            result: None,
            writing_mode: writing_mode,
        }
    }
}

impl FragmentBorderBoxIterator for MarginRetrievingFragmentBorderBoxIterator {
    fn process(&mut self, fragment: &Fragment, _: i32, _: &Rect<Au>) {
        let rect = match self.margin_padding {
            MarginPadding::Margin => &fragment.margin,
            MarginPadding::Padding => &fragment.border_padding
        };
        self.result = Some(match self.side {
                               Side::Left => rect.left(self.writing_mode),
                               Side::Right => rect.right(self.writing_mode),
                               Side::Bottom => rect.bottom(self.writing_mode),
                               Side::Top => rect.top(self.writing_mode)
        });
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.contains_node(self.node_address)
    }
}

pub fn process_content_box_request<N: LayoutNode>(
        requested_node: N, layout_root: &mut Flow) -> Rect<Au> {
    // FIXME(pcwalton): This has not been updated to handle the stacking context relative
    // stuff. So the position is wrong in most cases.
    let mut iterator = UnioningFragmentBorderBoxIterator::new(requested_node.opaque());
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    match iterator.rect {
        Some(rect) => rect,
        None       => Rect::zero()
    }
}

pub fn process_content_boxes_request<N: LayoutNode>(requested_node: N, layout_root: &mut Flow)
        -> Vec<Rect<Au>> {
    // FIXME(pcwalton): This has not been updated to handle the stacking context relative
    // stuff. So the position is wrong in most cases.
    let mut iterator = CollectingFragmentBorderBoxIterator::new(requested_node.opaque());
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    iterator.rects
}

struct FragmentLocatingFragmentIterator {
    node_address: OpaqueNode,
    client_rect: Rect<i32>,
}

impl FragmentLocatingFragmentIterator {
    fn new(node_address: OpaqueNode) -> FragmentLocatingFragmentIterator {
        FragmentLocatingFragmentIterator {
            node_address: node_address,
            client_rect: Rect::zero()
        }
    }
}

struct UnioningFragmentScrollAreaIterator {
    node_address: OpaqueNode,
    union_rect: Rect<i32>,
    origin_rect: Rect<i32>,
    level: Option<i32>,
    is_child: bool,
    overflow_direction: OverflowDirection
}

impl UnioningFragmentScrollAreaIterator {
    fn new(node_address: OpaqueNode) -> UnioningFragmentScrollAreaIterator {
        UnioningFragmentScrollAreaIterator {
            node_address: node_address,
            union_rect: Rect::zero(),
            origin_rect: Rect::zero(),
            level: None,
            is_child: false,
            overflow_direction: OverflowDirection::RightAndDown
        }
    }
}

struct ParentBorderBoxInfo {
    node_address: OpaqueNode,
    border_box: Rect<Au>,
}

struct ParentOffsetBorderBoxIterator {
    node_address: OpaqueNode,
    last_level: i32,
    has_found_node: bool,
    node_border_box: Rect<Au>,
    parent_nodes: Vec<Option<ParentBorderBoxInfo>>,
}

impl ParentOffsetBorderBoxIterator {
    fn new(node_address: OpaqueNode) -> ParentOffsetBorderBoxIterator {
        ParentOffsetBorderBoxIterator {
            node_address: node_address,
            last_level: -1,
            has_found_node: false,
            node_border_box: Rect::zero(),
            parent_nodes: Vec::new(),
        }
    }
}

impl FragmentBorderBoxIterator for FragmentLocatingFragmentIterator {
    fn process(&mut self, fragment: &Fragment, _: i32, border_box: &Rect<Au>) {
        let style_structs::Border {
            border_top_width: top_width,
            border_right_width: right_width,
            border_bottom_width: bottom_width,
            border_left_width: left_width,
            ..
        } = *fragment.style.get_border();
        self.client_rect.origin.y = top_width.to_px();
        self.client_rect.origin.x = left_width.to_px();
        self.client_rect.size.width = (border_box.size.width - left_width - right_width).to_px();
        self.client_rect.size.height = (border_box.size.height - top_width - bottom_width).to_px();
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.node == self.node_address
    }
}

// https://drafts.csswg.org/cssom-view/#scrolling-area
impl FragmentBorderBoxIterator for UnioningFragmentScrollAreaIterator {
    fn process(&mut self, fragment: &Fragment, level: i32, border_box: &Rect<Au>) {
        // In cases in which smaller child elements contain less padding than the parent
        // the a union of the two elements padding rectangles could result in an unwanted
        // increase in size. To work around this, we store the original elements padding
        // rectangle as `origin_rect` and the union of all child elements padding and
        // margin rectangles as `union_rect`.
        let style_structs::Border {
            border_top_width: top_border,
            border_right_width: right_border,
            border_bottom_width: bottom_border,
            border_left_width: left_border,
            ..
        } = *fragment.style.get_border();
        let right_padding = (border_box.size.width - right_border - left_border).to_px();
        let bottom_padding = (border_box.size.height - bottom_border - top_border).to_px();
        let top_padding = top_border.to_px();
        let left_padding = left_border.to_px();

        match self.level {
            Some(start_level) if level <= start_level => { self.is_child = false; }
            Some(_) => {
                let padding = Rect::new(Point2D::new(left_padding, top_padding),
                                        Size2D::new(right_padding, bottom_padding));
                let top_margin = fragment.margin.top(fragment.style.writing_mode).to_px();
                let left_margin = fragment.margin.left(fragment.style.writing_mode).to_px();
                let bottom_margin = fragment.margin.bottom(fragment.style.writing_mode).to_px();
                let right_margin = fragment.margin.right(fragment.style.writing_mode).to_px();
                let margin = Rect::new(Point2D::new(left_margin, top_margin),
                                       Size2D::new(right_margin, bottom_margin));
                self.union_rect = self.union_rect.union(&margin).union(&padding);
            }
            None => {
                self.level = Some(level);
                self.is_child = true;
                self.overflow_direction = overflow_direction(&fragment.style.writing_mode);
                self.origin_rect = Rect::new(Point2D::new(left_padding, top_padding),
                                             Size2D::new(right_padding, bottom_padding));
            },
        };
    }

    fn should_process(&mut self, fragment: &Fragment) -> bool {
        fragment.contains_node(self.node_address) || self.is_child
    }
}

// https://drafts.csswg.org/cssom-view/#extensions-to-the-htmlelement-interface
impl FragmentBorderBoxIterator for ParentOffsetBorderBoxIterator {
    fn process(&mut self, fragment: &Fragment, level: i32, border_box: &Rect<Au>) {
        if fragment.node == self.node_address {
            // Found the fragment in the flow tree that matches the
            // DOM node being looked for.
            self.has_found_node = true;
            self.node_border_box = *border_box;

            // offsetParent returns null if the node is fixed.
            if fragment.style.get_box().position == computed_values::position::T::fixed {
                self.parent_nodes.clear();
            }
        } else if level > self.last_level {
            // TODO(gw): Is there a less fragile way of checking whether this
            // fragment is the body element, rather than just checking that
            // the parent nodes stack contains the root node only?
            let is_body_element = self.parent_nodes.len() == 1;

            let is_valid_parent = match (is_body_element,
                                         fragment.style.get_box().position,
                                         &fragment.specific) {
                // Spec says it's valid if any of these are true:
                //  1) Is the body element
                //  2) Is static position *and* is a table or table cell
                //  3) Is not static position
                (true, _, _) |
                (false, computed_values::position::T::static_, &SpecificFragmentInfo::Table) |
                (false, computed_values::position::T::static_, &SpecificFragmentInfo::TableCell) |
                (false, computed_values::position::T::absolute, _) |
                (false, computed_values::position::T::relative, _) |
                (false, computed_values::position::T::fixed, _) => true,

                // Otherwise, it's not a valid parent
                (false, computed_values::position::T::static_, _) => false,
            };

            let parent_info = if is_valid_parent {
                Some(ParentBorderBoxInfo {
                    border_box: *border_box,
                    node_address: fragment.node,
                })
            } else {
                None
            };

            self.parent_nodes.push(parent_info);
        } else if level < self.last_level {
            self.parent_nodes.pop();
        }
    }

    fn should_process(&mut self, _: &Fragment) -> bool {
        !self.has_found_node
    }
}

pub fn process_node_geometry_request<N: LayoutNode>(requested_node: N, layout_root: &mut Flow)
        -> Rect<i32> {
    let mut iterator = FragmentLocatingFragmentIterator::new(requested_node.opaque());
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    iterator.client_rect
}

pub fn process_node_scroll_root_id_request<N: LayoutNode>(requested_node: N) -> ScrollRootId {
    let layout_node = requested_node.to_threadsafe();
    layout_node.scroll_root_id()
}

pub fn process_node_scroll_area_request< N: LayoutNode>(requested_node: N, layout_root: &mut Flow)
        -> Rect<i32> {
    let mut iterator = UnioningFragmentScrollAreaIterator::new(requested_node.opaque());
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    match iterator.overflow_direction {
        OverflowDirection::RightAndDown => {
            let right = max(iterator.union_rect.size.width, iterator.origin_rect.size.width);
            let bottom = max(iterator.union_rect.size.height, iterator.origin_rect.size.height);
            Rect::new(iterator.origin_rect.origin, Size2D::new(right, bottom))
        },
        OverflowDirection::LeftAndDown => {
            let bottom = max(iterator.union_rect.size.height, iterator.origin_rect.size.height);
            let left = max(iterator.union_rect.origin.x, iterator.origin_rect.origin.x);
            Rect::new(Point2D::new(left, iterator.origin_rect.origin.y),
                      Size2D::new(iterator.origin_rect.size.width, bottom))
        },
        OverflowDirection::LeftAndUp => {
            let top = min(iterator.union_rect.origin.y, iterator.origin_rect.origin.y);
            let left = min(iterator.union_rect.origin.x, iterator.origin_rect.origin.x);
            Rect::new(Point2D::new(left, top), iterator.origin_rect.size)
        },
        OverflowDirection::RightAndUp => {
            let top = min(iterator.union_rect.origin.y, iterator.origin_rect.origin.y);
            let right = max(iterator.union_rect.size.width, iterator.origin_rect.size.width);
            Rect::new(Point2D::new(iterator.origin_rect.origin.x, top),
                      Size2D::new(right, iterator.origin_rect.size.height))
        }
    }
}

/// Return the resolved value of property for a given (pseudo)element.
/// https://drafts.csswg.org/cssom/#resolved-value
pub fn process_resolved_style_request<'a, N>(shared: &SharedLayoutContext,
                                             requested_node: N,
                                             pseudo: &Option<PseudoElement>,
                                             property: &PropertyId,
                                             layout_root: &mut Flow) -> String
    where N: LayoutNode,
{
    use style::traversal::{clear_descendant_data, style_element_in_display_none_subtree};
    let element = requested_node.as_element().unwrap();

    // We call process_resolved_style_request after performing a whole-document
    // traversal, so the only reason we wouldn't have an up-to-date style here
    // is that the requested node is in a display:none subtree. We currently
    // maintain the invariant that elements in display:none subtrees always have
    // no ElementData, so we need to temporarily bend those invariants here, and
    // then throw them the style data away again before returning to preserve them.
    // We could optimize this later to keep the style data cached somehow, but
    // we'd need a mechanism to prevent detect when it's stale (since we don't
    // traverse display:none subtrees during restyle).
    let display_none_root = if element.get_data().is_none() {
        let mut tlc = ScopedThreadLocalLayoutContext::new(shared);
        let context = StyleContext {
            shared: &shared.style_context,
            thread_local: &mut tlc.style_context,
        };

        Some(style_element_in_display_none_subtree(&context, element,
                                                   &|e| e.as_node().initialize_data()))
    } else {
        None
    };

    let layout_el = requested_node.to_threadsafe().as_element().unwrap();
    let layout_el = match *pseudo {
        Some(PseudoElement::Before) => layout_el.get_before_pseudo(),
        Some(PseudoElement::After) => layout_el.get_after_pseudo(),
        Some(PseudoElement::DetailsSummary) |
        Some(PseudoElement::DetailsContent) |
        Some(PseudoElement::Selection) => None,
        _ => Some(layout_el)
    };

    let layout_el = match layout_el {
        None => {
            // The pseudo doesn't exist, return nothing.  Chrome seems to query
            // the element itself in this case, Firefox uses the resolved value.
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=29006
            return String::new();
        }
        Some(layout_el) => layout_el
    };

    let style = &*layout_el.resolved_style();

    let longhand_id = match *property {
        PropertyId::Longhand(id) => id,

        // Firefox returns blank strings for the computed value of shorthands,
        // so this should be web-compatible.
        PropertyId::Shorthand(_) => return String::new(),

        PropertyId::Custom(ref name) => {
            return style.computed_value_to_string(PropertyDeclarationId::Custom(name))
        }
    };

    // Clear any temporarily-resolved data to maintain our invariants. See the comment
    // at the top of this function.
    display_none_root.map(|r| clear_descendant_data(r, &|e| e.as_node().clear_data()));

    let positioned = match style.get_box().position {
        position::computed_value::T::relative |
        /*position::computed_value::T::sticky |*/
        position::computed_value::T::fixed |
        position::computed_value::T::absolute => true,
        _ => false
    };

    //TODO: determine whether requested property applies to the element.
    //      eg. width does not apply to non-replaced inline elements.
    // Existing browsers disagree about when left/top/right/bottom apply
    // (Chrome seems to think they never apply and always returns resolved values).
    // There are probably other quirks.
    let applies = true;

    fn used_value_for_position_property<N: LayoutNode>(
            layout_el: <N::ConcreteThreadSafeLayoutNode as ThreadSafeLayoutNode>::ConcreteThreadSafeLayoutElement,
            layout_root: &mut Flow,
            requested_node: N,
            longhand_id: LonghandId) -> String {
        let maybe_data = layout_el.borrow_layout_data();
        let position = maybe_data.map_or(Point2D::zero(), |data| {
            match (*data).flow_construction_result {
                ConstructionResult::Flow(ref flow_ref, _) =>
                    flow::base(flow_ref.deref()).stacking_relative_position,
                // TODO(dzbarsky) search parents until we find node with a flow ref.
                // https://github.com/servo/servo/issues/8307
                _ => Point2D::zero()
            }
        });
        let property = match longhand_id {
            LonghandId::Bottom => PositionProperty::Bottom,
            LonghandId::Top => PositionProperty::Top,
            LonghandId::Left => PositionProperty::Left,
            LonghandId::Right => PositionProperty::Right,
            LonghandId::Width => PositionProperty::Width,
            LonghandId::Height => PositionProperty::Height,
            _ => unreachable!()
        };
        let mut iterator =
            PositionRetrievingFragmentBorderBoxIterator::new(requested_node.opaque(),
                                                             property,
                                                             position);
        sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root,
                                                                    &mut iterator);
        iterator.result.map(|r| r.to_css_string()).unwrap_or(String::new())
    }

    // TODO: we will return neither the computed nor used value for margin and padding.
    match longhand_id {
        LonghandId::MarginBottom | LonghandId::MarginTop |
        LonghandId::MarginLeft | LonghandId::MarginRight |
        LonghandId::PaddingBottom | LonghandId::PaddingTop |
        LonghandId::PaddingLeft | LonghandId::PaddingRight
        if applies && style.get_box().display != display::computed_value::T::none => {
            let (margin_padding, side) = match longhand_id {
                LonghandId::MarginBottom => (MarginPadding::Margin, Side::Bottom),
                LonghandId::MarginTop => (MarginPadding::Margin, Side::Top),
                LonghandId::MarginLeft => (MarginPadding::Margin, Side::Left),
                LonghandId::MarginRight => (MarginPadding::Margin, Side::Right),
                LonghandId::PaddingBottom => (MarginPadding::Padding, Side::Bottom),
                LonghandId::PaddingTop => (MarginPadding::Padding, Side::Top),
                LonghandId::PaddingLeft => (MarginPadding::Padding, Side::Left),
                LonghandId::PaddingRight => (MarginPadding::Padding, Side::Right),
                _ => unreachable!()
            };
            let mut iterator =
                MarginRetrievingFragmentBorderBoxIterator::new(requested_node.opaque(),
                                                               side,
                                                               margin_padding,
                                                               style.writing_mode);
            sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root,
                                                                        &mut iterator);
            iterator.result.map(|r| r.to_css_string()).unwrap_or(String::new())
        },

        LonghandId::Bottom | LonghandId::Top | LonghandId::Right | LonghandId::Left
        if applies && positioned && style.get_box().display !=
                display::computed_value::T::none => {
            used_value_for_position_property(layout_el, layout_root, requested_node, longhand_id)
        }
        LonghandId::Width | LonghandId::Height
        if applies && style.get_box().display !=
                display::computed_value::T::none => {
            used_value_for_position_property(layout_el, layout_root, requested_node, longhand_id)
        }
        // FIXME: implement used value computation for line-height
        _ => {
            style.computed_value_to_string(PropertyDeclarationId::Longhand(longhand_id))
        }
    }
}

pub fn process_offset_parent_query<N: LayoutNode>(requested_node: N, layout_root: &mut Flow)
        -> OffsetParentResponse {
    let mut iterator = ParentOffsetBorderBoxIterator::new(requested_node.opaque());
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    let parent_info_index = iterator.parent_nodes.iter().rposition(|info| info.is_some());
    match parent_info_index {
        Some(parent_info_index) => {
            let parent = iterator.parent_nodes[parent_info_index].as_ref().unwrap();
            let origin = iterator.node_border_box.origin - parent.border_box.origin;
            let size = iterator.node_border_box.size;
            OffsetParentResponse {
                node_address: Some(parent.node_address.to_untrusted_node_address()),
                rect: Rect::new(origin, size),
            }
        }
        None => {
            OffsetParentResponse::empty()
        }
    }
}

pub fn process_node_overflow_request<N: LayoutNode>(requested_node: N) -> NodeOverflowResponse {
    let layout_node = requested_node.to_threadsafe();
    let style = &*layout_node.as_element().unwrap().resolved_style();
    let style_box = style.get_box();

    NodeOverflowResponse(Some((Point2D::new(style_box.overflow_x, style_box.overflow_y.0))))
}

pub fn process_margin_style_query<N: LayoutNode>(requested_node: N)
        -> MarginStyleResponse {
    let layout_node = requested_node.to_threadsafe();
    let style = &*layout_node.as_element().unwrap().resolved_style();
    let margin = style.get_margin();

    MarginStyleResponse {
        top: margin.margin_top,
        right: margin.margin_right,
        bottom: margin.margin_bottom,
        left: margin.margin_left,
    }
}
