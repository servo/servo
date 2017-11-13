/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to borders.

use app_units::Au;
use std::fmt;
use style_traits::ToCss;
use values::animated::ToAnimatedZero;
use values::computed::{Context, Number, NumberOrPercentage, ToComputedValue};
use values::computed::length::{LengthOrPercentage, NonNegativeLength};
use values::generics::border::BorderCornerRadius as GenericBorderCornerRadius;
use values::generics::border::BorderImageSideWidth as GenericBorderImageSideWidth;
use values::generics::border::BorderImageSlice as GenericBorderImageSlice;
use values::generics::border::BorderRadius as GenericBorderRadius;
use values::generics::border::BorderSpacing as GenericBorderSpacing;
use values::generics::rect::Rect;
use values::generics::size::Size;
use values::specified::border::{BorderImageRepeat as SpecifiedBorderImageRepeat, RepeatKeyword};

/// A computed value for the `border-image-width` property.
pub type BorderImageWidth = Rect<BorderImageSideWidth>;

/// A computed value for a single side of a `border-image-width` property.
pub type BorderImageSideWidth = GenericBorderImageSideWidth<LengthOrPercentage, Number>;

/// A computed value for the `border-image-slice` property.
pub type BorderImageSlice = GenericBorderImageSlice<NumberOrPercentage>;

/// A computed value for the `border-radius` property.
pub type BorderRadius = GenericBorderRadius<LengthOrPercentage>;

/// A computed value for the `border-*-radius` longhand properties.
pub type BorderCornerRadius = GenericBorderCornerRadius<LengthOrPercentage>;

/// A computed value for the `border-spacing` longhand property.
pub type BorderSpacing = GenericBorderSpacing<NonNegativeLength>;

impl BorderImageSideWidth {
    /// Returns `1`.
    #[inline]
    pub fn one() -> Self {
        GenericBorderImageSideWidth::Number(1.)
    }
}

impl BorderSpacing {
    /// Returns `0 0`.
    pub fn zero() -> Self {
        GenericBorderSpacing(Size::new(NonNegativeLength::zero(), NonNegativeLength::zero()))
    }

    /// Returns the horizontal spacing.
    pub fn horizontal(&self) -> Au {
        Au::from(*self.0.width())
    }

    /// Returns the vertical spacing.
    pub fn vertical(&self) -> Au {
        Au::from(*self.0.height())
    }
}

impl BorderCornerRadius {
    /// Returns `0 0`.
    pub fn zero() -> Self {
        GenericBorderCornerRadius(Size::new(LengthOrPercentage::zero(), LengthOrPercentage::zero()))
    }
}

impl ToAnimatedZero for BorderSpacing {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        // FIXME(emilio): Why?
        Err(())
    }
}

impl ToAnimatedZero for BorderCornerRadius {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        // FIXME(nox): Why?
        Err(())
    }
}

/// The computed value of the `border-image-repeat` property:
///
/// https://drafts.csswg.org/css-backgrounds/#the-border-image-repeat
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct BorderImageRepeat(pub RepeatKeyword, pub RepeatKeyword);

impl BorderImageRepeat {
    /// Returns the `repeat repeat` value.
    pub fn repeat() -> Self {
        BorderImageRepeat(RepeatKeyword::Repeat, RepeatKeyword::Repeat)
    }
}

impl ToCss for BorderImageRepeat {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let BorderImageRepeat(horizontal, vertical) = *self;
        horizontal.to_css(dest)?;
        if horizontal != vertical {
            dest.write_str(" ")?;
            vertical.to_css(dest)?;
        }
        Ok(())
    }
}

impl ToComputedValue for SpecifiedBorderImageRepeat {
    type ComputedValue = BorderImageRepeat;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> Self::ComputedValue {
        match *self {
            SpecifiedBorderImageRepeat(horizontal, vertical) => {
                BorderImageRepeat(horizontal, vertical.unwrap_or(horizontal))
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        SpecifiedBorderImageRepeat(computed.0, Some(computed.1))
    }
}
