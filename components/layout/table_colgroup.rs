/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use std::fmt;

use app_units::Au;
use euclid::default::Point2D;
use style::logical_geometry::LogicalSize;
use style::properties::ComputedValues;
use style::values::computed::Size;

use crate::context::LayoutContext;
use crate::display_list::{DisplayListBuildState, StackingContextCollectionState};
use crate::flow::{BaseFlow, Flow, FlowClass, ForceNonfloatedFlag, OpaqueFlow};
use crate::fragment::{Fragment, FragmentBorderBoxIterator, Overflow};
use crate::{layout_debug, layout_debug_scope};

#[allow(unsafe_code)]
unsafe impl crate::flow::HasBaseFlow for TableColGroupFlow {}

/// A table formatting context.
#[repr(C)]
pub struct TableColGroupFlow {
    /// Data common to all flows.
    pub base: BaseFlow,

    /// The associated fragment.
    pub fragment: Option<Fragment>,

    /// The table column fragments
    pub cols: Vec<Fragment>,

    /// The specified inline-sizes of table columns. (We use `LengthPercentageOrAuto` here in
    /// lieu of `ColumnInlineSize` because column groups do not establish minimum or preferred
    /// inline sizes.)
    pub inline_sizes: Vec<Size>,
}

impl TableColGroupFlow {
    pub fn from_fragments(fragment: Fragment, fragments: Vec<Fragment>) -> TableColGroupFlow {
        let writing_mode = fragment.style().writing_mode;
        TableColGroupFlow {
            base: BaseFlow::new(
                Some(fragment.style()),
                writing_mode,
                ForceNonfloatedFlag::ForceNonfloated,
            ),
            fragment: Some(fragment),
            cols: fragments,
            inline_sizes: vec![],
        }
    }
}

impl Flow for TableColGroupFlow {
    fn class(&self) -> FlowClass {
        FlowClass::TableColGroup
    }

    fn as_mut_table_colgroup(&mut self) -> &mut TableColGroupFlow {
        self
    }

    fn as_table_colgroup(&self) -> &TableColGroupFlow {
        self
    }

    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!(
            "table_colgroup::bubble_inline_sizes {:x}",
            self.base.debug_id()
        );

        for fragment in &self.cols {
            // Retrieve the specified value from the appropriate CSS property.
            let inline_size = fragment.style().content_inline_size();
            for _ in 0..fragment.column_span() {
                self.inline_sizes.push(inline_size.clone())
            }
        }
    }

    /// Table column inline-sizes are assigned in the table flow and propagated to table row flows
    /// and/or rowgroup flows. Therefore, table colgroup flows do not need to assign inline-sizes.
    fn assign_inline_sizes(&mut self, _: &LayoutContext) {}

    /// Table columns do not have block-size.
    fn assign_block_size(&mut self, _: &LayoutContext) {}

    fn update_late_computed_inline_position_if_necessary(&mut self, _: Au) {}

    fn update_late_computed_block_position_if_necessary(&mut self, _: Au) {}

    // Table columns are invisible.
    fn build_display_list(&mut self, _: &mut DisplayListBuildState) {}

    fn collect_stacking_contexts(&mut self, state: &mut StackingContextCollectionState) {
        self.base.stacking_context_id = state.current_stacking_context_id;
        self.base.clipping_and_scrolling = Some(state.current_clipping_and_scrolling);
    }

    fn repair_style(&mut self, _: &crate::ServoArc<ComputedValues>) {}

    fn compute_overflow(&self) -> Overflow {
        Overflow::new()
    }

    fn generated_containing_block_size(&self, _: OpaqueFlow) -> LogicalSize<Au> {
        panic!("Table column groups can't be containing blocks!")
    }

    fn iterate_through_fragment_border_boxes(
        &self,
        _: &mut dyn FragmentBorderBoxIterator,
        _: i32,
        _: &Point2D<Au>,
    ) {
    }

    fn mutate_fragments(&mut self, _: &mut dyn FnMut(&mut Fragment)) {}
}

impl fmt::Debug for TableColGroupFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.fragment {
            Some(ref rb) => write!(f, "TableColGroupFlow: {:?}", rb),
            None => write!(f, "TableColGroupFlow"),
        }
    }
}
