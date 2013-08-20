/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::box::{RenderBox};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{FlowData};
use layout::model::{MaybeAuto};
use layout::float_context::{FloatContext, PlacementInfo, FloatType};

use std::cell::Cell;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use gfx::geometry;

pub struct FloatFlowData {
    /// Data common to all flows.
    common: FlowData,

    /// The associated render box.
    box: Option<RenderBox>,

    containing_width: Au,

    /// Offset relative to where the parent tried to position this flow
    rel_pos: Point2D<Au>,

    /// Left or right?
    float_type: FloatType,

    /// Index into the box list for inline floats
    index: Option<uint>,

    /// Number of floated children
    floated_children: uint,

}

impl FloatFlowData {
    pub fn new(common: FlowData, float_type: FloatType) -> FloatFlowData {
        FloatFlowData {
            common: common,
            containing_width: Au(0),
            box: None,
            index: None,
            float_type: float_type,
            rel_pos: Point2D(Au(0), Au(0)),
            floated_children: 0,
        }
    }

    pub fn teardown(&mut self) {
        for box in self.box.iter() {
            box.teardown();
        }
        self.box = None;
        self.index = None;
    }
}

impl FloatFlowData {
    pub fn bubble_widths_float(&mut self, ctx: &LayoutContext) {
        let mut min_width = Au(0);
        let mut pref_width = Au(0);
        let mut num_floats = 0;

        for child_ctx in self.common.child_iter() {
            //assert!(child_ctx.starts_block_flow() || child_ctx.starts_inline_flow());

            do child_ctx.with_mut_base |child_node| {
                min_width = geometry::max(min_width, child_node.min_width);
                pref_width = geometry::max(pref_width, child_node.pref_width);
                num_floats = num_floats + child_node.num_floats;
            }
        }

        self.common.num_floats = 1;
        self.floated_children = num_floats;


        self.box.map(|&box| {
            let style = box.style();
            do box.with_model |model| {
                model.compute_borders(style)
            }

            min_width = min_width.add(&box.get_min_width(ctx));
            pref_width = pref_width.add(&box.get_pref_width(ctx));
        });

        self.common.min_width = min_width;
        self.common.pref_width = pref_width;
    }

    pub fn assign_widths_float(&mut self) { 
        debug!("assign_widths_float: assigning width for flow %?",  self.common.id);
        // position.size.width is set by parent even though we don't know
        // position.origin yet.
        let mut remaining_width = self.common.position.size.width;
        self.containing_width = remaining_width;
        let mut x_offset = Au(0);
        
        // Parent usually sets this, but floats are never inorder
        self.common.is_inorder = false;

        for &box in self.box.iter() {
            let style = box.style();
            do box.with_model |model| {
                // Can compute padding here since we know containing block width.
                model.compute_padding(style, remaining_width);

                // Margins for floats are 0 if auto.
                let margin_top = MaybeAuto::from_margin(style.margin_top(),
                                                        remaining_width,
                                                        style.font_size()).specified_or_zero();
                let margin_bottom = MaybeAuto::from_margin(style.margin_bottom(),
                                                           remaining_width,
                                                           style.font_size()).specified_or_zero();
                let margin_left = MaybeAuto::from_margin(style.margin_left(),
                                                         remaining_width,
                                                         style.font_size()).specified_or_zero();
                let margin_right = MaybeAuto::from_margin(style.margin_right(),
                                                          remaining_width,
                                                          style.font_size()).specified_or_zero();



                let shrink_to_fit = geometry::min(self.common.pref_width, 
                                                  geometry::max(self.common.min_width, 
                                                                remaining_width));


                let width = MaybeAuto::from_width(style.width(), 
                                                  remaining_width,
                                                  style.font_size()).specified_or_default(shrink_to_fit);
                debug!("assign_widths_float -- width: %?", width);

                model.margin.top = margin_top;
                model.margin.right = margin_right;
                model.margin.bottom = margin_bottom;
                model.margin.left = margin_left;

                x_offset = model.offset();
                remaining_width = width;
            }

            do box.with_mut_base |base| {
                //The associated box is the border box of this flow
                base.position.origin.x = base.model.margin.left;

                let pb = base.model.padding.left + base.model.padding.right +
                    base.model.border.left + base.model.border.right;
                base.position.size.width = remaining_width + pb;
            }
        }

        self.common.position.size.width = remaining_width;

        let has_inorder_children = self.common.num_floats > 0;
        for kid in self.common.child_iter() {
            //assert!(kid.starts_block_flow() || kid.starts_inline_flow());

            do kid.with_mut_base |child_node| {
                child_node.position.origin.x = x_offset;
                child_node.position.size.width = remaining_width;
                child_node.is_inorder = has_inorder_children;

                if !child_node.is_inorder {
                    child_node.floats_in = FloatContext::new(0);
                }
            }
        }
    }

