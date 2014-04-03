/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use layout::box_::Box;
use layout::block::{BlockFlow, MarginsMayNotCollapse, WidthAndMarginsComputer};
use layout::block::{WidthConstraintInput, WidthConstraintSolution};
use layout::construct::FlowConstructor;
use layout::context::LayoutContext;
use layout::display_list_builder::{DisplayListBuilder, DisplayListBuildingInfo};
use layout::floats::FloatKind;
use layout::flow::{TableWrapperFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use layout::model::{Specified, Auto, specified};
use layout::wrapper::ThreadSafeLayoutNode;

use gfx::display_list::StackingContext;
use servo_util::geometry::Au;
use servo_util::geometry;
use style::computed_values::table_layout;

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
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_height_table_wrapper_base(&mut self,
                                        layout_context: &mut LayoutContext,
                                        inorder: bool) {
        self.block_flow.assign_height_block_base(layout_context, inorder, MarginsMayNotCollapse);
    }

    pub fn build_display_list_table_wrapper(&mut self,
                                            stacking_context: &mut StackingContext,
                                            builder: &mut DisplayListBuilder,
                                            info: &DisplayListBuildingInfo) {
        debug!("build_display_list_table_wrapper: same process as block flow");
        self.block_flow.build_display_list_block(stacking_context, builder, info);
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
        // get column widths info from table flow
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

        // In case of fixed layout, column widths are calculated in table flow.
        let assigned_col_widths = match self.table_layout {
            FixedLayout => None,
            AutoLayout => Some(self.col_widths.clone())
        };
        self.block_flow.propagate_assigned_width_to_children(left_content_edge, content_width, assigned_col_widths);
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
        let computed_width = match table_wrapper.table_layout {
            FixedLayout => {
                let fixed_cells_width = table_wrapper.col_widths.iter().fold(Au(0),
                                                                             |sum, width| sum.add(width));

                let mut computed_width = input.computed_width.specified_or_zero();
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
                    // Compare border-edge widths. Because fixed_cells_width indicates content-width,
                    // padding and border values are added to fixed_cells_width.
                    computed_width = geometry::max(fixed_cells_width + padding_and_borders, computed_width);
                }
                computed_width
            },
            AutoLayout => {
                // Automatic table layout is calculated according to CSS 2.1 § 17.5.2.2.
                let mut cap_min = Au(0);
                let mut cols_min = Au(0);
                let mut cols_max = Au(0);
                let mut col_min_widths = &~[];
                let mut col_pref_widths = &~[];
                for kid in table_wrapper.block_flow.base.child_iter() {
                    if kid.is_table_caption() {
                        cap_min = kid.as_block().base.intrinsic_widths.minimum_width;
                    } else {
                        assert!(kid.is_table());
                        cols_min = kid.as_block().base.intrinsic_widths.minimum_width;
                        cols_max = kid.as_block().base.intrinsic_widths.preferred_width;
                        col_min_widths = kid.col_min_widths();
                        col_pref_widths = kid.col_pref_widths();
                    }
                }
                // 'extra_width': difference between the calculated table width and minimum width
                // required by all columns. It will be distributed over the columns.
                let (width, extra_width) = match input.computed_width {
                    Auto => {
                        if input.available_width > geometry::max(cols_max, cap_min) {
                            if cols_max > cap_min {
                                table_wrapper.col_widths = col_pref_widths.clone();
                                (cols_max, Au(0))
                            } else {
                                (cap_min, cap_min - cols_min)
                            }
                        } else {
                            let max = if cols_min >= input.available_width && cols_min >= cap_min {
                                table_wrapper.col_widths = col_min_widths.clone();
                                cols_min
                            } else {
                                geometry::max(input.available_width, cap_min)
                            };
                            (max, max - cols_min)
                        }
                    },
                    Specified(width) => {
                        let max = if cols_min >= width && cols_min >= cap_min {
                            table_wrapper.col_widths = col_min_widths.clone();
                            cols_min
                        } else {
                            geometry::max(width, cap_min)
                        };
                        (max, max - cols_min)
                    }
                };
                // The extra width is distributed over the columns
                if extra_width > Au(0) {
                    let cell_len = table_wrapper.col_widths.len() as f64;
                    table_wrapper.col_widths = col_min_widths.map(|width| {
                        width + extra_width.scale_by(1.0 / cell_len)
                    });
                }
                width
            }
        };
        input.computed_width = Specified(computed_width);
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
