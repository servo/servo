/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Layout for elements with a CSS `display` property of `flex`.

#![deny(unsafe_code)]

use app_units::Au;
use block::BlockFlow;
use context::LayoutContext;
use display_list_builder::FlexFlowDisplayListBuilding;
use euclid::{Point2D, Rect};
use floats::FloatKind;
use flow;
use flow::IS_ABSOLUTELY_POSITIONED;
use flow::ImmutableFlowUtils;
use flow::mut_base;
use flow::{Flow, FlowClass, OpaqueFlow};
use flow::{HAS_LEFT_FLOATED_DESCENDANTS, HAS_RIGHT_FLOATED_DESCENDANTS};
use flow_list::{MutFlowListIterator};
use fragment::{Fragment, FragmentBorderBoxIterator};
use gfx::display_list::DisplayList;
use incremental::{REFLOW, REFLOW_OUT_OF_FLOW};
use layout_debug;
use model::MaybeAuto;
use model::{IntrinsicISizes};
use std::cmp::{max, Ordering};
use std::sync::Arc;
use std::vec::IntoIter;
use style::computed_values::{flex_direction, float};
use style::properties::ComputedValues;
use style::properties::style_structs;
use style::values::computed::LengthOrPercentageOrAuto;
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
    /// Is the container reversed? In other words, is the flex direction `row-reverse` or
    /// `column-reverse`?
    reversed: bool,
    /// Information about each child of this flexbox. This Vec is in the same order as the child
    /// flows.
    items: Vec<FlexItem>,
}

/// Holds information about a child flow. Used to avoid repeated recalculations of flexbox
/// algorithm intermediate values like e.g. `hypothetical main size`.
#[derive(Debug)]
struct FlexItem;

impl FlexItem {
    fn new() -> Self {
        FlexItem
    }
}

type FlowFlexItemIterMut<'a> = IntoIter<(&'a mut Flow, &'a mut FlexItem)>;

fn compare_flows_by_order(a: &Flow, b: &Flow) -> Ordering {
    flex_item_style(flow_style(a)).order.cmp(&flex_item_style(flow_style(b)).order)
}

fn flow_flex_item_iter_mut<'a>(flows: MutFlowListIterator<'a>, items: &'a mut Vec<FlexItem>,
                               reverse: bool) -> FlowFlexItemIterMut<'a> {
    let mut pairs : Vec<(&'a mut Flow, &'a mut FlexItem)> = flows.zip(items.iter_mut()).collect();

    // Flexbox § 9.1: Re-order the flex items (and any absolutely positioned flex
    // container children) according to their order.
    // We implement this by sorting the flex items by the child's `order` property.
    pairs.sort_by(|a, b| compare_flows_by_order(a.0, b.0));
    if reverse {
        pairs.reverse();
    }
    pairs.into_iter()
}

fn flow_style<'a>(child: &'a Flow) -> &'a ComputedValues {
    &child.as_block().fragment.style
}


// Returns the style struct for the properties of a flex container's fragment.
fn flex_style(fragment: &Fragment) -> &style_structs::Flex {
    fragment.style.get_flex()
}

