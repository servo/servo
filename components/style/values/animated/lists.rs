/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Lists have various ways of being animated, this module implements them.
//!
//! See https://drafts.csswg.org/web-animations-1/#animating-properties

/// https://drafts.csswg.org/web-animations-1/#by-computed-value
pub mod by_computed_value {
    use crate::values::{
        animated::{Animate, Procedure},
        distance::{ComputeSquaredDistance, SquaredDistance},
    };
    use std::iter::FromIterator;

    #[allow(missing_docs)]
    pub fn animate<T, C>(left: &[T], right: &[T], procedure: Procedure) -> Result<C, ()>
    where
        T: Animate,
        C: FromIterator<T>,
    {
        if left.len() != right.len() {
            return Err(());
        }
        left.iter()
            .zip(right.iter())
            .map(|(left, right)| left.animate(right, procedure))
            .collect()
    }

    #[allow(missing_docs)]
    pub fn squared_distance<T>(left: &[T], right: &[T]) -> Result<SquaredDistance, ()>
    where
        T: ComputeSquaredDistance,
    {
        if left.len() != right.len() {
            return Err(());
        }
        left.iter()
            .zip(right.iter())
            .map(|(left, right)| left.compute_squared_distance(right))
            .sum()
    }
}

/// This is the animation used for some of the types like shadows and filters, where the
/// interpolation happens with the zero value if one of the sides is not present.
///
/// https://drafts.csswg.org/web-animations-1/#animating-shadow-lists
pub mod with_zero {
    use crate::values::animated::ToAnimatedZero;
    use crate::values::{
        animated::{Animate, Procedure},
        distance::{ComputeSquaredDistance, SquaredDistance},
    };
    use itertools::{EitherOrBoth, Itertools};
    use std::iter::FromIterator;

    #[allow(missing_docs)]
    pub fn animate<T, C>(left: &[T], right: &[T], procedure: Procedure) -> Result<C, ()>
    where
        T: Animate + Clone + ToAnimatedZero,
        C: FromIterator<T>,
    {
        if procedure == Procedure::Add {
            return Ok(left.iter().chain(right.iter()).cloned().collect());
        }
        left.iter()
            .zip_longest(right.iter())
            .map(|it| match it {
                EitherOrBoth::Both(left, right) => left.animate(right, procedure),
                EitherOrBoth::Left(left) => left.animate(&left.to_animated_zero()?, procedure),
                EitherOrBoth::Right(right) => right.to_animated_zero()?.animate(right, procedure),
            })
            .collect()
    }

    #[allow(missing_docs)]
    pub fn squared_distance<T>(left: &[T], right: &[T]) -> Result<SquaredDistance, ()>
    where
        T: ToAnimatedZero + ComputeSquaredDistance,
    {
        left.iter()
            .zip_longest(right.iter())
            .map(|it| match it {
                EitherOrBoth::Both(left, right) => left.compute_squared_distance(right),
                EitherOrBoth::Left(item) | EitherOrBoth::Right(item) => {
                    item.to_animated_zero()?.compute_squared_distance(item)
                },
            })
            .sum()
    }
}

/// https://drafts.csswg.org/web-animations-1/#repeatable-list
pub mod repeatable_list {
    use crate::values::{
        animated::{Animate, Procedure},
        distance::{ComputeSquaredDistance, SquaredDistance},
    };
    use std::iter::FromIterator;

    #[allow(missing_docs)]
    pub fn animate<T, C>(left: &[T], right: &[T], procedure: Procedure) -> Result<C, ()>
    where
        T: Animate,
        C: FromIterator<T>,
    {
        use num_integer::lcm;
        // If the length of either list is zero, the least common multiple is undefined.
        if left.is_empty() || right.is_empty() {
            return Err(());
        }
        let len = lcm(left.len(), right.len());
        left.iter()
            .cycle()
            .zip(right.iter().cycle())
            .take(len)
            .map(|(left, right)| left.animate(right, procedure))
            .collect()
    }

    #[allow(missing_docs)]
    pub fn squared_distance<T>(left: &[T], right: &[T]) -> Result<SquaredDistance, ()>
    where
        T: ComputeSquaredDistance,
    {
        use num_integer::lcm;
        if left.is_empty() || right.is_empty() {
            return Err(());
        }
        let len = lcm(left.len(), right.len());
        left.iter()
            .cycle()
            .zip(right.iter().cycle())
            .take(len)
            .map(|(left, right)| left.compute_squared_distance(right))
            .sum()
    }
}
