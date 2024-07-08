/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::From;
use std::fmt;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use app_units::Au;
use serde::Serialize;
use style::logical_geometry::{
    BlockFlowDirection, InlineBaseDirection, PhysicalCorner, WritingMode,
};
use style::values::computed::{CSSPixelLength, Length, LengthPercentage};
use style::values::generics::length::GenericLengthPercentageOrAuto as AutoOr;
use style::Zero;
use style_traits::CSSPixel;

use crate::ContainingBlock;

pub type PhysicalPoint<U> = euclid::Point2D<U, CSSPixel>;
pub type PhysicalSize<U> = euclid::Size2D<U, CSSPixel>;
pub type PhysicalRect<U> = euclid::Rect<U, CSSPixel>;
pub type PhysicalSides<U> = euclid::SideOffsets2D<U, CSSPixel>;
pub type AuOrAuto = AutoOr<Au>;
pub type LengthOrAuto = AutoOr<Length>;
pub type LengthPercentageOrAuto<'a> = AutoOr<&'a LengthPercentage>;

#[derive(Clone, Copy, Serialize)]
pub struct LogicalVec2<T> {
    pub inline: T,
    pub block: T,
}

#[derive(Clone, Copy, Serialize)]
pub struct LogicalRect<T> {
    pub start_corner: LogicalVec2<T>,
    pub size: LogicalVec2<T>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct LogicalSides<T> {
    pub inline_start: T,
    pub inline_end: T,
    pub block_start: T,
    pub block_end: T,
}

impl<T: fmt::Debug> fmt::Debug for LogicalVec2<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Not using f.debug_struct on purpose here, to keep {:?} output somewhat compact
        f.write_str("Vec2 { i: ")?;
        self.inline.fmt(f)?;
        f.write_str(", b: ")?;
        self.block.fmt(f)?;
        f.write_str(" }")
    }
}

impl<T: Clone> LogicalVec2<T> {
    pub fn from_physical_size(physical_size: &PhysicalSize<T>, mode: WritingMode) -> Self {
        // https://drafts.csswg.org/css-writing-modes/#logical-to-physical
        let (i, b) = if mode.is_horizontal() {
            (&physical_size.width, &physical_size.height)
        } else {
            (&physical_size.height, &physical_size.width)
        };
        LogicalVec2 {
            inline: i.clone(),
            block: b.clone(),
        }
    }

    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> LogicalVec2<U> {
        LogicalVec2 {
            inline: f(&self.inline),
            block: f(&self.block),
        }
    }
}

impl<T: Add<Output = T> + Copy> Add<LogicalVec2<T>> for LogicalVec2<T> {
    type Output = LogicalVec2<T>;
    fn add(self, other: Self) -> Self::Output {
        LogicalVec2 {
            inline: self.inline + other.inline,
            block: self.block + other.block,
        }
    }
}

impl<T: Sub<Output = T> + Copy> Sub<LogicalVec2<T>> for LogicalVec2<T> {
    type Output = LogicalVec2<T>;
    fn sub(self, other: Self) -> Self::Output {
        LogicalVec2 {
            inline: self.inline - other.inline,
            block: self.block - other.block,
        }
    }
}

impl<T: AddAssign<T> + Copy> AddAssign<LogicalVec2<T>> for LogicalVec2<T> {
    fn add_assign(&mut self, other: LogicalVec2<T>) {
        self.inline += other.inline;
        self.block += other.block;
    }
}

impl<T: SubAssign<T> + Copy> SubAssign<LogicalVec2<T>> for LogicalVec2<T> {
    fn sub_assign(&mut self, other: LogicalVec2<T>) {
        self.inline -= other.inline;
        self.block -= other.block;
    }
}

impl<T: Neg<Output = T> + Copy> Neg for LogicalVec2<T> {
    type Output = LogicalVec2<T>;
    fn neg(self) -> Self::Output {
        Self {
            inline: -self.inline,
            block: -self.block,
        }
    }
}

