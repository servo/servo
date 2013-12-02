/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::box::{RenderBox, RenderBoxUtils};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{FloatFlowClass, FlowClass, Flow, FlowData};
use layout::flow;
use layout::model::{MaybeAuto};
use layout::float_context::{FloatContext, PlacementInfo, FloatType};

use std::cell::Cell;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;
use servo_util::geometry;

pub struct FloatFlow {
    /// Data common to all flows.
    base: FlowData,

    /// The associated render box.
    box: Option<@RenderBox>,

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

impl FloatFlow {
    pub fn new(base: FlowData, float_type: FloatType) -> FloatFlow {
        FloatFlow {
            base: base,
            containing_width: Au(0),
            box: None,
            index: None,
            float_type: float_type,
            rel_pos: Point2D(Au(0), Au(0)),
            floated_children: 0,
        }
    }

    pub fn from_box(base: FlowData, float_type: FloatType, box: @RenderBox) -> FloatFlow {
        FloatFlow {
            base: base,
            containing_width: Au(0),
            box: Some(box),
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

    pub fn build_display_list_float<E:ExtraDisplayListData>(&mut self,
                                                            builder: &DisplayListBuilder,
                                                            dirty: &Rect<Au>, 
                                                            list: &Cell<DisplayList<E>>) 
                                                            -> bool {
        //TODO: implement iframe size messaging
        if self.base.node.is_iframe_element() {
            error!("float iframe size messaging not implemented yet");
        }
        let abs_rect = Rect(self.base.abs_position, self.base.position.size);
        if !abs_rect.intersects(dirty) {
            return true;
        }


        let offset = self.base.abs_position + self.rel_pos;
        // add box that starts block context
        for box in self.box.iter() {
            box.build_display_list(builder, dirty, &offset, list)
        }


        // TODO: handle any out-of-flow elements

        // go deeper into the flow tree
        for child in self.base.child_iter() {
            let child_base = flow::mut_base(*child);
            child_base.abs_position = offset + child_base.position.origin;
        }

        false
    }

    fn debug_str(&self) -> ~str {
        ~"FloatFlow"
    }
}

impl Flow for FloatFlow {
    fn class(&self) -> FlowClass {
        FloatFlowClass
    }

    fn as_float<'a>(&'a mut self) -> &'a mut FloatFlow {
        self
    }

    fn bubble_widths(&mut self, _: &mut LayoutContext) {
        let mut min_width = Au(0);
        let mut pref_width = Au(0);
        let mut num_floats = 0;

        for child_ctx in self.base.child_iter() {
            //assert!(child_ctx.starts_block_flow() || child_ctx.starts_inline_flow());

            let child_base = flow::mut_base(*child_ctx);
            min_width = geometry::max(min_width, child_base.min_width);
            pref_width = geometry::max(pref_width, child_base.pref_width);
            num_floats = num_floats + child_base.num_floats;
        }

        self.base.num_floats = 1;
        self.floated_children = num_floats;

        for box in self.box.iter() {
            {
                box.mut_base().compute_borders(box.base().style());
            }

            let (this_minimum_width, this_preferred_width) = box.minimum_and_preferred_widths();
            min_width = min_width + this_minimum_width;
            pref_width = pref_width + this_preferred_width;
        }

        self.base.min_width = min_width;
        self.base.pref_width = pref_width;
    }

    fn assign_widths(&mut self, _: &mut LayoutContext) {
        debug!("assign_widths_float: assigning width for flow {}",  self.base.id);
        // position.size.width is set by parent even though we don't know
        // position.origin yet.
        let mut remaining_width = self.base.position.size.width;
        self.containing_width = remaining_width;
        let mut x_offset = Au(0);
        
        // Parent usually sets this, but floats are never inorder
        self.base.is_inorder = false;

        for &box in self.box.iter() {
            let base = box.base();
            let mut_base = box.mut_base();
            let style = base.style();
            let mut position_ref = base.position.mutate();
            let position = &mut position_ref.ptr;

            // Can compute padding here since we know containing block width.
            mut_base.compute_padding(style, remaining_width);

            // Margins for floats are 0 if auto.
            let margin_top = MaybeAuto::from_style(style.Margin.margin_top,
                                                   remaining_width).specified_or_zero();
            let margin_bottom = MaybeAuto::from_style(style.Margin.margin_bottom,
                                                      remaining_width).specified_or_zero();
            let margin_left = MaybeAuto::from_style(style.Margin.margin_left,
                                                    remaining_width).specified_or_zero();
            let margin_right = MaybeAuto::from_style(style.Margin.margin_right,
                                                     remaining_width).specified_or_zero();


            let shrink_to_fit = geometry::min(self.base.pref_width, 
                                              geometry::max(self.base.min_width, remaining_width));


            let width = MaybeAuto::from_style(style.Box.width, 
                                              remaining_width).specified_or_default(shrink_to_fit);
            debug!("assign_widths_float -- width: {}", width);

            mut_base.margin.top = margin_top;
            mut_base.margin.right = margin_right;
            mut_base.margin.bottom = margin_bottom;
            mut_base.margin.left = margin_left;

            x_offset = base.offset();
            remaining_width = width;

            // The associated box is the border box of this flow.
            position.origin.x = base.margin.left;

            let padding_and_borders = base.padding.left + base.padding.right +
                base.border.left + base.border.right;
            position.size.width = remaining_width + padding_and_borders;
        }

        self.base.position.size.width = remaining_width;

        let has_inorder_children = self.base.num_floats > 0;
        for kid in self.base.child_iter() {
            //assert!(kid.starts_block_flow() || kid.starts_inline_flow());

            let child_base = flow::mut_base(*kid);
            child_base.position.origin.x = x_offset;
            child_base.position.size.width = remaining_width;
            child_base.is_inorder = has_inorder_children;

            if !child_base.is_inorder {
                child_base.floats_in = FloatContext::new(0);
            }
        }
    }

