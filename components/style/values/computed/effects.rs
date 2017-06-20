/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to effects.

use values::computed::{Angle, Number};
#[cfg(feature = "gecko")]
use values::computed::color::Color;
use values::computed::length::Length;
use values::generics::effects::Filter as GenericFilter;
use values::generics::effects::FilterList as GenericFilterList;

/// A computed value for the `filter` property.
pub type FilterList = GenericFilterList<Filter>;

/// A computed value for a single `filter`.
pub type Filter = GenericFilter<
    Angle,
    // FIXME: Should be `NumberOrPercentage`.
    Number,
    Length,
    DropShadow,
>;

/// A computed value for the `drop-shadow()` filter.
///
/// Currently unsupported outside of Gecko.
#[cfg(not(feature = "gecko"))]
#[cfg_attr(feature = "servo", derive(Deserialize, HeapSizeOf, Serialize))]
#[derive(Clone, Debug, PartialEq, ToCss)]
pub enum DropShadow {}

/// A computed value for the `drop-shadow()` filter.
///
/// Contrary to the canonical order from the spec, the color is serialised
/// first, like in Gecko and Webkit.
#[cfg(feature = "gecko")]
#[derive(Clone, Debug, PartialEq, ToCss)]
pub struct DropShadow {
    /// Color.
    pub color: Color,
    /// Horizontal radius.
    pub horizontal: Length,
    /// Vertical radius.
    pub vertical: Length,
    /// Blur radius.
    pub blur: Length,
}

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
