/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_blocks)]

use block::{BlockFlow, ISizeAndMarginsComputer, MarginsMayCollapseFlag};
use construct::FlowConstructor;
use context::LayoutContext;
use flow::{FlowClass, Flow};
use fragment::{Fragment, FragmentBorderBoxIterator};
use layout_debug;
use table::{ColumnComputedInlineSize, ColumnIntrinsicInlineSize, InternalTable};
use wrapper::ThreadSafeLayoutNode;

use geom::{Point2D, Rect};
use servo_util::geometry::Au;
use servo_util::logical_geometry::LogicalRect;
use std::fmt;
use style::properties::ComputedValues;
use std::sync::Arc;

/// A table formatting context.
#[derive(RustcEncodable)]
pub struct TableRowGroupFlow {
    /// Fields common to all block flows.
    pub block_flow: BlockFlow,

    /// Information about the intrinsic inline-sizes of each column.
    pub column_intrinsic_inline_sizes: Vec<ColumnIntrinsicInlineSize>,

    /// Information about the actual inline sizes of each column.
    pub column_computed_inline_sizes: Vec<ColumnComputedInlineSize>,
}

impl TableRowGroupFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode, fragment: Fragment)
                                  -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node_and_fragment(node, fragment),
            column_intrinsic_inline_sizes: Vec::new(),
            column_computed_inline_sizes: Vec::new(),
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor, node: &ThreadSafeLayoutNode)
                     -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node(constructor, node),
            column_intrinsic_inline_sizes: Vec::new(),
            column_computed_inline_sizes: Vec::new(),
        }
    }

    pub fn fragment<'a>(&'a mut self) -> &'a Fragment {
        &self.block_flow.fragment
    }

    /// Assign block-size for table-rowgroup flow.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods.
    #[inline(always)]
    fn assign_block_size_table_rowgroup_base<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        self.block_flow.assign_block_size_block_base(layout_context, MarginsMayCollapseFlag::MarginsMayNotCollapse)
    }
}

impl Flow for TableRowGroupFlow {
    fn class(&self) -> FlowClass {
        FlowClass::TableRowGroup
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

    fn column_intrinsic_inline_sizes<'a>(&'a mut self) -> &'a mut Vec<ColumnIntrinsicInlineSize> {
        &mut self.column_intrinsic_inline_sizes
    }

    fn column_computed_inline_sizes<'a>(&'a mut self) -> &'a mut Vec<ColumnComputedInlineSize> {
        &mut self.column_computed_inline_sizes
    }

    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("table_rowgroup::bubble_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());
        // Proper calculation of intrinsic sizes in table layout requires access to the entire
        // table, which we don't have yet. Defer to our parent.
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
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
                                                      layout_context,
                                                      containing_block_inline_size);

        self.block_flow.propagate_assigned_inline_size_to_children(
            layout_context,
            inline_start_content_edge,
            content_inline_size,
            Some(self.column_computed_inline_sizes.as_slice()));
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

    fn compute_overflow(&self) -> Rect<Au> {
        self.block_flow.compute_overflow()
    }

    fn generated_containing_block_rect(&self) -> LogicalRect<Au> {
        self.block_flow.generated_containing_block_rect()
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             stacking_context_position: &Point2D<Au>) {
        self.block_flow.iterate_through_fragment_border_boxes(iterator, stacking_context_position)
    }
}

impl fmt::Debug for TableRowGroupFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableRowGroupFlow: {:?}", self.block_flow.fragment)
    }
}
