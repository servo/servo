/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated types for CSS values related to effects.

use properties::animated_properties::{Animatable, IntermediateColor};
use properties::longhands::box_shadow::computed_value::T as ComputedBoxShadowList;
use properties::longhands::filter::computed_value::T as ComputedFilterList;
use properties::longhands::text_shadow::computed_value::T as ComputedTextShadowList;
use std::cmp;
#[cfg(not(feature = "gecko"))]
use values::Impossible;
use values::animated::{ToAnimatedValue, ToAnimatedZero};
use values::computed::{Angle, Number};
use values::computed::length::Length;
use values::generics::effects::BoxShadow as GenericBoxShadow;
use values::generics::effects::Filter as GenericFilter;
use values::generics::effects::SimpleShadow as GenericSimpleShadow;

/// An animated value for the `box-shadow` property.
pub type BoxShadowList = ShadowList<BoxShadow>;

/// An animated value for the `text-shadow` property.
pub type TextShadowList = ShadowList<SimpleShadow>;

/// An animated value for shadow lists.
///
/// https://drafts.csswg.org/css-transitions/#animtype-shadow-list
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq)]
pub struct ShadowList<Shadow>(Vec<Shadow>);

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

/// An animated value for the `drop-shadow()` filter.
pub type SimpleShadow = GenericSimpleShadow<IntermediateColor, Length, Length>;

impl ToAnimatedValue for ComputedBoxShadowList {
    type AnimatedValue = BoxShadowList;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        ShadowList(self.0.to_animated_value())
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        ComputedBoxShadowList(ToAnimatedValue::from_animated_value(animated.0))
    }
}

impl<S> Animatable for ShadowList<S>
where
    S: Animatable + Clone + ToAnimatedZero,
{
    #[inline]
    fn add_weighted(
        &self,
        other: &Self,
        self_portion: f64,
        other_portion: f64,
    ) -> Result<Self, ()> {
        let max_len = cmp::max(self.0.len(), other.0.len());
        let mut shadows = Vec::with_capacity(max_len);
        for i in 0..max_len {
            shadows.push(match (self.0.get(i), other.0.get(i)) {
                (Some(shadow), Some(other)) => {
                    shadow.add_weighted(other, self_portion, other_portion)?
                },
                (Some(shadow), None) => {
                    shadow.add_weighted(&shadow.to_animated_zero()?, self_portion, other_portion)?
                },
                (None, Some(shadow)) => {
                    shadow.to_animated_zero()?.add_weighted(&shadow, self_portion, other_portion)?
                },
                (None, None) => unreachable!(),
            });
        }
        Ok(ShadowList(shadows))
    }

    #[inline]
    fn add(&self, other: &Self) -> Result<Self, ()> {
        Ok(ShadowList(
            self.0.iter().cloned().chain(other.0.iter().cloned()).collect(),
        ))
    }
}

impl<S> ToAnimatedZero for ShadowList<S> {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(ShadowList(vec![]))
    }
}

impl ToAnimatedValue for ComputedTextShadowList {
    type AnimatedValue = TextShadowList;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        ShadowList(self.0.to_animated_value())
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        ComputedTextShadowList(ToAnimatedValue::from_animated_value(animated.0))
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

impl ToAnimatedZero for BoxShadow {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(BoxShadow {
            base: self.base.to_animated_zero()?,
            spread: self.spread.to_animated_zero()?,
            inset: self.inset,
        })
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

impl ToAnimatedZero for FilterList {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(FilterList(vec![]))
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

impl ToAnimatedZero for SimpleShadow {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(SimpleShadow {
            color: IntermediateColor::transparent(),
            horizontal: self.horizontal.to_animated_zero()?,
            vertical: self.vertical.to_animated_zero()?,
            blur: self.blur.to_animated_zero()?,
        })
    }
}
