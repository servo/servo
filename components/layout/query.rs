/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout_task::{LayoutTaskData, RWGuard};

use euclid::point::Point2D;
use euclid::rect::Rect;
use flow_ref::FlowRef;
use fragment::{Fragment, FragmentBorderBoxIterator};
use gfx::display_list::{DisplayItemMetadata, OpaqueNode};
use msg::constellation_msg::Msg as ConstellationMsg;
use msg::constellation_msg::ConstellationChan;
use opaque_node::OpaqueNodeMethods;
use script::layout_interface::{ContentBoxResponse, ContentBoxesResponse, NodeGeometryResponse};
use script::layout_interface::{HitTestResponse, LayoutRPC, MouseOverResponse};
use script::layout_interface::{ScriptLayoutChan, TrustedNodeAddress};
use sequential;

use std::sync::{Arc, Mutex};
use util::geometry::Au;
use util::cursor::Cursor;

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
    fn process(&mut self, _: &Fragment, border_box: &Rect<Au>) {
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
    fn process(&mut self, _: &Fragment, border_box: &Rect<Au>) {
        self.rects.push(*border_box);
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
