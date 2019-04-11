/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to borders.

use crate::values::generics::rect::Rect;
use crate::values::generics::size::Size2D;
use crate::Zero;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// A generic value for a single side of a `border-image-width` property.
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum BorderImageSideWidth<LengthPercentage, Number> {
    /// `<length-or-percentage>`
    Length(LengthPercentage),
    /// `<number>`
    Number(Number),
    /// `auto`
    Auto,
}

/// A generic value for the `border-image-slice` property.
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericBorderImageSlice<NumberOrPercentage> {
    /// The offsets.
    #[css(field_bound)]
    pub offsets: Rect<NumberOrPercentage>,
    /// Whether to fill the middle part.
    #[css(represents_keyword)]
    pub fill: bool,
}

pub use self::GenericBorderImageSlice as BorderImageSlice;

/// A generic value for the `border-*-radius` longhand properties.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericBorderCornerRadius<L>(
    #[css(field_bound)]
    #[shmem(field_bound)]
    pub Size2D<L>,
);

pub use self::GenericBorderCornerRadius as BorderCornerRadius;

impl<L> BorderCornerRadius<L> {
    /// Trivially create a `BorderCornerRadius`.
    pub fn new(w: L, h: L) -> Self {
        BorderCornerRadius(Size2D::new(w, h))
    }
}

impl<L: Zero> Zero for BorderCornerRadius<L> {
    fn zero() -> Self {
        BorderCornerRadius(Size2D::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

/// A generic value for the `border-spacing` property.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
pub struct BorderSpacing<L>(
    #[css(field_bound)]
    #[shmem(field_bound)]
    pub Size2D<L>,
);

impl<L> BorderSpacing<L> {
    /// Trivially create a `BorderCornerRadius`.
    pub fn new(w: L, h: L) -> Self {
        BorderSpacing(Size2D::new(w, h))
    }
}

/// A generic value for `border-radius`, `outline-radius` and `inset()`.
///
/// <https://drafts.csswg.org/css-backgrounds-3/#border-radius>
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericBorderRadius<LengthPercentage> {
    /// The top left radius.
    #[shmem(field_bound)]
    pub top_left: GenericBorderCornerRadius<LengthPercentage>,
    /// The top right radius.
    pub top_right: GenericBorderCornerRadius<LengthPercentage>,
    /// The bottom right radius.
    pub bottom_right: GenericBorderCornerRadius<LengthPercentage>,
    /// The bottom left radius.
    pub bottom_left: GenericBorderCornerRadius<LengthPercentage>,
}

pub use self::GenericBorderRadius as BorderRadius;

impl<L> BorderRadius<L> {
    /// Returns a new `BorderRadius<L>`.
    #[inline]
    pub fn new(
        tl: BorderCornerRadius<L>,
        tr: BorderCornerRadius<L>,
        br: BorderCornerRadius<L>,
        bl: BorderCornerRadius<L>,
    ) -> Self {
        BorderRadius {
            top_left: tl,
            top_right: tr,
            bottom_right: br,
            bottom_left: bl,
        }
    }

    /// Serialises two given rects following the syntax of the `border-radius``
    /// property.
    pub fn serialize_rects<W>(
        widths: Rect<&L>,
        heights: Rect<&L>,
        dest: &mut CssWriter<W>,
    ) -> fmt::Result
    where
        L: PartialEq + ToCss,
        W: Write,
    {
        widths.to_css(dest)?;
        if widths != heights {
            dest.write_str(" / ")?;
            heights.to_css(dest)?;
        }
        Ok(())
    }
}

impl<L: Zero> Zero for BorderRadius<L> {
    fn zero() -> Self {
        Self::new(
            BorderCornerRadius::<L>::zero(),
            BorderCornerRadius::<L>::zero(),
            BorderCornerRadius::<L>::zero(),
            BorderCornerRadius::<L>::zero(),
        )
    }

    fn is_zero(&self) -> bool {
        self.top_left.is_zero() &&
            self.top_right.is_zero() &&
            self.bottom_right.is_zero() &&
            self.bottom_left.is_zero()
    }
}

impl<L> ToCss for BorderRadius<L>
where
    L: PartialEq + ToCss,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        let BorderRadius {
            top_left: BorderCornerRadius(ref tl),
            top_right: BorderCornerRadius(ref tr),
            bottom_right: BorderCornerRadius(ref br),
            bottom_left: BorderCornerRadius(ref bl),
        } = *self;

        let widths = Rect::new(&tl.width, &tr.width, &br.width, &bl.width);
        let heights = Rect::new(&tl.height, &tr.height, &br.height, &bl.height);

        Self::serialize_rects(widths, heights, dest)
    }
}
