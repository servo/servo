/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Layout for elements with a CSS `display` property of `flex`.

#![deny(unsafe_code)]

use app_units::{Au, MAX_AU};
use block::{BlockFlow, MarginsMayCollapseFlag};
use context::{LayoutContext, SharedLayoutContext};
use display_list_builder::{DisplayListBuildState, FlexFlowDisplayListBuilding};
use euclid::Point2D;
use floats::FloatKind;
use flow;
use flow::{Flow, FlowClass, ImmutableFlowUtils, OpaqueFlow};
use flow::{INLINE_POSITION_IS_STATIC, IS_ABSOLUTELY_POSITIONED};
use fragment::{Fragment, FragmentBorderBoxIterator, Overflow};
use layout_debug;
use model::{IntrinsicISizes, MaybeAuto, SizeConstraint};
use model::{specified, specified_or_none};
use std::cmp::{max, min};
use std::ops::Range;
use std::sync::Arc;
use style::computed_values::{align_content, align_self, flex_direction, flex_wrap, justify_content};
use style::computed_values::border_collapse;
use style::context::SharedStyleContext;
use style::logical_geometry::{Direction, LogicalSize};
use style::properties::ServoComputedValues;
use style::servo::restyle_damage::{REFLOW, REFLOW_OUT_OF_FLOW};
use style::values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto};
use style::values::computed::{LengthOrPercentageOrAutoOrContent, LengthOrPercentageOrNone};