// Returns the style struct for the properties of a flex items's fragment.
fn flex_item_style(style: &ComputedValues) -> &style_structs::Flex {
    style.get_flex()
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

        let reversed = match flex_style(&fragment).flex_direction {
            flex_direction::T::row_reverse | flex_direction::T::column_reverse => true,
            flex_direction::T::row         | flex_direction::T::column         => false
        };

        FlexFlow {
            block_flow: BlockFlow::from_fragment(fragment, flotation),
            main_mode: main_mode,
            reversed: reversed,
            items: Vec::new()
        }
    }

    // Iterate through children in `display order`. This applies the `order` property, as well as
    // reversing the children if necessary.
    fn item_iter_mut<'a>(&'a mut self) -> FlowFlexItemIterMut<'a> {
        flow_flex_item_iter_mut(self.block_flow.base.child_iter(), &mut self.items,
                                self.reversed)
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
            for (kid, _) in self.item_iter_mut() {
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
            for (kid, _) in self.item_iter_mut() {
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

    // TODO(zentner): This function needs to be radically different for multi-line flexbox.
    // Currently, this is the core of BlockFlow::propagate_assigned_inline_size_to_children() with
    // all float and table logic stripped out.
    fn block_mode_assign_inline_sizes(&mut self,
                                      _layout_context: &LayoutContext,
                                      inline_start_content_edge: Au,
                                      _inline_end_content_edge: Au,
                                      content_inline_size: Au) {
        let _scope = layout_debug_scope!("flex::block_mode_assign_inline_sizes");
        debug!("block_mode_assign_inline_sizes");

        let containing_block_mode = self.block_flow.base.writing_mode;

        let block_container_explicit_block_size = self.block_flow.base.block_container_explicit_block_size;
        for (kid, _) in self.item_iter_mut() {
            // The inline-start margin edge of the child flow is at our inline-start content edge,
            // and its inline-size is our content inline-size.
            {
                let kid_base = flow::mut_base(kid);
                kid_base.block_container_inline_size = content_inline_size;
                kid_base.block_container_writing_mode = containing_block_mode;
                kid_base.block_container_explicit_block_size = block_container_explicit_block_size;
                kid_base.position.start.i = inline_start_content_edge;
            }
        }
    }

    // TODO(zentner): This function should actually flex elements!
    // Currently, this is the core of InlineFlow::propagate_assigned_inline_size_to_children() with
    // fragment logic stripped out.
    fn inline_mode_assign_inline_sizes(&mut self,
                                       _layout_context: &LayoutContext,
                                       inline_start_content_edge: Au,
                                       _inline_end_content_edge: Au,
                                       content_inline_size: Au) {
        let _scope = layout_debug_scope!("flex::inline_mode_assign_inline_sizes");
        debug!("inline_mode_assign_inline_sizes");

        debug!("content_inline_size = {:?}", content_inline_size);

        let child_count = ImmutableFlowUtils::child_count(self as &Flow) as i32;
        debug!("child_count = {:?}", child_count);
        if child_count == 0 {
            return;
        }

        let even_content_inline_size = content_inline_size / child_count;

        let inline_size = self.block_flow.base.block_container_inline_size;
        let container_mode = self.block_flow.base.block_container_writing_mode;
        self.block_flow.base.position.size.inline = inline_size;

        let block_container_explicit_block_size = self.block_flow.base.block_container_explicit_block_size;
        let mut inline_child_start = inline_start_content_edge;
        for (kid, _) in self.item_iter_mut() {
            let kid_base = flow::mut_base(kid);

            kid_base.block_container_inline_size = even_content_inline_size;
            kid_base.block_container_writing_mode = container_mode;
            kid_base.block_container_explicit_block_size = block_container_explicit_block_size;
            kid_base.position.start.i = inline_child_start;
            inline_child_start = inline_child_start + even_content_inline_size;
        }
    }

    // TODO(zentner): This function should actually flex elements!
    fn block_mode_assign_block_size<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        let _scope = layout_debug_scope!("flex::block_mode_assign_block_size");

        let mut cur_b = self.block_flow.fragment.border_padding.block_start;

        for (kid, _) in self.item_iter_mut() {
            let kid_base = flow::mut_base(kid);
            kid_base.position.start.b = cur_b;
            cur_b = cur_b + kid_base.position.size.block;
        }

        self.block_flow.assign_block_size(layout_context)
    }

    // TODO(zentner): This function should actually flex elements!
    // Currently, this is the core of TableRowFlow::assign_block_size() with
    // float related logic stripped out.
    fn inline_mode_assign_block_size<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        let _scope = layout_debug_scope!("flex::inline_mode_assign_block_size");

        let mut max_block_size = Au(0);
        let thread_id = self.block_flow.base.thread_id;
        for (kid, _) in self.item_iter_mut() {
            kid.assign_block_size_for_inorder_child_if_necessary(layout_context, thread_id);

            {
                let child_fragment = &mut kid.as_mut_block().fragment;
                // TODO: Percentage block-size
                let child_specified_block_size =
                    MaybeAuto::from_style(child_fragment.style().content_block_size(),
                                          Au(0)).specified_or_zero();
                max_block_size =
                    max(max_block_size,
                        child_specified_block_size +
                        child_fragment.border_padding.block_start_end());
            }
            let child_node = flow::mut_base(kid);
            child_node.position.start.b = Au(0);
            max_block_size = max(max_block_size, child_node.position.size.block);
        }

        let mut block_size = max_block_size;
        // TODO: Percentage block-size

        block_size = match MaybeAuto::from_style(self.block_flow
                                                     .fragment
                                                     .style()
                                                     .content_block_size(),
                                                 Au(0)) {
            MaybeAuto::Auto => block_size,
            MaybeAuto::Specified(value) => max(value, block_size),
        };

        // Assign the block-size of own fragment
        let mut position = self.block_flow.fragment.border_box;
        position.size.block = block_size;
        self.block_flow.fragment.border_box = position;
        self.block_flow.base.position.size.block = block_size;

        // Assign the block-size of kid fragments, which is the same value as own block-size.
        for (kid, _) in self.item_iter_mut() {
            {
                let kid_fragment = &mut kid.as_mut_block().fragment;
                let mut position = kid_fragment.border_box;
                position.size.block = block_size;
                kid_fragment.border_box = position;
            }

            // Assign the child's block size.
            flow::mut_base(kid).position.size.block = block_size
        }
    }
}