impl<T: Zero> LogicalVec2<T> {
    pub fn zero() -> Self {
        Self {
            inline: T::zero(),
            block: T::zero(),
        }
    }
}

impl<T: Clone> LogicalVec2<AutoOr<T>> {
    pub fn auto_is(&self, f: impl Fn() -> T) -> LogicalVec2<T> {
        self.map(|t| t.auto_is(&f))
    }
}

impl LogicalVec2<LengthPercentageOrAuto<'_>> {
    pub fn percentages_relative_to(
        &self,
        containing_block: &ContainingBlock,
    ) -> LogicalVec2<LengthOrAuto> {
        LogicalVec2 {
            inline: self
                .inline
                .percentage_relative_to(containing_block.inline_size.into()),
            block: self.block.maybe_percentage_relative_to(
                containing_block.block_size.map(|t| t.into()).non_auto(),
            ),
        }
    }
}

impl LogicalVec2<Option<&'_ LengthPercentage>> {
    pub fn percentages_relative_to(
        &self,
        containing_block: &ContainingBlock,
    ) -> LogicalVec2<Option<Length>> {
        LogicalVec2 {
            inline: self
                .inline
                .map(|lp| lp.percentage_relative_to(containing_block.inline_size.into())),
            block: self.block.and_then(|lp| {
                lp.maybe_percentage_relative_to(
                    containing_block.block_size.map(|t| t.into()).non_auto(),
                )
            }),
        }
    }
}

impl<T: Zero> LogicalRect<T> {
    pub fn zero() -> Self {
        Self {
            start_corner: LogicalVec2::zero(),
            size: LogicalVec2::zero(),
        }
    }
}

impl fmt::Debug for LogicalRect<Au> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Rect(i{}×b{} @ (i{},b{}))",
            self.size.inline.to_px(),
            self.size.block.to_px(),
            self.start_corner.inline.to_px(),
            self.start_corner.block.to_px(),
        )
    }
}

impl<T: Clone> LogicalVec2<T> {
    pub fn to_physical(&self, mode: WritingMode) -> PhysicalSize<T> {
        // https://drafts.csswg.org/css-writing-modes/#logical-to-physical
        let (x, y) = if mode.is_horizontal() {
            (&self.inline, &self.block)
        } else {
            (&self.block, &self.inline)
        };
        PhysicalSize::new(x.clone(), y.clone())
    }
}

impl<T: Clone> LogicalSides<T> {
    pub fn from_physical(sides: &PhysicalSides<T>, mode: WritingMode) -> Self {
        // https://drafts.csswg.org/css-writing-modes/#logical-to-physical
        let block_flow = mode.block_flow_direction();
        let (bs, be) = match mode.block_flow_direction() {
            BlockFlowDirection::TopToBottom => (&sides.top, &sides.bottom),
            BlockFlowDirection::RightToLeft => (&sides.right, &sides.left),
            BlockFlowDirection::LeftToRight => (&sides.left, &sides.right),
        };
        use BlockFlowDirection::TopToBottom;
        let (is, ie) = match (block_flow, mode.inline_base_direction()) {
            (TopToBottom, InlineBaseDirection::LeftToRight) => (&sides.left, &sides.right),
            (TopToBottom, InlineBaseDirection::RightToLeft) => (&sides.right, &sides.left),
            (_, InlineBaseDirection::LeftToRight) => (&sides.top, &sides.bottom),
            (_, InlineBaseDirection::RightToLeft) => (&sides.bottom, &sides.top),
        };
        LogicalSides {
            inline_start: is.clone(),
            inline_end: ie.clone(),
            block_start: bs.clone(),
            block_end: be.clone(),
        }
    }
}

impl<T> LogicalSides<T> {
    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> LogicalSides<U> {
        LogicalSides {
            inline_start: f(&self.inline_start),
            inline_end: f(&self.inline_end),
            block_start: f(&self.block_start),
            block_end: f(&self.block_end),
        }
    }

