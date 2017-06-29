/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated types for CSS values related to effects.

use app_units::Au;
use properties::animated_properties::{Animatable, IntermediateColor};
use properties::longhands::box_shadow::computed_value::T as ComputedBoxShadowList;
use properties::longhands::filter::computed_value::T as ComputedFilterList;
use properties::longhands::text_shadow::computed_value::T as ComputedTextShadowList;
use std::cmp;
#[cfg(not(feature = "gecko"))]
use values::Impossible;
use values::animated::ToAnimatedValue;
use values::computed::{Angle, Number};
use values::computed::length::Length;
use values::generics::effects::BoxShadow as GenericBoxShadow;
use values::generics::effects::Filter as GenericFilter;
use values::generics::effects::SimpleShadow as GenericSimpleShadow;

#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq)]
/// An animated value for the `box-shadow` property.
pub struct BoxShadowList(pub Vec<BoxShadow>);

/// An animated value for a single `box-shadow`.
pub type BoxShadow = GenericBoxShadow<IntermediateColor, Length, Length>;

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

#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq)]
/// An animated value for the `box-shadow` property.
pub struct TextShadowList(pub Vec<SimpleShadow>);

/// An animated value for the `drop-shadow()` filter.
pub type SimpleShadow = GenericSimpleShadow<IntermediateColor, Length, Length>;

impl ToAnimatedValue for ComputedBoxShadowList {
    type AnimatedValue = BoxShadowList;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        BoxShadowList(self.0.to_animated_value())
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        ComputedBoxShadowList(ToAnimatedValue::from_animated_value(animated.0))
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-shadow-list
impl Animatable for BoxShadowList {
    #[inline]
    fn add_weighted(
        &self,
        other: &Self,
        self_portion: f64,
        other_portion: f64,
    ) -> Result<Self, ()> {
        // The inset value must change
        let mut zero = BoxShadow {
            base: SimpleShadow {
                color: IntermediateColor::transparent(),
                horizontal: Au(0),
                vertical: Au(0),
                blur: Au(0),
            },
            spread: Au(0),
            inset: false,
        };

        let max_len = cmp::max(self.0.len(), other.0.len());
        let mut shadows = Vec::with_capacity(max_len);
        for i in 0..max_len {
            shadows.push(match (self.0.get(i), other.0.get(i)) {
                (Some(shadow), Some(other)) => {
                    shadow.add_weighted(other, self_portion, other_portion)?
                },
                (Some(shadow), None) => {
                    zero.inset = shadow.inset;
                    shadow.add_weighted(&zero, self_portion, other_portion)?
                },
                (None, Some(shadow)) => {
                    zero.inset = shadow.inset;
                    zero.add_weighted(&shadow, self_portion, other_portion)?
                },
                (None, None) => unreachable!(),
            });
        }

        Ok(BoxShadowList(shadows))
    }

    #[inline]
    fn add(&self, other: &Self) -> Result<Self, ()> {
        Ok(BoxShadowList(
            self.0.iter().cloned().chain(other.0.iter().cloned()).collect(),
        ))
    }
}

impl ToAnimatedValue for ComputedTextShadowList {
    type AnimatedValue = TextShadowList;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        TextShadowList(self.0.to_animated_value())
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        ComputedTextShadowList(ToAnimatedValue::from_animated_value(animated.0))
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-shadow-list
impl Animatable for TextShadowList {
    #[inline]
    fn add_weighted(
        &self,
        other: &Self,
        self_portion: f64,
        other_portion: f64,
    ) -> Result<Self, ()> {
        let zero = SimpleShadow {
            color: IntermediateColor::transparent(),
            horizontal: Au(0),
            vertical: Au(0),
            blur: Au(0),
        };

        let max_len = cmp::max(self.0.len(), other.0.len());
        let mut shadows = Vec::with_capacity(max_len);
        for i in 0..max_len {
            shadows.push(match (self.0.get(i), other.0.get(i)) {
                (Some(shadow), Some(other)) => {
                    shadow.add_weighted(other, self_portion, other_portion)?
                },
                (Some(shadow), None) => {
                    shadow.add_weighted(&zero, self_portion, other_portion)?
                },
                (None, Some(shadow)) => {
                    zero.add_weighted(&shadow, self_portion, other_portion)?
                },
                (None, None) => unreachable!(),
            });
        }

        Ok(TextShadowList(shadows))
    }

    #[inline]
    fn add(&self, other: &Self) -> Result<Self, ()> {
        Ok(TextShadowList(
            self.0.iter().cloned().chain(other.0.iter().cloned()).collect(),
        ))
    }
}

impl Animatable for BoxShadow {
    #[inline]
    fn add_weighted(
        &self,
        other: &Self,
        self_portion: f64,
        other_portion: f64,
    ) -> Result<Self, ()> {
        if self.inset != other.inset {
            return Err(());
        }
        Ok(BoxShadow {
            base: self.base.add_weighted(&other.base, self_portion, other_portion)?,
            spread: self.spread.add_weighted(&other.spread, self_portion, other_portion)?,
            inset: self.inset,
        })
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        if self.inset != other.inset {
            return Err(());
        }
        Ok(
            self.base.compute_squared_distance(&other.base)? +
            self.spread.compute_squared_distance(&other.spread)?,
        )
    }
}


impl ToAnimatedValue for ComputedFilterList {
    type AnimatedValue = FilterList;

    #[cfg(not(feature = "gecko"))]
    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        FilterList(self.0)
    }

    #[cfg(feature = "gecko")]
    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        FilterList(self.0.to_animated_value())
    }

    #[cfg(not(feature = "gecko"))]
    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        ComputedFilterList(animated.0)
    }

    #[cfg(feature = "gecko")]
    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        ComputedFilterList(ToAnimatedValue::from_animated_value(animated.0))
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
