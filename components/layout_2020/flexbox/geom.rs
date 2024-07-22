/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! <https://drafts.csswg.org/css-flexbox/#box-model>

use style::properties::longhands::flex_direction::computed_value::T as FlexDirection;

use crate::geom::{LogicalRect, LogicalSides, LogicalVec2};

#[derive(Clone, Copy)]
pub(super) struct FlexRelativeVec2<T> {
    pub main: T,
    pub cross: T,
}

#[derive(Clone, Copy)]
pub(super) struct FlexRelativeSides<T> {
    pub cross_start: T,
    pub main_start: T,
    pub cross_end: T,
    pub main_end: T,
}

pub(super) struct FlexRelativeRect<T> {
    pub start_corner: FlexRelativeVec2<T>,
    pub size: FlexRelativeVec2<T>,
}

impl<T> std::ops::Add for FlexRelativeVec2<T>
where
    T: std::ops::Add,
{
    type Output = FlexRelativeVec2<T::Output>;
    fn add(self, rhs: Self) -> Self::Output {
        FlexRelativeVec2 {
            main: self.main + rhs.main,
            cross: self.cross + rhs.cross,
        }
    }
}

impl<T> std::ops::Sub for FlexRelativeVec2<T>
where
    T: std::ops::Sub,
{
    type Output = FlexRelativeVec2<T::Output>;
    fn sub(self, rhs: Self) -> Self::Output {
        FlexRelativeVec2 {
            main: self.main - rhs.main,
            cross: self.cross - rhs.cross,
        }
    }
}

impl<T> FlexRelativeSides<T> {
    pub fn sum_by_axis(self) -> FlexRelativeVec2<T::Output>
    where
        T: std::ops::Add,
    {
        FlexRelativeVec2 {
            main: self.main_start + self.main_end,
            cross: self.cross_start + self.cross_end,
        }
    }
}

/// One of the two bits set by the `flex-direction` property
/// (The other is "forward" v.s. reverse.)
#[derive(Clone, Copy, PartialEq)]
pub(super) enum FlexAxis {
    /// The main axis is the inline axis of the container (not necessarily of flex items!),
    /// cross is block.
    Row,
    /// The main axis is the block axis, cross is inline.
    Column,
}

/// Which flow-relative sides map to the main-start and cross-start sides, respectively.
/// See <https://drafts.csswg.org/css-flexbox/#box-model>
#[derive(Clone, Copy)]
pub(super) enum MainStartCrossStart {
    InlineStartBlockStart,
    InlineStartBlockEnd,
    BlockStartInlineStart,
    BlockStartInlineEnd,
    InlineEndBlockStart,
    InlineEndBlockEnd,
    BlockEndInlineStart,
    BlockEndInlineEnd,
}

impl FlexAxis {
    pub fn from(flex_direction: FlexDirection) -> Self {
        match flex_direction {
            FlexDirection::Row | FlexDirection::RowReverse => FlexAxis::Row,
            FlexDirection::Column | FlexDirection::ColumnReverse => FlexAxis::Column,
        }
    }

    pub fn vec2_to_flex_relative<T>(self, flow_relative: LogicalVec2<T>) -> FlexRelativeVec2<T> {
        let LogicalVec2 { inline, block } = flow_relative;
        match self {
            FlexAxis::Row => FlexRelativeVec2 {
                main: inline,
                cross: block,
            },
            FlexAxis::Column => FlexRelativeVec2 {
                main: block,
                cross: inline,
            },
        }
    }

    pub fn vec2_to_flow_relative<T>(self, flex_relative: FlexRelativeVec2<T>) -> LogicalVec2<T> {
        let FlexRelativeVec2 { main, cross } = flex_relative;
        match self {
            FlexAxis::Row => LogicalVec2 {
                inline: main,
                block: cross,
            },
            FlexAxis::Column => LogicalVec2 {
                block: main,
                inline: cross,
            },
        }
    }
}

macro_rules! sides_mapping_methods {
    (
        $(
            $variant: path => {
                $( $flex_relative_side: ident <=> $flow_relative_side: ident, )+
            },
        )+
    ) => {
        pub fn sides_to_flex_relative<T>(self, flow_relative: LogicalSides<T>) -> FlexRelativeSides<T> {
            match self {
                $(
                    $variant => FlexRelativeSides {
                        $( $flex_relative_side: flow_relative.$flow_relative_side, )+
                    },
                )+
            }
        }

        pub fn sides_to_flow_relative<T>(self, flex_relative: FlexRelativeSides<T>) -> LogicalSides<T> {
            match self {
                $(
                    $variant => LogicalSides {
                        $( $flow_relative_side: flex_relative.$flex_relative_side, )+
                    },
                )+
            }
        }
    }
}

