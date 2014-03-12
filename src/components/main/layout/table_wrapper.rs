/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::BlockFlow;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::flow::{BaseFlow, TableWrapperFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::model::{MaybeAuto, Specified, Auto, specified};
use layout::float_context::{FloatType};

use std::cell::RefCell;
use style::computed_values::table_layout;
use geom::{Point2D, Rect};
use gfx::display_list::DisplayList;
use servo_util::geometry::Au;
use servo_util::geometry;

pub enum TableLayout {
    FixedLayout,
    AutoLayout
}

/// A table wrapper flow based on a block formatting context.
pub struct TableWrapperFlow {
    block_flow: BlockFlow,

    /// Column widths
    col_widths: ~[Au],

    /// Table-layout property
    table_layout: TableLayout,
}

impl TableWrapperFlow {
    pub fn new(base: BaseFlow) -> TableWrapperFlow {
        TableWrapperFlow {
            block_flow: BlockFlow::new(base),
            col_widths: ~[],
            table_layout: AutoLayout,
        }
    }

    pub fn from_box(base: BaseFlow, box_: Box, is_fixed: bool) -> TableWrapperFlow {
        let table_layout = if (box_.style().Table.table_layout == table_layout::fixed) {
            FixedLayout
        } else {
            AutoLayout
        };
        TableWrapperFlow {
            block_flow: BlockFlow::from_box(base, box_, is_fixed),
            col_widths: ~[],
            table_layout: table_layout,
        }
    }

    pub fn float_from_box(base: BaseFlow, float_type: FloatType, box_: Box) -> TableWrapperFlow {
        let table_layout = if (box_.style().Table.table_layout == table_layout::fixed) {
            FixedLayout
        } else {
            AutoLayout
        };
        TableWrapperFlow {
            block_flow: BlockFlow::float_from_box(base, float_type, box_),
            col_widths: ~[],
            table_layout: table_layout,
        }
    }

    pub fn new_float(base: BaseFlow, float_type: FloatType) -> TableWrapperFlow {
        TableWrapperFlow {
            block_flow: BlockFlow::new_float(base, float_type),
            col_widths: ~[],
            table_layout: AutoLayout,
        }
    }

    pub fn is_float(&self) -> bool {
        self.block_flow.float.is_some()
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown();
        self.col_widths = ~[];
    }

    // inline(always) because this is only ever called by in-order or non-in-order top-level
    // methods
    #[inline(always)]
    fn assign_height_table_wrapper_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {
        let (clearance, top_offset, bottom_offset, left_offset) = self.block_flow.initialize_offset(false);

        let mut float_ctx = self.block_flow.handle_children_floats_if_inorder(ctx,
                                                                              Point2D(-left_offset, -top_offset),
                                                                              inorder);

        // Table wrapper flow has margin but is not collapsed with kids(table caption and table).
        let (margin_top, margin_bottom, _, _) = self.block_flow.precompute_margin();

        let mut cur_y = top_offset;

        for kid in self.block_flow.base.child_iter() {
            let child_node = flow::mut_base(*kid);
            child_node.position.origin.y = cur_y;
            cur_y = cur_y + child_node.position.size.height;
        }

        let mut height = cur_y - top_offset;

        for box_ in self.block_flow.box_.iter() {
            let style = box_.style();

            // At this point, `height` is the height of the containing block, so passing `height`
            // as the second argument here effectively makes percentages relative to the containing
            // block per CSS 2.1 § 10.5.
            height = match MaybeAuto::from_style(style.Box.height, height) {
                Auto => height,
                Specified(value) => geometry::max(value, height)
            };
        }

        self.block_flow.compute_height_position(&mut height,
                                                ctx.screen_size.height,
                                                Au(0),
                                                margin_top,
                                                margin_bottom,
                                                clearance,
                                                top_offset);

        self.block_flow.set_floats_out(&mut float_ctx, height, cur_y, top_offset,
                                       bottom_offset, left_offset, inorder);
    }

    pub fn build_display_list_table_wrapper<E:ExtraDisplayListData>(
                                    &mut self,
                                    builder: &DisplayListBuilder,
                                    dirty: &Rect<Au>,
                                    list: &RefCell<DisplayList<E>>)
                                    -> bool {
        debug!("build_display_list_table_wrapper: same process as block flow");
        self.block_flow.build_display_list_block(builder, dirty, list)
    }
}

impl Flow for TableWrapperFlow {
    fn class(&self) -> FlowClass {
        TableWrapperFlowClass
    }

    fn as_table_wrapper<'a>(&'a mut self) -> &'a mut TableWrapperFlow {
        self
    }

    /* Recursively (bottom-up) determine the context's preferred and
    minimum widths.  When called on this context, all child contexts
    have had their min/pref widths set. This function must decide
    min/pref widths based on child context widths and dimensions of
    any boxes it is responsible for flowing.  */

    fn bubble_widths(&mut self, ctx: &mut LayoutContext) {
        /* find max width from child block contexts */
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_caption() || kid.is_table());

            if kid.is_table() {
                self.col_widths.push_all(kid.as_table().col_widths);
            }
        }

        self.block_flow.bubble_widths(ctx);
    }

    /// Recursively (top-down) determines the actual width of child contexts and boxes. When called
    /// on this context, the context has had its width set by the parent context.
    ///
    /// Dual boxes consume some width first, and the remainder is assigned to all child (block)
    /// contexts.
    fn assign_widths(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_widths({}): assigning width for flow {}",
               if self.is_float() {
                   "floated table_wrapper"
               } else {
                   "table_wrapper"
               },
               self.block_flow.base.id);

        // The position was set to the containing block by the flow's parent.
        let mut remaining_width = self.block_flow.base.position.size.width;
        let mut x_offset = Au::new(0);

        let fixed_cells_width = self.col_widths.iter().fold(Au(0), |sum, width| sum.add(width));

        self.block_flow.set_containing_width_if_float(remaining_width);

        for box_ in self.block_flow.box_.iter() {
            let style = box_.style();

            // The text alignment of a table_wrapper flow is the text alignment of its box's style.
            self.block_flow.base.flags_info.flags.set_text_align(style.Text.text_align);
            let width = self.block_flow.initial_box_setting(box_, style, remaining_width, false, true);

            let screen_size = ctx.screen_size;
            let (x, w) = box_.get_x_coord_and_new_width_if_fixed(screen_size.width,
                                                                 screen_size.height,
                                                                 width,
                                                                 box_.offset(),
                                                                 self.block_flow.is_fixed);
            x_offset = x;

            // Get left and right paddings, borders for table
            let padding_left = specified(style.Padding.padding_left, remaining_width);
            let padding_right = specified(style.Padding.padding_right, remaining_width);
            let border_left = style.Border.border_left_width;
            let border_right = style.Border.border_right_width;
            let padding_and_borders = padding_left + padding_right + border_left + border_right;
            remaining_width = geometry::max(fixed_cells_width + padding_and_borders, w);

            let border_x = if self.block_flow.is_fixed {
                x_offset + box_.margin.get().left
            } else {
                box_.margin.get().left
            };
            self.block_flow.set_box_x_and_width(box_, border_x, remaining_width);
        }

        match self.table_layout {
            FixedLayout | _ if self.is_float() =>
                self.block_flow.base.position.size.width = remaining_width,
            _ => {}
        }

        self.block_flow.propagate_assigned_width_to_children(x_offset, remaining_width);
    }

    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        if self.is_float() {
            debug!("assign_height_inorder_float: assigning height for floated table_wrapper {}", self.block_flow.base.id);
            self.block_flow.assign_height_float_inorder();
        } else {
            debug!("assign_height_inorder: assigning height for table_wrapper {}", self.block_flow.base.id);
            self.assign_height_table_wrapper_base(ctx, true);
        }
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        //assign height for box
        for box_ in self.block_flow.box_.iter() {
            box_.assign_height();
        }

        if self.is_float() {
            debug!("assign_height_float: assigning height for floated table_wrapper {}", self.block_flow.base.id);
            self.block_flow.assign_height_float(ctx);
        } else {
            debug!("assign_height: assigning height for table_wrapper {}", self.block_flow.base.id);
            self.assign_height_table_wrapper_base(ctx, false);
        }
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
        let txt = if self.is_float() {
            ~"TableWrapperFlow(Float): "
        } else {
            ~"TableWrapperFlow: "
        };
        txt.append(match self.block_flow.box_ {
            Some(ref rb) => rb.debug_str(),
            None => ~"",
        })
    }
}

