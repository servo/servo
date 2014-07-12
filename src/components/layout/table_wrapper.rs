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

    /// Column isizes
    pub col_isizes: Vec<Au>,

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
            col_isizes: vec!(),
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
            col_isizes: vec!(),
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
            col_isizes: vec!(),
            table_layout: table_layout
        }
    }

    pub fn is_float(&self) -> bool {
        self.block_flow.float.is_some()
    }

    /// Assign bsize for table-wrapper flow.
    /// `Assign bsize` of table-wrapper flow follows a similar process to that of block flow.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_bsize_table_wrapper_base(&mut self, layout_context: &mut LayoutContext) {
        self.block_flow.assign_bsize_block_base(layout_context, MarginsMayNotCollapse);
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
    minimum isizes.  When called on this context, all child contexts
    have had their min/pref isizes set. This function must decide
    min/pref isizes based on child context isizes and dimensions of
    any fragments it is responsible for flowing.  */

    fn bubble_isizes(&mut self, ctx: &mut LayoutContext) {
        // get column isizes info from table flow
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_caption() || kid.is_table());

            if kid.is_table() {
                self.col_isizes.push_all(kid.as_table().col_isizes.as_slice());
            }
        }

        self.block_flow.bubble_isizes(ctx);
    }

    /// Recursively (top-down) determines the actual isize of child contexts and fragments. When
    /// called on this context, the context has had its isize set by the parent context.
    ///
    /// Dual fragments consume some isize first, and the remainder is assigned to all child (block)
    /// contexts.
    fn assign_isizes(&mut self, ctx: &mut LayoutContext) {
        debug!("assign_isizes({}): assigning isize for flow",
               if self.is_float() {
                   "floated table_wrapper"
               } else {
                   "table_wrapper"
               });

        // The position was set to the containing block by the flow's parent.
        let containing_block_isize = self.block_flow.base.position.size.isize;

        let isize_computer = TableWrapper;
        isize_computer.compute_used_isize_table_wrapper(self, ctx, containing_block_isize);

        let istart_content_edge = self.block_flow.fragment.border_box.start.i;
        let content_isize = self.block_flow.fragment.border_box.size.isize;

        match self.table_layout {
            FixedLayout | _ if self.is_float() =>
                self.block_flow.base.position.size.isize = content_isize,
            _ => {}
        }

        // In case of fixed layout, column isizes are calculated in table flow.
        let assigned_col_isizes = match self.table_layout {
            FixedLayout => None,
            AutoLayout => Some(self.col_isizes.clone())
        };
        self.block_flow.propagate_assigned_isize_to_children(istart_content_edge, content_isize, assigned_col_isizes);
    }

    fn assign_bsize(&mut self, ctx: &mut LayoutContext) {
        if self.is_float() {
            debug!("assign_bsize_float: assigning bsize for floated table_wrapper");
            self.block_flow.assign_bsize_float(ctx);
        } else {
            debug!("assign_bsize: assigning bsize for table_wrapper");
            self.assign_bsize_table_wrapper_base(ctx);
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
    fn compute_used_isize_table_wrapper(&self,
                                        table_wrapper: &mut TableWrapperFlow,
                                        ctx: &mut LayoutContext,
                                        parent_flow_isize: Au) {
        let input = self.compute_isize_constraint_inputs_table_wrapper(table_wrapper,
                                                                       parent_flow_isize,
                                                                       ctx);

        let solution = self.solve_isize_constraints(&mut table_wrapper.block_flow, &input);

        self.set_isize_constraint_solutions(&mut table_wrapper.block_flow, solution);
        self.set_flow_x_coord_if_necessary(&mut table_wrapper.block_flow, solution);
    }

    fn compute_isize_constraint_inputs_table_wrapper(&self,
                                                     table_wrapper: &mut TableWrapperFlow,
                                                     parent_flow_isize: Au,
                                                     ctx: &mut LayoutContext)
                                                     -> ISizeConstraintInput {
        let mut input = self.compute_isize_constraint_inputs(&mut table_wrapper.block_flow,
                                                             parent_flow_isize,
                                                             ctx);
        let computed_isize = match table_wrapper.table_layout {
            FixedLayout => {
                let fixed_cells_isize = table_wrapper.col_isizes.iter().fold(Au(0),
                                                                             |sum, isize| sum.add(isize));

                let mut computed_isize = input.computed_isize.specified_or_zero();
                let style = table_wrapper.block_flow.fragment.style();

                // Get istart and iend paddings, borders for table.
                // We get these values from the fragment's style since table_wrapper doesn't have it's own border or padding.
                // input.available_isize is same as containing_block_isize in table_wrapper.
                let padding = style.logical_padding();
                let border = style.logical_border_width();
                let padding_and_borders =
                    specified(padding.istart, input.available_isize) +
                    specified(padding.iend, input.available_isize) +
                    border.istart +
                    border.iend;
                // Compare border-edge isizes. Because fixed_cells_isize indicates content-isize,
                // padding and border values are added to fixed_cells_isize.
                computed_isize = geometry::max(
                    fixed_cells_isize + padding_and_borders, computed_isize);
                computed_isize
            },
            AutoLayout => {
                // Automatic table layout is calculated according to CSS 2.1 § 17.5.2.2.
                let mut cap_min = Au(0);
                let mut cols_min = Au(0);
                let mut cols_max = Au(0);
                let mut col_min_isizes = &vec!();
                let mut col_pref_isizes = &vec!();
                for kid in table_wrapper.block_flow.base.child_iter() {
                    if kid.is_table_caption() {
                        cap_min = kid.as_block().base.intrinsic_isizes.minimum_isize;
                    } else {
                        assert!(kid.is_table());
                        cols_min = kid.as_block().base.intrinsic_isizes.minimum_isize;
                        cols_max = kid.as_block().base.intrinsic_isizes.preferred_isize;
                        col_min_isizes = kid.col_min_isizes();
                        col_pref_isizes = kid.col_pref_isizes();
                    }
                }
                // 'extra_isize': difference between the calculated table isize and minimum isize
                // required by all columns. It will be distributed over the columns.
                let (isize, extra_isize) = match input.computed_isize {
                    Auto => {
                        if input.available_isize > geometry::max(cols_max, cap_min) {
                            if cols_max > cap_min {
                                table_wrapper.col_isizes = col_pref_isizes.clone();
                                (cols_max, Au(0))
                            } else {
                                (cap_min, cap_min - cols_min)
                            }
                        } else {
                            let max = if cols_min >= input.available_isize && cols_min >= cap_min {
                                table_wrapper.col_isizes = col_min_isizes.clone();
                                cols_min
                            } else {
                                geometry::max(input.available_isize, cap_min)
                            };
                            (max, max - cols_min)
                        }
                    },
                    Specified(isize) => {
                        let max = if cols_min >= isize && cols_min >= cap_min {
                            table_wrapper.col_isizes = col_min_isizes.clone();
                            cols_min
                        } else {
                            geometry::max(isize, cap_min)
                        };
                        (max, max - cols_min)
                    }
                };
                // The extra isize is distributed over the columns
                if extra_isize > Au(0) {
                    let cell_len = table_wrapper.col_isizes.len() as f64;
                    table_wrapper.col_isizes = col_min_isizes.iter().map(|isize| {
                        isize + extra_isize.scale_by(1.0 / cell_len)
                    }).collect();
                }
                isize
            }
        };
        input.computed_isize = Specified(computed_isize);
        input
    }
}

impl ISizeAndMarginsComputer for TableWrapper {
    /// Solve the isize and margins constraints for this block flow.
    fn solve_isize_constraints(&self, block: &mut BlockFlow, input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        self.solve_block_isize_constraints(block, input)
    }
}