    pub fn assign_height_inorder_float(&mut self) {
        debug!("assign_height_inorder_float: assigning height for float %?", self.common.id);
        // assign_height_float was already called by the traversal function
        // so this is well-defined

        let mut height = Au(0);
        let mut clearance = Au(0);
        let mut full_noncontent_width = Au(0);
        let mut margin_height = Au(0);

        self.box.map(|&box| {
            height = do box.with_base |base| {
                base.position.size.height
            };
            clearance = match box.clear() {
                None => Au(0),
                Some(clear) => {
                    self.common.floats_in.clearance(clear)
                }
            };

            do box.with_base |base| {
                let noncontent_width = base.model.padding.left + base.model.padding.right +
                    base.model.border.left + base.model.border.right;

                full_noncontent_width = noncontent_width + base.model.margin.left + base.model.margin.right;
                margin_height = base.model.margin.top + base.model.margin.bottom;
            }

        });

        let info = PlacementInfo {
            width: self.common.position.size.width + full_noncontent_width,
            height: height + margin_height,
            ceiling: clearance,
            max_width: self.containing_width,
            f_type: self.float_type,
        };

        // Place the float and return the FloatContext back to the parent flow.
        // After, grab the position and use that to set our position.
        self.common.floats_out = self.common.floats_in.add_float(&info);
        self.rel_pos = self.common.floats_out.last_float_pos();
    }

    pub fn assign_height_float(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_float: assigning height for float %?", self.common.id);
        let has_inorder_children = self.common.num_floats > 0;
        if has_inorder_children {
            let mut float_ctx = FloatContext::new(self.floated_children);
            for kid in self.common.child_iter() {
                do kid.with_mut_base |child_node| {
                    child_node.floats_in = float_ctx.clone();
                }
                kid.assign_height_inorder(ctx);
                do kid.with_mut_base |child_node| {
                    float_ctx = child_node.floats_out.clone();
                }
            }
        }

        let mut cur_y = Au(0);
        let mut top_offset = Au(0);

        for &box in self.box.iter() {
            do box.with_model |model| {
                top_offset = model.margin.top + model.border.top + model.padding.top;
                cur_y = cur_y + top_offset;
            }
        }

        for kid in self.common.child_iter() {
            do kid.with_mut_base |child_node| {
                child_node.position.origin.y = cur_y;
                cur_y = cur_y + child_node.position.size.height;
            };
        }

        let mut height = cur_y - top_offset;

        let mut noncontent_width = Au(0);
        let mut noncontent_height = Au(0);
        self.box.map(|&box| {
            do box.with_mut_base |base| {
                //The associated box is the border box of this flow
                base.position.origin.y = base.model.margin.top;

                noncontent_width = base.model.padding.left + base.model.padding.right +
                    base.model.border.left + base.model.border.right;
                noncontent_height = base.model.padding.top + base.model.padding.bottom +
                    base.model.border.top + base.model.border.bottom;
                base.position.size.height = height + noncontent_height;

            }
        });

        
        //TODO(eatkinson): compute heights properly using the 'height' property.
        for &box in self.box.iter() {
            let height_prop = 
                MaybeAuto::from_height(box.style().height(),
                                       Au(0),
                                       box.style().font_size()).specified_or_zero();

            height = geometry::max(height, height_prop) + noncontent_height;
            debug!("assign_height_float -- height: %?", height);
            do box.with_mut_base |base| {
                base.position.size.height = height;
            }
        }
    }

    pub fn build_display_list_float<E:ExtraDisplayListData>(&mut self,
                                                            builder: &DisplayListBuilder,
                                                            dirty: &Rect<Au>, 
                                                            list: &Cell<DisplayList<E>>) 
                                                            -> bool {

        //TODO: implement iframe size messaging
        if self.common.node.is_iframe_element() {
            error!("float iframe size messaging not implemented yet");
        }
        let abs_rect = Rect(self.common.abs_position, self.common.position.size);
        if !abs_rect.intersects(dirty) {
            return true;
        }


        let offset = self.common.abs_position + self.rel_pos;
        // add box that starts block context
        self.box.map(|&box| {
            box.build_display_list(builder, dirty, &offset, list)
        });


        // TODO: handle any out-of-flow elements

        // go deeper into the flow tree
        for child in self.common.child_iter() {
            do child.with_mut_base |base| {
                base.abs_position = offset + base.position.origin;
            }
        }

        false
    }
}

