/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated types for CSS values related to effects.

use properties::animated_properties::{Animatable, IntermediateColor};
use properties::longhands::filter::computed_value::T as ComputedFilterList;
#[cfg(not(feature = "gecko"))]
use values::Impossible;
use values::computed::{Angle, Number};
#[cfg(feature = "gecko")]
use values::computed::effects::Filter as ComputedFilter;
use values::computed::effects::SimpleShadow as ComputedSimpleShadow;
use values::computed::length::Length;
use values::generics::effects::Filter as GenericFilter;
use values::generics::effects::SimpleShadow as GenericSimpleShadow;

/// An animated value for the `filter` property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq)]
pub struct FilterList(pub Vec<Filter>);

/// An animated value for a single `filter`.
#[cfg(feature = "gecko")]
pub type Filter = GenericFilter<Angle, Number, Length, SimpleShadow>;

/// An animated value for a single `filter`.
#[cfg(not(feature = "gecko"))]
pub type Filter = GenericFilter<Angle, Number, Length, Impossible>;

/// An animated value for the `drop-shadow()` filter.
pub type SimpleShadow = GenericSimpleShadow<IntermediateColor, Length, Length>;

impl From<ComputedFilterList> for FilterList {
    #[cfg(not(feature = "gecko"))]
    #[inline]
    fn from(filters: ComputedFilterList) -> Self {
        FilterList(filters.0)
    }

    #[cfg(feature = "gecko")]
    #[inline]
    fn from(filters: ComputedFilterList) -> Self {
        FilterList(filters.0.into_iter().map(|f| f.into()).collect())
    }
}

impl From<FilterList> for ComputedFilterList {
    #[cfg(not(feature = "gecko"))]
    #[inline]
    fn from(filters: FilterList) -> Self {
        ComputedFilterList(filters.0)
    }

    #[cfg(feature = "gecko")]
    #[inline]
    fn from(filters: FilterList) -> Self {
        ComputedFilterList(filters.0.into_iter().map(|f| f.into()).collect())
    }
}

#[cfg(feature = "gecko")]
impl From<ComputedFilter> for Filter {
    #[inline]
    fn from(filter: ComputedFilter) -> Self {
        match filter {
            GenericFilter::Blur(angle) => GenericFilter::Blur(angle),
            GenericFilter::Brightness(factor) => GenericFilter::Brightness(factor),
            GenericFilter::Contrast(factor) => GenericFilter::Contrast(factor),
            GenericFilter::Grayscale(factor) => GenericFilter::Grayscale(factor),
            GenericFilter::HueRotate(factor) => GenericFilter::HueRotate(factor),
            GenericFilter::Invert(factor) => GenericFilter::Invert(factor),
            GenericFilter::Opacity(factor) => GenericFilter::Opacity(factor),
            GenericFilter::Saturate(factor) => GenericFilter::Saturate(factor),
            GenericFilter::Sepia(factor) => GenericFilter::Sepia(factor),
            GenericFilter::DropShadow(shadow) => {
                GenericFilter::DropShadow(shadow.into())
            },
            #[cfg(feature = "gecko")]
            GenericFilter::Url(url) => GenericFilter::Url(url),
        }
    }
}

#[cfg(feature = "gecko")]
impl From<Filter> for ComputedFilter {
    #[inline]
    fn from(filter: Filter) -> Self {
        match filter {
            GenericFilter::Blur(angle) => GenericFilter::Blur(angle),
            GenericFilter::Brightness(factor) => GenericFilter::Brightness(factor),
            GenericFilter::Contrast(factor) => GenericFilter::Contrast(factor),
            GenericFilter::Grayscale(factor) => GenericFilter::Grayscale(factor),
            GenericFilter::HueRotate(factor) => GenericFilter::HueRotate(factor),
            GenericFilter::Invert(factor) => GenericFilter::Invert(factor),
            GenericFilter::Opacity(factor) => GenericFilter::Opacity(factor),
            GenericFilter::Saturate(factor) => GenericFilter::Saturate(factor),
            GenericFilter::Sepia(factor) => GenericFilter::Sepia(factor),
            GenericFilter::DropShadow(shadow) => {
                GenericFilter::DropShadow(shadow.into())
            },
            #[cfg(feature = "gecko")]
            GenericFilter::Url(url) => GenericFilter::Url(url.clone())
        }
    }
}

impl From<ComputedSimpleShadow> for SimpleShadow {
    #[inline]
    fn from(shadow: ComputedSimpleShadow) -> Self {
        SimpleShadow {
            color: shadow.color.into(),
            horizontal: shadow.horizontal,
            vertical: shadow.vertical,
            blur: shadow.blur,
        }
    }
}

impl From<SimpleShadow> for ComputedSimpleShadow {
    #[inline]
    fn from(shadow: SimpleShadow) -> Self {
        ComputedSimpleShadow {
            color: shadow.color.into(),
            horizontal: shadow.horizontal,
            vertical: shadow.vertical,
            blur: shadow.blur,
        }
    }
}

impl Animatable for SimpleShadow {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        let color = self.color.add_weighted(&other.color, self_portion, other_portion)?;
        let horizontal = self.horizontal.add_weighted(&other.horizontal, self_portion, other_portion)?;
        let vertical = self.vertical.add_weighted(&other.vertical, self_portion, other_portion)?;
        let blur = self.blur.add_weighted(&other.blur, self_portion, other_portion)?;

        Ok(SimpleShadow {
            color: color,
            horizontal: horizontal,
            vertical: vertical,
            blur: blur,
        })
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok(
            self.color.compute_squared_distance(&other.color)? +
            self.horizontal.compute_squared_distance(&other.horizontal)? +
            self.vertical.compute_squared_distance(&other.vertical)? +
            self.blur.compute_squared_distance(&other.blur)?
        )
    }
}
