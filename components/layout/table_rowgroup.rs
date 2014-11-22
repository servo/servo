/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_blocks)]

use block::BlockFlow;
use block::ISizeAndMarginsComputer;
use construct::FlowConstructor;
use context::LayoutContext;
use flow::{TableRowGroupFlowClass, FlowClass, Flow, ImmutableFlowUtils};
use flow;
use fragment::{Fragment, FragmentBoundsIterator};
use layout_debug;
use model::IntrinsicISizesContribution;
use table::{ColumnInlineSize, InternalTable, TableFlow};
use wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use std::fmt;
use style::ComputedValues;
use sync::Arc;

/// A table formatting context.
#[deriving(Encodable)]
pub struct TableRowGroupFlow {
    pub block_flow: BlockFlow,

    /// Information about the inline-sizes of each column.
    pub column_inline_sizes: Vec<ColumnInlineSize>,
}

impl TableRowGroupFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                  fragment: Fragment)
                                  -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node_and_fragment(node, fragment),
            column_inline_sizes: Vec::new(),
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node(constructor, node),
            column_inline_sizes: Vec::new(),
        }
    }

    pub fn fragment<'a>(&'a mut self) -> &'a Fragment {
        &self.block_flow.fragment
    }

    fn initialize_offsets(&mut self) -> (Au, Au, Au) {
        // TODO: If border-collapse: collapse, block-start_offset, block-end_offset, and
        // inline-start_offset should be updated. Currently, they are set as Au(0).
        (Au(0), Au(0), Au(0))
    }

    /// Assign block-size for table-rowgroup flow.
    ///
    /// FIXME(pcwalton): This doesn't handle floats right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_block_size_table_rowgroup_base<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        let (block_start_offset, _, _) = self.initialize_offsets();

        let mut cur_y = block_start_offset;

        for kid in self.block_flow.base.child_iter() {
            kid.place_float_if_applicable(layout_context);
            if !flow::base(kid).flags.is_float() {
                kid.assign_block_size_for_inorder_child_if_necessary(layout_context);
            }

            let child_node = flow::mut_base(kid);
            child_node.position.start.b = cur_y;
            cur_y = cur_y + child_node.position.size.block;
        }

        let block_size = cur_y - block_start_offset;

        let mut position = self.block_flow.fragment.border_box;
        position.size.block = block_size;
        self.block_flow.fragment.border_box = position;
        self.block_flow.base.position.size.block = block_size;
    }
}

impl Flow for TableRowGroupFlow {
    fn class(&self) -> FlowClass {
        TableRowGroupFlowClass
    }

    fn as_table_rowgroup<'a>(&'a mut self) -> &'a mut TableRowGroupFlow {
        self
    }

    fn as_immutable_table_rowgroup<'a>(&'a self) -> &'a TableRowGroupFlow {
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
    ///
    /// Min/pref inline-sizes set by this function are used in automatic table layout calculation.
    ///
    /// Also, this function finds the specified column inline-sizes from the first row. These are
    /// used in fixed table layout calculation.
    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("table_rowgroup::bubble_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());

        let mut computation = IntrinsicISizesContribution::new();
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_row());

            // Calculate minimum and preferred inline sizes for automatic table layout.
            if self.column_inline_sizes.is_empty() {
                // We're the first row.
                debug_assert!(self.column_inline_sizes.is_empty());
                self.column_inline_sizes = kid.column_inline_sizes().clone();
            } else {
                let mut child_intrinsic_sizes =
                    TableFlow::update_column_inline_sizes(&mut self.column_inline_sizes,
                                                          kid.column_inline_sizes());

                // update the number of column inline-sizes from table-rows.
                let column_count = self.column_inline_sizes.len();
                let child_column_count = kid.column_inline_sizes().len();
                for i in range(column_count, child_column_count) {
                    let this_column_inline_size = (*kid.column_inline_sizes())[i];

                    // FIXME(pcwalton): Ignoring the percentage here seems dubious.
                    child_intrinsic_sizes.minimum_inline_size =
                        child_intrinsic_sizes.minimum_inline_size +
                        this_column_inline_size.minimum_length;
                    child_intrinsic_sizes.preferred_inline_size =
                        child_intrinsic_sizes.preferred_inline_size +
                        this_column_inline_size.preferred;
                    self.column_inline_sizes.push(this_column_inline_size);
                }

                computation.union_block(&child_intrinsic_sizes)
            }
        }

        self.block_flow.base.intrinsic_inline_sizes = computation.finish()
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, ctx: &LayoutContext) {
        let _scope = layout_debug_scope!("table_rowgroup::assign_inline_sizes {:x}",
                                            self.block_flow.base.debug_id());
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table_rowgroup");

        // The position was set to the containing block by the flow's parent.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;
        // FIXME: In case of border-collapse: collapse, inline-start_content_edge should be
        // the border width on the inline-start side.
        let inline_start_content_edge = Au::new(0);
        let content_inline_size = containing_block_inline_size;

        let inline_size_computer = InternalTable;
        inline_size_computer.compute_used_inline_size(&mut self.block_flow,
                                                      ctx,
                                                      containing_block_inline_size);

        self.block_flow.propagate_assigned_inline_size_to_children(
            inline_start_content_edge,
            content_inline_size,
            Some(self.column_inline_sizes.as_slice()));
    }

    fn assign_block_size<'a>(&mut self, ctx: &'a LayoutContext<'a>) {
        debug!("assign_block_size: assigning block_size for table_rowgroup");
        self.assign_block_size_table_rowgroup_base(ctx);
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
        debug!("build_display_list_table_rowgroup: same process as block flow");
        self.block_flow.build_display_list(layout_context)
    }

    fn repair_style(&mut self, new_style: &Arc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn iterate_through_fragment_bounds(&self, iterator: &mut FragmentBoundsIterator) {
        self.block_flow.iterate_through_fragment_bounds(iterator);
    }
}

impl fmt::Show for TableRowGroupFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableRowGroupFlow: {}", self.block_flow.fragment)
    }
}
