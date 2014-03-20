/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::BlockFlow;
use layout::block::{WidthAndMarginsComputer, WidthConstraintInput, WidthConstraintSolution};
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, ExtraDisplayListData};
use layout::floats::{FloatKind};
use layout::flow::{TableWrapperFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::flow;
use layout::model::{MaybeAuto, Specified, Auto, specified};
use layout::wrapper::ThreadSafeLayoutNode;

use std::cell::RefCell;
use style::computed_values::table_layout;
use geom::{Point2D, Rect, Size2D};
use gfx::display_list::DisplayListCollection;
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
    pub fn from_node_and_box(node: &ThreadSafeLayoutNode,
                             box_: Box)
                             -> TableWrapperFlow {
        let mut block_flow = BlockFlow::from_node_and_box(node, box_);
        let table_layout = if block_flow.box_().style().Table.get().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableWrapperFlow {
            block_flow: block_flow,
            col_widths: ~[],
            table_layout: table_layout
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableWrapperFlow {
        let mut block_flow = BlockFlow::from_node(constructor, node);
        let table_layout = if block_flow.box_().style().Table.get().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableWrapperFlow {
            block_flow: block_flow,
            col_widths: ~[],
            table_layout: table_layout
        }
    }

    pub fn float_from_node(constructor: &mut FlowConstructor,
                           node: &ThreadSafeLayoutNode,
                           float_kind: FloatKind)
                           -> TableWrapperFlow {
        let mut block_flow = BlockFlow::float_from_node(constructor, node, float_kind);
        let table_layout = if block_flow.box_().style().Table.get().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableWrapperFlow {
            block_flow: block_flow,
            col_widths: ~[],
            table_layout: table_layout
        }
    }

    pub fn is_float(&self) -> bool {
        self.block_flow.float.is_some()
    }

    pub fn teardown(&mut self) {
        self.block_flow.teardown();
        self.col_widths = ~[];
    }

    /// Assign height for table-wrapper flow.
    /// `Assign height` of table-wrapper flow follows a similar process to that of block flow.
    /// However, table-wrapper flow doesn't consider collapsing margins for flow's children
    /// and calculating padding/border.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_height_table_wrapper_base(&mut self, ctx: &mut LayoutContext, inorder: bool) {

        // Note: Ignoring clearance for absolute flows as of now.
        let ignore_clear = self.is_absolutely_positioned();
        let (clearance, top_offset, bottom_offset, left_offset) = self.block_flow.initialize_offsets(ignore_clear);

        self.block_flow.handle_children_floats_if_necessary(ctx, inorder,
                                                            left_offset, top_offset);

        // Table wrapper flow has margin but is not collapsed with kids(table caption and table).
        let (margin_top, margin_bottom, _, _) = self.block_flow.precompute_margin();

        let mut cur_y = top_offset;

        for kid in self.block_flow.base.child_iter() {
            let child_node = flow::mut_base(kid);
            child_node.position.origin.y = cur_y;
            cur_y = cur_y + child_node.position.size.height;
        }

        // top_offset: top margin-edge of the topmost child.
        // hence, height = content height
        let mut height = cur_y - top_offset;

        // For an absolutely positioned element, store the content height and stop the function.
        if self.block_flow.store_content_height_if_absolutely_positioned(height) {
            return;
        }

        for box_ in self.block_flow.box_.iter() {
            let style = box_.style();

            // At this point, `height` is the height of the containing block, so passing `height`
            // as the second argument here effectively makes percentages relative to the containing
            // block per CSS 2.1 § 10.5.
            height = match MaybeAuto::from_style(style.Box.get().height, height) {
                Auto => height,
                Specified(value) => geometry::max(value, height)
            };
        }

        self.block_flow.compute_height_position(&mut height,
                                                Au(0),
                                                margin_top,
                                                margin_bottom,
                                                clearance);

        self.block_flow.set_floats_out_if_inorder(inorder, height, cur_y,
                                                  top_offset, bottom_offset, left_offset);
        self.block_flow.assign_height_absolute_flows(ctx);
    }

    pub fn build_display_list_table_wrapper<E:ExtraDisplayListData>(
                                            &mut self,
                                            builder: &DisplayListBuilder,
                                            container_block_size: &Size2D<Au>,
                                            absolute_cb_abs_position: Point2D<Au>,
                                            dirty: &Rect<Au>,
                                            index: uint,
                                            lists: &RefCell<DisplayListCollection<E>>)
                                            -> uint {
        debug!("build_display_list_table_wrapper: same process as block flow");
        self.block_flow.build_display_list_block(builder, container_block_size,
                                                 absolute_cb_abs_position,
                                                 dirty, index, lists)
    }
}

impl Flow for TableWrapperFlow {
    fn class(&self) -> FlowClass {
        TableWrapperFlowClass
    }

    fn as_table_wrapper<'a>(&'a mut self) -> &'a mut TableWrapperFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
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
        debug!("assign_widths({}): assigning width for flow",
               if self.is_float() {
                   "floated table_wrapper"
               } else {
                   "table_wrapper"
               });

        // The position was set to the containing block by the flow's parent.
        let containing_block_width = self.block_flow.base.position.size.width;
        let mut left_content_edge = Au::new(0);
        let mut content_width = containing_block_width;

        self.block_flow.set_containing_width_if_float(containing_block_width);

        let width_computer = TableWrapper;
        width_computer.compute_used_width_table_wrapper(self, ctx, containing_block_width);

        for box_ in self.block_flow.box_.iter() {
            left_content_edge = box_.border_box.get().origin.x;
            content_width = box_.border_box.get().size.width;
        }

        match self.table_layout {
            FixedLayout | _ if self.is_float() =>
                self.block_flow.base.position.size.width = content_width,
            _ => {}
        }

        self.block_flow.propagate_assigned_width_to_children(left_content_edge, content_width, None);
    }

    /// This is called on kid flows by a parent.
    ///
    /// Hence, we can assume that assign_height has already been called on the
    /// kid (because of the bottom-up traversal).
    fn assign_height_inorder(&mut self, ctx: &mut LayoutContext) {
        if self.is_float() {
            debug!("assign_height_inorder_float: assigning height for floated table_wrapper");
            self.block_flow.assign_height_float_inorder();
        } else {
            debug!("assign_height_inorder: assigning height for table_wrapper");
            self.assign_height_table_wrapper_base(ctx, true);
        }
    }

    fn assign_height(&mut self, ctx: &mut LayoutContext) {
        if self.is_float() {
            debug!("assign_height_float: assigning height for floated table_wrapper");
            self.block_flow.assign_height_float(ctx);
        } else {
            debug!("assign_height: assigning height for table_wrapper");
            self.assign_height_table_wrapper_base(ctx, false);
        }
    }

    // CSS Section 8.3.1 - Collapsing Margins
    // `self`: the Flow whose margins we want to collapse.
    // `collapsing`: value to be set by this function. This tells us how much
    // of the top margin has collapsed with a previous margin.
    // `collapsible`: Potential collapsible margin at the bottom of this flow's box.
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

