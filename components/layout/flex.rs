/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Layout for elements with a CSS `display` property of `flex`.

#![deny(unsafe_code)]

use app_units::{Au, MAX_AU};
use block::BlockFlow;
use context::LayoutContext;
use display_list_builder::{DisplayListBuildState, FlexFlowDisplayListBuilding};
use euclid::Point2D;
use floats::FloatKind;
use flow;
use flow::{Flow, FlowClass, ImmutableFlowUtils, OpaqueFlow};
use flow::{INLINE_POSITION_IS_STATIC, IS_ABSOLUTELY_POSITIONED};
use flow_ref::{self, FlowRef};
use fragment::{Fragment, FragmentBorderBoxIterator, Overflow};
use gfx::display_list::StackingContext;
use gfx_traits::StackingContextId;
use layout_debug;
use model::{IntrinsicISizes, MaybeAuto, MinMaxConstraint};
use model::{specified, specified_or_none};
use script_layout_interface::restyle_damage::{REFLOW, REFLOW_OUT_OF_FLOW};
use std::cmp::{max, min};
use std::sync::Arc;
use style::computed_values::flex_direction;
use style::computed_values::{box_sizing, border_collapse};
use style::logical_geometry::LogicalSize;
use style::properties::{ComputedValues, ServoComputedValues};
use style::servo::SharedStyleContext;
use style::values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto};
use style::values::computed::{LengthOrPercentageOrAutoOrContent, LengthOrPercentageOrNone};


/// The size of an axis. May be a specified size, a min/max
/// constraint, or an unlimited size
#[derive(Debug)]
enum AxisSize {
    Definite(Au),
    MinMax(MinMaxConstraint),
    Infinite,
}

impl AxisSize {
    /// Generate a new available cross or main axis size from the specified size of the container,
    /// containing block size, min constraint, and max constraint
    pub fn new(size: LengthOrPercentageOrAuto, content_size: Option<Au>, min: LengthOrPercentage,
               max: LengthOrPercentageOrNone) -> AxisSize {
        match size {
            LengthOrPercentageOrAuto::Length(length) => AxisSize::Definite(length),
            LengthOrPercentageOrAuto::Percentage(percent) => {
                match content_size {
                    Some(size) => AxisSize::Definite(size.scale_by(percent)),
                    None => AxisSize::Infinite
                }
            },
            LengthOrPercentageOrAuto::Calc(calc) => {
                match content_size {
                    Some(size) => AxisSize::Definite(size.scale_by(calc.percentage())),
                    None => AxisSize::Infinite
                }
            },
            LengthOrPercentageOrAuto::Auto => {
                AxisSize::MinMax(MinMaxConstraint::new(content_size, min, max))
            }
        }
    }
}

// A mode describes which logical axis a flex axis is parallel with.
// The logical axises are inline and block, the flex axises are main and cross.
// When the flex container has flex-direction: column or flex-direction: column-reverse, the main axis
// should be block. Otherwise, it should be inline.
#[derive(Debug, Clone, Copy)]
enum Mode {
    Inline,
    Block
}

/// This function accepts the flex-basis and the size property in main direction from style,
/// and the container size, then return the used value of flex basis. it can be used to help
/// determining the flex base size and to indicate whether the main size of the item
/// is definite after flex size resolving.
fn from_flex_basis(flex_basis: LengthOrPercentageOrAutoOrContent,
                   main_length: LengthOrPercentageOrAuto,
                   containing_length: Option<Au>) -> MaybeAuto {
    match (flex_basis, containing_length) {
        (LengthOrPercentageOrAutoOrContent::Length(length), _) =>
            MaybeAuto::Specified(length),
        (LengthOrPercentageOrAutoOrContent::Percentage(percent), Some(size)) =>
            MaybeAuto::Specified(size.scale_by(percent)),
        (LengthOrPercentageOrAutoOrContent::Percentage(_), None) =>
            MaybeAuto::Auto,
        (LengthOrPercentageOrAutoOrContent::Calc(calc), Some(size)) =>
            MaybeAuto::Specified(calc.length() + size.scale_by(calc.percentage())),
        (LengthOrPercentageOrAutoOrContent::Calc(_), None) =>
            MaybeAuto::Auto,
        (LengthOrPercentageOrAutoOrContent::Content, _) =>
            MaybeAuto::Auto,
        (LengthOrPercentageOrAutoOrContent::Auto, Some(size)) =>
            MaybeAuto::from_style(main_length, size),
        (LengthOrPercentageOrAutoOrContent::Auto, None) => {
            if let LengthOrPercentageOrAuto::Length(length) = main_length {
                MaybeAuto::Specified(length)
            } else {
                MaybeAuto::Auto
            }
        }
    }
}

