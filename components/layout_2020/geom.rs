/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::ContainingBlock;
use std::fmt;
use std::ops::{Add, AddAssign, Sub};
use style::logical_geometry::{BlockFlowDirection, InlineBaseDirection};
use style::logical_geometry::{PhysicalCorner, WritingMode};
use style::values::computed::{Length, LengthPercentage};
use style::values::generics::length::GenericLengthPercentageOrAuto as AutoOr;
use style::Zero;
use style_traits::CSSPixel;

pub type PhysicalPoint<U> = euclid::Point2D<U, CSSPixel>;
pub type PhysicalSize<U> = euclid::Size2D<U, CSSPixel>;
pub type PhysicalRect<U> = euclid::Rect<U, CSSPixel>;
pub type PhysicalSides<U> = euclid::SideOffsets2D<U, CSSPixel>;
pub type LengthOrAuto = AutoOr<Length>;
pub type LengthPercentageOrAuto<'a> = AutoOr<&'a LengthPercentage>;

pub mod flow_relative {
    #[derive(Clone, Serialize)]
    pub struct Vec2<T> {
        pub inline: T,
        pub block: T,
    }

    #[derive(Clone, Serialize)]
    pub struct Rect<T> {
        pub start_corner: Vec2<T>,
        pub size: Vec2<T>,
    }

    #[derive(Clone, Serialize)]
    pub struct Sides<T> {
        pub inline_start: T,
        pub inline_end: T,
        pub block_start: T,
        pub block_end: T,
    }
}

impl<T: fmt::Debug> fmt::Debug for flow_relative::Vec2<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Not using f.debug_struct on purpose here, to keep {:?} output somewhat compact
        f.write_str("Vec2 { i: ")?;
        self.inline.fmt(f)?;
        f.write_str(", b: ")?;
        self.block.fmt(f)?;
        f.write_str(" }")
    }
}

impl<T: Clone> flow_relative::Vec2<T> {
    pub fn from_physical_size(physical_size: &PhysicalSize<T>, mode: WritingMode) -> Self {
        // https://drafts.csswg.org/css-writing-modes/#logical-to-physical
        let (i, b) = if mode.is_horizontal() {
            (&physical_size.width, &physical_size.height)
        } else {
            (&physical_size.height, &physical_size.width)
        };
        flow_relative::Vec2 {
            inline: i.clone(),
            block: b.clone(),
        }
    }
}

impl<T> Add<&'_ flow_relative::Vec2<T>> for &'_ flow_relative::Vec2<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = flow_relative::Vec2<T>;

    fn add(self, other: &'_ flow_relative::Vec2<T>) -> Self::Output {
        flow_relative::Vec2 {
            inline: self.inline + other.inline,
            block: self.block + other.block,
        }
    }
}

impl<T> Sub<&'_ flow_relative::Vec2<T>> for &'_ flow_relative::Vec2<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = flow_relative::Vec2<T>;

    fn sub(self, other: &'_ flow_relative::Vec2<T>) -> Self::Output {
        flow_relative::Vec2 {
            inline: self.inline - other.inline,
            block: self.block - other.block,
        }
    }
}

impl<T> AddAssign<&'_ flow_relative::Vec2<T>> for flow_relative::Vec2<T>
where
    T: AddAssign<T> + Copy,
{
    fn add_assign(&mut self, other: &'_ flow_relative::Vec2<T>) {
        self.inline += other.inline;
        self.block += other.block;
    }
}

impl flow_relative::Vec2<Length> {
    pub fn zero() -> Self {
        Self {
            inline: Length::zero(),
            block: Length::zero(),
        }
    }
}

impl flow_relative::Vec2<LengthOrAuto> {
    pub fn auto_is(&self, f: impl Fn() -> Length) -> flow_relative::Vec2<Length> {
        flow_relative::Vec2 {
            inline: self.inline.auto_is(&f),
            block: self.block.auto_is(&f),
        }
    }
}

