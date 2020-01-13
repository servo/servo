/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::ContainingBlock;
use std::fmt;
use std::ops::{Add, AddAssign, Sub};
use style::logical_geometry::{BlockFlowDirection, InlineBaseDirection};
use style::logical_geometry::{PhysicalCorner, WritingMode};
use style::values::computed::{Length, LengthOrAuto, LengthPercentage, LengthPercentageOrAuto};
use style::values::generics::length::MaxSize;
use style::Zero;
use style_traits::CSSPixel;

pub type Point<U> = euclid::Point2D<f32, U>;
pub type Size<U> = euclid::Size2D<f32, U>;
pub type Rect<U> = euclid::Rect<f32, U>;

pub(crate) mod physical {
    #[derive(Clone)]
    pub(crate) struct Vec2<T> {
        pub x: T,
        pub y: T,
    }

    #[derive(Clone, Debug)]
    pub(crate) struct Rect<T> {
        pub top_left: Vec2<T>,
        pub size: Vec2<T>,
    }

    #[derive(Clone, Debug)]
    pub(crate) struct Sides<T> {
        pub top: T,
        pub left: T,
        pub bottom: T,
        pub right: T,
    }
}

pub(crate) mod flow_relative {
    #[derive(Clone)]
    pub(crate) struct Vec2<T> {
        pub inline: T,
        pub block: T,
    }

    #[derive(Clone)]
    pub(crate) struct Rect<T> {
        pub start_corner: Vec2<T>,
        pub size: Vec2<T>,
    }

    #[derive(Clone, Debug)]
    pub(crate) struct Sides<T> {
        pub inline_start: T,
        pub inline_end: T,
        pub block_start: T,
        pub block_end: T,
    }
}

impl<T: fmt::Debug> fmt::Debug for physical::Vec2<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Not using f.debug_struct on purpose here, to keep {:?} output somewhat compact
        f.write_str("Vec2 { x: ")?;
        self.x.fmt(f)?;
        f.write_str(", y: ")?;
        self.y.fmt(f)?;
        f.write_str(" }")
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

impl<T> Add<&'_ physical::Vec2<T>> for &'_ physical::Vec2<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = physical::Vec2<T>;

    fn add(self, other: &'_ physical::Vec2<T>) -> Self::Output {
        physical::Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: Clone> physical::Vec2<T> {
    pub fn size_to_flow_relative(&self, mode: WritingMode) -> flow_relative::Vec2<T> {
        // https://drafts.csswg.org/css-writing-modes/#logical-to-physical
        let (i, b) = if mode.is_horizontal() {
            (&self.x, &self.y)
        } else {
            (&self.y, &self.x)
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

impl flow_relative::Vec2<LengthPercentageOrAuto> {
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

impl flow_relative::Vec2<MaxSize<LengthPercentage>> {
    pub fn percentages_relative_to(
        &self,
        containing_block: &ContainingBlock,
    ) -> flow_relative::Vec2<Option<Length>> {
        flow_relative::Vec2 {
            inline: match self.inline {
                MaxSize::None => None,
                MaxSize::LengthPercentage(ref lp) => {
                    Some(lp.percentage_relative_to(containing_block.inline_size))
                },
            },
            block: match self.block {
                MaxSize::None => None,
                MaxSize::LengthPercentage(ref lp) => {
                    lp.maybe_percentage_relative_to(containing_block.block_size.non_auto())
                },
            },
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
    pub fn size_to_physical(&self, mode: WritingMode) -> physical::Vec2<T> {
        // https://drafts.csswg.org/css-writing-modes/#logical-to-physical
        let (x, y) = if mode.is_horizontal() {
            (&self.inline, &self.block)
        } else {
            (&self.block, &self.inline)
        };
        physical::Vec2 {
            x: x.clone(),
            y: y.clone(),
        }
    }
}

impl From<physical::Vec2<Length>> for Point<CSSPixel> {
    fn from(v: physical::Vec2<Length>) -> Self {
        Point::from_lengths(v.x.into(), v.y.into())
    }
}

impl<T: Clone> physical::Sides<T> {
    pub fn to_flow_relative(&self, mode: WritingMode) -> flow_relative::Sides<T> {
        // https://drafts.csswg.org/css-writing-modes/#logical-to-physical
        let block_flow = mode.block_flow_direction();
        let (bs, be) = match mode.block_flow_direction() {
            BlockFlowDirection::TopToBottom => (&self.top, &self.bottom),
            BlockFlowDirection::RightToLeft => (&self.right, &self.left),
            BlockFlowDirection::LeftToRight => (&self.left, &self.right),
        };
        use BlockFlowDirection::TopToBottom;
        let (is, ie) = match (block_flow, mode.inline_base_direction()) {
            (TopToBottom, InlineBaseDirection::LeftToRight) => (&self.left, &self.right),
            (TopToBottom, InlineBaseDirection::RightToLeft) => (&self.right, &self.left),
            (_, InlineBaseDirection::LeftToRight) => (&self.top, &self.bottom),
            (_, InlineBaseDirection::RightToLeft) => (&self.bottom, &self.top),
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
}

impl flow_relative::Sides<LengthPercentage> {
    pub fn percentages_relative_to(&self, basis: Length) -> flow_relative::Sides<Length> {
        self.map(|s| s.percentage_relative_to(basis))
    }
}

impl flow_relative::Sides<LengthPercentageOrAuto> {
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
        _containing_block: &physical::Rect<T>,
    ) -> physical::Rect<T>
    where
        T: Clone,
    {
        // Top-left corner
        let (tl_x, tl_y) = match mode.start_start_physical_corner() {
            PhysicalCorner::TopLeft => (&self.start_corner.inline, &self.start_corner.block),
            _ => unimplemented!(),
        };
        physical::Rect {
            top_left: physical::Vec2 {
                x: tl_x.clone(),
                y: tl_y.clone(),
            },
            size: self.size.size_to_physical(mode),
        }
    }
}

impl<T> physical::Rect<T> {
    pub fn translate(&self, by: &physical::Vec2<T>) -> Self
    where
        T: Add<Output = T> + Copy,
    {
        physical::Rect {
            top_left: &self.top_left + by,
            size: self.size.clone(),
        }
    }
}

impl From<physical::Rect<Length>> for Rect<CSSPixel> {
    fn from(r: physical::Rect<Length>) -> Self {
        Rect {
            origin: Point::new(r.top_left.x.px(), r.top_left.y.px()),
            size: Size::new(r.size.x.px(), r.size.y.px()),
        }
    }
}

impl From<physical::Rect<Length>> for webrender_api::units::LayoutRect {
    fn from(r: physical::Rect<Length>) -> Self {
        Rect {
            origin: Point::new(r.top_left.x.px(), r.top_left.y.px()),
            size: Size::new(r.size.x.px(), r.size.y.px()),
        }
    }
}