/// Represents a child in a flex container. Most fields here are used in
/// flex size resolving, and items are sorted by the 'order' property.
#[derive(Debug)]
struct FlexItem {
    /// Main size of a flex item, used to store results of flexible length calcuation.
    pub main_size: Au,
    /// Used flex base size.
    pub base_size: Au,
    /// The minimal size in main direction.
    pub min_size: Au,
    /// The maximal main size. If this property is not actually set by style
    /// It will be the largest size available for code reuse.
    pub max_size: Au,
    /// Reference to the actual flow.
    pub flow: FlowRef,
    /// Style of the child flow, stored here to reduce overhead.
    pub style: Arc<ServoComputedValues>,
    /// The 'flex-grow' property of this item.
    pub flex_grow: f32,
    /// The 'flex-shrink' property of this item.
    pub flex_shrink: f32,
    /// The 'order' property of this item.
    pub order: i32,
    /// Whether the main size has met its constraint.
    pub is_frozen: bool,
    /// True if this flow has property 'visibility::collapse'.
    pub is_strut: bool
}

impl FlexItem {
    pub fn new(flow: FlowRef) -> FlexItem {
        let style = flow.as_block().fragment.style.clone();
        let flex_grow = style.get_position().flex_grow;
        let flex_shrink = style.get_position().flex_shrink;
        let order = style.get_position().order;
        // TODO(stshine): for item with visibility:collapse, set is_strut to true.

        FlexItem {
            main_size: Au(0),
            base_size: Au(0),
            min_size: Au(0),
            max_size: MAX_AU,
            flow: flow,
            style: style,
            flex_grow: flex_grow,
            flex_shrink: flex_shrink,
            order: order,
            is_frozen: false,
            is_strut: false
        }
    }

    /// Initialize the used flex base size, minimal main size and maximal main size.
    /// For block mode container this method should be called in assign_block_size()
    /// pass so that the item has already been layouted.
    pub fn init_sizes(&mut self, containing_length: Au, mode: Mode) {
        let block = flow_ref::deref_mut(&mut self.flow).as_mut_block();
        match mode {
            // TODO(stshine): the definition of min-{width, height} in style component
            // should change to LengthOrPercentageOrAuto for automatic implied minimal size.
            // https://drafts.csswg.org/css-flexbox-1/#min-size-auto
            Mode::Inline => {
                let basis = from_flex_basis(self.style.get_position().flex_basis,
                                            self.style.content_inline_size(),
                                            Some(containing_length));

                // These methods compute auto margins to zero length, which is exactly what we want.
                block.fragment.compute_border_and_padding(containing_length,
                                                          border_collapse::T::separate);
                block.fragment.compute_inline_direction_margins(containing_length);
                block.fragment.compute_block_direction_margins(containing_length);

                let adjustment = match self.style.get_position().box_sizing {
                    box_sizing::T::content_box => Au(0),
                    box_sizing::T::border_box =>
                        block.fragment.border_padding.inline_start_end()
                };
                let content_size = block.base.intrinsic_inline_sizes.preferred_inline_size
                    - block.fragment.surrounding_intrinsic_inline_size() + adjustment;
                self.base_size = basis.specified_or_default(content_size);
                self.max_size = specified_or_none(self.style.max_inline_size(), containing_length)
                    .unwrap_or(MAX_AU);
                self.min_size = specified(self.style.min_inline_size(), containing_length);
            },
            Mode::Block => {
                let basis = from_flex_basis(self.style.get_position().flex_basis,
                                            self.style.content_block_size(),
                                            Some(containing_length));
                let content_size = match self.style.get_position().box_sizing {
                    box_sizing::T::border_box => block.fragment.border_box.size.block,
                    box_sizing::T::content_box => block.fragment.border_box.size.block
                        - block.fragment.border_padding.block_start_end(),
                };
                self.base_size = basis.specified_or_default(content_size);
                self.max_size = specified_or_none(self.style.max_block_size(), containing_length)
                    .unwrap_or(MAX_AU);
                self.min_size = specified(self.style.min_block_size(), containing_length);
            }
        }
    }