impl flow_relative::Vec2<LengthPercentageOrAuto<'_>> {
    pub fn percentages_relative_to(
        &self,
        containing_block: &ContainingBlock,
    ) -> flow_relative::Vec2<LengthOrAuto> {
        flow_relative::Vec2 {
            inline: self
                .inline
                .percentage_relative_to(containing_block.inline_size),
            block: self
                .block
                .maybe_percentage_relative_to(containing_block.block_size.non_auto()),
        }
    }
}

impl flow_relative::Vec2<Option<&'_ LengthPercentage>> {
    pub fn percentages_relative_to(
        &self,
        containing_block: &ContainingBlock,
    ) -> flow_relative::Vec2<Option<Length>> {
        flow_relative::Vec2 {
            inline: self
                .inline
                .map(|lp| lp.percentage_relative_to(containing_block.inline_size)),
            block: self.block.and_then(|lp| {
                lp.maybe_percentage_relative_to(containing_block.block_size.non_auto())
            }),
        }
    }
}

impl flow_relative::Rect<Length> {
    pub fn zero() -> Self {
        Self {
            start_corner: flow_relative::Vec2::zero(),
            size: flow_relative::Vec2::zero(),
        }
    }
}

impl fmt::Debug for flow_relative::Rect<Length> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Rect(i{}×b{} @ (i{},b{}))",
            self.size.inline.px(),
            self.size.block.px(),
            self.start_corner.inline.px(),
            self.start_corner.block.px(),
        )
    }
}

impl<T: Clone> flow_relative::Vec2<T> {
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

impl<T: Clone> flow_relative::Sides<T> {
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
        flow_relative::Sides {
            inline_start: is.clone(),
            inline_end: ie.clone(),
            block_start: bs.clone(),
            block_end: be.clone(),
        }
    }
}

impl<T> flow_relative::Sides<T> {
    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> flow_relative::Sides<U> {
        flow_relative::Sides {
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
    ) -> flow_relative::Sides<U> {
        flow_relative::Sides {
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

    pub fn sum(&self) -> flow_relative::Vec2<T::Output>
    where
        T: Add + Copy,
    {
        flow_relative::Vec2 {
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

impl<T> flow_relative::Sides<T>
where
    T: Copy,
{
    pub fn start_offset(&self) -> flow_relative::Vec2<T> {
        flow_relative::Vec2 {
            inline: self.inline_start,
            block: self.block_start,
        }
    }
}

impl flow_relative::Sides<&'_ LengthPercentage> {
    pub fn percentages_relative_to(&self, basis: Length) -> flow_relative::Sides<Length> {
        self.map(|s| s.percentage_relative_to(basis))
    }
}

impl flow_relative::Sides<LengthPercentageOrAuto<'_>> {
    pub fn percentages_relative_to(&self, basis: Length) -> flow_relative::Sides<LengthOrAuto> {
        self.map(|s| s.percentage_relative_to(basis))
    }
}

impl flow_relative::Sides<LengthOrAuto> {
    pub fn auto_is(&self, f: impl Fn() -> Length) -> flow_relative::Sides<Length> {
        self.map(|s| s.auto_is(&f))
    }
}

impl<T> Add<&'_ flow_relative::Sides<T>> for &'_ flow_relative::Sides<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = flow_relative::Sides<T>;

    fn add(self, other: &'_ flow_relative::Sides<T>) -> Self::Output {
        flow_relative::Sides {
            inline_start: self.inline_start + other.inline_start,
            inline_end: self.inline_end + other.inline_end,
            block_start: self.block_start + other.block_start,
            block_end: self.block_end + other.block_end,
        }
    }
}

impl<T> flow_relative::Rect<T> {
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

    pub fn inflate(&self, sides: &flow_relative::Sides<T>) -> Self
    where
        T: Add<Output = T> + Copy,
        T: Sub<Output = T> + Copy,
    {
        flow_relative::Rect {
            start_corner: flow_relative::Vec2 {
                inline: self.start_corner.inline - sides.inline_start,
                block: self.start_corner.block - sides.block_start,
            },
            size: flow_relative::Vec2 {
                inline: self.size.inline + sides.inline_sum(),
                block: self.size.block + sides.block_sum(),
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
