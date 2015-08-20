/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities for querying the layout, as needed by the layout task.

use layout_task::{LayoutTaskData, RWGuard};

use euclid::point::Point2D;
use euclid::rect::Rect;
use flow_ref::FlowRef;
use fragment::{Fragment, FragmentBorderBoxIterator};
use gfx::display_list::{DisplayItemMetadata, OpaqueNode};
use msg::constellation_msg::ConstellationChan;
use msg::constellation_msg::Msg as ConstellationMsg;
use opaque_node::OpaqueNodeMethods;
use script::layout_interface::{ContentBoxResponse, ContentBoxesResponse, NodeGeometryResponse};
use script::layout_interface::{HitTestResponse, LayoutRPC, MouseOverResponse, OffsetParentResponse};
use script::layout_interface::{ResolvedStyleResponse, ScriptLayoutChan, TrustedNodeAddress};
use sequential;

use std::sync::{Arc, Mutex};
use util::cursor::Cursor;
use util::geometry::Au;
use util::logical_geometry::WritingMode;

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
    fn hit_test(&self, _: TrustedNodeAddress, point: Point2D<f32>) -> Result<HitTestResponse, ()> {
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

    fn mouse_over(&self, _: TrustedNodeAddress, point: Point2D<f32>)
                  -> Result<MouseOverResponse, ()> {
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

pub struct UnioningFragmentBorderBoxIterator {
    pub node_address: OpaqueNode,
    pub rect: Option<Rect<Au>>,
}

impl UnioningFragmentBorderBoxIterator {
    pub fn new(node_address: OpaqueNode) -> UnioningFragmentBorderBoxIterator {
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

pub struct CollectingFragmentBorderBoxIterator {
    pub node_address: OpaqueNode,
    pub rects: Vec<Rect<Au>>,
}

impl CollectingFragmentBorderBoxIterator {
    pub fn new(node_address: OpaqueNode) -> CollectingFragmentBorderBoxIterator {
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

pub enum Side {
    Left,
    Right,
    Bottom,
    Top
}

pub enum MarginPadding {
    Margin,
    Padding
}

pub enum PositionProperty {
    Left,
    Right,
    Top,
    Bottom,
    Width,
    Height,
}

pub struct PositionRetrievingFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    pub result: Option<Au>,
    position: Point2D<Au>,
    property: PositionProperty,
}

impl PositionRetrievingFragmentBorderBoxIterator {
    pub fn new(node_address: OpaqueNode,
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
    fn process(&mut self, _: &Fragment, _: i32, border_box: &Rect<Au>) {
        self.result =
            Some(match self.property {
                     PositionProperty::Left => self.position.x,
                     PositionProperty::Top => self.position.y,
                     PositionProperty::Width => border_box.size.width,
                     PositionProperty::Height => border_box.size.height,
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

pub struct MarginRetrievingFragmentBorderBoxIterator {
    node_address: OpaqueNode,
    pub result: Option<Au>,
    writing_mode: WritingMode,
    margin_padding: MarginPadding,
    side: Side,
}

impl MarginRetrievingFragmentBorderBoxIterator {
    pub fn new(node_address: OpaqueNode, side: Side, margin_padding:
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

pub fn process_content_box_request<'a>(requested_node: TrustedNodeAddress,
                                       layout_root: &mut FlowRef,
                                       rw_data: &mut RWGuard<'a>) {
    // FIXME(pcwalton): This has not been updated to handle the stacking context relative
    // stuff. So the position is wrong in most cases.
    let requested_node: OpaqueNode = OpaqueNodeMethods::from_script_node(requested_node);
    let mut iterator = UnioningFragmentBorderBoxIterator::new(requested_node);
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    rw_data.content_box_response = match iterator.rect {
        Some(rect) => rect,
        None       => Rect::zero()
    };
}

pub fn process_content_boxes_request<'a>(requested_node: TrustedNodeAddress,
                                         layout_root: &mut FlowRef,
                                         rw_data: &mut RWGuard<'a>) {
    // FIXME(pcwalton): This has not been updated to handle the stacking context relative
    // stuff. So the position is wrong in most cases.
    let requested_node: OpaqueNode = OpaqueNodeMethods::from_script_node(requested_node);
    let mut iterator = CollectingFragmentBorderBoxIterator::new(requested_node);
    sequential::iterate_through_flow_tree_fragment_border_boxes(layout_root, &mut iterator);
    rw_data.content_boxes_response = iterator.rects;
}
