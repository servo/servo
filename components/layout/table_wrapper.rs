/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS tables.

#![deny(unsafe_block)]

use block::{BlockFlow, BlockNonReplaced, FloatNonReplaced, ISizeAndMarginsComputer};
use block::{ISizeConstraintInput, MarginsMayNotCollapse};
use construct::FlowConstructor;
use context::LayoutContext;
use floats::FloatKind;
use flow::{TableWrapperFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use fragment::Fragment;
use model::{Specified, Auto, specified};
use wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use std::cmp::max;
use std::fmt;
use style::computed_values::{clear, float, table_layout};

#[deriving(Encodable)]
pub enum TableLayout {
    FixedLayout,
    AutoLayout
}

/// A table wrapper flow based on a block formatting context.
#[deriving(Encodable)]
pub struct TableWrapperFlow {
    pub block_flow: BlockFlow,

    /// Column inline-sizes
    pub col_inline_sizes: Vec<Au>,

    /// Table-layout property
    pub table_layout: TableLayout,
}

impl TableWrapperFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                  fragment: Fragment)
                                  -> TableWrapperFlow {
        let mut block_flow = BlockFlow::from_node_and_fragment(node, fragment);
        let table_layout = if block_flow.fragment().style().get_table().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableWrapperFlow {
            block_flow: block_flow,
            col_inline_sizes: vec!(),
            table_layout: table_layout
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableWrapperFlow {
        let mut block_flow = BlockFlow::from_node(constructor, node);
        let table_layout = if block_flow.fragment().style().get_table().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableWrapperFlow {
            block_flow: block_flow,
            col_inline_sizes: vec!(),
            table_layout: table_layout
        }
    }

    pub fn float_from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                        fragment: Fragment,
                                        float_kind: FloatKind)
                                        -> TableWrapperFlow {
        let mut block_flow = BlockFlow::float_from_node_and_fragment(node, fragment, float_kind);
        let table_layout = if block_flow.fragment().style().get_table().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableWrapperFlow {
            block_flow: block_flow,
            col_inline_sizes: vec!(),
            table_layout: table_layout
        }
    }

    pub fn build_display_list_table_wrapper(&mut self, layout_context: &LayoutContext) {
        debug!("build_display_list_table_wrapper: same process as block flow");
        self.block_flow.build_display_list_block(layout_context);
    }

    fn calculate_table_column_sizes(&mut self, mut input: ISizeConstraintInput)
                                    -> ISizeConstraintInput {
        let style = self.block_flow.fragment.style();

        // Get inline-start and inline-end paddings, borders for table.
        // We get these values from the fragment's style since table_wrapper doesn't have its own
        // border or padding. input.available_inline_size is same as containing_block_inline_size
        // in table_wrapper.
        let padding = style.logical_padding();
        let border = style.logical_border_width();
        let padding_and_borders =
            specified(padding.inline_start, input.available_inline_size) +
            specified(padding.inline_end, input.available_inline_size) +
            border.inline_start +
            border.inline_end;

        let computed_inline_size = match self.table_layout {
            FixedLayout => {
                let fixed_cells_inline_size = self.col_inline_sizes
                                                  .iter()
                                                  .fold(Au(0), |sum, inline_size| {
                        sum.add(inline_size)
                    });

                let mut computed_inline_size = input.computed_inline_size.specified_or_zero();

                // Compare border-edge inline-sizes. Because fixed_cells_inline_size indicates
                // content-inline-size, padding and border values are added to
                // fixed_cells_inline_size.
                computed_inline_size = max(
                    fixed_cells_inline_size + padding_and_borders, computed_inline_size);
                computed_inline_size
            },
            AutoLayout => {
                // Automatic table layout is calculated according to CSS 2.1 § 17.5.2.2.
                let mut cap_min = Au(0);
                let mut cols_min = Au(0);
                let mut cols_max = Au(0);
                let mut col_min_inline_sizes = &vec!();
                let mut col_pref_inline_sizes = &vec!();
                for kid in self.block_flow.base.child_iter() {
                    if kid.is_table_caption() {
                        cap_min = kid.as_block().base.intrinsic_inline_sizes.minimum_inline_size;
                    } else {
                        assert!(kid.is_table());
                        cols_min = kid.as_block().base.intrinsic_inline_sizes.minimum_inline_size;
                        cols_max = kid.as_block()
                                      .base
                                      .intrinsic_inline_sizes
                                      .preferred_inline_size;
                        col_min_inline_sizes = kid.col_min_inline_sizes();
                        col_pref_inline_sizes = kid.col_pref_inline_sizes();
                    }
                }
                // 'extra_inline-size': difference between the calculated table inline-size and
                // minimum inline-size required by all columns. It will be distributed over the
                // columns.
                let (inline_size, extra_inline_size) = match input.computed_inline_size {
                    Auto => {
                        if input.available_inline_size > max(cols_max, cap_min) {
                            if cols_max > cap_min {
                                self.col_inline_sizes = col_pref_inline_sizes.clone();
                                (cols_max, Au(0))
                            } else {
                                (cap_min, cap_min - cols_min)
                            }
                        } else {
                            let max = if cols_min >= input.available_inline_size &&
                                    cols_min >= cap_min {
                                self.col_inline_sizes = col_min_inline_sizes.clone();
                                cols_min
                            } else {
                                max(input.available_inline_size, cap_min)
                            };
                            (max, max - cols_min)
                        }
                    },
                    Specified(inline_size) => {
                        let max = if cols_min >= inline_size && cols_min >= cap_min {
                            self.col_inline_sizes = col_min_inline_sizes.clone();
                            cols_min
                        } else {
                            max(inline_size, cap_min)
                        };
                        (max, max - cols_min)
                    }
                };
                // The extra inline-size is distributed over the columns
                if extra_inline_size > Au(0) {
                    let cell_len = self.col_inline_sizes.len() as f64;
                    self.col_inline_sizes = col_min_inline_sizes.iter()
                                                                         .map(|inline_size| {
                        inline_size + extra_inline_size.scale_by(1.0 / cell_len)
                    }).collect();
                }
                inline_size + padding_and_borders
            }
        };
        input.computed_inline_size = Specified(computed_inline_size);
        input
    }

    fn compute_used_inline_size(&mut self,
                                layout_context: &LayoutContext,
                                parent_flow_inline_size: Au) {
        // Delegate to the appropriate inline size computer to find the constraint inputs.
        let mut input = if self.is_float() {
            FloatNonReplaced.compute_inline_size_constraint_inputs(&mut self.block_flow,
                                                                   parent_flow_inline_size,
                                                                   layout_context)
        } else {
            BlockNonReplaced.compute_inline_size_constraint_inputs(&mut self.block_flow,
                                                                   parent_flow_inline_size,
                                                                   layout_context)
        };

        // Compute the inline sizes of the columns.
        input = self.calculate_table_column_sizes(input);

        // Delegate to the appropriate inline size computer to write the constraint solutions in.
        if self.is_float() {
            let solution = FloatNonReplaced.solve_inline_size_constraints(&mut self.block_flow,
                                                                          &input);
            FloatNonReplaced.set_inline_size_constraint_solutions(&mut self.block_flow, solution);
            FloatNonReplaced.set_flow_x_coord_if_necessary(&mut self.block_flow, solution);
        } else {
            let solution = BlockNonReplaced.solve_inline_size_constraints(&mut self.block_flow,
                                                                          &input);
            BlockNonReplaced.set_inline_size_constraint_solutions(&mut self.block_flow, solution);
            BlockNonReplaced.set_flow_x_coord_if_necessary(&mut self.block_flow, solution);
        }
    }
}

