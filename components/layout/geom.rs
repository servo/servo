/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{LazyCell, OnceCell};
use std::convert::From;
use std::fmt;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use app_units::{Au, MAX_AU};
use malloc_size_of_derive::MallocSizeOf;
use style::Zero;
use style::logical_geometry::{BlockFlowDirection, Direction, InlineBaseDirection, WritingMode};
use style::values::computed::{
    CSSPixelLength, LengthPercentage, MaxSize as StyleMaxSize, Percentage, Size as StyleSize,
};
use style::values::generics::length::GenericLengthPercentageOrAuto as AutoOr;
use style_traits::CSSPixel;

use crate::ContainingBlock;
use crate::sizing::ContentSizes;
use crate::style_ext::Clamp;

pub type PhysicalPoint<U> = euclid::Point2D<U, CSSPixel>;
pub type PhysicalSize<U> = euclid::Size2D<U, CSSPixel>;
pub type PhysicalVec<U> = euclid::Vector2D<U, CSSPixel>;
pub type PhysicalRect<U> = euclid::Rect<U, CSSPixel>;
pub type PhysicalSides<U> = euclid::SideOffsets2D<U, CSSPixel>;
pub type AuOrAuto = AutoOr<Au>;
pub type LengthPercentageOrAuto<'a> = AutoOr<&'a LengthPercentage>;

#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub struct LogicalVec2<T> {
    pub inline: T,
    pub block: T,
}

