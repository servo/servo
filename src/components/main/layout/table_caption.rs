/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::block::BlockFlow;
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{TableCaptionFlowClass, FlowClass, Flow};
use layout::wrapper::ThreadSafeLayoutNode;

use std::cell::RefCell;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::DisplayListCollection;
use servo_util::geometry::Au;

/// A table formatting context.
pub struct TableCaptionFlow {
    block_flow: BlockFlow,
}

impl TableCaptionFlow {
    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableCaptionFlow {
        TableCaptionFlow {
            block_flow: BlockFlow::from_node(constructor, node)
        }
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown();
    }

    pub fn build_display_list_table_caption<E:ExtraDisplayListData>(
                                           &mut self,
                                           builder: &DisplayListBuilder,
                                           container_block_size: &Size2D<Au>,
                                           absolute_cb_abs_position: Point2D<Au>,
                                           dirty: &Rect<Au>,
                                           index: uint,
                                           lists: &RefCell<DisplayListCollection<E>>)
                                           -> uint {
        debug!("build_display_list_table_caption: same process as block flow");
        self.block_flow.build_display_list_block(builder, container_block_size,
                                                 absolute_cb_abs_position,
                                                 dirty, index, lists)
    }
}

impl Flow for TableCaptionFlow {
    fn class(&self) -> FlowClass {
        TableCaptionFlowClass
    }

    fn as_table_caption<'a>(&'a mut self) -> &'a mut TableCaptionFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn bubble_widths(&mut self, ctx: &mut LayoutContext) {
        self.block_flow.bubble_widths(ctx);
    }

    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow", "table_caption");
        self.block_flow.assign_widths(ctx);
    }

    /// This is called on kid flows by a parent.
    ///
    /// Hence, we can assume that assign_height has already been called on the
    /// kid (because of the bottom-up traversal).
    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for table_caption");
        self.block_flow.assign_height_inorder(ctx);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table_caption");
        self.block_flow.assign_height(ctx);
    }

    /// table-caption has margins but is not collapsed with a sibling(table)
    /// or its parents(table-wrapper).
    /// Therefore, margins to be collapsed do not exist.
    fn collapse_margins(&mut self, _: bool, _: &mut bool, _: &mut Au,
                        _: &mut Au, _: &mut Au, _: &mut Au) {
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableCaptionFlow: ";
        txt.append(match self.block_flow.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}
