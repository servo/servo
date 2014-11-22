/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_blocks)]

use block::BlockFlow;
use block::ISizeAndMarginsComputer;
use construct::FlowConstructor;
use context::LayoutContext;
use flow::{TableRowFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use flow;
use fragment::{Fragment, FragmentBoundsIterator};
use layout_debug;
use table::{ColumnInlineSize, InternalTable};
use model::{MaybeAuto, Specified, Auto};
use wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use std::cmp::max;
use std::fmt;
use style::ComputedValues;
use style::computed_values::{LPA_Auto, LPA_Length, LPA_Percentage};
use sync::Arc;

/// A single row of a table.
#[deriving(Encodable)]
pub struct TableRowFlow {
    pub block_flow: BlockFlow,

    /// Information about the inline-sizes of each column.
    pub column_inline_sizes: Vec<ColumnInlineSize>,
}

impl TableRowFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                  fragment: Fragment)
                                  -> TableRowFlow {
        TableRowFlow {
            block_flow: BlockFlow::from_node_and_fragment(node, fragment),
            column_inline_sizes: Vec::new()
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableRowFlow {
        TableRowFlow {
            block_flow: BlockFlow::from_node(constructor, node),
            column_inline_sizes: Vec::new()
        }
    }

    pub fn fragment<'a>(&'a mut self) -> &'a Fragment {
        &self.block_flow.fragment
    }

    fn initialize_offsets(&mut self) -> (Au, Au, Au) {
        // TODO: If border-collapse: collapse, block_start_offset, block_end_offset, and
        // inline_start_offset should be updated. Currently, they are set as Au(0).
        (Au(0), Au(0), Au(0))
    }

    /// Assign block-size for table-row flow.
    ///
    /// TODO(pcwalton): This doesn't handle floats and positioned elements right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_block_size_table_row_base<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        let (block_start_offset, _, _) = self.initialize_offsets();

        let /* mut */ cur_y = block_start_offset;

        // Per CSS 2.1 § 17.5.3, find max_y = max(computed `block-size`, minimum block-size of all
        // cells).
        let mut max_y = Au(0);
        for kid in self.block_flow.base.child_iter() {
            kid.place_float_if_applicable(layout_context);
            if !flow::base(kid).flags.is_float() {
                kid.assign_block_size_for_inorder_child_if_necessary(layout_context);
            }

            {
                let child_fragment = kid.as_table_cell().fragment();
                // TODO: Percentage block-size
                let child_specified_block_size =
                    MaybeAuto::from_style(child_fragment.style().content_block_size(),
                                          Au::new(0)).specified_or_zero();
                max_y = max(max_y,
                            child_specified_block_size +
                                child_fragment.border_padding.block_start_end());
            }
            let child_node = flow::mut_base(kid);
            child_node.position.start.b = cur_y;
            max_y = max(max_y, child_node.position.size.block);
        }

        let mut block_size = max_y;
        // TODO: Percentage block-size
        block_size = match MaybeAuto::from_style(self.block_flow
                                                     .fragment
                                                     .style()
                                                     .content_block_size(),
                                                 Au(0)) {
            Auto => block_size,
            Specified(value) => max(value, block_size)
        };
        // cur_y = cur_y + block-size;

        // Assign the block-size of own fragment
        //
        // FIXME(pcwalton): Take `cur_y` into account.
        let mut position = self.block_flow.fragment.border_box;
        position.size.block = block_size;
        self.block_flow.fragment.border_box = position;
        self.block_flow.base.position.size.block = block_size;

        // Assign the block-size of kid fragments, which is the same value as own block-size.
        for kid in self.block_flow.base.child_iter() {
            {
                let kid_fragment = kid.as_table_cell().mut_fragment();
                let mut position = kid_fragment.border_box;
                position.size.block = block_size;
                kid_fragment.border_box = position;
            }
            let child_node = flow::mut_base(kid);
            child_node.position.size.block = block_size;
        }
    }
}