impl Flow for FlexFlow {
    fn class(&self) -> FlowClass {
        FlowClass::Flex
    }

    fn as_mut_block(&mut self) -> &mut BlockFlow {
        &mut self.block_flow
    }

    fn mark_as_root(&mut self) {
        self.block_flow.mark_as_root();
    }

    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("flex::bubble_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());

        // Flexbox § 9.0: Generate anonymous flex items:
        // This part was handled in the flow constructor.

        // Allocate the flex items. It's efficient to do this here, since we know the number of
        // children. After this point, `self.item_iter_mut()` should be used to iterate through
        // child flows, in order to conform to Flexbox § 9.1 (the `order` property).
        let child_count = ImmutableFlowUtils::child_count(self as &Flow);
        self.items.reserve (child_count);
        for _ in self.block_flow.base.child_iter() {
            self.items.push(FlexItem::new());
        }

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
        let _scope = layout_debug_scope!("flex::assign_inline_sizes {:x}", self.block_flow.base.debug_id());
        debug!("assign_inline_sizes");

        if !self.block_flow.base.restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW) {
            return
        }

        // Our inline-size was set to the inline-size of the containing block by the flow's parent.
        // Now compute the real value.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;
        self.block_flow.compute_used_inline_size(layout_context, containing_block_inline_size);
        if self.block_flow.base.flags.is_float() {
            self.block_flow.float.as_mut().unwrap().containing_inline_size = containing_block_inline_size
        }

        // Move in from the inline-start border edge.
        let inline_start_content_edge = self.block_flow.fragment.border_box.start.i +
            self.block_flow.fragment.border_padding.inline_start;

        debug!("inline_start_content_edge = {:?}", inline_start_content_edge);

        let padding_and_borders = self.block_flow.fragment.border_padding.inline_start_end();

        // Distance from the inline-end margin edge to the inline-end content edge.
        let inline_end_content_edge =
            self.block_flow.fragment.margin.inline_end +
            self.block_flow.fragment.border_padding.inline_end;

        debug!("padding_and_borders = {:?}", padding_and_borders);
        debug!("self.block_flow.fragment.border_box.size.inline = {:?}",
               self.block_flow.fragment.border_box.size.inline);
        let content_inline_size = self.block_flow.fragment.border_box.size.inline - padding_and_borders;

        match self.main_mode {
            Mode::Inline =>
                self.inline_mode_assign_inline_sizes(layout_context,
                                                     inline_start_content_edge,
                                                     inline_end_content_edge,
                                                     content_inline_size),
            Mode::Block  =>
                self.block_mode_assign_inline_sizes(layout_context,
                                                    inline_start_content_edge,
                                                    inline_end_content_edge,
                                                    content_inline_size)
        }
    }

    fn assign_block_size<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        self.block_flow.assign_block_size(layout_context);
        match self.main_mode {
            Mode::Inline =>
                self.inline_mode_assign_block_size(layout_context),
            Mode::Block  =>
                self.block_mode_assign_block_size(layout_context)
        }
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
