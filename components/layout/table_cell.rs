/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_blocks)]

use block::{BlockFlow, ISizeAndMarginsComputer, MarginsMayCollapseFlag};
use context::LayoutContext;
use flow::{Flow, FlowClass};
use fragment::{Fragment, FragmentBorderBoxIterator};
use model::{MaybeAuto};
use layout_debug;
use table::InternalTable;
use wrapper::ThreadSafeLayoutNode;

use geom::{Point2D, Rect};
use servo_util::geometry::Au;
use servo_util::logical_geometry::LogicalRect;
use std::fmt;
use style::properties::ComputedValues;
use style::legacy::UnsignedIntegerAttribute;
use std::sync::Arc;

/// A table formatting context.
#[derive(RustcEncodable)]
pub struct TableCellFlow {
    /// Data common to all block flows.
    pub block_flow: BlockFlow,
    /// The column span of this cell.
    pub column_span: u32,
    /// Whether this cell is visible. If false, the value of `empty-cells` means that we must not
    /// display this cell.
    pub visible: bool,
}

impl TableCellFlow {
    pub fn from_node_fragment_and_visibility_flag(node: &ThreadSafeLayoutNode,
                                                  fragment: Fragment,
                                                  visible: bool)
                                                  -> TableCellFlow {
        TableCellFlow {
            block_flow: BlockFlow::from_node_and_fragment(node, fragment),
            column_span: node.get_unsigned_integer_attribute(UnsignedIntegerAttribute::ColSpan)
                             .unwrap_or(1),
            visible: visible,
        }
    }

    pub fn fragment<'a>(&'a mut self) -> &'a Fragment {
        &self.block_flow.fragment
    }

    pub fn mut_fragment<'a>(&'a mut self) -> &'a mut Fragment {
        &mut self.block_flow.fragment
    }

    /// Assign block-size for table-cell flow.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods.
    #[inline(always)]
    fn assign_block_size_table_cell_base<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        self.block_flow.assign_block_size_block_base(
            layout_context,
            MarginsMayCollapseFlag::MarginsMayNotCollapse)
    }
}

impl Flow for TableCellFlow {
    fn class(&self) -> FlowClass {
        FlowClass::TableCell
    }

    fn as_table_cell<'a>(&'a mut self) -> &'a mut TableCellFlow {
        self
    }

    fn as_immutable_table_cell<'a>(&'a self) -> &'a TableCellFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    /// Minimum/preferred inline-sizes set by this function are used in automatic table layout
    /// calculation.
    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("table_cell::bubble_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());

        self.block_flow.bubble_inline_sizes();
        let specified_inline_size = MaybeAuto::from_style(self.block_flow
                                                              .fragment
                                                              .style()
                                                              .content_inline_size(),
                                                          Au(0)).specified_or_zero();
        if self.block_flow.base.intrinsic_inline_sizes.minimum_inline_size <
                specified_inline_size {
            self.block_flow.base.intrinsic_inline_sizes.minimum_inline_size = specified_inline_size
        }
        if self.block_flow.base.intrinsic_inline_sizes.preferred_inline_size <
                self.block_flow.base.intrinsic_inline_sizes.minimum_inline_size {
            self.block_flow.base.intrinsic_inline_sizes.preferred_inline_size =
                self.block_flow.base.intrinsic_inline_sizes.minimum_inline_size;
        }
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent table
    /// row.
    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("table_cell::assign_inline_sizes {:x}",
                                            self.block_flow.base.debug_id());
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table_cell");

        // The position was set to the column inline-size by the parent flow, table row flow.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;

        let inline_size_computer = InternalTable;

        inline_size_computer.compute_used_inline_size(&mut self.block_flow,
                                                      layout_context,
                                                      containing_block_inline_size);

        let inline_start_content_edge =
            self.block_flow.fragment.border_box.start.i +
            self.block_flow.fragment.border_padding.inline_start;
        let padding_and_borders = self.block_flow.fragment.border_padding.inline_start_end();
        let content_inline_size =
            self.block_flow.fragment.border_box.size.inline - padding_and_borders;

        self.block_flow.propagate_assigned_inline_size_to_children(layout_context,
                                                                   inline_start_content_edge,
                                                                   content_inline_size,
                                                                   None);
    }

    fn assign_block_size<'a>(&mut self, ctx: &'a LayoutContext<'a>) {
        debug!("assign_block_size: assigning block_size for table_cell");
        self.assign_block_size_table_cell_base(ctx);
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
        if self.visible {
            self.block_flow.build_display_list(layout_context)
        }
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

impl fmt::Debug for TableCellFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableCellFlow: {:?}", self.block_flow)
    }
}
