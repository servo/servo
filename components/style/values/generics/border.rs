/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to borders.

use euclid::Size2D;
use properties::animated_properties::Animatable;
use std::fmt;
use style_traits::ToCss;
use values::generics::rect::Rect;

/// A generic value for a single side of a `border-image-width` property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue, ToCss)]
pub enum BorderImageSideWidth<LengthOrPercentage, Number> {
    /// `<length-or-percentage>`
    Length(LengthOrPercentage),
    /// `<number>`
    Number(Number),
    /// `auto`
    Auto,
}

/// A generic value for the `border-image-slice` property.
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct BorderImageSlice<NumberOrPercentage> {
    /// The offsets.
    pub offsets: Rect<NumberOrPercentage>,
    /// Whether to fill the middle part.
    pub fill: bool,
}

/// A generic value for `border-radius`, `outline-radius` and `inset()`.
///
/// https://drafts.csswg.org/css-backgrounds-3/#border-radius
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
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

#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
/// A generic value for `border-*-radius` longhand properties.
pub struct BorderCornerRadius<L>(pub Size2D<L>);

impl<N> From<N> for BorderImageSlice<N>
    where N: Clone,
{
    #[inline]
    fn from(value: N) -> Self {
        Self {
            offsets: value.into(),
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
    pub fn new(tl: BorderCornerRadius<L>,
               tr: BorderCornerRadius<L>,
               br: BorderCornerRadius<L>,
               bl: BorderCornerRadius<L>)
               -> Self {
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

impl<L> Animatable for BorderRadius<L>
where
    L: Animatable + Copy,
{
    fn add_weighted(
        &self,
        other: &Self,
        self_portion: f64,
        other_portion: f64,
    ) -> Result<Self, ()> {
        let tl = self.top_left.add_weighted(&other.top_left, self_portion, other_portion)?;
        let tr = self.top_right.add_weighted(&other.top_right, self_portion, other_portion)?;
        let br = self.bottom_right.add_weighted(&other.bottom_right, self_portion, other_portion)?;
        let bl = self.bottom_left.add_weighted(&other.bottom_left, self_portion, other_portion)?;
        Ok(BorderRadius::new(tl, tr, br, bl))
    }

    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok(self.compute_squared_distance(other)?.sqrt())
    }

    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok(
            self.top_left.compute_squared_distance(&other.top_left)? +
            self.top_right.compute_squared_distance(&other.top_right)? +
            self.bottom_right.compute_squared_distance(&other.bottom_right)? +
            self.bottom_left.compute_squared_distance(&other.bottom_left)?,
        )
    }
}

impl<L> ToCss for BorderRadius<L>
    where L: PartialEq + ToCss
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let BorderRadius {
            top_left: ref tl,
            top_right: ref tr,
            bottom_right: ref br,
            bottom_left: ref bl,
        } = *self;

        let widths = Rect::new(&tl.0.width, &tr.0.width, &br.0.width, &bl.0.width);
        let heights = Rect::new(&tl.0.height, &tr.0.height, &br.0.height, &bl.0.height);

        Self::serialize_rects(widths, heights, dest)
    }
}

impl<L> BorderCornerRadius<L> {
    #[inline]
    /// Create a new `BorderCornerRadius` for an area of given width and height.
    pub fn new(width: L, height: L) -> BorderCornerRadius<L> {
        BorderCornerRadius(Size2D::new(width, height))
    }
}

impl<L: Clone> From<L> for BorderCornerRadius<L> {
    fn from(radius: L) -> Self {
        Self::new(radius.clone(), radius)
    }
}

impl<L> Animatable for BorderCornerRadius<L>
where
    L: Animatable + Copy,
{
    #[inline]
    fn add_weighted(
        &self,
        other: &Self,
        self_portion: f64,
        other_portion: f64,
    ) -> Result<Self, ()> {
        Ok(BorderCornerRadius(self.0.add_weighted(&other.0, self_portion, other_portion)?))
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_distance(&other.0)
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_squared_distance(&other.0)
    }
}

impl<L> ToCss for BorderCornerRadius<L>
    where L: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        self.0.width.to_css(dest)?;
        dest.write_str(" ")?;
        self.0.height.to_css(dest)
    }
}