impl Flow for TableWrapperFlow {
    fn class(&self) -> FlowClass {
        TableWrapperFlowClass
    }

    fn is_float(&self) -> bool {
        self.block_flow.is_float()
    }

    fn as_table_wrapper<'a>(&'a mut self) -> &'a mut TableWrapperFlow {
        self
    }

    fn as_immutable_table_wrapper<'a>(&'a self) -> &'a TableWrapperFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn float_clearance(&self) -> clear::T {
        self.block_flow.float_clearance()
    }

    fn float_kind(&self) -> float::T {
        self.block_flow.float_kind()
    }

    fn bubble_inline_sizes(&mut self, ctx: &LayoutContext) {
        // get column inline-sizes info from table flow
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_caption() || kid.is_table());

            if kid.is_table() {
                self.col_inline_sizes.push_all(kid.as_table().col_inline_sizes.as_slice());
            }
        }

        self.block_flow.bubble_inline_sizes(ctx);
    }

    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        debug!("assign_inline_sizes({}): assigning inline_size for flow",
               if self.is_float() {
                   "floated table_wrapper"
               } else {
                   "table_wrapper"
               });

        // Table wrappers are essentially block formatting contexts and are therefore never
        // impacted by floats.
        self.block_flow.base.flags.set_impacted_by_left_floats(false);
        self.block_flow.base.flags.set_impacted_by_right_floats(false);

        // Our inline-size was set to the inline-size of the containing block by the flow's parent.
        // Now compute the real value.
        let containing_block_inline_size = self.block_flow.base.position.size.inline;
        if self.is_float() {
            self.block_flow.float.as_mut().unwrap().containing_inline_size =
                containing_block_inline_size;
        }

        self.compute_used_inline_size(layout_context, containing_block_inline_size);

        let inline_start_content_edge = self.block_flow.fragment.border_box.start.i;
        let content_inline_size = self.block_flow.fragment.border_box.size.inline;

        // In case of fixed layout, column inline-sizes are calculated in table flow.
        let assigned_col_inline_sizes = match self.table_layout {
            FixedLayout => None,
            AutoLayout => Some(self.col_inline_sizes.clone())
        };

        self.block_flow.propagate_assigned_inline_size_to_children(inline_start_content_edge,
                                                                   content_inline_size,
                                                                   assigned_col_inline_sizes);
    }

    fn assign_block_size<'a>(&mut self, ctx: &'a LayoutContext<'a>) {
        debug!("assign_block_size: assigning block_size for table_wrapper");
        self.block_flow.assign_block_size_block_base(ctx, MarginsMayNotCollapse);
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }

    fn assign_block_size_for_inorder_child_if_necessary<'a>(&mut self,
                                                            layout_context: &'a LayoutContext<'a>)
                                                            -> bool {
        if self.block_flow.is_float() {
            self.block_flow.place_float();
            return true
        }

        let impacted = self.block_flow.base.flags.impacted_by_floats();
        if impacted {
            self.assign_block_size(layout_context);
        }
        impacted
    }
}

impl fmt::Show for TableWrapperFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_float() {
            write!(f, "TableWrapperFlow(Float): {}", self.block_flow.fragment)
        } else {
            write!(f, "TableWrapperFlow: {}", self.block_flow.fragment)
        }
    }
}