    /// Return the outer main size of the item, including paddings and margins,
    /// clamped by max and min size.
    pub fn outer_main_size(&self, mode: Mode) -> Au {
        let ref fragment = self.flow.as_block().fragment;
        let adjustment = match mode {
            Mode::Inline => {
                match self.style.get_position().box_sizing {
                    box_sizing::T::content_box =>
                        fragment.border_padding.inline_start_end() + fragment.margin.inline_start_end(),
                    box_sizing::T::border_box =>
                        fragment.margin.inline_start_end()
                }
            },
            Mode::Block => {
                match self.style.get_position().box_sizing {
                    box_sizing::T::content_box =>
                        fragment.border_padding.block_start_end() + fragment.margin.block_start_end(),
                    box_sizing::T::border_box =>
                        fragment.margin.block_start_end()
                }
            }
        };
        max(self.min_size, min(self.base_size, self.max_size)) + adjustment
    }

    pub fn auto_margin_num(&self, mode: Mode) -> i32 {
        let margin = self.style.logical_margin();
        let mut margin_count = 0;
        match mode {
            Mode::Inline => {
                if margin.inline_start == LengthOrPercentageOrAuto::Auto {
                    margin_count += 1;
                }
                if margin.inline_end == LengthOrPercentageOrAuto::Auto {
                    margin_count += 1;
                }
            }
            Mode::Block => {
                if margin.block_start == LengthOrPercentageOrAuto::Auto {
                    margin_count += 1;
                }
                if margin.block_end == LengthOrPercentageOrAuto::Auto {
                    margin_count += 1;
                }
            }
        }
        margin_count
    }
}

/// A block with the CSS `display` property equal to `flex`.
#[derive(Debug)]
pub struct FlexFlow {
    /// Data common to all block flows.
    block_flow: BlockFlow,
    /// The logical axis which the main axis will be parallel with.
    /// The cross axis will be parallel with the opposite logical axis.
    main_mode: Mode,
    /// The available main axis size
    available_main_size: AxisSize,
    /// The available cross axis size
    available_cross_size: AxisSize,
    /// List of flex-items that belong to this flex-container
    items: Vec<FlexItem>,
    /// True if the flex-direction is *-reversed
    is_reverse: bool
}

impl FlexFlow {
    pub fn from_fragment(fragment: Fragment,
                         flotation: Option<FloatKind>)
                         -> FlexFlow {
        let (main_mode, is_reverse) = match fragment.style.get_position().flex_direction {
            flex_direction::T::row            => (Mode::Inline, false),
            flex_direction::T::row_reverse    => (Mode::Inline, true),
            flex_direction::T::column         => (Mode::Block, false),
            flex_direction::T::column_reverse => (Mode::Block, true),
        };

        FlexFlow {
            block_flow: BlockFlow::from_fragment(fragment, flotation),
            main_mode: main_mode,
            available_main_size: AxisSize::Infinite,
            available_cross_size: AxisSize::Infinite,
            items: Vec::new(),
            is_reverse: is_reverse
        }
    }

    // TODO(zentner): This function should use flex-basis.
    // Currently, this is the core of BlockFlow::bubble_inline_sizes() with all float logic
    // stripped out, and max replaced with union_nonbreaking_inline.
    fn inline_mode_bubble_inline_sizes(&mut self) {
        let fixed_width = match self.block_flow.fragment.style().get_position().width {
            LengthOrPercentageOrAuto::Length(_) => true,
            _ => false,
        };

        let mut computation = self.block_flow.fragment.compute_intrinsic_inline_sizes();
        if !fixed_width {
            for kid in &mut self.items {
                let base = flow::mut_base(flow_ref::deref_mut(&mut kid.flow));
                let is_absolutely_positioned = base.flags.contains(IS_ABSOLUTELY_POSITIONED);
                if !is_absolutely_positioned {
                    let flex_item_inline_sizes = IntrinsicISizes {
                        minimum_inline_size: base.intrinsic_inline_sizes.minimum_inline_size,
                        preferred_inline_size: base.intrinsic_inline_sizes.preferred_inline_size,
                    };
                    computation.union_nonbreaking_inline(&flex_item_inline_sizes);
                }
            }
        }
        self.block_flow.base.intrinsic_inline_sizes = computation.finish();
    }

