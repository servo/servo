/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::From;
use std::fmt;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use app_units::Au;
use serde::Serialize;
use style::logical_geometry::{BlockFlowDirection, InlineBaseDirection, WritingMode};
use style::values::computed::{
    CSSPixelLength, LengthPercentage, MaxSize as StyleMaxSize, Size as StyleSize,
};
use style::values::generics::length::GenericLengthPercentageOrAuto as AutoOr;
use style::Zero;
use style_traits::CSSPixel;

use crate::sizing::ContentSizes;
use crate::ContainingBlock;

pub type PhysicalPoint<U> = euclid::Point2D<U, CSSPixel>;
pub type PhysicalSize<U> = euclid::Size2D<U, CSSPixel>;
pub type PhysicalVec<U> = euclid::Vector2D<U, CSSPixel>;
pub type PhysicalRect<U> = euclid::Rect<U, CSSPixel>;
pub type PhysicalSides<U> = euclid::SideOffsets2D<U, CSSPixel>;
pub type AuOrAuto = AutoOr<Au>;
pub type LengthPercentageOrAuto<'a> = AutoOr<&'a LengthPercentage>;

#[derive(Clone, Copy, PartialEq, Serialize)]
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

impl<T: Default> Default for LogicalVec2<T> {
    fn default() -> Self {
        Self {
            inline: T::default(),
            block: T::default(),
        }
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

impl LogicalVec2<Au> {
    #[inline]
    pub fn to_physical_point(
        &self,
        containing_block: Option<&ContainingBlock>,
    ) -> PhysicalPoint<Au> {
        let mode = containing_block.map_or_else(WritingMode::horizontal_tb, |containing_block| {
            containing_block.style.writing_mode
        });
        if mode.is_vertical() {
            // TODO: Bottom-to-top writing modes are not supported yet.
            PhysicalPoint::new(self.block, self.inline)
        } else {
            let y = self.block;
            let x = match containing_block {
                Some(containing_block) if !mode.is_bidi_ltr() => {
                    containing_block.inline_size - self.inline
                },
                _ => self.inline,
            };
            PhysicalPoint::new(x, y)
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
}

impl LogicalSides<&'_ LengthPercentage> {
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
    pub fn to_physical(&self, containing_block: Option<&ContainingBlock<'_>>) -> PhysicalRect<Au> {
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
                    containing_block.inline_size - self.max_inline_position()
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
                    containing_block.inline_size - self.x
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
                inline_start = containing_block.inline_size - (self.origin.x + self.size.width);
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
#[derive(Clone, PartialEq)]
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
    pub(crate) fn is_numeric(&self) -> bool {
        matches!(self, Self::Numeric(_))
    }

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
    pub(crate) fn to_auto_or(&self) -> AutoOr<T> {
        self.to_numeric()
            .map_or(AutoOr::Auto, AutoOr::LengthPercentage)
    }

    #[inline]
    pub fn map<U>(&self, f: impl FnOnce(T) -> U) -> Size<U> {
        match self {
            Size::Initial => Size::Initial,
            Size::MinContent => Size::MinContent,
            Size::MaxContent => Size::MaxContent,
            Size::FitContent => Size::FitContent,
            Size::Stretch => Size::Stretch,
            Size::Numeric(numeric) => Size::Numeric(f(numeric.clone())),
        }
    }

    #[inline]
    pub fn maybe_map<U>(&self, f: impl FnOnce(T) -> Option<U>) -> Option<Size<U>> {
        Some(match self {
            Size::Numeric(numeric) => Size::Numeric(f(numeric.clone())?),
            _ => self.map(|_| unreachable!("This shouldn't be called for keywords")),
        })
    }
}

impl From<StyleSize> for Size<LengthPercentage> {
    fn from(size: StyleSize) -> Self {
        match size {
            StyleSize::LengthPercentage(length) => Size::Numeric(length.0),
            StyleSize::Auto => Size::Initial,
            StyleSize::MinContent => Size::MinContent,
            StyleSize::MaxContent => Size::MaxContent,
            StyleSize::FitContent => Size::FitContent,
            StyleSize::Stretch => Size::Stretch,
            StyleSize::AnchorSizeFunction(_) => unreachable!("anchor-size() should be disabled"),
        }
    }
}

impl From<StyleMaxSize> for Size<LengthPercentage> {
    fn from(max_size: StyleMaxSize) -> Self {
        match max_size {
            StyleMaxSize::LengthPercentage(length) => Size::Numeric(length.0),
            StyleMaxSize::None => Size::Initial,
            StyleMaxSize::MinContent => Size::MinContent,
            StyleMaxSize::MaxContent => Size::MaxContent,
            StyleMaxSize::FitContent => Size::FitContent,
            StyleMaxSize::Stretch => Size::Stretch,
            StyleMaxSize::AnchorSizeFunction(_) => unreachable!("anchor-size() should be disabled"),
        }
    }
}

impl LogicalVec2<Size<LengthPercentage>> {
    pub(crate) fn percentages_relative_to(
        &self,
        containing_block: &ContainingBlock,
    ) -> LogicalVec2<Size<Au>> {
        LogicalVec2 {
            inline: self
                .inline
                .map(|lp| lp.to_used_value(containing_block.inline_size)),
            block: self
                .block
                .maybe_map(|lp| lp.maybe_to_used_value(containing_block.block_size.non_auto()))
                .unwrap_or_default(),
        }
    }

    pub(crate) fn maybe_percentages_relative_to_basis(
        &self,
        basis: &LogicalVec2<Option<Au>>,
    ) -> LogicalVec2<Size<Au>> {
        LogicalVec2 {
            inline: self
                .inline
                .maybe_map(|v| v.maybe_to_used_value(basis.inline))
                .unwrap_or_default(),
            block: self
                .block
                .maybe_map(|v| v.maybe_to_used_value(basis.block))
                .unwrap_or_default(),
        }
    }

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
    /// Resolves any size into a numerical value.
    #[inline]
    pub(crate) fn resolve(
        &self,
        initial_behavior: Self,
        stretch_size: Au,
        get_content_size: &mut impl FnMut() -> ContentSizes,
    ) -> Au {
        if self.is_initial() {
            assert!(!initial_behavior.is_initial());
            initial_behavior.resolve_non_initial(stretch_size, get_content_size)
        } else {
            self.resolve_non_initial(stretch_size, get_content_size)
        }
        .unwrap()
    }

    /// Resolves a non-initial size into a numerical value.
    /// Returns `None` if the size is the initial one.
    #[inline]
    pub(crate) fn resolve_non_initial(
        &self,
        stretch_size: Au,
        get_content_size: &mut impl FnMut() -> ContentSizes,
    ) -> Option<Au> {
        match self {
            Self::Initial => None,
            Self::MinContent => Some(get_content_size().min_content),
            Self::MaxContent => Some(get_content_size().max_content),
            Self::FitContent => Some(get_content_size().shrink_to_fit(stretch_size)),
            Self::Stretch => Some(stretch_size),
            Self::Numeric(numeric) => Some(*numeric),
        }
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
            Self::Initial | Self::MinContent | Self::MaxContent | Self::FitContent => None,
            Self::Stretch => stretch_size,
            Self::Numeric(numeric) => Some(*numeric),
        }
    }
}
