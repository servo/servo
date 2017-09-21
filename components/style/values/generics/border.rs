/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to borders.

use std::fmt;
use style_traits::ToCss;
use values::generics::rect::Rect;
use values::generics::size::Size;

/// A generic value for a single side of a `border-image-width` property.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, ToComputedValue, ToCss)]
pub enum BorderImageSideWidth<LengthOrPercentage, Number> {
    /// `<length-or-percentage>`
    Length(LengthOrPercentage),
    /// `<number>`
    Number(Number),
    /// `auto`
    Auto,
}

/// A generic value for the `border-image-slice` property.
#[derive(Clone, Copy, Debug, PartialEq, ToComputedValue)]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct BorderImageSlice<NumberOrPercentage> {
    /// The offsets.
    pub offsets: Rect<NumberOrPercentage>,
    /// Whether to fill the middle part.
    pub fill: bool,
}

/// A generic value for the `border-*-radius` longhand properties.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug)]
#[derive(PartialEq, ToComputedValue, ToCss)]
pub struct BorderCornerRadius<L>(pub Size<L>);

impl<L> BorderCornerRadius<L> {
    /// Trivially create a `BorderCornerRadius`.
    pub fn new(w: L, h: L) -> Self {
        BorderCornerRadius(Size::new(w, h))
    }
}

/// A generic value for the `border-spacing` property.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug)]
#[derive(PartialEq, ToAnimatedValue, ToComputedValue, ToCss)]
pub struct BorderSpacing<L>(pub Size<L>);

impl<L> BorderSpacing<L> {
    /// Trivially create a `BorderCornerRadius`.
    pub fn new(w: L, h: L) -> Self {
        BorderSpacing(Size::new(w, h))
    }
}

/// A generic value for `border-radius`, `outline-radius` and `inset()`.
///
/// https://drafts.csswg.org/css-backgrounds-3/#border-radius
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug)]
#[derive(PartialEq, ToComputedValue)]
pub struct BorderRadius<LengthOrPercentage> {
    /// The top left radius.
    pub top_left: BorderCornerRadius<LengthOrPercentage>,
    /// The top right radius.
    pub top_right: BorderCornerRadius<LengthOrPercentage>,
    /// The bottom right radius.
    pub bottom_right: BorderCornerRadius<LengthOrPercentage>,
    /// The bottom left radius.
    pub bottom_left: BorderCornerRadius<LengthOrPercentage>,
}

impl<N> From<N> for BorderImageSlice<N>
    where N: Clone,
{
    #[inline]
    fn from(value: N) -> Self {
        Self {
            offsets: Rect::all(value),
            fill: false,
        }
    }
}

impl<N> ToCss for BorderImageSlice<N>
    where N: PartialEq + ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        self.offsets.to_css(dest)?;
        if self.fill {
            dest.write_str(" fill")?;
        }
        Ok(())
    }
}

impl<L> BorderRadius<L> {
    /// Returns a new `BorderRadius<L>`.
    #[inline]
    pub fn new(
        tl: BorderCornerRadius<L>,
        tr: BorderCornerRadius<L>,
        br: BorderCornerRadius<L>,
        bl: BorderCornerRadius<L>
    ) -> Self {
        BorderRadius {
            top_left: tl,
            top_right: tr,
            bottom_right: br,
            bottom_left: bl,
        }
    }
}

impl<L> BorderRadius<L>
    where L: PartialEq + ToCss
{
    /// Serialises two given rects following the syntax of the `border-radius``
    /// property.
    pub fn serialize_rects<W>(widths: Rect<&L>, heights: Rect<&L>, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        widths.to_css(dest)?;
        if widths.0 != heights.0 || widths.1 != heights.1 || widths.2 != heights.2 || widths.3 != heights.3 {
            dest.write_str(" / ")?;
            heights.to_css(dest)?;
        }
        Ok(())
    }
}

impl<L> ToCss for BorderRadius<L>
    where L: PartialEq + ToCss
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let BorderRadius {
            top_left: BorderCornerRadius(ref tl),
            top_right: BorderCornerRadius(ref tr),
            bottom_right: BorderCornerRadius(ref br),
            bottom_left: BorderCornerRadius(ref bl),
        } = *self;

        let widths = Rect::new(&tl.0.width, &tr.0.width, &br.0.width, &bl.0.width);
        let heights = Rect::new(&tl.0.height, &tr.0.height, &br.0.height, &bl.0.height);

        Self::serialize_rects(widths, heights, dest)
    }
}