    // TODO(zentner): This function should use flex-basis.
    // Currently, this is the core of BlockFlow::bubble_inline_sizes() with all float logic
    // stripped out.
    fn block_mode_bubble_inline_sizes(&mut self) {
        let fixed_width = match self.block_flow.fragment.style().get_position().width {
            LengthOrPercentageOrAuto::Length(_) => true,
            _ => false,
        };

        let mut computation = self.block_flow.fragment.compute_intrinsic_inline_sizes();
        if !fixed_width {
            for kid in &mut self.items {
                let base = flow::mut_base(flow_ref::deref_mut(&mut kid.flow));
                let is_absolutely_positioned = base.flags.contains(IS_ABSOLUTELY_POSITIONED);
                if !is_absolutely_positioned {
                    computation.content_intrinsic_sizes.minimum_inline_size =
                        max(computation.content_intrinsic_sizes.minimum_inline_size,
                            base.intrinsic_inline_sizes.minimum_inline_size);

                    computation.content_intrinsic_sizes.preferred_inline_size =
                        max(computation.content_intrinsic_sizes.preferred_inline_size,
                            base.intrinsic_inline_sizes.preferred_inline_size);
                }
            }
        }
        self.block_flow.base.intrinsic_inline_sizes = computation.finish();
    }

    // TODO(zentner): This function needs to be radically different for multi-line flexbox.
    // Currently, this is the core of BlockFlow::propagate_assigned_inline_size_to_children() with
    // all float and table logic stripped out.
    fn block_mode_assign_inline_sizes(&mut self,
                                      _shared_context: &SharedStyleContext,
                                      inline_start_content_edge: Au,
                                      inline_end_content_edge: Au,
                                      content_inline_size: Au) {
        let _scope = layout_debug_scope!("flex::block_mode_assign_inline_sizes");
        debug!("block_mode_assign_inline_sizes");

        // FIXME (mbrubeck): Get correct mode for absolute containing block
        let containing_block_mode = self.block_flow.base.writing_mode;

        let container_block_size = match self.available_main_size {
            AxisSize::Definite(length) => Some(length),
            _ => None
        };
        let container_inline_size = match self.available_cross_size {
            AxisSize::Definite(length) => length,
            AxisSize::MinMax(ref constraint) => constraint.clamp(content_inline_size),
            AxisSize::Infinite => content_inline_size
        };
        for kid in &mut self.items {
            {
                let kid_base = flow::mut_base(flow_ref::deref_mut(&mut kid.flow));
                kid_base.block_container_explicit_block_size = container_block_size;
                if kid_base.flags.contains(INLINE_POSITION_IS_STATIC) {
                    // The inline-start margin edge of the child flow is at our inline-start content edge,
                    // and its inline-size is our content inline-size.
                    kid_base.position.start.i =
                        if kid_base.writing_mode.is_bidi_ltr() == containing_block_mode.is_bidi_ltr() {
                            inline_start_content_edge
                        } else {
                            // The kid's inline 'start' is at the parent's 'end'
                            inline_end_content_edge
                        };
                }
                kid_base.block_container_inline_size = container_inline_size;
                kid_base.block_container_writing_mode = containing_block_mode;
                kid_base.position.start.i = inline_start_content_edge;
            }
        }
    }

    // TODO(zentner): This function should actually flex elements!
    // Currently, this is the core of InlineFlow::propagate_assigned_inline_size_to_children() with
    // fragment logic stripped out.
    fn inline_mode_assign_inline_sizes(&mut self,
                                       _shared_context: &SharedStyleContext,
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

        let inline_size = match self.available_main_size {
            AxisSize::Definite(length) => length,
            AxisSize::MinMax(ref constraint) => constraint.clamp(content_inline_size),
            AxisSize::Infinite => content_inline_size,
        };

        let even_content_inline_size = inline_size / child_count;

        let container_mode = self.block_flow.base.block_container_writing_mode;
        self.block_flow.base.position.size.inline = inline_size;

        let block_container_explicit_block_size = self.block_flow.base.block_container_explicit_block_size;
        let mut inline_child_start = if !self.is_reverse {
            inline_start_content_edge
        } else {
            self.block_flow.fragment.border_box.size.inline
        };
        for kid in &mut self.items {
            let base = flow::mut_base(flow_ref::deref_mut(&mut kid.flow));

            base.block_container_inline_size = even_content_inline_size;
            base.block_container_writing_mode = container_mode;
            base.block_container_explicit_block_size = block_container_explicit_block_size;
            if !self.is_reverse {
              base.position.start.i = inline_child_start;
              inline_child_start = inline_child_start + even_content_inline_size;
            } else {
              base.position.start.i = inline_child_start - base.intrinsic_inline_sizes.preferred_inline_size;
              inline_child_start = inline_child_start - even_content_inline_size;
            };
        }
    }

