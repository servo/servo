/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities for querying the layout, as needed by the layout task.

use app_units::Au;
use construct::ConstructionResult;
use euclid::point::Point2D;
use euclid::rect::Rect;
use flow;
use flow_ref::FlowRef;
use fragment::{Fragment, FragmentBorderBoxIterator, SpecificFragmentInfo};
use gfx::display_list::{DisplayItemMetadata, OpaqueNode};
use layout_task::LayoutTaskData;
use msg::constellation_msg::ConstellationChan;
use msg::constellation_msg::ScriptMsg as ConstellationMsg;
use opaque_node::OpaqueNodeMethods;
use script::layout_interface::{ContentBoxResponse, ContentBoxesResponse, NodeGeometryResponse};
use script::layout_interface::{HitTestResponse, LayoutRPC, MouseOverResponse, OffsetParentResponse};
use script::layout_interface::{ResolvedStyleResponse, ScriptLayoutChan};
use selectors::parser::PseudoElement;
use sequential;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use string_cache::Atom;
use style::computed_values;
use style::properties::longhands::{display, position};
use style::properties::style_structs;
use style::values::AuExtensionMethods;
use util::cursor::Cursor;
use util::geometry::ZERO_POINT;
use util::logical_geometry::WritingMode;
use wrapper::{LayoutNode, ServoLayoutNode, ThreadSafeLayoutNode};

