/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_blocks)]

use context::LayoutContext;
use css::node_style::StyledNode;
use flow::{BaseFlow, FlowClass, Flow, ForceNonfloatedFlag};
use fragment::{Fragment, FragmentBorderBoxIterator, SpecificFragmentInfo};
use layout_debug;
use wrapper::ThreadSafeLayoutNode;

use geom::{Point2D, Rect};
use servo_util::geometry::{Au, ZERO_RECT};
use std::cmp::max;
use std::fmt;
use style::computed_values::LengthOrPercentageOrAuto;
use style::ComputedValues;
use std::sync::Arc;

/// A table formatting context.
pub struct TableColGroupFlow {
    /// Data common to all flows.
    pub base: BaseFlow,

    /// The associated fragment.
    pub fragment: Option<Fragment>,

    /// The table column fragments
    pub cols: Vec<Fragment>,

    /// The specified inline-sizes of table columns. (We use `LengthOrPercentageOrAuto` here in
    /// lieu of `ColumnInlineSize` because column groups do not establish minimum or preferred
    /// inline sizes.)
    pub inline_sizes: Vec<LengthOrPercentageOrAuto>,
}

impl TableColGroupFlow {
    pub fn from_node_and_fragments(node: &ThreadSafeLayoutNode,
                                   fragment: Fragment,
                                   fragments: Vec<Fragment>)
                                   -> TableColGroupFlow {
        let writing_mode = node.style().writing_mode;
        TableColGroupFlow {
            base: BaseFlow::new(Some((*node).clone()), writing_mode, ForceNonfloatedFlag::ForceNonfloated),
            fragment: Some(fragment),
            cols: fragments,
            inline_sizes: vec!(),
        }
    }
}

impl Flow for TableColGroupFlow {
    fn class(&self) -> FlowClass {
        FlowClass::TableColGroup
    }

    fn as_table_colgroup<'a>(&'a mut self) -> &'a mut TableColGroupFlow {
        self
    }

    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("table_colgroup::bubble_inline_sizes {:x}",
                                            self.base.debug_id());

        for fragment in self.cols.iter() {
            // Retrieve the specified value from the appropriate CSS property.
            let inline_size = fragment.style().content_inline_size();
            let span: int = match fragment.specific {
                SpecificFragmentInfo::TableColumn(col_fragment) => max(col_fragment.span, 1),
                _ => panic!("non-table-column fragment inside table column?!"),
            };
            for _ in range(0, span) {
                self.inline_sizes.push(inline_size)
            }
        }
    }

    /// Table column inline-sizes are assigned in the table flow and propagated to table row flows
    /// and/or rowgroup flows. Therefore, table colgroup flows do not need to assign inline-sizes.
    fn assign_inline_sizes(&mut self, _: &LayoutContext) {
    }

    /// Table columns do not have block-size.
    fn assign_block_size(&mut self, _: &LayoutContext) {
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, _: Au) {}

    fn update_late_computed_block_position_if_necessary(&mut self, _: Au) {}

    // Table columns are invisible.
    fn build_display_list(&mut self, _: &LayoutContext) {}

    fn repair_style(&mut self, _: &Arc<ComputedValues>) {}

    fn compute_overflow(&self) -> Rect<Au> {
        ZERO_RECT
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             _: &mut FragmentBorderBoxIterator,
                                             _: &Point2D<Au>) {}
}

impl fmt::Show for TableColGroupFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.fragment {
            Some(ref rb) => write!(f, "TableColGroupFlow: {:?}", rb),
            None => write!(f, "TableColGroupFlow"),
        }
    }
}