struct TableWrapper;
impl TableWrapper {
    fn compute_used_width_table_wrapper(&self,
                                        table_wrapper: &mut TableWrapperFlow,
                                        ctx: &mut LayoutContext,
                                        parent_flow_width: Au) {
        let input = self.compute_width_constraint_inputs_table_wrapper(table_wrapper,
                                                                           parent_flow_width,
                                                                           ctx);

        let solution = self.solve_width_constraints(&mut table_wrapper.block_flow, input);

        self.set_width_constraint_solutions(&mut table_wrapper.block_flow, solution);
        self.set_flow_x_coord_if_necessary(&mut table_wrapper.block_flow, solution);
    }

    fn compute_width_constraint_inputs_table_wrapper(&self,
                                                     table_wrapper: &mut TableWrapperFlow,
                                                     parent_flow_width: Au,
                                                     ctx: &mut LayoutContext)
                                                     -> WidthConstraintInput {
        let mut input = self.compute_width_constraint_inputs(&mut table_wrapper.block_flow,
                                                             parent_flow_width,
                                                             ctx);
        match table_wrapper.table_layout {
            FixedLayout => {
                let fixed_cells_width = table_wrapper.col_widths.iter().fold(Au(0),
                                                                             |sum, width| sum.add(width));
                for box_ in table_wrapper.block_flow.box_.iter() {
                    let style = box_.style();

                    // Get left and right paddings, borders for table.
                    // We get these values from the box's style since table_wrapper doesn't have it's own border or padding.
                    // input.available_width is same as containing_block_width in table_wrapper.
                    let padding_left = specified(style.Padding.get().padding_left,
                                                 input.available_width);
                    let padding_right = specified(style.Padding.get().padding_right,
                                                  input.available_width);
                    let border_left = style.Border.get().border_left_width;
                    let border_right = style.Border.get().border_right_width;
                    let padding_and_borders = padding_left + padding_right + border_left + border_right;
                    let mut computed_width = input.computed_width.specified_or_zero();
                    // Compare border-edge widths. Because fixed_cells_width indicates content-width,
                    // padding and border values are added to fixed_cells_width.
                    computed_width = geometry::max(fixed_cells_width + padding_and_borders, computed_width);
                    input.computed_width = Specified(computed_width);
                }
            },
            _ => {}
        }
        input
    }
}

impl WidthAndMarginsComputer for TableWrapper {
    /// Solve the width and margins constraints for this block flow.
    fn solve_width_constraints(&self,
                               block: &mut BlockFlow,
                               input: WidthConstraintInput)
                               -> WidthConstraintSolution {
        self.solve_block_width_constraints(block, input)
    }
}
