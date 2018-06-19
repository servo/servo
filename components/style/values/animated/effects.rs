/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated types for CSS values related to effects.

#[cfg(not(feature = "gecko"))]
use values::Impossible;
use values::animated::color::Color;
use values::computed::{Angle, Number};
use values::computed::length::Length;
#[cfg(feature = "gecko")]
use values::computed::url::ComputedUrl;
use values::distance::{ComputeSquaredDistance, SquaredDistance};
use values::generics::effects::BoxShadow as GenericBoxShadow;
use values::generics::effects::Filter as GenericFilter;
use values::generics::effects::SimpleShadow as GenericSimpleShadow;

/// An animated value for a single `box-shadow`.
pub type BoxShadow = GenericBoxShadow<Color, Length, Length, Length>;

/// An animated value for a single `filter`.
#[cfg(feature = "gecko")]
pub type Filter = GenericFilter<Angle, Number, Length, SimpleShadow, ComputedUrl>;

/// An animated value for a single `filter`.
#[cfg(not(feature = "gecko"))]
pub type Filter = GenericFilter<Angle, Number, Length, Impossible, Impossible>;

/// An animated value for the `drop-shadow()` filter.
pub type SimpleShadow = GenericSimpleShadow<Color, Length, Length>;

impl ComputeSquaredDistance for BoxShadow {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        if self.inset != other.inset {
            return Err(());
        }
        Ok(self.base.compute_squared_distance(&other.base)? +
            self.spread.compute_squared_distance(&other.spread)?)
    }
}
