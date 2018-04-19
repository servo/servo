/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed angles.

use num_traits::Zero;
use std::{f32, f64};
use std::f64::consts::PI;
use std::ops::Add;
use values::CSSFloat;
use values::animated::{Animate, Procedure};
use values::distance::{ComputeSquaredDistance, SquaredDistance};

/// A computed angle.
#[animate(fallback = "Self::animate_fallback")]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Animate, Clone, Copy, Debug, MallocSizeOf, PartialEq, PartialOrd, ToAnimatedZero, ToCss)]
pub enum Angle {
    /// An angle with degree unit.
    #[css(dimension)]
    Deg(CSSFloat),
    /// An angle with gradian unit.
    #[css(dimension)]
    Grad(CSSFloat),
    /// An angle with radian unit.
    #[css(dimension)]
    Rad(CSSFloat),
    /// An angle with turn unit.
    #[css(dimension)]
    Turn(CSSFloat),
}

impl Angle {
    /// Creates a computed `Angle` value from a radian amount.
    pub fn from_radians(radians: CSSFloat) -> Self {
        Angle::Rad(radians)
    }

    /// Returns the amount of radians this angle represents.
    #[inline]
    pub fn radians(&self) -> CSSFloat {
        self.radians64().min(f32::MAX as f64).max(f32::MIN as f64) as f32
    }

    /// Returns the amount of radians this angle represents as a `f64`.
    ///
    /// Gecko stores angles as singles, but does this computation using doubles.
    /// See nsCSSValue::GetAngleValueInRadians.
    /// This is significant enough to mess up rounding to the nearest
    /// quarter-turn for 225 degrees, for example.
    #[inline]
    pub fn radians64(&self) -> f64 {
        const RAD_PER_DEG: f64 = PI / 180.0;
        const RAD_PER_GRAD: f64 = PI / 200.0;
        const RAD_PER_TURN: f64 = PI * 2.0;

        let radians = match *self {
            Angle::Deg(val) => val as f64 * RAD_PER_DEG,
            Angle::Grad(val) => val as f64 * RAD_PER_GRAD,
            Angle::Turn(val) => val as f64 * RAD_PER_TURN,
            Angle::Rad(val) => val as f64,
        };
        radians.min(f64::MAX).max(f64::MIN)
    }

    /// Return the value in degrees.
    pub fn degrees(&self) -> f32 {
        use std::f32::consts::PI;
        self.radians() * 360. / (2. * PI)
    }

    /// <https://drafts.csswg.org/css-transitions/#animtype-number>
    #[inline]
    fn animate_fallback(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Angle::from_radians(self.radians().animate(&other.radians(), procedure)?))
    }
}

impl AsRef<Angle> for Angle {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Add for Angle {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Angle::Deg(x), Angle::Deg(y)) => Angle::Deg(x + y),
            (Angle::Grad(x), Angle::Grad(y)) => Angle::Grad(x + y),
            (Angle::Turn(x), Angle::Turn(y)) => Angle::Turn(x + y),
            (Angle::Rad(x), Angle::Rad(y)) => Angle::Rad(x + y),
            _ => Angle::from_radians(self.radians() + rhs.radians()),
        }
    }
}

impl Zero for Angle {
    #[inline]
    fn zero() -> Self {
        Angle::from_radians(0.0)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        match *self {
            Angle::Deg(val) | Angle::Grad(val) | Angle::Turn(val) | Angle::Rad(val) => val == 0.,
        }
    }
}

impl ComputeSquaredDistance for Angle {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // Use the formula for calculating the distance between angles defined in SVG:
        // https://www.w3.org/TR/SVG/animate.html#complexDistances
        self.radians64()
            .compute_squared_distance(&other.radians64())
    }
}
