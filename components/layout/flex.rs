/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Layout for elements with a CSS `display` property of `flex`.

#![deny(unsafe_code)]

use block::BlockFlow;
use context::LayoutContext;
use display_list_builder::FlexFlowDisplayListBuilding;
use floats::FloatKind;
use flow;
use flow::{Flow, FlowClass, OpaqueFlow};
use flow::{HAS_LEFT_FLOATED_DESCENDANTS, HAS_RIGHT_FLOATED_DESCENDANTS};
use flow::IS_ABSOLUTELY_POSITIONED;
use flow::mut_base;
use fragment::{Fragment, FragmentBorderBoxIterator};
use layout_debug;
use model::{IntrinsicISizes};
use style::computed_values::{flex_direction, float};
use style::properties::style_structs;
use style::values::computed::LengthOrPercentageOrAuto;

use euclid::{Point2D, Rect};
use gfx::display_list::DisplayList;
use std::cmp::max;
use std::sync::Arc;
use style::properties::ComputedValues;
use util::geometry::Au;
use util::logical_geometry::LogicalSize;
use util::opts;

// A mode describes which logical axis a flex axis is parallel with.
// The logical axises are inline and block, the flex axises are main and cross.
// When the flex container has flex-direction: column or flex-direction: column-reverse, the main axis
// should be block. Otherwise, it should be inline.
#[derive(Debug)]
enum Mode {
    Inline,
    Block
}

/// A block with the CSS `display` property equal to `flex`.
#[derive(Debug)]
pub struct FlexFlow {
    /// Data common to all block flows.
    block_flow: BlockFlow,
    /// The logical axis which the main axis will be parallel with.
    /// The cross axis will be parallel with the opposite logical axis.
    main_mode: Mode,
}

fn flex_style(fragment: &Fragment) -> &style_structs::Flex {
    fragment.style.get_flex()
}

// TODO(zentner): This function should use flex-basis.
fn flex_item_inline_sizes(flow: &mut Flow) -> IntrinsicISizes {
    let _scope = layout_debug_scope!("flex::flex_item_inline_sizes");
    debug!("flex_item_inline_sizes");
    let base = flow::mut_base(flow);

    debug!("FlexItem intrinsic inline sizes: {:?}, {:?}",
           base.intrinsic_inline_sizes.minimum_inline_size,
           base.intrinsic_inline_sizes.preferred_inline_size);

    IntrinsicISizes {
        minimum_inline_size: base.intrinsic_inline_sizes.minimum_inline_size,
        preferred_inline_size: base.intrinsic_inline_sizes.preferred_inline_size,
    }
}

impl FlexFlow {
    pub fn from_fragment(fragment: Fragment,
                         flotation: Option<FloatKind>)
                         -> FlexFlow {

        let main_mode = match flex_style(&fragment).flex_direction {
            flex_direction::T::row_reverse    => Mode::Inline,
            flex_direction::T::row            => Mode::Inline,
            flex_direction::T::column_reverse => Mode::Block,
            flex_direction::T::column         => Mode::Block
        };

        let this = FlexFlow {
            block_flow: BlockFlow::from_fragment(fragment, flotation),
            main_mode: main_mode
        };

        this
    }

    // TODO(zentner): This function should use flex-basis.
    // Currently, this is the core of BlockFlow::bubble_inline_sizes() with all float logic
    // stripped out, and max replaced with union_nonbreaking_inline.
    fn inline_mode_bubble_inline_sizes(&mut self) {
        let fixed_width = match self.block_flow.fragment.style().get_box().width {
            LengthOrPercentageOrAuto::Length(_) => true,
            _ => false,
        };

        let mut computation = self.block_flow.fragment.compute_intrinsic_inline_sizes();
        if !fixed_width {
            for kid in self.block_flow.base.child_iter() {
                let is_absolutely_positioned =
                    flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED);
                if !is_absolutely_positioned {
                    computation.union_nonbreaking_inline(&flex_item_inline_sizes(kid));
                }
            }
        }
        self.block_flow.base.intrinsic_inline_sizes = computation.finish();
    }

    // TODO(zentner): This function should use flex-basis.
    // Currently, this is the core of BlockFlow::bubble_inline_sizes() with all float logic
    // stripped out.
    fn block_mode_bubble_inline_sizes(&mut self) {
        let fixed_width = match self.block_flow.fragment.style().get_box().width {
            LengthOrPercentageOrAuto::Length(_) => true,
            _ => false,
        };

        let mut computation = self.block_flow.fragment.compute_intrinsic_inline_sizes();
        if !fixed_width {
            for kid in self.block_flow.base.child_iter() {
                let is_absolutely_positioned =
                    flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED);
                let child_base = flow::mut_base(kid);
                if !is_absolutely_positioned {
                    computation.content_intrinsic_sizes.minimum_inline_size =
                        max(computation.content_intrinsic_sizes.minimum_inline_size,
                            child_base.intrinsic_inline_sizes.minimum_inline_size);

                    computation.content_intrinsic_sizes.preferred_inline_size =
                        max(computation.content_intrinsic_sizes.preferred_inline_size,
                            child_base.intrinsic_inline_sizes.preferred_inline_size);
                }
            }
        }
        self.block_flow.base.intrinsic_inline_sizes = computation.finish();
    }
}

impl Flow for FlexFlow {
    fn class(&self) -> FlowClass {
        FlowClass::Flex
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("flex::bubble_inline_sizes {:x}", self.block_flow.base.debug_id());

        // Flexbox Section 9.0: Generate anonymous flex items:
        // This part was handled in the flow constructor.

        // Flexbox Section 9.1: Re-order the flex items (and any absolutely positioned flex
        // container children) according to their order.
        // TODO(zentner): We need to re-order the items at some point. However, all the operations
        // here ignore order, so we can afford to do it later, if necessary.

        // `flex item`s (our children) cannot be floated. Furthermore, they all establish BFC's.
        // Therefore, we do not have to handle any floats here.

        let mut flags = self.block_flow.base.flags;
        flags.remove(HAS_LEFT_FLOATED_DESCENDANTS);
        flags.remove(HAS_RIGHT_FLOATED_DESCENDANTS);

        match self.main_mode {
            Mode::Inline => self.inline_mode_bubble_inline_sizes(),
            Mode::Block  => self.block_mode_bubble_inline_sizes()
        }

        // Although our children can't be floated, we can.
        match self.block_flow.fragment.style().get_box().float {
            float::T::none => {}
            float::T::left => flags.insert(HAS_LEFT_FLOATED_DESCENDANTS),
            float::T::right => flags.insert(HAS_RIGHT_FLOATED_DESCENDANTS),
        }
        self.block_flow.base.flags = flags
    }

    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        self.block_flow.assign_inline_sizes(layout_context);
    }

    fn assign_block_size<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        self.block_flow.assign_block_size(layout_context);
    }

    fn compute_absolute_position(&mut self, layout_context: &LayoutContext) {
        self.block_flow.compute_absolute_position(layout_context)
    }

    fn place_float_if_applicable<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        self.block_flow.place_float_if_applicable(layout_context)
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, layout_context: &LayoutContext) {
        self.build_display_list_for_flex(Box::new(DisplayList::new()), layout_context);

        if opts::get().validate_display_list_geometry {
            self.block_flow.base.validate_display_list_geometry();
        }
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
        self.block_flow.iterate_through_fragment_border_boxes(iterator, level, stacking_context_position);
    }

    fn mutate_fragments(&mut self, mutator: &mut FnMut(&mut Fragment)) {
        self.block_flow.mutate_fragments(mutator);
    }
}
