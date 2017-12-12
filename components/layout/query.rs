/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities for querying the layout, as needed by the layout thread.

use app_units::Au;
use construct::ConstructionResult;
use context::LayoutContext;
use euclid::{Point2D, Vector2D, Rect, Size2D};
use flow::{self, Flow};
use fragment::{Fragment, FragmentBorderBoxIterator, SpecificFragmentInfo};
use gfx::display_list::{DisplayList, OpaqueNode, ScrollOffsetMap};
use inline::InlineFragmentNodeFlags;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use opaque_node::OpaqueNodeMethods;
use script_layout_interface::rpc::{ContentBoxResponse, ContentBoxesResponse, LayoutRPC};
use script_layout_interface::rpc::{MarginStyleResponse, NodeGeometryResponse};
use script_layout_interface::rpc::{NodeOverflowResponse, NodeScrollRootIdResponse};
use script_layout_interface::rpc::{OffsetParentResponse, ResolvedStyleResponse, TextIndexResponse};
use script_layout_interface::wrapper_traits::{LayoutNode, ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use script_traits::LayoutMsg as ConstellationMsg;
use script_traits::UntrustedNodeAddress;
use sequential;
use std::cmp::{min, max};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use style::computed_values::display::T as Display;
use style::computed_values::position::T as Position;
use style::context::{StyleContext, ThreadLocalStyleContext};
use style::dom::TElement;
use style::logical_geometry::{WritingMode, BlockFlowDirection, InlineBaseDirection};
use style::properties::{style_structs, PropertyId, PropertyDeclarationId, LonghandId};
use style::selector_parser::PseudoElement;
use style_traits::ToCss;
use webrender_api::ClipId;
use wrapper::LayoutNodeLayoutData;

/// Mutable data belonging to the LayoutThread.
///
/// This needs to be protected by a mutex so we can do fast RPCs.
pub struct LayoutThreadData {
    /// The channel on which messages can be sent to the constellation.
    pub constellation_chan: IpcSender<ConstellationMsg>,

    /// The root stacking context.
    pub display_list: Option<Arc<DisplayList>>,

    /// A queued response for the union of the content boxes of a node.
    pub content_box_response: Option<Rect<Au>>,

    /// A queued response for the content boxes of a node.
    pub content_boxes_response: Vec<Rect<Au>>,

    /// A queued response for the client {top, left, width, height} of a node in pixels.
    pub client_rect_response: Rect<i32>,

    /// A queued response for the scroll root id for a given node.
    pub scroll_root_id_response: Option<ClipId>,

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

    /// Scroll offsets of scrolling regions.
    pub scroll_offsets: ScrollOffsetMap,

    /// Index in a text fragment. We need this do determine the insertion point.
    pub text_index_response: TextIndexResponse,

    /// A queued response for the list of nodes at a given point.
    pub nodes_from_point_response: Vec<UntrustedNodeAddress>,
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

    fn nodes_from_point_response(&self) -> Vec<UntrustedNodeAddress> {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        rw_data.nodes_from_point_response.clone()
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

    fn text_index(&self) -> TextIndexResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        rw_data.text_index_response.clone()
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
        requested_node: N, layout_root: &mut Flow) -> Option<Rect<Au>> {
    // FIXME(pcwalton): This has not been updated to handle the stacking context relative
    // stuff. So the position is wrong in most cases.
    let mut iterator = UnioningFragmentBorderBoxIterator::new(requested_node.opaque());
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    iterator.rect
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

struct NodeOffsetBoxInfo {
    offset: Point2D<Au>,
    rectangle: Rect<Au>,
}

struct ParentBorderBoxInfo {
    node_address: OpaqueNode,
    origin: Point2D<Au>,
}

struct ParentOffsetBorderBoxIterator {
    node_address: OpaqueNode,
    has_processed_node: bool,
    node_offset_box: Option<NodeOffsetBoxInfo>,
    parent_nodes: Vec<Option<ParentBorderBoxInfo>>,
}

impl ParentOffsetBorderBoxIterator {
    fn new(node_address: OpaqueNode) -> ParentOffsetBorderBoxIterator {
        ParentOffsetBorderBoxIterator {
            node_address: node_address,
            has_processed_node: false,
            node_offset_box: None,
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
        let (left_width, right_width) = (left_width.px(), right_width.px());
        let (top_width, bottom_width) = (top_width.px(), bottom_width.px());
        self.client_rect.origin.y = top_width as i32;
        self.client_rect.origin.x = left_width as i32;
        self.client_rect.size.width =
            (border_box.size.width.to_f32_px() - left_width - right_width) as i32;
        self.client_rect.size.height =
            (border_box.size.height.to_f32_px() - top_width - bottom_width) as i32;
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
        let (left_border, right_border) = (left_border.px(), right_border.px());
        let (top_border, bottom_border) = (top_border.px(), bottom_border.px());
        let right_padding = (border_box.size.width.to_f32_px() - right_border - left_border) as i32;
        let bottom_padding = (border_box.size.height.to_f32_px() - bottom_border - top_border) as i32;
        let top_padding = top_border as i32;
        let left_padding = left_border as i32;

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
        if self.node_offset_box.is_none() {
            // We haven't found the node yet, so we're still looking
            // for its parent. Remove all nodes at this level or
            // higher, as they can't be parents of this node.
            self.parent_nodes.truncate(level as usize);
            assert_eq!(self.parent_nodes.len(), level as usize,
                "Skipped at least one level in the flow tree!");
        }

        if !fragment.is_primary_fragment() {
            // This fragment doesn't correspond to anything worth
            // taking measurements from.

            if self.node_offset_box.is_none() {
                // If this is the only fragment in the flow, we need to
                // do this to avoid failing the above assertion.
                self.parent_nodes.push(None);
            }

            return;
        }

        if fragment.node == self.node_address {
            // Found the fragment in the flow tree that matches the
            // DOM node being looked for.

            assert!(self.node_offset_box.is_none(),
                "Node was being treated as inline, but it has an associated fragment!");

            self.has_processed_node = true;
            self.node_offset_box = Some(NodeOffsetBoxInfo {
                offset: border_box.origin,
                rectangle: *border_box,
            });

            // offsetParent returns null if the node is fixed.
            if fragment.style.get_box().position == Position::Fixed {
                self.parent_nodes.clear();
            }
        } else if let Some(node) = fragment.inline_context.as_ref().and_then(|inline_context| {
            inline_context.nodes.iter().find(|node| node.address == self.node_address)
        }) {
            // TODO: Handle cases where the `offsetParent` is an inline
            // element. This will likely be impossible until
            // https://github.com/servo/servo/issues/13982 is fixed.

            // Found a fragment in the flow tree whose inline context
            // contains the DOM node we're looking for, i.e. the node
            // is inline and contains this fragment.
            match self.node_offset_box {
                Some(NodeOffsetBoxInfo { ref mut rectangle, .. }) => {
                    *rectangle = rectangle.union(border_box);
                },
                None => {
                    // https://github.com/servo/servo/issues/13982 will
                    // cause this assertion to fail sometimes, so it's
                    // commented out for now.
                    /*assert!(node.flags.contains(FIRST_FRAGMENT_OF_ELEMENT),
                        "First fragment of inline node found wasn't its first fragment!");*/

                    self.node_offset_box = Some(NodeOffsetBoxInfo {
                        offset: border_box.origin,
                        rectangle: *border_box,
                    });
                },
            }

            if node.flags.contains(InlineFragmentNodeFlags::LAST_FRAGMENT_OF_ELEMENT) {
                self.has_processed_node = true;
            }
        } else if self.node_offset_box.is_none() {
            // TODO(gw): Is there a less fragile way of checking whether this
            // fragment is the body element, rather than just checking that
            // it's at level 1 (below the root node)?
            let is_body_element = level == 1;

            let is_valid_parent = match (is_body_element,
                                         fragment.style.get_box().position,
                                         &fragment.specific) {
                // Spec says it's valid if any of these are true:
                //  1) Is the body element
                //  2) Is static position *and* is a table or table cell
                //  3) Is not static position
                (true, _, _) |
                (false, Position::Static, &SpecificFragmentInfo::Table) |
                (false, Position::Static, &SpecificFragmentInfo::TableCell) |
                (false, Position::Sticky, _) |
                (false, Position::Absolute, _) |
                (false, Position::Relative, _) |
                (false, Position::Fixed, _) => true,

                // Otherwise, it's not a valid parent
                (false, Position::Static, _) => false,
            };

            let parent_info = if is_valid_parent {
                let border_width = fragment.border_width().to_physical(fragment.style.writing_mode);

                Some(ParentBorderBoxInfo {
                    node_address: fragment.node,
                    origin: border_box.origin + Vector2D::new(border_width.left, border_width.top),
                })
            } else {
                None
            };

            self.parent_nodes.push(parent_info);
        }
    }

    fn should_process(&mut self, _: &Fragment) -> bool {
        !self.has_processed_node
    }
}


pub fn process_node_geometry_request<N: LayoutNode>(requested_node: N, layout_root: &mut Flow)
        -> Rect<i32> {
    let mut iterator = FragmentLocatingFragmentIterator::new(requested_node.opaque());
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    iterator.client_rect
}

pub fn process_node_scroll_root_id_request<N: LayoutNode>(id: PipelineId,
                                                          requested_node: N)
                                                          -> ClipId {
    let layout_node = requested_node.to_threadsafe();
    layout_node.generate_scroll_root_id(id)
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
/// <https://drafts.csswg.org/cssom/#resolved-value>
pub fn process_resolved_style_request<'a, N>(context: &LayoutContext,
                                             node: N,
                                             pseudo: &Option<PseudoElement>,
                                             property: &PropertyId,
                                             layout_root: &mut Flow) -> String
    where N: LayoutNode,
{
    use style::stylist::RuleInclusion;
    use style::traversal::resolve_style;

    let element = node.as_element().unwrap();

    // We call process_resolved_style_request after performing a whole-document
    // traversal, so in the common case, the element is styled.
    if element.get_data().is_some() {
        return process_resolved_style_request_internal(node, pseudo, property, layout_root);
    }

    // In a display: none subtree. No pseudo-element exists.
    if pseudo.is_some() {
        return String::new();
    }

    let mut tlc = ThreadLocalStyleContext::new(&context.style_context);
    let mut context = StyleContext {
        shared: &context.style_context,
        thread_local: &mut tlc,
    };

    let styles = resolve_style(&mut context, element, RuleInclusion::All, false, pseudo.as_ref());
    let style = styles.primary();
    let longhand_id = match *property {
        PropertyId::LonghandAlias(id, _) |
        PropertyId::Longhand(id) => id,
        // Firefox returns blank strings for the computed value of shorthands,
        // so this should be web-compatible.
        PropertyId::ShorthandAlias(..) |
        PropertyId::Shorthand(_) => return String::new(),
        PropertyId::Custom(ref name) => {
            return style.computed_value_to_string(PropertyDeclarationId::Custom(name))
        }
    };

    // No need to care about used values here, since we're on a display: none
    // subtree, use the resolved value.
    style.computed_value_to_string(PropertyDeclarationId::Longhand(longhand_id))
}

/// The primary resolution logic, which assumes that the element is styled.
fn process_resolved_style_request_internal<'a, N>(
    requested_node: N,
    pseudo: &Option<PseudoElement>,
    property: &PropertyId,
    layout_root: &mut Flow,
) -> String
where
    N: LayoutNode,
{
    let layout_el = requested_node.to_threadsafe().as_element().unwrap();
    let layout_el = match *pseudo {
        Some(PseudoElement::Before) => layout_el.get_before_pseudo(),
        Some(PseudoElement::After) => layout_el.get_after_pseudo(),
        Some(PseudoElement::DetailsSummary) |
        Some(PseudoElement::DetailsContent) |
        Some(PseudoElement::Selection) => None,
        // FIXME(emilio): What about the other pseudos? Probably they shouldn't
        // just return the element's style!
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
        PropertyId::LonghandAlias(id, _) |
        PropertyId::Longhand(id) => id,
        // Firefox returns blank strings for the computed value of shorthands,
        // so this should be web-compatible.
        PropertyId::ShorthandAlias(..) |
        PropertyId::Shorthand(_) => return String::new(),
        PropertyId::Custom(ref name) => {
            return style.computed_value_to_string(PropertyDeclarationId::Custom(name))
        }
    };

    let positioned = match style.get_box().position {
        Position::Relative |
        Position::Sticky |
        Position::Fixed |
        Position::Absolute => true,
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
                    flow::base(flow_ref.deref()).stacking_relative_position.to_point(),
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
        if applies && style.get_box().display != Display::None => {
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
        if applies && positioned && style.get_box().display != Display::None => {
            used_value_for_position_property(layout_el, layout_root, requested_node, longhand_id)
        }
        LonghandId::Width | LonghandId::Height
        if applies && style.get_box().display != Display::None => {
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

    let node_offset_box = iterator.node_offset_box;
    let parent_info = iterator.parent_nodes.into_iter().rev().filter_map(|info| info).next();
    match (node_offset_box, parent_info) {
        (Some(node_offset_box), Some(parent_info)) => {
            let origin = node_offset_box.offset - parent_info.origin.to_vector();
            let size = node_offset_box.rectangle.size;
            OffsetParentResponse {
                node_address: Some(parent_info.node_address.to_untrusted_node_address()),
                rect: Rect::new(origin, size),
            }
        }
        _ => {
            OffsetParentResponse::empty()
        }
    }
}

pub fn process_node_overflow_request<N: LayoutNode>(requested_node: N) -> NodeOverflowResponse {
    let layout_node = requested_node.to_threadsafe();
    let style = &*layout_node.as_element().unwrap().resolved_style();
    let style_box = style.get_box();

    NodeOverflowResponse(Some((Point2D::new(style_box.overflow_x, style_box.overflow_y))))
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