impl Flow for TableRowFlow {
    fn class(&self) -> FlowClass {
        TableRowFlowClass
    }

    fn as_table_row<'a>(&'a mut self) -> &'a mut TableRowFlow {
        self
    }

    fn as_immutable_table_row<'a>(&'a self) -> &'a TableRowFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn column_inline_sizes<'a>(&'a mut self) -> &'a mut Vec<ColumnInlineSize> {
        &mut self.column_inline_sizes
    }

    /// Recursively (bottom-up) determines the context's preferred and minimum inline-sizes. When
    /// called on this context, all child contexts have had their min/pref inline-sizes set. This
    /// function must decide min/pref inline-sizes based on child context inline-sizes and
    /// dimensions of any fragments it is responsible for flowing.
    /// Min/pref inline-sizes set by this function are used in automatic table layout calculation.
    /// The specified column inline-sizes of children cells are used in fixed table layout
    /// calculation.
    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("table_row::bubble_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());

        // Bubble up the specified inline-sizes from child table cells.
        let (mut min_inline_size, mut pref_inline_size) = (Au(0), Au(0));
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_cell());

            // Collect the specified column inline-size of the cell. This is used in both fixed and
            // automatic table layout calculation.
            let child_specified_inline_size = kid.as_table_cell()
                                                 .fragment()
                                                 .style()
                                                 .content_inline_size();

            // Collect minimum and preferred inline-sizes of the cell for automatic table layout
            // calculation.
            let child_base = flow::mut_base(kid);
            let child_column_inline_size = ColumnInlineSize {
                minimum_length: match child_specified_inline_size {
                    LPA_Auto | LPA_Percentage(_) => {
                        child_base.intrinsic_inline_sizes.minimum_inline_size
                    }
                    LPA_Length(length) => length,
                },
                percentage: match child_specified_inline_size {
                    LPA_Auto | LPA_Length(_) => 0.0,
                    LPA_Percentage(percentage) => percentage,
                },
                preferred: child_base.intrinsic_inline_sizes.preferred_inline_size,
                constrained: match child_specified_inline_size {
                    LPA_Length(_) => true,
                    LPA_Auto | LPA_Percentage(_) => false,
                },
            };
            min_inline_size = min_inline_size + child_column_inline_size.minimum_length;
            pref_inline_size = pref_inline_size + child_column_inline_size.preferred;
            self.column_inline_sizes.push(child_column_inline_size);
        }
        self.block_flow.base.intrinsic_inline_sizes.minimum_inline_size = min_inline_size;
        self.block_flow.base.intrinsic_inline_sizes.preferred_inline_size = max(min_inline_size,
                                                                                pref_inline_size);
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, ctx: &LayoutContext) {
        let _scope = layout_debug_scope!("table_row::assign_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table_row");

        // The position was set to the containing block by the flow's parent.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;
        // FIXME: In case of border-collapse: collapse, inline_start_content_edge should be
        // border_inline_start.
        let inline_start_content_edge = Au(0);

        let inline_size_computer = InternalTable;
        inline_size_computer.compute_used_inline_size(&mut self.block_flow,
                                                      ctx,
                                                      containing_block_inline_size);

        self.block_flow
            .propagate_assigned_inline_size_to_children(inline_start_content_edge,
                                                        containing_block_inline_size,
                                                        Some(self.column_inline_sizes.as_slice()));
    }

    fn assign_block_size<'a>(&mut self, ctx: &'a LayoutContext<'a>) {
        debug!("assign_block_size: assigning block_size for table_row");
        self.assign_block_size_table_row_base(ctx);
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, layout_context: &LayoutContext) {
        self.block_flow.build_display_list(layout_context)
    }

    fn repair_style(&mut self, new_style: &Arc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn iterate_through_fragment_bounds(&self, iterator: &mut FragmentBoundsIterator) {
        self.block_flow.iterate_through_fragment_bounds(iterator);
    }
}

impl fmt::Show for TableRowFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableRowFlow: {}", self.block_flow.fragment)
    }
}