    fn assign_height_inorder(&mut self, _: &mut LayoutContext) {
        debug!("assign_height_inorder_float: assigning height for float {}", self.base.id);
        // assign_height_float was already called by the traversal function
        // so this is well-defined

        let mut height = Au(0);
        let mut clearance = Au(0);
        let mut full_noncontent_width = Au(0);
        let mut margin_height = Au(0);

        for box in self.box.iter() {
            let base = box.base();
            height = base.position.borrow().ptr.size.height;
            clearance = match base.clear() {
                None => Au(0),
                Some(clear) => self.base.floats_in.clearance(clear),
            };

            let noncontent_width = base.padding.left + base.padding.right + base.border.left +
                base.border.right;

            full_noncontent_width = noncontent_width + base.margin.left + base.margin.right;
            margin_height = base.margin.top + base.margin.bottom;
        }

        let info = PlacementInfo {
            width: self.base.position.size.width + full_noncontent_width,
            height: height + margin_height,
            ceiling: clearance,
            max_width: self.containing_width,
            f_type: self.float_type,
        };

        // Place the float and return the FloatContext back to the parent flow.
        // After, grab the position and use that to set our position.
        self.base.floats_out = self.base.floats_in.add_float(&info);
        self.rel_pos = self.base.floats_out.last_float_pos();
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        // Now that we've determined our height, propagate that out.
        let has_inorder_children = self.base.num_floats > 0;
        if has_inorder_children {
            let mut float_ctx = FloatContext::new(self.floated_children);
            for kid in self.base.child_iter() {
                flow::mut_base(*kid).floats_in = float_ctx.clone();
                kid.assign_height_inorder(ctx);
                float_ctx = flow::mut_base(*kid).floats_out.clone();
            }
        }
        debug!("assign_height_float: assigning height for float {}", self.base.id);
        let mut cur_y = Au(0);
        let mut top_offset = Au(0);

        for &box in self.box.iter() {
            let base = box.base();
            top_offset = base.margin.top + base.border.top + base.padding.top;
            cur_y = cur_y + top_offset;
        }

        for kid in self.base.child_iter() {
            let child_base = flow::mut_base(*kid);
            child_base.position.origin.y = cur_y;
            cur_y = cur_y + child_base.position.size.height;
        }

        let mut height = cur_y - top_offset;

        let mut noncontent_height;
        let box = self.box.as_ref().unwrap();
        let base = box.base();
        let mut position_ref = base.position.mutate();
        let position = &mut position_ref.ptr;

        // The associated box is the border box of this flow.
        position.origin.y = base.margin.top;

        noncontent_height = base.padding.top + base.padding.bottom + base.border.top +
            base.border.bottom;
    
        //TODO(eatkinson): compute heights properly using the 'height' property.
        let height_prop = MaybeAuto::from_style(base.style().Box.height,
                                                Au::new(0)).specified_or_zero();

        height = geometry::max(height, height_prop) + noncontent_height;
        debug!("assign_height_float -- height: {}", height);

        position.size.height = height;

    }

    fn collapse_margins(&mut self,
                        _: bool,
                        _: &mut bool,
                        _: &mut Au,
                        _: &mut Au,
                        collapsing: &mut Au,
                        _: &mut Au) {
        // Margins between a floated box and any other box do not collapse.
        *collapsing = Au::new(0);
    }

    fn debug_str(&self) -> ~str {
        ~"FloatFlow"
    }
}

