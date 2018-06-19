/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to effects.

#[cfg(not(feature = "gecko"))]
use values::Impossible;
use values::computed::{Angle, NonNegativeNumber};
use values::computed::color::Color;
use values::computed::length::{Length, NonNegativeLength};
#[cfg(feature = "gecko")]
use values::computed::url::ComputedUrl;
use values::generics::effects::BoxShadow as GenericBoxShadow;
use values::generics::effects::Filter as GenericFilter;
use values::generics::effects::SimpleShadow as GenericSimpleShadow;

/// A computed value for a single shadow of the `box-shadow` property.
pub type BoxShadow = GenericBoxShadow<Color, Length, NonNegativeLength, Length>;

/// A computed value for a single `filter`.
#[cfg(feature = "gecko")]
pub type Filter = GenericFilter<Angle, NonNegativeNumber, NonNegativeLength, SimpleShadow, ComputedUrl>;

/// A computed value for a single `filter`.
#[cfg(not(feature = "gecko"))]
pub type Filter = GenericFilter<Angle, NonNegativeNumber, NonNegativeLength, Impossible, Impossible>;

/// A computed value for the `drop-shadow()` filter.
pub type SimpleShadow = GenericSimpleShadow<Color, Length, NonNegativeLength>;
