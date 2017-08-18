/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for text properties.

use values::{CSSInteger, CSSFloat};
use values::animated::{Animate, Procedure, ToAnimatedZero};
use values::computed::{NonNegativeAu, NonNegativeNumber};
use values::computed::length::{Length, LengthOrPercentage};
use values::generics::text::InitialLetter as GenericInitialLetter;
use values::generics::text::LineHeight as GenericLineHeight;
use values::generics::text::Spacing;

/// A computed value for the `initial-letter` property.
pub type InitialLetter = GenericInitialLetter<CSSFloat, CSSInteger>;

/// A computed value for the `letter-spacing` property.
pub type LetterSpacing = Spacing<Length>;

/// A computed value for the `word-spacing` property.
pub type WordSpacing = Spacing<LengthOrPercentage>;

/// A computed value for the `line-height` property.
pub type LineHeight = GenericLineHeight<NonNegativeNumber, NonNegativeAu>;

impl Animate for LineHeight {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (&GenericLineHeight::Length(ref this), &GenericLineHeight::Length(ref other)) => {
                Ok(GenericLineHeight::Length(this.animate(other, procedure)?))
            },
            (&GenericLineHeight::Number(ref this), &GenericLineHeight::Number(ref other)) => {
                Ok(GenericLineHeight::Number(this.animate(other, procedure)?))
            },
            (&GenericLineHeight::Normal, &GenericLineHeight::Normal) => {
                Ok(GenericLineHeight::Normal)
            },
            #[cfg(feature = "gecko")]
            (&GenericLineHeight::MozBlockHeight, &GenericLineHeight::MozBlockHeight) => {
                Ok(GenericLineHeight::MozBlockHeight)
            },
            _ => Err(()),
        }
    }
}

impl ToAnimatedZero for LineHeight {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
}
