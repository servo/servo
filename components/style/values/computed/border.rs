/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to borders.

use app_units::Au;
use crate::values::animated::ToAnimatedZero;
use crate::values::computed::length::{LengthOrPercentage, NonNegativeLength};
use crate::values::computed::{Number, NumberOrPercentage};
use crate::values::generics::border::BorderCornerRadius as GenericBorderCornerRadius;
use crate::values::generics::border::BorderImageSideWidth as GenericBorderImageSideWidth;
use crate::values::generics::border::BorderImageSlice as GenericBorderImageSlice;
use crate::values::generics::border::BorderRadius as GenericBorderRadius;
use crate::values::generics::border::BorderSpacing as GenericBorderSpacing;
use crate::values::generics::rect::Rect;
use crate::values::generics::size::Size;

pub use crate::values::specified::border::BorderImageRepeat;

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
        GenericBorderSpacing(Size::new(
            NonNegativeLength::zero(),
            NonNegativeLength::zero(),
        ))
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
        GenericBorderCornerRadius(Size::new(
            LengthOrPercentage::zero(),
            LengthOrPercentage::zero(),
        ))
    }
}

impl ToAnimatedZero for BorderCornerRadius {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        // FIXME(nox): Why?
        Err(())
    }
}

impl BorderRadius {
    /// Returns whether all the values are `0px`.
    pub fn all_zero(&self) -> bool {
        fn all(corner: &BorderCornerRadius) -> bool {
            fn is_zero(l: &LengthOrPercentage) -> bool {
                *l == LengthOrPercentage::zero()
            }
            is_zero(corner.0.width()) && is_zero(corner.0.height())
        }
        all(&self.top_left) &&
            all(&self.top_right) &&
            all(&self.bottom_left) &&
            all(&self.bottom_right)
    }
}