pub struct LayoutRPCImpl(pub Arc<Mutex<LayoutTaskData>>);

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

    fn node_geometry(&self) -> NodeGeometryResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        NodeGeometryResponse {
            client_rect: rw_data.client_rect_response
        }
    }

    /// Retrieves the resolved value for a CSS style property.
    fn resolved_style(&self) -> ResolvedStyleResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        ResolvedStyleResponse(rw_data.resolved_style_response.clone())
    }

    /// Requests the node containing the point of interest.
    fn hit_test(&self, point: Point2D<f32>) -> Result<HitTestResponse, ()> {
        let point = Point2D::new(Au::from_f32_px(point.x), Au::from_f32_px(point.y));
        let resp = {
            let &LayoutRPCImpl(ref rw_data) = self;
            let rw_data = rw_data.lock().unwrap();
            match rw_data.stacking_context {
                None => panic!("no root stacking context!"),
                Some(ref stacking_context) => {
                    let mut result = Vec::new();
                    stacking_context.hit_test(point, &mut result, true);
                    if !result.is_empty() {
                        Some(HitTestResponse(result[0].node.to_untrusted_node_address()))
                    } else {
                        None
                    }
                }
            }
        };

        if resp.is_some() {
            return Ok(resp.unwrap());
        }
        Err(())
    }

    fn mouse_over(&self, point: Point2D<f32>) -> Result<MouseOverResponse, ()> {
        let mut mouse_over_list: Vec<DisplayItemMetadata> = vec!();
        let point = Point2D::new(Au::from_f32_px(point.x), Au::from_f32_px(point.y));
        {
            let &LayoutRPCImpl(ref rw_data) = self;
            let rw_data = rw_data.lock().unwrap();
            match rw_data.stacking_context {
                None => panic!("no root stacking context!"),
                Some(ref stacking_context) => {
                    stacking_context.hit_test(point, &mut mouse_over_list, false);
                }
            }

            // Compute the new cursor.
            let cursor = if !mouse_over_list.is_empty() {
                mouse_over_list[0].pointing.unwrap()
            } else {
                Cursor::DefaultCursor
            };
            let ConstellationChan(ref constellation_chan) = rw_data.constellation_chan;
            constellation_chan.send(ConstellationMsg::SetCursor(cursor)).unwrap();
        }

        if mouse_over_list.is_empty() {
            Err(())
        } else {
            let response_list =
                mouse_over_list.iter()
                               .map(|metadata| metadata.node.to_untrusted_node_address())
                               .collect();
            Ok(MouseOverResponse(response_list))
        }
    }

    fn offset_parent(&self) -> OffsetParentResponse {
        let &LayoutRPCImpl(ref rw_data) = self;
        let rw_data = rw_data.lock().unwrap();
        rw_data.offset_parent_response.clone()
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

pub fn process_content_box_request(requested_node: ServoLayoutNode,
                                   layout_root: &mut FlowRef)
                                   -> Rect<Au> {
    // FIXME(pcwalton): This has not been updated to handle the stacking context relative
    // stuff. So the position is wrong in most cases.
    let mut iterator = UnioningFragmentBorderBoxIterator::new(requested_node.opaque());
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    match iterator.rect {
        Some(rect) => rect,
        None       => Rect::zero()
    }
}

pub fn process_content_boxes_request(requested_node: ServoLayoutNode,
                                     layout_root: &mut FlowRef)
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

pub fn process_node_geometry_request(requested_node: ServoLayoutNode,
                                     layout_root: &mut FlowRef)
                                     -> Rect<i32> {
    let mut iterator = FragmentLocatingFragmentIterator::new(requested_node.opaque());
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    iterator.client_rect
}

/// Return the resolved value of property for a given (pseudo)element.
/// https://drafts.csswg.org/cssom/#resolved-value
pub fn process_resolved_style_request(requested_node: ServoLayoutNode,
                                      pseudo: &Option<PseudoElement>,
                                      property: &Atom,
                                      layout_root: &mut FlowRef)
                                      -> Option<String> {
    let layout_node = ThreadSafeLayoutNode::new(&requested_node);
    let layout_node = match pseudo {
        &Some(PseudoElement::Before) => layout_node.get_before_pseudo(),
        &Some(PseudoElement::After) => layout_node.get_after_pseudo(),
        _ => Some(layout_node)
    };

    let layout_node = match layout_node {
        None => {
            // The pseudo doesn't exist, return nothing.  Chrome seems to query
            // the element itself in this case, Firefox uses the resolved value.
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=29006
            return None;
        }
        Some(layout_node) => layout_node
    };

    let style = &*layout_node.style();

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

    fn used_value_for_position_property(layout_node: ThreadSafeLayoutNode,
                                        layout_root: &mut FlowRef,
                                        requested_node: ServoLayoutNode,
                                        property: &Atom) -> Option<String> {
        let layout_data = layout_node.borrow_layout_data();
        let position = layout_data.as_ref().map(|layout_data| {
            match layout_data.data.flow_construction_result {
                ConstructionResult::Flow(ref flow_ref, _) =>
                    flow::base(flow_ref.deref()).stacking_relative_position,
                // TODO(dzbarsky) search parents until we find node with a flow ref.
                // https://github.com/servo/servo/issues/8307
                _ => ZERO_POINT
            }
        }).unwrap_or(ZERO_POINT);
        let property = match *property {
            atom!("bottom") => PositionProperty::Bottom,
            atom!("top") => PositionProperty::Top,
            atom!("left") => PositionProperty::Left,
            atom!("right") => PositionProperty::Right,
            atom!("width") => PositionProperty::Width,
            atom!("height") => PositionProperty::Height,
            _ => unreachable!()
        };
        let mut iterator =
            PositionRetrievingFragmentBorderBoxIterator::new(requested_node.opaque(),
                                                             property,
                                                             position);
        sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root,
                                                                    &mut iterator);
        iterator.result.map(|r| r.to_css_string())
    }

    // TODO: we will return neither the computed nor used value for margin and padding.
    // Firefox returns blank strings for the computed value of shorthands,
    // so this should be web-compatible.
    match *property {
        atom!("margin-bottom") | atom!("margin-top") |
        atom!("margin-left") | atom!("margin-right") |
        atom!("padding-bottom") | atom!("padding-top") |
        atom!("padding-left") | atom!("padding-right")
        if applies && style.get_box().display != display::computed_value::T::none => {
            let (margin_padding, side) = match *property {
                atom!("margin-bottom") => (MarginPadding::Margin, Side::Bottom),
                atom!("margin-top") => (MarginPadding::Margin, Side::Top),
                atom!("margin-left") => (MarginPadding::Margin, Side::Left),
                atom!("margin-right") => (MarginPadding::Margin, Side::Right),
                atom!("padding-bottom") => (MarginPadding::Padding, Side::Bottom),
                atom!("padding-top") => (MarginPadding::Padding, Side::Top),
                atom!("padding-left") => (MarginPadding::Padding, Side::Left),
                atom!("padding-right") => (MarginPadding::Padding, Side::Right),
                _ => unreachable!()
            };
            let mut iterator =
                MarginRetrievingFragmentBorderBoxIterator::new(requested_node.opaque(),
                                                               side,
                                                               margin_padding,
                                                               style.writing_mode);
            sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root,
                                                                        &mut iterator);
            iterator.result.map(|r| r.to_css_string())
        },

        atom!("bottom") | atom!("top") | atom!("right") |
        atom!("left")
        if applies && positioned && style.get_box().display !=
                display::computed_value::T::none => {
            used_value_for_position_property(layout_node, layout_root, requested_node, property)
        }
        atom!("width") | atom!("height")
        if applies && style.get_box().display !=
                display::computed_value::T::none => {
            used_value_for_position_property(layout_node, layout_root, requested_node, property)
        }
        // FIXME: implement used value computation for line-height
        ref property => {
            style.computed_value_to_string(property.as_slice()).ok()
        }
    }
}

pub fn process_offset_parent_query(requested_node: ServoLayoutNode,
                                   layout_root: &mut FlowRef)
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
