/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::box::{RenderBox};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{FloatFlow, FlowData};
use layout::model::{MaybeAuto};
use layout::float_context::{FloatContext, PlacementInfo, FloatType};

use std::cell::Cell;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use gfx::geometry;
use servo_util::tree::{TreeNodeRef, TreeUtils};

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
        }
    }

    pub fn teardown(&mut self) {
        self.common.teardown();
        for self.box.iter().advance |box| {
            box.teardown();
        }
        self.box = None;
        self.index = None;
    }
}

impl FloatFlowData {
    pub fn bubble_widths_float(@mut self, ctx: &LayoutContext) {
        let mut min_width = Au(0);
        let mut pref_width = Au(0);

        self.common.num_floats = 1;

        for FloatFlow(self).each_child |child_ctx| {
            //assert!(child_ctx.starts_block_flow() || child_ctx.starts_inline_flow());

            do child_ctx.with_mut_base |child_node| {
                min_width = geometry::max(min_width, child_node.min_width);
                pref_width = geometry::max(pref_width, child_node.pref_width);
                child_node.floats_in = FloatContext::new(child_node.num_floats);
            }
        }

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

    pub fn assign_widths_float(@mut self, _: &LayoutContext) { 
        debug!("assign_widths_float: assigning width for flow %?",  self.common.id);
        // position.size.width is set by parent even though we don't know
        // position.origin yet.
        let mut remaining_width = self.common.position.size.width;
        self.containing_width = remaining_width;
        let mut x_offset = Au(0);

        for self.box.iter().advance |&box| {
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

        for FloatFlow(self).each_child |kid| {
            //assert!(kid.starts_block_flow() || kid.starts_inline_flow());

            do kid.with_mut_base |child_node| {
                child_node.position.origin.x = x_offset;
                child_node.position.size.width = remaining_width;
            }
        }
    }

    pub fn assign_height_float(@mut self, ctx: &mut LayoutContext) {
        for FloatFlow(self).each_child |kid| {
            kid.assign_height(ctx);
        }

        let mut cur_y = Au(0);
        let mut top_offset = Au(0);

        for self.box.iter().advance |&box| {
            do box.with_model |model| {
                top_offset = model.margin.top + model.border.top + model.padding.top;
                cur_y = cur_y + top_offset;
            }
        }

        for FloatFlow(self).each_child |kid| {
            do kid.with_mut_base |child_node| {
                child_node.position.origin.y = cur_y;
                cur_y = cur_y + child_node.position.size.height;
            };
        }

        let mut height = cur_y - top_offset;
        
        let mut noncontent_height = Au(0);
        self.box.map(|&box| {
            do box.with_mut_base |base| {
                //The associated box is the border box of this flow
                base.position.origin.y = base.model.margin.top;

                noncontent_height = base.model.padding.top + base.model.padding.bottom +
                    base.model.border.top + base.model.border.bottom;
                base.position.size.height = height + noncontent_height;

                noncontent_height = noncontent_height + base.model.margin.top + base.model.margin.bottom;
            }
        });

        
        //TODO(eatkinson): compute heights properly using the 'height' property.
        for self.box.iter().advance |&box| {
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

        let info = PlacementInfo {
            width: self.common.position.size.width,
            height: height,
            ceiling: Au(0),
            max_width: self.containing_width,
            f_type: self.float_type,
        };

        // Place the float and return the FloatContext back to the parent flow.
        // After, grab the position and use that to set our position.
        self.common.floats_out = self.common.floats_in.add_float(&info);
        self.rel_pos = self.common.floats_out.last_float_pos();
    }

    pub fn build_display_list_float<E:ExtraDisplayListData>(@mut self,
                                                            builder: &DisplayListBuilder,
                                                            dirty: &Rect<Au>, 
                                                            list: &Cell<DisplayList<E>>) 
                                                            -> bool {

        let abs_rect = Rect(self.common.abs_position, self.common.position.size);
        if !abs_rect.intersects(dirty) {
            return false;
        }


        let offset = self.common.abs_position + self.rel_pos;
        // add box that starts block context
        self.box.map(|&box| {
            box.build_display_list(builder, dirty, &offset, list)
        });


        // TODO: handle any out-of-flow elements

        // go deeper into the flow tree
        let flow = FloatFlow(self);
        for flow.each_child |child| {
            do child.with_mut_base |base| {
                base.abs_position = offset + base.position.origin;
            }
        }

        true
    }
}

