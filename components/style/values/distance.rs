/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery to compute distances between animatable values.

use app_units::Au;
use euclid::Size2D;
use std::iter::Sum;
use std::ops::Add;

/// A trait to compute squared distances between two animatable values.
///
/// This trait is derivable with `#[derive(ComputeSquaredDistance)]`. The derived
/// implementation uses a `match` expression with identical patterns for both
/// `self` and `other`, calling `ComputeSquaredDistance::compute_squared_distance`
/// on each fields of the values.
///
/// If a variant is annotated with `#[animation(error)]`, the corresponding
/// `match` arm is not generated.
///
/// If the two values are not similar, an error is returned unless a fallback
/// function has been specified through `#[distance(fallback)]`.
///
/// Trait bounds for type parameter `Foo` can be opted out of with
/// `#[animation(no_bound(Foo))]` on the type definition, trait bounds for
/// fields can be opted into with `#[distance(field_bound)]` on the field.
pub trait ComputeSquaredDistance {
    /// Computes the squared distance between two animatable values.
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()>;
}

/// A distance between two animatable values.
#[derive(Clone, Copy, Debug)]
pub struct SquaredDistance {
    value: f64,
}

impl SquaredDistance {
    /// Returns a squared distance from its square root.
    #[inline]
    pub fn from_sqrt(sqrt: f64) -> Self {
        Self { value: sqrt * sqrt }
    }
}

impl ComputeSquaredDistance for u16 {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        Ok(SquaredDistance::from_sqrt(
            ((*self as f64) - (*other as f64)).abs(),
        ))
    }
}

impl ComputeSquaredDistance for i32 {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        Ok(SquaredDistance::from_sqrt((*self - *other).abs() as f64))
    }
}

impl ComputeSquaredDistance for f32 {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        Ok(SquaredDistance::from_sqrt((*self - *other).abs() as f64))
    }
}

impl ComputeSquaredDistance for f64 {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        Ok(SquaredDistance::from_sqrt((*self - *other).abs()))
    }
}

impl ComputeSquaredDistance for Au {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        self.0.compute_squared_distance(&other.0)
    }
}

impl<T> ComputeSquaredDistance for Option<T>
where
    T: ComputeSquaredDistance,
{
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self.as_ref(), other.as_ref()) {
            (Some(this), Some(other)) => this.compute_squared_distance(other),
            (None, None) => Ok(SquaredDistance::from_sqrt(0.)),
            _ => Err(()),
        }
    }
}

impl<T> ComputeSquaredDistance for Size2D<T>
where
    T: ComputeSquaredDistance,
{
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        Ok(self.width.compute_squared_distance(&other.width)? +
            self.height.compute_squared_distance(&other.height)?)
    }
}

impl SquaredDistance {
    /// Returns the square root of this squared distance.
    #[inline]
    pub fn sqrt(self) -> f64 {
        self.value.sqrt()
    }
}

impl From<SquaredDistance> for f64 {
    #[inline]
    fn from(distance: SquaredDistance) -> Self {
        distance.value
    }
}

impl Add for SquaredDistance {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        SquaredDistance {
            value: self.value + rhs.value,
        }
    }
}

impl Sum for SquaredDistance {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(SquaredDistance::from_sqrt(0.), Add::add)
    }
}
