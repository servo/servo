/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::BlockFlow;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BaseFlow, TableCaptionFlowClass, FlowClass, Flow};

use std::cell::RefCell;
use geom::{Rect};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;

/// A table formatting context.
pub struct TableCaptionFlow {
    block_flow: BlockFlow,
}

impl TableCaptionFlow {
    pub fn new(base: BaseFlow) -> TableCaptionFlow {
        TableCaptionFlow {
            block_flow: BlockFlow::new(base),
        }
    }

    pub fn from_box(base: BaseFlow, box_: Box) -> TableCaptionFlow {
        TableCaptionFlow {
            block_flow: BlockFlow::from_box(base, box_, false),
        }
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown();
    }

    pub fn build_display_list_table_caption<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    dirty: &Rect<Au>,
                                    list: &RefCell<DisplayList<E>>)
                                    -> bool {
        debug!("build_display_list_table_caption: same process as block flow");
        self.block_flow.build_display_list_block(builder, dirty, list)
    }
}

impl Flow for TableCaptionFlow {
    fn class(&self) -> FlowClass {
        TableCaptionFlowClass
    }

    fn as_table_caption<'a>(&'a mut self) -> &'a mut TableCaptionFlow {
        self
    }

    fn bubble_widths(&mut self, ctx: &mut LayoutContext) {
        self.block_flow.bubble_widths(ctx);
    }

    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow {}", "table_caption", self.block_flow.base.id);
        self.block_flow.assign_widths(ctx);
    }

    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height_inorder: assigning height for table_caption {}", self.block_flow.base.id);
        self.block_flow.assign_height_inorder(ctx);
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_height: assigning height for table_caption {}", self.block_flow.base.id);
        self.block_flow.assign_height(ctx);
    }

    fn collapse_margins(&mut self,
                        top_margin_collapsible: bool,
                        first_in_flow: &mut bool,
                        margin_top: &mut Au,
                        top_offset: &mut Au,
                        collapsing: &mut Au,
                        collapsible: &mut Au) {
        self.block_flow.collapse_margins(top_margin_collapsible,
                                         first_in_flow,
                                         margin_top,
                                         top_offset,
                                         collapsing,
                                         collapsible);
    }

    fn debug_str(&self) -> ~str {
        let txt = ~"TableCaptionFlow: ";
        txt.append(match self.block_flow.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}
