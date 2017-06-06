/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for text properties.

use app_units::Au;
use properties::animated_properties::Animatable;
use values::{CSSInteger, CSSFloat};
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
pub type LineHeight = GenericLineHeight<CSSFloat, Au>;

impl Animatable for LineHeight {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (GenericLineHeight::Length(ref this), GenericLineHeight::Length(ref other)) => {
                this.add_weighted(other, self_portion, other_portion).map(GenericLineHeight::Length)
            },
            (GenericLineHeight::Number(ref this), GenericLineHeight::Number(ref other)) => {
                this.add_weighted(other, self_portion, other_portion).map(GenericLineHeight::Number)
            },
            (GenericLineHeight::Normal, GenericLineHeight::Normal) => {
                Ok(GenericLineHeight::Normal)
            },
            #[cfg(feature = "gecko")]
            (GenericLineHeight::MozBlockHeight, GenericLineHeight::MozBlockHeight) => {
                Ok(GenericLineHeight::MozBlockHeight)
            },
            _ => Err(()),
        }
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (GenericLineHeight::Length(ref this), GenericLineHeight::Length(ref other)) => {
                this.compute_distance(other)
            },
            (GenericLineHeight::Number(ref this), GenericLineHeight::Number(ref other)) => {
                this.compute_distance(other)
            },
            (GenericLineHeight::Normal, GenericLineHeight::Normal) => Ok(0.),
            #[cfg(feature = "gecko")]
            (GenericLineHeight::MozBlockHeight, GenericLineHeight::MozBlockHeight) => Ok(0.),
            _ => Err(()),
        }
    }
}
