/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS block layout.

use layout::box::{RenderBox};
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, FlowDisplayListBuilderMethods};
use layout::flow::{BlockFlow, FlowContext, FlowData, InlineBlockFlow, RootFlow};
use layout::inline::InlineLayout;

use au = gfx::geometry;
use core::cell::Cell;
use geom::point::Point2D;
use geom::rect::Rect;
use gfx::display_list::DisplayList;
use gfx::geometry::Au;
use servo_util::tree::{TreeNodeRef, TreeUtils};

pub struct BlockFlowData {
    /// Data common to all flows.
    common: FlowData,

    /// The associated render box.
    box: Option<@mut RenderBox>
}

impl BlockFlowData {
    pub fn new(common: FlowData) -> BlockFlowData {
        BlockFlowData {
            common: common,
            box: None,
        }
    }
}

/// NB: These are part of FlowContext, not part of BlockFlowData, because the root flow calls these
/// as well. It is not clear to me whether this needs to be the case, or whether `RootFlow` can be
/// merged into this.
pub trait BlockLayout {
    fn starts_block_flow(&self) -> bool;
    fn with_block_box(&self, &fn(box: &@mut RenderBox) -> ()) -> ();

    fn bubble_widths_block(&self, ctx: &LayoutContext);
    fn assign_widths_block(&self, ctx: &LayoutContext);
    fn assign_height_block(&self, ctx: &LayoutContext);
    fn build_display_list_block(&self,
                                a: &DisplayListBuilder,
                                b: &Rect<Au>,
                                c: &Point2D<Au>,
                                d: &Cell<DisplayList>);
}

impl BlockLayout for FlowContext {
    fn starts_block_flow(&self) -> bool {
        match *self {
            RootFlow(*) | BlockFlow(*) | InlineBlockFlow(*) => true,
            _ => false 
        }
    }

    /// Get the current flow's corresponding block box, if it exists, and do something with it. 
    /// This works on both BlockFlow and RootFlow, since they are mostly the same.
    fn with_block_box(&self, callback: &fn(box: &@mut RenderBox) -> ()) -> () {
        match *self {
            BlockFlow(*) => {
                let box = self.block().box;
                for box.each |b| {
                    callback(b);
                }
            },                
            RootFlow(*) => {
                let mut box = self.root().box;
                for box.each |b| {
                    callback(b);
                }
            },
            _  => fail!(fmt!("Tried to do something with_block_box(), but this is a %?", self))
        }
    }

    /* Recursively (bottom-up) determine the context's preferred and
    minimum widths.  When called on this context, all child contexts
    have had their min/pref widths set. This function must decide
    min/pref widths based on child context widths and dimensions of
    any boxes it is responsible for flowing.  */

    /* TODO: floats */
    /* TODO: absolute contexts */
    /* TODO: inline-blocks */
    fn bubble_widths_block(&self, ctx: &LayoutContext) {
        assert!(self.starts_block_flow());

        let mut min_width = Au(0);
        let mut pref_width = Au(0);

        /* find max width from child block contexts */
        for self.each_child |child_ctx| {
            assert!(child_ctx.starts_block_flow() || child_ctx.starts_inline_flow());

            do child_ctx.with_immutable_node |child_node| {
                min_width = au::max(min_width, child_node.min_width);
                pref_width = au::max(pref_width, child_node.pref_width);
            }
        }

        /* if not an anonymous block context, add in block box's widths.
           these widths will not include child elements, just padding etc. */
        do self.with_block_box |box| {
            min_width = min_width.add(&box.get_min_width(ctx));
            pref_width = pref_width.add(&box.get_pref_width(ctx));
        }

        do self.with_mutable_node |this_node| {
            this_node.min_width = min_width;
            this_node.pref_width = pref_width;
        }
    }
 
    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    ///
    /// Dual boxes consume some width first, and the remainder is assigned to all child (block)
    /// contexts.
    fn assign_widths_block(&self, _: &LayoutContext) { 
        assert!(self.starts_block_flow());

        let mut remaining_width = self.with_immutable_node(|this| this.position.size.width);
        let mut _right_used = Au(0);
        let mut left_used = Au(0);

        // Let the box consume some width. It will return the amount remaining for its children.
        do self.with_block_box |box| {
            box.d().position.size.width = remaining_width;
            let (left_used, right_used) = box.get_used_width();
            remaining_width -= left_used.add(&right_used);
        }

        for self.each_child |kid| {
            assert!(kid.starts_block_flow() || kid.starts_inline_flow());

            do kid.with_mutable_node |child_node| {
                child_node.position.origin.x = left_used;
                child_node.position.size.width = remaining_width;
            }
        }
    }

    fn assign_height_block(&self, _ctx: &LayoutContext) {
        assert!(self.starts_block_flow());

        let mut cur_y = Au(0);

        for self.each_child |kid| {
            do kid.with_mutable_node |child_node| {
                child_node.position.origin.y = cur_y;
                cur_y += child_node.position.size.height;
            }
        }

        do self.with_mutable_node |this_node| {
            this_node.position.size.height = cur_y;
        }

        let _used_top = Au(0);
        let _used_bot = Au(0);
        
        do self.with_block_box |box| {
            box.d().position.origin.y = Au(0);
            box.d().position.size.height = cur_y;
            let (_used_top, _used_bot) = box.get_used_height();
        }
    }

    fn build_display_list_block(&self,
                                builder: &DisplayListBuilder,
                                dirty: &Rect<Au>, 
                                offset: &Point2D<Au>,
                                list: &Cell<DisplayList>) {
        assert!(self.starts_block_flow());
        
        // add box that starts block context
        do self.with_block_box |box| {
            box.build_display_list(builder, dirty, offset, list)
        }

        // TODO: handle any out-of-flow elements

        // go deeper into the flow tree
        for self.each_child |child| {
            self.build_display_list_for_child(builder, child, dirty, offset, list)
        }
    }
}