/// The size of an axis. May be a specified size, a min/max
/// constraint, or an unlimited size
#[derive(Debug, Serialize)]
enum AxisSize {
    Definite(Au),
    MinMax(SizeConstraint),
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
            }
            LengthOrPercentageOrAuto::Calc(calc) => {
                match content_size {
                    Some(size) => AxisSize::Definite(size.scale_by(calc.percentage())),
                    None => AxisSize::Infinite
                }
            }
            LengthOrPercentageOrAuto::Auto => {
                AxisSize::MinMax(SizeConstraint::new(content_size, min, max, None))
            }
        }
    }
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
#[derive(Debug, Serialize)]
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
    /// The index of the actual flow in our child list.
    pub index: usize,
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
    pub fn new(index: usize, flow: &Flow) -> FlexItem {
        let style = &flow.as_block().fragment.style;
        let flex_grow = style.get_position().flex_grow;
        let flex_shrink = style.get_position().flex_shrink;
        let order = style.get_position().order;
        // TODO(stshine): for item with 'visibility:collapse', set is_strut to true.

        FlexItem {
            main_size: Au(0),
            base_size: Au(0),
            min_size: Au(0),
            max_size: MAX_AU,
            index: index,
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
    pub fn init_sizes(&mut self, flow: &mut Flow, containing_length: Au, direction: Direction) {
        let block = flow.as_mut_block();
        match direction {
            // TODO(stshine): the definition of min-{width, height} in style component
            // should change to LengthOrPercentageOrAuto for automatic implied minimal size.
            // https://drafts.csswg.org/css-flexbox-1/#min-size-auto
            Direction::Inline => {
                let basis = from_flex_basis(block.fragment.style.get_position().flex_basis,
                                            block.fragment.style.content_inline_size(),
                                            Some(containing_length));

                // These methods compute auto margins to zero length, which is exactly what we want.
                block.fragment.compute_border_and_padding(containing_length,
                                                          border_collapse::T::separate);
                block.fragment.compute_inline_direction_margins(containing_length);
                block.fragment.compute_block_direction_margins(containing_length);

                let (border_padding, margin) = block.fragment.surrounding_intrinsic_inline_size();
                let content_size = block.base.intrinsic_inline_sizes.preferred_inline_size
                    - border_padding
                    - margin
                    + block.fragment.box_sizing_boundary(direction);
                self.base_size = basis.specified_or_default(content_size);
                self.max_size = specified_or_none(block.fragment.style.max_inline_size(),
                                                  containing_length).unwrap_or(MAX_AU);
                self.min_size = specified(block.fragment.style.min_inline_size(),
                                          containing_length);
            }
            Direction::Block => {
                let basis = from_flex_basis(block.fragment.style.get_position().flex_basis,
                                            block.fragment.style.content_block_size(),
                                            Some(containing_length));
                let content_size = block.fragment.border_box.size.block
                    - block.fragment.border_padding.block_start_end()
                    + block.fragment.box_sizing_boundary(direction);
                self.base_size = basis.specified_or_default(content_size);
                self.max_size = specified_or_none(block.fragment.style.max_block_size(),
                                                  containing_length).unwrap_or(MAX_AU);
                self.min_size = specified(block.fragment.style.min_block_size(),
                                          containing_length);
            }
        }
    }

    /// Returns the outer main size of the item, including paddings and margins,
    /// clamped by max and min size.
    pub fn outer_main_size(&self, flow: &Flow, direction: Direction) -> Au {
        let ref fragment = flow.as_block().fragment;
        let outer_width = match direction {
            Direction::Inline => {
                fragment.border_padding.inline_start_end() + fragment.margin.inline_start_end()
            }
            Direction::Block => {
                fragment.border_padding.block_start_end() + fragment.margin.block_start_end()
            }
        };
        max(self.min_size, min(self.base_size, self.max_size))
            - fragment.box_sizing_boundary(direction) + outer_width
    }

    /// Returns the number of auto margins in given direction.
    pub fn auto_margin_count(&self, flow: &Flow, direction: Direction) -> i32 {
        let margin = flow.as_block().fragment.style.logical_margin();
        let mut margin_count = 0;
        match direction {
            Direction::Inline => {
                if margin.inline_start == LengthOrPercentageOrAuto::Auto {
                    margin_count += 1;
                }
                if margin.inline_end == LengthOrPercentageOrAuto::Auto {
                    margin_count += 1;
                }
            }
            Direction::Block => {
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

/// A line in a flex container.
// TODO(stshine): More fields are required to handle collapsed items and baseline alignment.
#[derive(Debug, Serialize)]
struct FlexLine {
    /// Range of items belong to this line in 'self.items'.
    pub range: Range<usize>,
    /// Remaining free space of this line, items will grow or shrink based on it being positive or negative.
    pub free_space: Au,
    /// The number of auto margins of items.
    pub auto_margin_count: i32,
    /// Line size in the block direction.
    pub cross_size: Au,
}

impl FlexLine {
    pub fn new(range: Range<usize>, free_space: Au, auto_margin_count: i32) -> FlexLine {
        FlexLine {
            range: range,
            auto_margin_count: auto_margin_count,
            free_space: free_space,
            cross_size: Au(0)
        }
    }

    /// This method implements the flexible lengths resolving algorithm.
    /// The 'collapse' parameter is used to indicate whether items with 'visibility: collapse'
    /// is included in length resolving. The result main size is stored in 'item.main_size'.
    /// https://drafts.csswg.org/css-flexbox/#resolve-flexible-lengths
    pub fn flex_resolve(&mut self, items: &mut [FlexItem], collapse: bool) {
        let mut total_grow = 0.0;
        let mut total_shrink = 0.0;
        let mut total_scaled = 0.0;
        let mut active_count = 0;
        // Iterate through items, collect total factors and freeze those that have already met
        // their constraints or won't grow/shrink in corresponding scenario.
        // https://drafts.csswg.org/css-flexbox/#resolve-flexible-lengths
        for item in items.iter_mut().filter(|i| !(i.is_strut && collapse)) {
            item.main_size = max(item.min_size, min(item.base_size, item.max_size));
            if (self.free_space > Au(0) && (item.flex_grow == 0.0 || item.base_size >= item.max_size)) ||
                (self.free_space < Au(0) && (item.flex_shrink == 0.0 || item.base_size <= item.min_size)) {
                    item.is_frozen = true;
                } else {
                    item.is_frozen = false;
                    total_grow += item.flex_grow;
                    total_shrink += item.flex_shrink;
                    // The scaled factor is used to calculate flex shrink
                    total_scaled += item.flex_shrink * item.base_size.0 as f32;
                    active_count += 1;
                }
        }

        let initial_free_space = self.free_space;
        let mut total_variation = Au(1);
        // If there is no remaining free space or all items are frozen, stop loop.
        while total_variation != Au(0) && self.free_space != Au(0) && active_count > 0 {
            self.free_space =
                // https://drafts.csswg.org/css-flexbox/#remaining-free-space
                if self.free_space > Au(0) {
                    min(initial_free_space.scale_by(total_grow), self.free_space)
                } else {
                    max(initial_free_space.scale_by(total_shrink), self.free_space)
                };

            total_variation = Au(0);
            for item in items.iter_mut().filter(|i| !i.is_frozen).filter(|i| !(i.is_strut && collapse)) {
                // Use this and the 'abs()' below to make the code work in both grow and shrink scenarios.
                let (factor, end_size) = if self.free_space > Au(0) {
                    (item.flex_grow / total_grow, item.max_size)
                } else {
                    (item.flex_shrink * item.base_size.0 as f32 / total_scaled, item.min_size)
                };
                let variation = self.free_space.scale_by(factor);
                if variation.0.abs() >= (end_size - item.main_size).0.abs() {
                    // Use constraint as the target main size, and freeze item.
                    total_variation += end_size - item.main_size;
                    item.main_size = end_size;
                    item.is_frozen = true;
                    active_count -= 1;
                    total_shrink -= item.flex_shrink;
                    total_grow -= item.flex_grow;
                    total_scaled -= item.flex_shrink * item.base_size.0 as f32;
                } else {
                    total_variation += variation;
                    item.main_size += variation;
                }
            }
            self.free_space -= total_variation;
        }
    }
}

/// A block with the CSS `display` property equal to `flex`.
#[derive(Debug, Serialize)]
pub struct FlexFlow {
    /// Data common to all block flows.
    block_flow: BlockFlow,
    /// The logical axis which the main axis will be parallel with.
    /// The cross axis will be parallel with the opposite logical axis.
    main_mode: Direction,
    /// The available main axis size
    available_main_size: AxisSize,
    /// The available cross axis size
    available_cross_size: AxisSize,
    /// List of flex lines in the container.
    lines: Vec<FlexLine>,
    /// List of flex-items that belong to this flex-container
    items: Vec<FlexItem>,
    /// True if the flex-direction is *-reversed
    main_reverse: bool,
    /// True if this flex container can be multiline.
    is_wrappable: bool,
    /// True if the cross direction is reversed.
    cross_reverse: bool
}

impl FlexFlow {
    pub fn from_fragment(fragment: Fragment,
                         flotation: Option<FloatKind>)
                         -> FlexFlow {
        let main_mode;
        let main_reverse;
        let is_wrappable;
        let cross_reverse;
        {
            let style = fragment.style();
            let (mode, reverse) = match style.get_position().flex_direction {
                flex_direction::T::row            => (Direction::Inline, false),
                flex_direction::T::row_reverse    => (Direction::Inline, true),
                flex_direction::T::column         => (Direction::Block, false),
                flex_direction::T::column_reverse => (Direction::Block, true),
            };
            main_mode = mode;
            main_reverse =
                reverse == style.writing_mode.is_bidi_ltr();
            let (wrappable, reverse) = match fragment.style.get_position().flex_wrap {
                flex_wrap::T::nowrap              => (false, false),
                flex_wrap::T::wrap                => (true, false),
                flex_wrap::T::wrap_reverse        => (true, true),
            };
            is_wrappable = wrappable;
            // TODO(stshine): Handle vertical writing mode.
            cross_reverse = reverse;
        }

        FlexFlow {
            block_flow: BlockFlow::from_fragment_and_float_kind(fragment, flotation),
            main_mode: main_mode,
            available_main_size: AxisSize::Infinite,
            available_cross_size: AxisSize::Infinite,
            lines: Vec::new(),
            items: Vec::new(),
            main_reverse: main_reverse,
            is_wrappable: is_wrappable,
            cross_reverse: cross_reverse
        }
    }

    pub fn main_mode(&self) -> Direction {
        self.main_mode
    }

    /// Returns a line start after the last item that is already in a line.
    /// Note that when the container main size is infinite(i.e. A column flexbox with auto height),
    /// we do not need to do flex resolving and this can be considered as a fast-path, so the
    /// 'container_size' param does not need to be 'None'. A line has to contain at least one item;
    /// (except this) if the container can be multi-line the sum of outer main size of items should
    /// be less than the container size; a line should be filled by items as much as possible.
    /// After been collected in a line a item should have its main sizes initialized.
    fn get_flex_line(&mut self, container_size: Au) -> Option<FlexLine> {
        let start = self.lines.last().map(|line| line.range.end).unwrap_or(0);
        if start == self.items.len() {
            return None;
        }
        let mut end = start;
        let mut total_line_size = Au(0);
        let mut margin_count = 0;

        let items = &mut self.items[start..];
        let mut children = self.block_flow.base.children.random_access_mut();
        for mut item in items {
            let kid = children.get(item.index);
            item.init_sizes(kid, container_size, self.main_mode);
            let outer_main_size = item.outer_main_size(kid, self.main_mode);
            if total_line_size + outer_main_size > container_size && end != start && self.is_wrappable {
                break;
            }
            margin_count += item.auto_margin_count(kid, self.main_mode);
            total_line_size += outer_main_size;
            end += 1;
        }

        let line = FlexLine::new(start..end, container_size - total_line_size, margin_count);
        Some(line)
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
            for kid in self.block_flow.base.children.iter_mut() {
                let base = flow::mut_base(kid);
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
            for kid in self.block_flow.base.children.iter_mut() {
                let base = flow::mut_base(kid);
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
        debug!("flex::block_mode_assign_inline_sizes");

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

        let mut children = self.block_flow.base.children.random_access_mut();
        for kid in &mut self.items {
            let kid_base = flow::mut_base(children.get(kid.index));
            kid_base.block_container_explicit_block_size = container_block_size;
            if kid_base.flags.contains(INLINE_POSITION_IS_STATIC) {
                // The inline-start margin edge of the child flow is at our inline-start content
                // edge, and its inline-size is our content inline-size.
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

        let container_mode = self.block_flow.base.block_container_writing_mode;
        self.block_flow.base.position.size.inline = inline_size;

        // Calculate non-auto block size to pass to children.
        let box_border = self.block_flow.fragment.box_sizing_boundary(Direction::Block);

        let parent_container_size = self.block_flow.explicit_block_containing_size(_shared_context);
        // https://drafts.csswg.org/css-ui-3/#box-sizing
        let explicit_content_size = self
                                    .block_flow
                                    .explicit_block_size(parent_container_size)
                                    .map(|x| max(x - box_border, Au(0)));
        let containing_block_text_align =
            self.block_flow.fragment.style().get_inheritedtext().text_align;

        while let Some(mut line) = self.get_flex_line(inline_size) {
            let items = &mut self.items[line.range.clone()];
            line.flex_resolve(items, false);
            // TODO(stshine): if this flex line contain children that have
            // property visibility:collapse, exclude them and resolve again.

            let item_count = items.len() as i32;
            let mut cur_i = inline_start_content_edge;
            let item_interval = if line.free_space >= Au(0) && line.auto_margin_count == 0 {
                match self.block_flow.fragment.style().get_position().justify_content {
                    justify_content::T::space_between => {
                        if item_count == 1 {
                            Au(0)
                        } else {
                            line.free_space / (item_count - 1)
                        }
                    }
                    justify_content::T::space_around => {
                        line.free_space / item_count
                    }
                    _ => Au(0),
                }
            } else {
                Au(0)
            };

            match self.block_flow.fragment.style().get_position().justify_content {
                // Overflow equally in both ends of line.
                justify_content::T::center | justify_content::T::space_around => {
                    cur_i += (line.free_space - item_interval * (item_count - 1)) / 2;
                }
                justify_content::T::flex_end => {
                    cur_i += line.free_space;
                }
                _ => {}
            }

            let mut children = self.block_flow.base.children.random_access_mut();
            for item in items.iter_mut() {
                let mut block = children.get(item.index).as_mut_block();

                block.base.block_container_writing_mode = container_mode;
                block.base.block_container_inline_size = inline_size;
                block.base.block_container_explicit_block_size = explicit_content_size;
                // Per CSS 2.1 § 16.3.1, text alignment propagates to all children in flow.
                //
                // TODO(#2265, pcwalton): Do this in the cascade instead.
                block.base.flags.set_text_align(containing_block_text_align);

                let margin = block.fragment.style().logical_margin();
                let auto_len =
                    if line.auto_margin_count == 0 || line.free_space <= Au(0) {
                        Au(0)
                    } else {
                        line.free_space / line.auto_margin_count
                    };
                let margin_inline_start = MaybeAuto::from_style(margin.inline_start, inline_size)
                    .specified_or_default(auto_len);
                let margin_inline_end = MaybeAuto::from_style(margin.inline_end, inline_size)
                    .specified_or_default(auto_len);
                let item_inline_size = item.main_size
                    - block.fragment.box_sizing_boundary(self.main_mode)
                    + block.fragment.border_padding.inline_start_end();
                let item_outer_size = item_inline_size + block.fragment.margin.inline_start_end();

                block.fragment.margin.inline_start = margin_inline_start;
                block.fragment.margin.inline_end = margin_inline_end;
                block.fragment.border_box.start.i = margin_inline_start;
                block.fragment.border_box.size.inline = item_inline_size;
                block.base.position.start.i = if !self.main_reverse {
                    cur_i
                } else {
                    inline_start_content_edge * 2 + content_inline_size - cur_i  - item_outer_size
                };
                block.base.position.size.inline = item_outer_size;
                cur_i += item_outer_size + item_interval;
            }
            self.lines.push(line);
        }
    }

    // TODO(zentner): This function should actually flex elements!
    fn block_mode_assign_block_size(&mut self) {
        let mut cur_b = if !self.main_reverse {
            self.block_flow.fragment.border_padding.block_start
        } else {
            self.block_flow.fragment.border_box.size.block
        };

        let mut children = self.block_flow.base.children.random_access_mut();
        for item in &mut self.items {
            let mut base = flow::mut_base(children.get(item.index));
            if !self.main_reverse {
                base.position.start.b = cur_b;
                cur_b = cur_b + base.position.size.block;
            } else {
                cur_b = cur_b - base.position.size.block;
                base.position.start.b = cur_b;
            }
        }
    }

    fn inline_mode_assign_block_size<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        let _scope = layout_debug_scope!("flex::inline_mode_assign_block_size");

        let line_count = self.lines.len() as i32;
        let line_align = self.block_flow.fragment.style().get_position().align_content;
        let mut cur_b = self.block_flow.fragment.border_padding.block_start;
        let mut total_cross_size = Au(0);
        let mut line_interval = Au(0);

        {
            let mut children = self.block_flow.base.children.random_access_mut();
            for line in self.lines.iter_mut() {
                for item in &self.items[line.range.clone()] {
                    let fragment = &children.get(item.index).as_block().fragment;
                    line.cross_size = max(line.cross_size,
                                          fragment.border_box.size.block +
                                          fragment.margin.block_start_end());
                }
                total_cross_size += line.cross_size;
            }
        }

        let box_border = self.block_flow.fragment.box_sizing_boundary(Direction::Block);
        let parent_container_size =
            self.block_flow.explicit_block_containing_size(layout_context.shared_context());
        // https://drafts.csswg.org/css-ui-3/#box-sizing
        let explicit_content_size = self
                                    .block_flow
                                    .explicit_block_size(parent_container_size)
                                    .map(|x| max(x - box_border, Au(0)));

        if let Some(container_block_size) = explicit_content_size {
            let free_space = container_block_size - total_cross_size;
            total_cross_size = container_block_size;

            if line_align == align_content::T::stretch && free_space > Au(0) {
                for line in self.lines.iter_mut() {
                    line.cross_size += free_space / line_count;
                }
            }

            line_interval = match line_align {
                align_content::T::space_between => {
                    if line_count == 1 {
                        Au(0)
                    } else {
                        free_space / (line_count - 1)
                    }
                }
                align_content::T::space_around => {
                    free_space / line_count
                }
                _ => Au(0),
            };

            match line_align {
                align_content::T::center | align_content::T::space_around => {
                    cur_b += (free_space - line_interval * (line_count - 1)) / 2;
                }
                align_content::T::flex_end => {
                    cur_b += free_space;
                }
                _ => {}
            }
        }

        let mut children = self.block_flow.base.children.random_access_mut();
        for line in &self.lines {
            for item in self.items[line.range.clone()].iter_mut() {
                let block = children.get(item.index).as_mut_block();
                let auto_margin_count = item.auto_margin_count(block, Direction::Block);
                let margin = block.fragment.style().logical_margin();

                let mut margin_block_start = block.fragment.margin.block_start;
                let mut margin_block_end = block.fragment.margin.block_end;
                let mut free_space = line.cross_size - block.base.position.size.block
                    - block.fragment.margin.block_start_end();

                // The spec is a little vague here, but if I understand it correctly, the outer
                // cross size of item should equal to the line size if any auto margin exists.
                // https://drafts.csswg.org/css-flexbox/#algo-cross-margins
                if auto_margin_count > 0 {
                    if margin.block_start == LengthOrPercentageOrAuto::Auto {
                        margin_block_start = if free_space < Au(0) {
                            Au(0)
                        } else {
                            free_space / auto_margin_count
                        };
                    }
                    margin_block_end = line.cross_size - margin_block_start - block.base.position.size.block;
                    free_space = Au(0);
                }

                let self_align = block.fragment.style().get_position().align_self;
                if self_align == align_self::T::stretch &&
                    block.fragment.style().content_block_size() == LengthOrPercentageOrAuto::Auto {
                        free_space = Au(0);
                        block.base.block_container_explicit_block_size = Some(line.cross_size);
                        block.base.position.size.block =
                            line.cross_size - margin_block_start - margin_block_end;
                        block.fragment.border_box.size.block = block.base.position.size.block;
                        // FIXME(stshine): item with 'align-self: stretch' and auto cross size should act
                        // as if it has a fixed cross size, all child blocks should resolve against it.
                        // block.assign_block_size(layout_context);
                    }
                block.base.position.start.b = margin_block_start +
                    if !self.cross_reverse {
                        cur_b
                    } else {
                        self.block_flow.fragment.border_padding.block_start * 2
                            + total_cross_size - cur_b - line.cross_size
                    };
                // TODO(stshine): support baseline alignment.
                if free_space != Au(0) {
                    let flex_cross = match self_align {
                        align_self::T::flex_end => free_space,
                        align_self::T::center => free_space / 2,
                        _ => Au(0),
                    };
                    block.base.position.start.b +=
                        if !self.cross_reverse {
                            flex_cross
                        } else {
                            free_space - flex_cross
                        };
                }
            }
            cur_b += line_interval + line.cross_size;
        }
        let total_block_size = total_cross_size + self.block_flow.fragment.border_padding.block_start_end();
        self.block_flow.fragment.border_box.size.block = total_block_size;
        self.block_flow.base.position.size.block = total_block_size;

    }
}

impl Flow for FlexFlow {
    fn class(&self) -> FlowClass {
        FlowClass::Flex
    }

    fn as_mut_flex(&mut self) -> &mut FlexFlow {
        self
    }

    fn as_flex(&self) -> &FlexFlow {
        self
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

        // Flexbox Section 9.1: Re-order flex items according to their order.
        // FIXME(stshine): This should be done during flow construction.
        let mut items: Vec<FlexItem> =
            self.block_flow
                .base
                .children
                .iter()
                .enumerate()
                .filter(|&(_, flow)| {
                    !flow.as_block().base.flags.contains(IS_ABSOLUTELY_POSITIONED)
                })
                .map(|(index, flow)| FlexItem::new(index, flow))
                .collect();

        items.sort_by_key(|item| item.order);
        self.items = items;

        match self.main_mode {
            Direction::Inline => self.inline_mode_bubble_inline_sizes(),
            Direction::Block  => self.block_mode_bubble_inline_sizes()
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
            Direction::Inline => {
                self.available_main_size = available_inline_size;
                self.available_cross_size = available_block_size;
                self.inline_mode_assign_inline_sizes(shared_context,
                                                     inline_start_content_edge,
                                                     inline_end_content_edge,
                                                     content_inline_size)
            }
            Direction::Block  => {
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
        self.block_flow
            .assign_block_size_block_base(layout_context,
                                          None,
                                          MarginsMayCollapseFlag::MarginsMayNotCollapse);
        match self.main_mode {
            Direction::Inline => self.inline_mode_assign_block_size(layout_context),
            Direction::Block => self.block_mode_assign_block_size(),
        }
    }

    fn compute_absolute_position(&mut self, layout_context: &SharedLayoutContext) {
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

    fn collect_stacking_contexts(&mut self, state: &mut DisplayListBuildState) {
        self.block_flow.collect_stacking_contexts(state);
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
