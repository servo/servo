/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to effects.

#[cfg(not(feature = "gecko"))]
use values::Impossible;
use values::computed::{Angle, Number};
use values::computed::color::Color;
use values::computed::length::Length;
use values::generics::effects::Filter as GenericFilter;
use values::generics::effects::FilterList as GenericFilterList;
use values::generics::effects::SimpleShadow as GenericSimpleShadow;

/// A computed value for the `filter` property.
pub type FilterList = GenericFilterList<Filter>;

/// A computed value for a single `filter`.
#[cfg(feature = "gecko")]
pub type Filter = GenericFilter<Angle, Number, Length, SimpleShadow>;

/// A computed value for a single `filter`.
#[cfg(not(feature = "gecko"))]
pub type Filter = GenericFilter<Angle, Number, Length, Impossible>;

/// A computed value for the `drop-shadow()` filter.
pub type SimpleShadow = GenericSimpleShadow<Color, Length, Length>;

impl FilterList {
    /// Returns the resulting opacity of this filter pipeline.
    pub fn opacity(&self) -> Number {
        let mut opacity = 0.;
        for filter in &*self.0 {
            if let GenericFilter::Opacity(factor) = *filter {
                opacity *= factor
            }
        }
        opacity
    }
}
