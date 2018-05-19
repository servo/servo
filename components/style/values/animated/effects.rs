/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated types for CSS values related to effects.

use properties::longhands::box_shadow::computed_value::T as ComputedBoxShadowList;
use properties::longhands::filter::computed_value::T as ComputedFilterList;
use properties::longhands::text_shadow::computed_value::T as ComputedTextShadowList;
use std::cmp;
#[cfg(not(feature = "gecko"))]
use values::Impossible;
use values::animated::{Animate, Procedure, ToAnimatedValue, ToAnimatedZero};
use values::animated::color::RGBA;
use values::computed::{Angle, Number};
use values::computed::length::Length;
#[cfg(feature = "gecko")]
use values::computed::url::ComputedUrl;
use values::distance::{ComputeSquaredDistance, SquaredDistance};
use values::generics::effects::BoxShadow as GenericBoxShadow;
use values::generics::effects::Filter as GenericFilter;
use values::generics::effects::SimpleShadow as GenericSimpleShadow;

/// An animated value for the `box-shadow` property.
pub type BoxShadowList = ShadowList<BoxShadow>;

/// An animated value for the `text-shadow` property.
pub type TextShadowList = ShadowList<SimpleShadow>;

/// An animated value for shadow lists.
///
/// <https://drafts.csswg.org/css-transitions/#animtype-shadow-list>
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq)]
pub struct ShadowList<Shadow>(Vec<Shadow>);

/// An animated value for a single `box-shadow`.
pub type BoxShadow = GenericBoxShadow<Option<RGBA>, Length, Length, Length>;

/// An animated value for the `filter` property.
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq)]
pub struct FilterList(pub Vec<Filter>);

/// An animated value for a single `filter`.
#[cfg(feature = "gecko")]
pub type Filter = GenericFilter<Angle, Number, Length, SimpleShadow, ComputedUrl>;

/// An animated value for a single `filter`.
#[cfg(not(feature = "gecko"))]
pub type Filter = GenericFilter<Angle, Number, Length, Impossible, Impossible>;

/// An animated value for the `drop-shadow()` filter.
pub type SimpleShadow = GenericSimpleShadow<Option<RGBA>, Length, Length>;

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

impl<S> Animate for ShadowList<S>
where
    S: Animate + Clone + ToAnimatedZero,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if procedure == Procedure::Add {
            return Ok(ShadowList(self.0.iter().chain(&other.0).cloned().collect()));
        }
        // FIXME(nox): Use itertools here, to avoid the need for `unreachable!`.
        let max_len = cmp::max(self.0.len(), other.0.len());
        let mut shadows = Vec::with_capacity(max_len);
        for i in 0..max_len {
            shadows.push(match (self.0.get(i), other.0.get(i)) {
                (Some(shadow), Some(other)) => shadow.animate(other, procedure)?,
                (Some(shadow), None) => shadow.animate(&shadow.to_animated_zero()?, procedure)?,
                (None, Some(shadow)) => shadow.to_animated_zero()?.animate(shadow, procedure)?,
                (None, None) => unreachable!(),
            });
        }
        Ok(ShadowList(shadows))
    }
}

impl<S> ComputeSquaredDistance for ShadowList<S>
where
    S: ComputeSquaredDistance + ToAnimatedZero,
{
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        use itertools::{EitherOrBoth, Itertools};

        self.0
            .iter()
            .zip_longest(other.0.iter())
            .map(|it| match it {
                EitherOrBoth::Both(from, to) => from.compute_squared_distance(to),
                EitherOrBoth::Left(list) | EitherOrBoth::Right(list) => {
                    list.compute_squared_distance(&list.to_animated_zero()?)
                },
            })
            .sum()
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

impl ToAnimatedValue for ComputedFilterList {
    type AnimatedValue = FilterList;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        FilterList(self.0.to_animated_value())
    }

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
