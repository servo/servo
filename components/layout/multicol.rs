/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS Multi-column layout http://dev.w3.org/csswg/css-multicol/

#![deny(unsafe_code)]

use block::BlockFlow;
use context::LayoutContext;
use floats::FloatKind;
use flow::{FlowClass, Flow, OpaqueFlow};
use fragment::{Fragment, FragmentBorderBoxIterator};

use euclid::{Point2D, Rect};
use std::fmt;
use std::sync::Arc;
use style::properties::ComputedValues;
use util::geometry::Au;
use util::logical_geometry::LogicalSize;

pub struct MulticolFlow {
    pub block_flow: BlockFlow,
}

impl MulticolFlow {
    pub fn from_fragment(fragment: Fragment, float_kind: Option<FloatKind>) -> MulticolFlow {
        MulticolFlow {
            block_flow: BlockFlow::from_fragment(fragment, float_kind)
        }
    }
}

impl Flow for MulticolFlow {
    fn class(&self) -> FlowClass {
        FlowClass::Multicol
    }

    fn as_mut_multicol<'a>(&'a mut self) -> &'a mut MulticolFlow {
        self
    }

    fn as_mut_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn as_block<'a>(&'a self) -> &'a BlockFlow {
        &self.block_flow
    }

    fn bubble_inline_sizes(&mut self) {
        // FIXME(SimonSapin) http://dev.w3.org/csswg/css-sizing/#multicol-intrinsic
        self.block_flow.bubble_inline_sizes();
    }

    fn assign_inline_sizes(&mut self, ctx: &LayoutContext) {
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "multicol");
        self.block_flow.assign_inline_sizes(ctx);
    }

    fn assign_block_size<'a>(&mut self, ctx: &'a LayoutContext<'a>) {
        debug!("assign_block_size: assigning block_size for multicol");
        self.block_flow.assign_block_size(ctx);
    }

    fn compute_absolute_position(&mut self, layout_context: &LayoutContext) {
        self.block_flow.compute_absolute_position(layout_context)
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, layout_context: &LayoutContext) {
        debug!("build_display_list_multicol: same process as block flow");
        self.block_flow.build_display_list(layout_context)
    }

    fn repair_style(&mut self, new_style: &Arc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Rect<Au> {
        self.block_flow.compute_overflow()
    }

    fn generated_containing_block_size(&self, flow: OpaqueFlow) -> LogicalSize<Au> {
        self.block_flow.generated_containing_block_size(flow)
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             level: i32,
                                             stacking_context_position: &Point2D<Au>) {
        self.block_flow.iterate_through_fragment_border_boxes(iterator, level, stacking_context_position)
    }

    fn mutate_fragments(&mut self, mutator: &mut FnMut(&mut Fragment)) {
        self.block_flow.mutate_fragments(mutator)
    }
}

impl fmt::Debug for MulticolFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MulticolFlow: {:?}", self.block_flow)
    }
}
