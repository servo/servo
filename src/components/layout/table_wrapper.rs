/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_block)]

use block::{BlockFlow, MarginsMayNotCollapse, ISizeAndMarginsComputer};
use block::{ISizeConstraintInput, ISizeConstraintSolution};
use construct::FlowConstructor;
use context::LayoutContext;
use floats::FloatKind;
use flow::{TableWrapperFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use fragment::Fragment;
use model::{Specified, Auto, specified};
use wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use servo_util::geometry;
use std::fmt;
use style::computed_values::table_layout;

pub enum TableLayout {
    FixedLayout,
    AutoLayout
}

/// A table wrapper flow based on a block formatting context.
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

    pub fn float_from_node(constructor: &mut FlowConstructor,
                           node: &ThreadSafeLayoutNode,
                           float_kind: FloatKind)
                           -> TableWrapperFlow {
        let mut block_flow = BlockFlow::float_from_node(constructor, node, float_kind);
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

    pub fn is_float(&self) -> bool {
        self.block_flow.float.is_some()
    }

    /// Assign block-size for table-wrapper flow.
    /// `Assign block-size` of table-wrapper flow follows a similar process to that of block flow.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_block_size_table_wrapper_base(&mut self, layout_context: &mut LayoutContext) {
        self.block_flow.assign_block_size_block_base(layout_context, MarginsMayNotCollapse);
    }

    pub fn build_display_list_table_wrapper(&mut self, layout_context: &LayoutContext) {
        debug!("build_display_list_table_wrapper: same process as block flow");
        self.block_flow.build_display_list_block(layout_context);
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
    minimum inline_sizes.  When called on this context, all child contexts
    have had their min/pref inline_sizes set. This function must decide
    min/pref inline_sizes based on child context inline_sizes and dimensions of
    any fragments it is responsible for flowing.  */

    fn bubble_inline_sizes(&mut self, ctx: &mut LayoutContext) {
        // get column inline-sizes info from table flow
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_caption() || kid.is_table());

            if kid.is_table() {
                self.col_inline_sizes.push_all(kid.as_table().col_inline_sizes.as_slice());
            }
        }

        self.block_flow.bubble_inline_sizes(ctx);
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments. When
    /// called on this context, the context has had its inline-size set by the parent context.
    ///
    /// Dual fragments consume some inline-size first, and the remainder is assigned to all child (block)
    /// contexts.
    fn assign_inline_sizes(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_inline_sizes({}): assigning inline_size for flow",
               if self.is_float() {
                   "floated table_wrapper"
               } else {
                   "table_wrapper"
               });

        // The position was set to the containing block by the flow's parent.
        let containing_block_inline_size = self.block_flow.base.position.size.inline;

        let inline_size_computer = TableWrapper;
        inline_size_computer.compute_used_inline_size_table_wrapper(self, ctx, containing_block_inline_size);

        let inline_start_content_edge = self.block_flow.fragment.border_box.start.i;
        let content_inline_size = self.block_flow.fragment.border_box.size.inline;

        match self.table_layout {
            FixedLayout | _ if self.is_float() =>
                self.block_flow.base.position.size.inline = content_inline_size,
            _ => {}
        }

        // In case of fixed layout, column inline-sizes are calculated in table flow.
        let assigned_col_inline_sizes = match self.table_layout {
            FixedLayout => None,
            AutoLayout => Some(self.col_inline_sizes.clone())
        };
        self.block_flow.propagate_assigned_inline_size_to_children(inline_start_content_edge, content_inline_size, assigned_col_inline_sizes);
    }

    fn assign_block_size(&mut self, ctx: &mut LayoutContext) {
        if self.is_float() {
            debug!("assign_block_size_float: assigning block_size for floated table_wrapper");
            self.block_flow.assign_block_size_float(ctx);
        } else {
            debug!("assign_block_size: assigning block_size for table_wrapper");
            self.assign_block_size_table_wrapper_base(ctx);
        }
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
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

struct TableWrapper;

impl TableWrapper {
    fn compute_used_inline_size_table_wrapper(&self,
                                        table_wrapper: &mut TableWrapperFlow,
                                        ctx: &mut LayoutContext,
                                        parent_flow_inline_size: Au) {
        let input = self.compute_inline_size_constraint_inputs_table_wrapper(table_wrapper,
                                                                       parent_flow_inline_size,
                                                                       ctx);

        let solution = self.solve_inline_size_constraints(&mut table_wrapper.block_flow, &input);

        self.set_inline_size_constraint_solutions(&mut table_wrapper.block_flow, solution);
        self.set_flow_x_coord_if_necessary(&mut table_wrapper.block_flow, solution);
    }

    fn compute_inline_size_constraint_inputs_table_wrapper(&self,
                                                     table_wrapper: &mut TableWrapperFlow,
                                                     parent_flow_inline_size: Au,
                                                     ctx: &mut LayoutContext)
                                                     -> ISizeConstraintInput {
        let mut input = self.compute_inline_size_constraint_inputs(&mut table_wrapper.block_flow,
                                                             parent_flow_inline_size,
                                                             ctx);
        let computed_inline_size = match table_wrapper.table_layout {
            FixedLayout => {
                let fixed_cells_inline_size = table_wrapper.col_inline_sizes.iter().fold(Au(0),
                                                                             |sum, inline_size| sum.add(inline_size));

                let mut computed_inline_size = input.computed_inline_size.specified_or_zero();
                let style = table_wrapper.block_flow.fragment.style();

                // Get inline-start and inline-end paddings, borders for table.
                // We get these values from the fragment's style since table_wrapper doesn't have it's own border or padding.
                // input.available_inline-size is same as containing_block_inline-size in table_wrapper.
                let padding = style.logical_padding();
                let border = style.logical_border_width();
                let padding_and_borders =
                    specified(padding.inline_start, input.available_inline_size) +
                    specified(padding.inline_end, input.available_inline_size) +
                    border.inline_start +
                    border.inline_end;
                // Compare border-edge inline-sizes. Because fixed_cells_inline-size indicates content-inline-size,
                // padding and border values are added to fixed_cells_inline-size.
                computed_inline_size = geometry::max(
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
                for kid in table_wrapper.block_flow.base.child_iter() {
                    if kid.is_table_caption() {
                        cap_min = kid.as_block().base.intrinsic_inline_sizes.minimum_inline_size;
                    } else {
                        assert!(kid.is_table());
                        cols_min = kid.as_block().base.intrinsic_inline_sizes.minimum_inline_size;
                        cols_max = kid.as_block().base.intrinsic_inline_sizes.preferred_inline_size;
                        col_min_inline_sizes = kid.col_min_inline_sizes();
                        col_pref_inline_sizes = kid.col_pref_inline_sizes();
                    }
                }
                // 'extra_inline-size': difference between the calculated table inline-size and minimum inline-size
                // required by all columns. It will be distributed over the columns.
                let (inline_size, extra_inline_size) = match input.computed_inline_size {
                    Auto => {
                        if input.available_inline_size > geometry::max(cols_max, cap_min) {
                            if cols_max > cap_min {
                                table_wrapper.col_inline_sizes = col_pref_inline_sizes.clone();
                                (cols_max, Au(0))
                            } else {
                                (cap_min, cap_min - cols_min)
                            }
                        } else {
                            let max = if cols_min >= input.available_inline_size && cols_min >= cap_min {
                                table_wrapper.col_inline_sizes = col_min_inline_sizes.clone();
                                cols_min
                            } else {
                                geometry::max(input.available_inline_size, cap_min)
                            };
                            (max, max - cols_min)
                        }
                    },
                    Specified(inline_size) => {
                        let max = if cols_min >= inline_size && cols_min >= cap_min {
                            table_wrapper.col_inline_sizes = col_min_inline_sizes.clone();
                            cols_min
                        } else {
                            geometry::max(inline_size, cap_min)
                        };
                        (max, max - cols_min)
                    }
                };
                // The extra inline-size is distributed over the columns
                if extra_inline_size > Au(0) {
                    let cell_len = table_wrapper.col_inline_sizes.len() as f64;
                    table_wrapper.col_inline_sizes = col_min_inline_sizes.iter().map(|inline_size| {
                        inline_size + extra_inline_size.scale_by(1.0 / cell_len)
                    }).collect();
                }
                inline_size
            }
        };
        input.computed_inline_size = Specified(computed_inline_size);
        input
    }
}

impl ISizeAndMarginsComputer for TableWrapper {
    /// Solve the inline-size and margins constraints for this block flow.
    fn solve_inline_size_constraints(&self, block: &mut BlockFlow, input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        self.solve_block_inline_size_constraints(block, input)
    }
}