#[derive(Clone, Copy)]
pub struct LogicalRect<T> {
    pub start_corner: LogicalVec2<T>,
    pub size: LogicalVec2<T>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LogicalSides<T> {
    pub inline_start: T,
    pub inline_end: T,
    pub block_start: T,
    pub block_end: T,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LogicalSides1D<T> {
    pub start: T,
    pub end: T,
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

impl<T: Default> Default for LogicalVec2<T> {
    fn default() -> Self {
        Self {
            inline: T::default(),
            block: T::default(),
        }
    }
}

impl<T: Copy> From<T> for LogicalVec2<T> {
    fn from(value: T) -> Self {
        Self {
            inline: value,
            block: value,
        }
    }
}

impl<T> LogicalVec2<T> {
    pub fn map_inline_and_block_axes<U>(
        &self,
        inline_f: impl FnOnce(&T) -> U,
        block_f: impl FnOnce(&T) -> U,
    ) -> LogicalVec2<U> {
        LogicalVec2 {
            inline: inline_f(&self.inline),
            block: block_f(&self.block),
        }
    }
}

impl<T: Clone> LogicalVec2<Size<T>> {
    pub fn map_inline_and_block_sizes<U>(
        &self,
        inline_f: impl FnOnce(T) -> U,
        block_f: impl FnOnce(T) -> U,
    ) -> LogicalVec2<Size<U>> {
        self.map_inline_and_block_axes(|size| size.map(inline_f), |size| size.map(block_f))
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

    pub(crate) fn map_with<U, V>(
        &self,
        other: &LogicalVec2<U>,
        f: impl Fn(&T, &U) -> V,
    ) -> LogicalVec2<V> {
        LogicalVec2 {
            inline: f(&self.inline, &other.inline),
            block: f(&self.block, &other.block),
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
            self.size.inline.to_f32_px(),
            self.size.block.to_f32_px(),
            self.start_corner.inline.to_f32_px(),
            self.start_corner.block.to_f32_px(),
        )
    }
}

impl<T: Clone> LogicalVec2<T> {
    pub fn to_physical_size(&self, mode: WritingMode) -> PhysicalSize<T> {
        // https://drafts.csswg.org/css-writing-modes/#logical-to-physical
        let (x, y) = if mode.is_horizontal() {
            (&self.inline, &self.block)
        } else {
            (&self.block, &self.inline)
        };
        PhysicalSize::new(x.clone(), y.clone())
    }
}

impl<T: Copy + Neg<Output = T>> LogicalVec2<T> {
    pub fn to_physical_vector(&self, mode: WritingMode) -> PhysicalVec<T> {
        if mode.is_horizontal() {
            if mode.is_bidi_ltr() {
                PhysicalVec::new(self.inline, self.block)
            } else {
                PhysicalVec::new(-self.inline, self.block)
            }
        } else if mode.is_inline_tb() {
            PhysicalVec::new(self.block, self.inline)
        } else {
            PhysicalVec::new(-self.block, self.inline)
        }
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

    #[inline]
    pub(crate) fn inline_sides(&self) -> LogicalSides1D<T> {
        LogicalSides1D::new(self.inline_start, self.inline_end)
    }

    #[inline]
    pub(crate) fn block_sides(&self) -> LogicalSides1D<T> {
        LogicalSides1D::new(self.block_start, self.block_end)
    }
}

impl LogicalSides<LengthPercentage> {
    pub fn percentages_relative_to(&self, basis: Au) -> LogicalSides<Au> {
        self.map(|value| value.to_used_value(basis))
    }
}

impl LogicalSides<LengthPercentageOrAuto<'_>> {
    pub fn percentages_relative_to(&self, basis: Au) -> LogicalSides<AuOrAuto> {
        self.map(|value| value.map(|value| value.to_used_value(basis)))
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

impl<T> LogicalSides1D<T> {
    #[inline]
    pub(crate) fn new(start: T, end: T) -> Self {
        Self { start, end }
    }
}

impl<T> LogicalSides1D<AutoOr<T>> {
    #[inline]
    pub(crate) fn either_specified(&self) -> bool {
        !self.start.is_auto() || !self.end.is_auto()
    }

    #[inline]
    pub(crate) fn either_auto(&self) -> bool {
        self.start.is_auto() || self.end.is_auto()
    }
}

impl<T: Add + Copy> LogicalSides1D<T> {
    #[inline]
    pub(crate) fn sum(&self) -> T::Output {
        self.start + self.end
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
}

impl LogicalRect<Au> {
    pub(crate) fn as_physical(
        &self,
        containing_block: Option<&ContainingBlock<'_>>,
    ) -> PhysicalRect<Au> {
        let mode = containing_block.map_or_else(WritingMode::horizontal_tb, |containing_block| {
            containing_block.style.writing_mode
        });
        let (x, y, width, height) = if mode.is_vertical() {
            // TODO: Bottom-to-top writing modes are not supported.
            (
                self.start_corner.block,
                self.start_corner.inline,
                self.size.block,
                self.size.inline,
            )
        } else {
            let y = self.start_corner.block;
            let x = match containing_block {
                Some(containing_block) if !mode.is_bidi_ltr() => {
                    containing_block.size.inline - self.max_inline_position()
                },
                _ => self.start_corner.inline,
            };
            (x, y, self.size.inline, self.size.block)
        };

        PhysicalRect::new(PhysicalPoint::new(x, y), PhysicalSize::new(width, height))
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

pub(crate) trait ToLogical<Unit, LogicalType> {
    fn to_logical(&self, writing_mode: WritingMode) -> LogicalType;
}

impl<Unit: Copy> ToLogical<Unit, LogicalVec2<Unit>> for PhysicalSize<Unit> {
    fn to_logical(&self, writing_mode: WritingMode) -> LogicalVec2<Unit> {
        LogicalVec2::from_physical_size(self, writing_mode)
    }
}

impl<Unit: Copy> ToLogical<Unit, LogicalSides<Unit>> for PhysicalSides<Unit> {
    fn to_logical(&self, writing_mode: WritingMode) -> LogicalSides<Unit> {
        LogicalSides::from_physical(self, writing_mode)
    }
}

pub(crate) trait ToLogicalWithContainingBlock<LogicalType> {
    fn to_logical(&self, containing_block: &ContainingBlock) -> LogicalType;
}

impl ToLogicalWithContainingBlock<LogicalVec2<Au>> for PhysicalPoint<Au> {
    fn to_logical(&self, containing_block: &ContainingBlock) -> LogicalVec2<Au> {
        let writing_mode = containing_block.style.writing_mode;
        // TODO: Bottom-to-top and right-to-left vertical writing modes are not supported yet.
        if writing_mode.is_vertical() {
            LogicalVec2 {
                inline: self.y,
                block: self.x,
            }
        } else {
            LogicalVec2 {
                inline: if writing_mode.is_bidi_ltr() {
                    self.x
                } else {
                    containing_block.size.inline - self.x
                },
                block: self.y,
            }
        }
    }
}

impl ToLogicalWithContainingBlock<LogicalRect<Au>> for PhysicalRect<Au> {
    fn to_logical(&self, containing_block: &ContainingBlock) -> LogicalRect<Au> {
        let inline_start;
        let block_start;
        let inline;
        let block;

        let writing_mode = containing_block.style.writing_mode;
        if writing_mode.is_vertical() {
            // TODO: Bottom-to-top and right-to-left vertical writing modes are not supported yet.
            inline = self.size.height;
            block = self.size.width;
            block_start = self.origin.x;
            inline_start = self.origin.y;
        } else {
            inline = self.size.width;
            block = self.size.height;
            block_start = self.origin.y;
            if writing_mode.is_bidi_ltr() {
                inline_start = self.origin.x;
            } else {
                inline_start = containing_block.size.inline - (self.origin.x + self.size.width);
            }
        }
        LogicalRect {
            start_corner: LogicalVec2 {
                inline: inline_start,
                block: block_start,
            },
            size: LogicalVec2 { inline, block },
        }
    }
}

/// The possible values accepted by the sizing properties.
/// <https://drafts.csswg.org/css-sizing/#sizing-properties>
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Size<T> {
    /// Represents an `auto` value for the preferred and minimum size properties,
    /// or `none` for the maximum size properties.
    /// <https://drafts.csswg.org/css-sizing/#valdef-width-auto>
    /// <https://drafts.csswg.org/css-sizing/#valdef-max-width-none>
    Initial,
    /// <https://drafts.csswg.org/css-sizing/#valdef-width-min-content>
    MinContent,
    /// <https://drafts.csswg.org/css-sizing/#valdef-width-max-content>
    MaxContent,
    /// <https://drafts.csswg.org/css-sizing-4/#valdef-width-fit-content>
    FitContent,
    /// <https://drafts.csswg.org/css-sizing-3/#funcdef-width-fit-content>
    FitContentFunction(T),
    /// <https://drafts.csswg.org/css-sizing-4/#valdef-width-stretch>
    Stretch,
    /// Represents a numeric `<length-percentage>`, but resolved as a `T`.
    /// <https://drafts.csswg.org/css-sizing/#valdef-width-length-percentage-0>
    Numeric(T),
}

impl<T: Copy> Copy for Size<T> {}

impl<T> Default for Size<T> {
    #[inline]
    fn default() -> Self {
        Self::Initial
    }
}

impl<T> Size<T> {
    #[inline]
    pub(crate) fn is_initial(&self) -> bool {
        matches!(self, Self::Initial)
    }
}

impl<T: Clone> Size<T> {
    #[inline]
    pub(crate) fn to_numeric(&self) -> Option<T> {
        match self {
            Self::Numeric(numeric) => Some(numeric).cloned(),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn map<U>(&self, f: impl FnOnce(T) -> U) -> Size<U> {
        match self {
            Size::Initial => Size::Initial,
            Size::MinContent => Size::MinContent,
            Size::MaxContent => Size::MaxContent,
            Size::FitContent => Size::FitContent,
            Size::FitContentFunction(size) => Size::FitContentFunction(f(size.clone())),
            Size::Stretch => Size::Stretch,
            Size::Numeric(numeric) => Size::Numeric(f(numeric.clone())),
        }
    }
}

impl From<StyleSize> for Size<LengthPercentage> {
    fn from(size: StyleSize) -> Self {
        match size {
            StyleSize::LengthPercentage(lp) => Size::Numeric(lp.0),
            StyleSize::Auto => Size::Initial,
            StyleSize::MinContent => Size::MinContent,
            StyleSize::MaxContent => Size::MaxContent,
            StyleSize::FitContent => Size::FitContent,
            StyleSize::FitContentFunction(lp) => Size::FitContentFunction(lp.0),
            StyleSize::Stretch => Size::Stretch,
            StyleSize::AnchorSizeFunction(_) | StyleSize::AnchorContainingCalcFunction(_) => {
                unreachable!("anchor-size() should be disabled")
            },
        }
    }
}

impl From<StyleMaxSize> for Size<LengthPercentage> {
    fn from(max_size: StyleMaxSize) -> Self {
        match max_size {
            StyleMaxSize::LengthPercentage(lp) => Size::Numeric(lp.0),
            StyleMaxSize::None => Size::Initial,
            StyleMaxSize::MinContent => Size::MinContent,
            StyleMaxSize::MaxContent => Size::MaxContent,
            StyleMaxSize::FitContent => Size::FitContent,
            StyleMaxSize::FitContentFunction(lp) => Size::FitContentFunction(lp.0),
            StyleMaxSize::Stretch => Size::Stretch,
            StyleMaxSize::AnchorSizeFunction(_) | StyleMaxSize::AnchorContainingCalcFunction(_) => {
                unreachable!("anchor-size() should be disabled")
            },
        }
    }
}

impl Size<LengthPercentage> {
    #[inline]
    pub(crate) fn to_percentage(&self) -> Option<Percentage> {
        self.to_numeric()
            .and_then(|length_percentage| length_percentage.to_percentage())
    }

    /// Resolves percentages in a preferred size, against the provided basis.
    /// If the basis is missing, percentages are considered cyclic.
    /// <https://www.w3.org/TR/css-sizing-3/#preferred-size-properties>
    /// <https://www.w3.org/TR/css-sizing-3/#cyclic-percentage-size>
    #[inline]
    pub(crate) fn resolve_percentages_for_preferred(&self, basis: Option<Au>) -> Size<Au> {
        match self {
            Size::Numeric(numeric) => numeric
                .maybe_to_used_value(basis)
                .map_or(Size::Initial, Size::Numeric),
            Size::FitContentFunction(numeric) => {
                // Under discussion in https://github.com/w3c/csswg-drafts/issues/11805
                numeric
                    .maybe_to_used_value(basis)
                    .map_or(Size::FitContent, Size::FitContentFunction)
            },
            _ => self.map(|_| unreachable!("This shouldn't be called for keywords")),
        }
    }

    /// Resolves percentages in a maximum size, against the provided basis.
    /// If the basis is missing, percentages are considered cyclic.
    /// <https://www.w3.org/TR/css-sizing-3/#preferred-size-properties>
    /// <https://www.w3.org/TR/css-sizing-3/#cyclic-percentage-size>
    #[inline]
    pub(crate) fn resolve_percentages_for_max(&self, basis: Option<Au>) -> Size<Au> {
        match self {
            Size::Numeric(numeric) => numeric
                .maybe_to_used_value(basis)
                .map_or(Size::Initial, Size::Numeric),
            Size::FitContentFunction(numeric) => {
                // Under discussion in https://github.com/w3c/csswg-drafts/issues/11805
                numeric
                    .maybe_to_used_value(basis)
                    .map_or(Size::MaxContent, Size::FitContentFunction)
            },
            _ => self.map(|_| unreachable!("This shouldn't be called for keywords")),
        }
    }
}

impl LogicalVec2<Size<LengthPercentage>> {
    pub(crate) fn percentages_relative_to_basis(
        &self,
        basis: &LogicalVec2<Au>,
    ) -> LogicalVec2<Size<Au>> {
        LogicalVec2 {
            inline: self.inline.map(|value| value.to_used_value(basis.inline)),
            block: self.block.map(|value| value.to_used_value(basis.block)),
        }
    }
}

impl Size<Au> {
    /// Resolves a preferred size into a numerical value.
    /// <https://www.w3.org/TR/css-sizing-3/#preferred-size-properties>
    #[inline]
    pub(crate) fn resolve_for_preferred<F: FnOnce() -> ContentSizes>(
        &self,
        automatic_size: Size<Au>,
        stretch_size: Option<Au>,
        content_size: &LazyCell<ContentSizes, F>,
    ) -> Au {
        match self {
            Self::Initial => {
                assert!(!automatic_size.is_initial());
                automatic_size.resolve_for_preferred(automatic_size, stretch_size, content_size)
            },
            Self::MinContent => content_size.min_content,
            Self::MaxContent => content_size.max_content,
            Self::FitContentFunction(size) => content_size.shrink_to_fit(*size),
            Self::FitContent => {
                content_size.shrink_to_fit(stretch_size.unwrap_or_else(|| content_size.max_content))
            },
            Self::Stretch => stretch_size.unwrap_or_else(|| content_size.max_content),
            Self::Numeric(numeric) => *numeric,
        }
    }

    /// Resolves a minimum size into a numerical value.
    /// <https://www.w3.org/TR/css-sizing-3/#min-size-properties>
    #[inline]
    pub(crate) fn resolve_for_min<F: FnOnce() -> ContentSizes>(
        &self,
        get_automatic_minimum_size: impl FnOnce() -> Au,
        stretch_size: Option<Au>,
        content_size: &LazyCell<ContentSizes, F>,
        is_table: bool,
    ) -> Au {
        let result = match self {
            Self::Initial => get_automatic_minimum_size(),
            Self::MinContent => content_size.min_content,
            Self::MaxContent => content_size.max_content,
            Self::FitContentFunction(size) => content_size.shrink_to_fit(*size),
            Self::FitContent => content_size.shrink_to_fit(stretch_size.unwrap_or_default()),
            Self::Stretch => stretch_size.unwrap_or_default(),
            Self::Numeric(numeric) => *numeric,
        };
        if is_table {
            // In addition to the specified minimum, the inline size of a table is forced to be
            // at least as big as its min-content size.
            //
            // Note that if there are collapsed columns, only the inline size of the table grid will
            // shrink, while the size of the table wrapper (being computed here) won't be affected.
            // However, collapsed rows should typically affect the block size of the table wrapper,
            // so it might be wrong to use this function for that case.
            // This is being discussed in https://github.com/w3c/csswg-drafts/issues/11408
            result.max(content_size.min_content)
        } else {
            result
        }
    }

    /// Resolves a maximum size into a numerical value.
    /// <https://www.w3.org/TR/css-sizing-3/#max-size-properties>
    #[inline]
    pub(crate) fn resolve_for_max<F: FnOnce() -> ContentSizes>(
        &self,
        stretch_size: Option<Au>,
        content_size: &LazyCell<ContentSizes, F>,
    ) -> Option<Au> {
        Some(match self {
            Self::Initial => return None,
            Self::MinContent => content_size.min_content,
            Self::MaxContent => content_size.max_content,
            Self::FitContentFunction(size) => content_size.shrink_to_fit(*size),
            Self::FitContent => content_size.shrink_to_fit(stretch_size.unwrap_or(MAX_AU)),
            Self::Stretch => return stretch_size,
            Self::Numeric(numeric) => *numeric,
        })
    }

    /// Tries to resolve an extrinsic size into a numerical value.
    /// Extrinsic sizes are those based on the context of an element, without regard for its contents.
    /// <https://drafts.csswg.org/css-sizing-3/#extrinsic>
    ///
    /// Returns `None` if either:
    /// - The size is intrinsic.
    /// - The size is the initial one.
    ///   TODO: should we allow it to behave as `stretch` instead of assuming it's intrinsic?
    /// - The provided `stretch_size` is `None` but we need its value.
    #[inline]
    pub(crate) fn maybe_resolve_extrinsic(&self, stretch_size: Option<Au>) -> Option<Au> {
        match self {
            Self::Initial |
            Self::MinContent |
            Self::MaxContent |
            Self::FitContent |
            Self::FitContentFunction(_) => None,
            Self::Stretch => stretch_size,
            Self::Numeric(numeric) => Some(*numeric),
        }
    }
}

/// Represents the sizing constraint that the preferred, min and max sizing properties
/// impose on one axis.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum SizeConstraint {
    /// Represents a definite preferred size, clamped by minimum and maximum sizes (if any).
    Definite(Au),
    /// Represents an indefinite preferred size that allows a range of values between
    /// the first argument (minimum size) and the second one (maximum size).
    MinMax(Au, Option<Au>),
}

impl Default for SizeConstraint {
    #[inline]
    fn default() -> Self {
        Self::MinMax(Au::default(), None)
    }
}

impl SizeConstraint {
    #[inline]
    pub(crate) fn new(preferred_size: Option<Au>, min_size: Au, max_size: Option<Au>) -> Self {
        preferred_size.map_or_else(
            || Self::MinMax(min_size, max_size),
            |size| Self::Definite(size.clamp_between_extremums(min_size, max_size)),
        )
    }

    #[inline]
    pub(crate) fn is_definite(self) -> bool {
        matches!(self, Self::Definite(_))
    }

    #[inline]
    pub(crate) fn to_definite(self) -> Option<Au> {
        match self {
            Self::Definite(size) => Some(size),
            _ => None,
        }
    }
}

impl From<Option<Au>> for SizeConstraint {
    fn from(size: Option<Au>) -> Self {
        size.map(SizeConstraint::Definite).unwrap_or_default()
    }
}

#[derive(Clone, Default)]
pub(crate) struct Sizes {
    /// <https://drafts.csswg.org/css-sizing-3/#preferred-size-properties>
    pub preferred: Size<Au>,
    /// <https://drafts.csswg.org/css-sizing-3/#min-size-properties>
    pub min: Size<Au>,
    /// <https://drafts.csswg.org/css-sizing-3/#max-size-properties>
    pub max: Size<Au>,
}

impl Sizes {
    #[inline]
    pub(crate) fn new(preferred: Size<Au>, min: Size<Au>, max: Size<Au>) -> Self {
        Self {
            preferred,
            min,
            max,
        }
    }

    /// Resolves the three sizes into a single numerical value.
    #[inline]
    pub(crate) fn resolve(
        &self,
        axis: Direction,
        automatic_size: Size<Au>,
        get_automatic_minimum_size: impl FnOnce() -> Au,
        stretch_size: Option<Au>,
        get_content_size: impl FnOnce() -> ContentSizes,
        is_table: bool,
    ) -> Au {
        if is_table && axis == Direction::Block {
            // The intrinsic block size of a table already takes sizing properties into account,
            // but it can be a smaller amount if there are collapsed rows.
            // Therefore, disregard sizing properties and just defer to the intrinsic size.
            // This is being discussed in https://github.com/w3c/csswg-drafts/issues/11408
            return get_content_size().max_content;
        }
        let (preferred, min, max) = self.resolve_each(
            automatic_size,
            get_automatic_minimum_size,
            stretch_size,
            get_content_size,
            is_table,
        );
        preferred.clamp_between_extremums(min, max)
    }

    /// Resolves each of the three sizes into a numerical value, separately.
    /// - The 1st returned value is the resolved preferred size.
    /// - The 2nd returned value is the resolved minimum size.
    /// - The 3rd returned value is the resolved maximum size. `None` means no maximum.
    #[inline]
    pub(crate) fn resolve_each(
        &self,
        automatic_size: Size<Au>,
        get_automatic_minimum_size: impl FnOnce() -> Au,
        stretch_size: Option<Au>,
        get_content_size: impl FnOnce() -> ContentSizes,
        is_table: bool,
    ) -> (Au, Au, Option<Au>) {
        // The provided `get_content_size` is a FnOnce but we may need its result multiple times.
        // A LazyCell will only invoke it once if needed, and then reuse the result.
        let content_size = LazyCell::new(get_content_size);
        (
            self.preferred
                .resolve_for_preferred(automatic_size, stretch_size, &content_size),
            self.min.resolve_for_min(
                get_automatic_minimum_size,
                stretch_size,
                &content_size,
                is_table,
            ),
            self.max.resolve_for_max(stretch_size, &content_size),
        )
    }

    /// Tries to extrinsically resolve the three sizes into a single [`SizeConstraint`].
    /// Values that are intrinsic or need `stretch_size` when it's `None` are handled as such:
    /// - On the preferred size, they make the returned value be an indefinite [`SizeConstraint::MinMax`].
    /// - On the min size, they are treated as `auto`, enforcing the automatic minimum size.
    /// - On the max size, they are treated as `none`, enforcing no maximum.
    #[inline]
    pub(crate) fn resolve_extrinsic(
        &self,
        automatic_size: Size<Au>,
        automatic_minimum_size: Au,
        stretch_size: Option<Au>,
    ) -> SizeConstraint {
        let (preferred, min, max) =
            self.resolve_each_extrinsic(automatic_size, automatic_minimum_size, stretch_size);
        SizeConstraint::new(preferred, min, max)
    }

    /// Tries to extrinsically resolve each of the three sizes into a numerical value, separately.
    /// This can't resolve values that are intrinsic or need `stretch_size` but it's `None`.
    /// - The 1st returned value is the resolved preferred size. If it can't be resolved then
    ///   the returned value is `None`. Note that this is different than treating it as `auto`.
    ///   TODO: This needs to be discussed in <https://github.com/w3c/csswg-drafts/issues/11387>.
    /// - The 2nd returned value is the resolved minimum size. If it can't be resolved then we
    ///   treat it as the initial `auto`, returning the automatic minimum size.
    /// - The 3rd returned value is the resolved maximum size. If it can't be resolved then we
    ///   treat it as the initial `none`, returning `None`.
    #[inline]
    pub(crate) fn resolve_each_extrinsic(
        &self,
        automatic_size: Size<Au>,
        automatic_minimum_size: Au,
        stretch_size: Option<Au>,
    ) -> (Option<Au>, Au, Option<Au>) {
        (
            if self.preferred.is_initial() {
                automatic_size.maybe_resolve_extrinsic(stretch_size)
            } else {
                self.preferred.maybe_resolve_extrinsic(stretch_size)
            },
            self.min
                .maybe_resolve_extrinsic(stretch_size)
                .unwrap_or(automatic_minimum_size),
            self.max.maybe_resolve_extrinsic(stretch_size),
        )
    }
}

struct LazySizeData<'a> {
    sizes: &'a Sizes,
    axis: Direction,
    automatic_size: Size<Au>,
    get_automatic_minimum_size: fn() -> Au,
    stretch_size: Option<Au>,
    is_table: bool,
}

/// Represents a size that can't be fully resolved until the intrinsic size
/// is known. This is useful in the block axis, since the intrinsic size
/// depends on layout, but the other inputs are known beforehand.
pub(crate) struct LazySize<'a> {
    result: OnceCell<Au>,
    data: Option<LazySizeData<'a>>,
}

impl<'a> LazySize<'a> {
    pub(crate) fn new(
        sizes: &'a Sizes,
        axis: Direction,
        automatic_size: Size<Au>,
        get_automatic_minimum_size: fn() -> Au,
        stretch_size: Option<Au>,
        is_table: bool,
    ) -> Self {
        Self {
            result: OnceCell::new(),
            data: Some(LazySizeData {
                sizes,
                axis,
                automatic_size,
                get_automatic_minimum_size,
                stretch_size,
                is_table,
            }),
        }
    }

    /// Creates a [`LazySize`] that will resolve to the intrinsic size.
    /// Should be equivalent to [`LazySize::new()`] with default parameters,
    /// but avoiding the trouble of getting a reference to a [`Sizes::default()`]
    /// which lives long enough.
    ///
    /// TODO: It's not clear what this should do if/when [`LazySize::resolve()`]
    /// is changed to accept a [`ContentSizes`] as the intrinsic size.
    pub(crate) fn intrinsic() -> Self {
        Self {
            result: OnceCell::new(),
            data: None,
        }
    }

    /// Resolves the [`LazySize`] into [`Au`], caching the result.
    /// The argument is a callback that computes the intrinsic size lazily.
    ///
    /// TODO: The intrinsic size should probably be a [`ContentSizes`] instead of [`Au`].
    pub(crate) fn resolve(&self, get_content_size: impl FnOnce() -> Au) -> Au {
        *self.result.get_or_init(|| {
            let Some(ref data) = self.data else {
                return get_content_size();
            };
            data.sizes.resolve(
                data.axis,
                data.automatic_size,
                data.get_automatic_minimum_size,
                data.stretch_size,
                || get_content_size().into(),
                data.is_table,
            )
        })
    }
}

impl From<Au> for LazySize<'_> {
    /// Creates a [`LazySize`] that will resolve to the given [`Au`],
    /// ignoring the intrinsic size.
    fn from(value: Au) -> Self {
        let result = OnceCell::new();
        result.set(value).unwrap();
        LazySize { result, data: None }
    }
}