impl MainStartCrossStart {
    pub fn from(flex_direction: FlexDirection, flex_wrap_reverse: bool) -> Self {
        match (flex_direction, flex_wrap_reverse) {
            // See definition of each keyword in
            // https://drafts.csswg.org/css-flexbox/#flex-direction-property and
            // https://drafts.csswg.org/css-flexbox/#flex-wrap-property,
            // or the tables (though they map to physical rather than flow-relative) at
            // https://drafts.csswg.org/css-flexbox/#axis-mapping
            (FlexDirection::Row, true) => MainStartCrossStart::InlineStartBlockEnd,
            (FlexDirection::Row, false) => MainStartCrossStart::InlineStartBlockStart,
            (FlexDirection::Column, true) => MainStartCrossStart::BlockStartInlineEnd,
            (FlexDirection::Column, false) => MainStartCrossStart::BlockStartInlineStart,
            (FlexDirection::RowReverse, true) => MainStartCrossStart::InlineEndBlockEnd,
            (FlexDirection::RowReverse, false) => MainStartCrossStart::InlineEndBlockStart,
            (FlexDirection::ColumnReverse, true) => MainStartCrossStart::BlockEndInlineEnd,
            (FlexDirection::ColumnReverse, false) => MainStartCrossStart::BlockEndInlineStart,
        }
    }

    sides_mapping_methods! {
        MainStartCrossStart::InlineStartBlockStart => {
            main_start <=> inline_start,
            cross_start <=> block_start,
            main_end <=> inline_end,
            cross_end <=> block_end,
        },
        MainStartCrossStart::InlineStartBlockEnd => {
            main_start <=> inline_start,
            cross_start <=> block_end,
            main_end <=> inline_end,
            cross_end <=> block_start,
        },
        MainStartCrossStart::BlockStartInlineStart => {
            main_start <=> block_start,
            cross_start <=> inline_start,
            main_end <=> block_end,
            cross_end <=> inline_end,
        },
        MainStartCrossStart::BlockStartInlineEnd => {
            main_start <=> block_start,
            cross_start <=> inline_end,
            main_end <=> block_end,
            cross_end <=> inline_start,
        },
        MainStartCrossStart::InlineEndBlockStart => {
            main_start <=> inline_end,
            cross_start <=> block_start,
            main_end <=> inline_start,
            cross_end <=> block_end,
        },
        MainStartCrossStart::InlineEndBlockEnd => {
            main_start <=> inline_end,
            cross_start <=> block_end,
            main_end <=> inline_start,
            cross_end <=> block_start,
        },
        MainStartCrossStart::BlockEndInlineStart => {
            main_start <=> block_end,
            cross_start <=> inline_start,
            main_end <=> block_start,
            cross_end <=> inline_end,
        },
        MainStartCrossStart::BlockEndInlineEnd => {
            main_start <=> block_end,
            cross_start <=> inline_end,
            main_end <=> block_start,
            cross_end <=> inline_start,
        },
    }
}

/// The start corner coordinates in both the input rectangle and output rectangle
/// are relative to some “base rectangle” whose size is passed here.
pub(super) fn rect_to_flow_relative<T>(
    flex_axis: FlexAxis,
    main_start_cross_start_sides_are: MainStartCrossStart,
    base_rect_size: FlexRelativeVec2<T>,
    rect: FlexRelativeRect<T>,
) -> LogicalRect<T>
where
    T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
{
    // First, convert from (start corner, size) to offsets from the edges of the base rectangle

    let end_corner_position = rect.start_corner + rect.size;
    let end_corner_offsets = base_rect_size - end_corner_position;
    // No-ops, but hopefully clarifies to human readers:
    let start_corner_position = rect.start_corner;
    let start_corner_offsets = start_corner_position;

    // Then, convert to flow-relative using methods above
    let flow_relative_offsets =
        main_start_cross_start_sides_are.sides_to_flow_relative(FlexRelativeSides {
            main_start: start_corner_offsets.main,
            cross_start: start_corner_offsets.cross,
            main_end: end_corner_offsets.main,
            cross_end: end_corner_offsets.cross,
        });
    let flow_relative_base_rect_size = flex_axis.vec2_to_flow_relative(base_rect_size);

    // Finally, convert back to (start corner, size)
    let start_corner = LogicalVec2 {
        inline: flow_relative_offsets.inline_start,
        block: flow_relative_offsets.block_start,
    };
    let end_corner_position = LogicalVec2 {
        inline: flow_relative_base_rect_size.inline - flow_relative_offsets.inline_end,
        block: flow_relative_base_rect_size.block - flow_relative_offsets.block_end,
    };
    let size = end_corner_position - start_corner;
    LogicalRect { start_corner, size }
}