    pub fn map_inline_and_block_axes<U>(
        &self,
        inline_f: impl Fn(&T) -> U,
        block_f: impl Fn(&T) -> U,
    ) -> LogicalSides<U> {
        LogicalSides {
            inline_start: inline_f(&self.inline_start),
            inline_end: inline_f(&self.inline_end),
            block_start: block_f(&self.block_start),
            block_end: block_f(&self.block_end),
        }
    }

    pub fn inline_sum(&self) -> T::Output
    where
        T: Add + Copy,
    {
        self.inline_start + self.inline_end
    }

    pub fn block_sum(&self) -> T::Output
    where
        T: Add + Copy,
    {
        self.block_start + self.block_end
    }

    pub fn sum(&self) -> LogicalVec2<T::Output>
    where
        T: Add + Copy,
    {
        LogicalVec2 {
            inline: self.inline_sum(),
            block: self.block_sum(),
        }
    }

    pub fn to_physical(&self, mode: WritingMode) -> PhysicalSides<T>
    where
        T: Clone,
    {
        let top;
        let right;
        let bottom;
        let left;
        if mode.is_vertical() {
            if mode.is_vertical_lr() {
                left = self.block_start.clone();
                right = self.block_end.clone();
            } else {
                right = self.block_start.clone();
                left = self.block_end.clone();
            }

            if mode.is_inline_tb() {
                top = self.inline_start.clone();
                bottom = self.inline_end.clone();
            } else {
                bottom = self.inline_start.clone();
                top = self.inline_end.clone();
            }
        } else {
            top = self.block_start.clone();
            bottom = self.block_end.clone();
            if mode.is_bidi_ltr() {
                left = self.inline_start.clone();
                right = self.inline_end.clone();
            } else {
                right = self.inline_start.clone();
                left = self.inline_end.clone();
            }
        }
        PhysicalSides::new(top, right, bottom, left)
    }
}

impl<T: Copy> LogicalSides<T> {
    pub fn start_offset(&self) -> LogicalVec2<T> {
        LogicalVec2 {
            inline: self.inline_start,
            block: self.block_start,
        }
    }
}

impl LogicalSides<&'_ LengthPercentage> {
    pub fn percentages_relative_to(&self, basis: Length) -> LogicalSides<Length> {
        self.map(|s| s.percentage_relative_to(basis))
    }
}

impl LogicalSides<LengthPercentageOrAuto<'_>> {
    pub fn percentages_relative_to(&self, basis: Length) -> LogicalSides<LengthOrAuto> {
        self.map(|s| s.percentage_relative_to(basis))
    }
}

impl<T: Clone> LogicalSides<AutoOr<T>> {
    pub fn auto_is(&self, f: impl Fn() -> T) -> LogicalSides<T> {
        self.map(|s| s.auto_is(&f))
    }
}

impl<T: Add<Output = T> + Copy> Add<LogicalSides<T>> for LogicalSides<T> {
    type Output = LogicalSides<T>;

    fn add(self, other: Self) -> Self::Output {
        LogicalSides {
            inline_start: self.inline_start + other.inline_start,
            inline_end: self.inline_end + other.inline_end,
            block_start: self.block_start + other.block_start,
            block_end: self.block_end + other.block_end,
        }
    }
}

impl<T: Sub<Output = T> + Copy> Sub<LogicalSides<T>> for LogicalSides<T> {
    type Output = LogicalSides<T>;

    fn sub(self, other: Self) -> Self::Output {
        LogicalSides {
            inline_start: self.inline_start - other.inline_start,
            inline_end: self.inline_end - other.inline_end,
            block_start: self.block_start - other.block_start,
            block_end: self.block_end - other.block_end,
        }
    }
}

impl<T: Neg<Output = T> + Copy> Neg for LogicalSides<T> {
    type Output = LogicalSides<T>;
    fn neg(self) -> Self::Output {
        Self {
            inline_start: -self.inline_start,
            inline_end: -self.inline_end,
            block_start: -self.block_start,
            block_end: -self.block_end,
        }
    }
}

impl<T: Zero> LogicalSides<T> {
    pub(crate) fn zero() -> LogicalSides<T> {
        Self {
            inline_start: T::zero(),
            inline_end: T::zero(),
            block_start: T::zero(),
            block_end: T::zero(),
        }
    }
}