    // TODO(zentner): This function should actually flex elements!
    fn block_mode_assign_block_size<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        let mut cur_b = if !self.is_reverse {
            self.block_flow.fragment.border_padding.block_start
        } else {
            self.block_flow.fragment.border_box.size.block
        };
        for kid in &mut self.items {
            let base = flow::mut_base(flow_ref::deref_mut(&mut kid.flow));
            if !self.is_reverse {
                base.position.start.b = cur_b;
                cur_b = cur_b + base.position.size.block;
            } else {
                cur_b = cur_b - base.position.size.block;
                base.position.start.b = cur_b;
            }
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
        for kid in self.block_flow.base.child_iter_mut() {
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
        for kid in self.block_flow.base.child_iter_mut() {
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

    fn as_block(&self) -> &BlockFlow {
        &self.block_flow
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

        // Flexbox Section 9.0: Generate anonymous flex items:
        // This part was handled in the flow constructor.

        // Flexbox Section 9.1: Re-order the flex items (and any absolutely positioned flex
        // container children) according to their order.

        let mut items = self.block_flow.base.children.iter_flow_ref_mut().map(|flow| {
            FlexItem::new(flow.clone())
        }).collect::<Vec<FlexItem>>();

        items.sort_by(|item1, item2| {
            item1.flow.as_block().fragment.style.get_position().order.cmp(
                &item2.flow.as_block().fragment.style.get_position().order
                )
        });

        self.items = items;

        match self.main_mode {
            Mode::Inline => self.inline_mode_bubble_inline_sizes(),
            Mode::Block  => self.block_mode_bubble_inline_sizes()
        }
    }

    fn assign_inline_sizes(&mut self, shared_context: &SharedStyleContext) {
        let _scope = layout_debug_scope!("flex::assign_inline_sizes {:x}", self.block_flow.base.debug_id());
        debug!("assign_inline_sizes");

        if !self.block_flow.base.restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW) {
            return
        }

        // Our inline-size was set to the inline-size of the containing block by the flow's parent.
        // Now compute the real value.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;
        self.block_flow.compute_used_inline_size(shared_context, containing_block_inline_size);
        if self.block_flow.base.flags.is_float() {
            self.block_flow.float.as_mut().unwrap().containing_inline_size = containing_block_inline_size
        }

        let (available_block_size, available_inline_size) = {
            let style = &self.block_flow.fragment.style;
            let (specified_block_size, specified_inline_size) = if style.writing_mode.is_vertical() {
                (style.get_position().width, style.get_position().height)
            } else {
                (style.get_position().height, style.get_position().width)
            };

            let available_inline_size = AxisSize::new(specified_inline_size,
                                                      Some(self.block_flow.base.block_container_inline_size),
                                                      style.min_inline_size(),
                                                      style.max_inline_size());

            let available_block_size = AxisSize::new(specified_block_size,
                                                     self.block_flow.base.block_container_explicit_block_size,
                                                     style.min_block_size(),
                                                     style.max_block_size());
            (available_block_size, available_inline_size)
        };

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
            Mode::Inline => {
                self.available_main_size = available_inline_size;
                self.available_cross_size = available_block_size;
                self.inline_mode_assign_inline_sizes(shared_context,
                                                     inline_start_content_edge,
                                                     inline_end_content_edge,
                                                     content_inline_size)
            },
            Mode::Block  => {
                self.available_main_size = available_block_size;
                self.available_cross_size = available_inline_size;
                self.block_mode_assign_inline_sizes(shared_context,
                                                    inline_start_content_edge,
                                                    inline_end_content_edge,
                                                    content_inline_size)
            }
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

    fn place_float_if_applicable<'a>(&mut self) {
        self.block_flow.place_float_if_applicable()
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, state: &mut DisplayListBuildState) {
        self.build_display_list_for_flex(state);
    }

    fn collect_stacking_contexts(&mut self,
                                 parent_id: StackingContextId,
                                 contexts: &mut Vec<Box<StackingContext>>)
                                 -> StackingContextId {
        self.block_flow.collect_stacking_contexts(parent_id, contexts)
    }

    fn repair_style(&mut self, new_style: &Arc<ServoComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Overflow {
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