impl From<LogicalSides<CSSPixelLength>> for LogicalSides<Au> {
    fn from(value: LogicalSides<CSSPixelLength>) -> Self {
        Self {
            inline_start: value.inline_start.into(),
            inline_end: value.inline_end.into(),
            block_start: value.block_start.into(),
            block_end: value.block_end.into(),
        }
    }
}

impl From<LogicalSides<Au>> for LogicalSides<CSSPixelLength> {
    fn from(value: LogicalSides<Au>) -> Self {
        Self {
            inline_start: value.inline_start.into(),
            inline_end: value.inline_end.into(),
            block_start: value.block_start.into(),
            block_end: value.block_end.into(),
        }
    }
}

impl<T> LogicalRect<T> {
    pub fn max_inline_position(&self) -> T
    where
        T: Add<Output = T> + Copy,
    {
        self.start_corner.inline + self.size.inline
    }

    pub fn max_block_position(&self) -> T
    where
        T: Add<Output = T> + Copy,
    {
        self.start_corner.block + self.size.block
    }

    pub fn inflate(&self, sides: &LogicalSides<T>) -> Self
    where
        T: Add<Output = T> + Copy,
        T: Sub<Output = T> + Copy,
    {
        Self {
            start_corner: LogicalVec2 {
                inline: self.start_corner.inline - sides.inline_start,
                block: self.start_corner.block - sides.block_start,
            },
            size: LogicalVec2 {
                inline: self.size.inline + sides.inline_sum(),
                block: self.size.block + sides.block_sum(),
            },
        }
    }

    pub fn deflate(&self, sides: &LogicalSides<T>) -> Self
    where
        T: Add<Output = T> + Copy,
        T: Sub<Output = T> + Copy,
    {
        LogicalRect {
            start_corner: LogicalVec2 {
                inline: self.start_corner.inline + sides.inline_start,
                block: self.start_corner.block + sides.block_start,
            },
            size: LogicalVec2 {
                inline: self.size.inline - sides.inline_sum(),
                block: self.size.block - sides.block_sum(),
            },
        }
    }

    pub fn to_physical(
        &self,
        mode: WritingMode,
        // Will be needed for other writing modes
        // FIXME: what if the containing block has a different mode?
        // https://drafts.csswg.org/css-writing-modes/#orthogonal-flows
        _containing_block: &PhysicalRect<T>,
    ) -> PhysicalRect<T>
    where
        T: Clone,
    {
        // Top-left corner
        let (tl_x, tl_y) = match mode.start_start_physical_corner() {
            PhysicalCorner::TopLeft => (&self.start_corner.inline, &self.start_corner.block),
            _ => unimplemented!(),
        };
        PhysicalRect::new(
            PhysicalPoint::new(tl_x.clone(), tl_y.clone()),
            self.size.to_physical(mode),
        )
    }
}

impl From<LogicalVec2<CSSPixelLength>> for LogicalVec2<Au> {
    fn from(value: LogicalVec2<CSSPixelLength>) -> Self {
        LogicalVec2 {
            inline: value.inline.into(),
            block: value.block.into(),
        }
    }
}

impl From<LogicalVec2<Au>> for LogicalVec2<CSSPixelLength> {
    fn from(value: LogicalVec2<Au>) -> Self {
        LogicalVec2 {
            inline: value.inline.into(),
            block: value.block.into(),
        }
    }
}

impl From<LogicalRect<Au>> for LogicalRect<CSSPixelLength> {
    fn from(value: LogicalRect<Au>) -> Self {
        LogicalRect {
            start_corner: value.start_corner.into(),
            size: value.size.into(),
        }
    }
}

impl From<LogicalRect<CSSPixelLength>> for LogicalRect<Au> {
    fn from(value: LogicalRect<CSSPixelLength>) -> Self {
        LogicalRect {
            start_corner: value.start_corner.into(),
            size: value.size.into(),
        }
    }
}
