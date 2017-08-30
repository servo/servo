/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed angles.

use euclid::Radians;
use std::{f32, f64, fmt};
use std::f64::consts::PI;
use style_traits::ToCss;
use values::CSSFloat;
use values::animated::{Animate, Procedure};
use values::distance::{ComputeSquaredDistance, SquaredDistance};

/// A computed angle.
#[animate(fallback = "Self::animate_fallback")]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
#[derive(Animate, Clone, Copy, Debug, PartialEq)]
#[derive(PartialOrd, ToAnimatedZero)]
pub enum Angle {
    /// An angle with degree unit.
    Degree(CSSFloat),
    /// An angle with gradian unit.
    Gradian(CSSFloat),
    /// An angle with radian unit.
    Radian(CSSFloat),
    /// An angle with turn unit.
    Turn(CSSFloat),
}

impl Angle {
    /// Creates a computed `Angle` value from a radian amount.
    pub fn from_radians(radians: CSSFloat) -> Self {
        Angle::Radian(radians)
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
            Angle::Degree(val) => val as f64 * RAD_PER_DEG,
            Angle::Gradian(val) => val as f64 * RAD_PER_GRAD,
            Angle::Turn(val) => val as f64 * RAD_PER_TURN,
            Angle::Radian(val) => val as f64,
        };
        radians.min(f64::MAX).max(f64::MIN)
    }

    /// Returns an angle that represents a rotation of zero radians.
    pub fn zero() -> Self {
        Angle::Radian(0.0)
    }

    /// https://drafts.csswg.org/css-transitions/#animtype-number
    #[inline]
    fn animate_fallback(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Angle::from_radians(self.radians().animate(&other.radians(), procedure)?))
    }
}

impl ComputeSquaredDistance for Angle {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // Use the formula for calculating the distance between angles defined in SVG:
        // https://www.w3.org/TR/SVG/animate.html#complexDistances
        self.radians64().compute_squared_distance(&other.radians64())
    }
}

impl ToCss for Angle {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        let mut write = |value: CSSFloat, unit: &str| {
            value.to_css(dest)?;
            dest.write_str(unit)
        };

        match *self {
            Angle::Degree(val) => write(val, "deg"),
            Angle::Gradian(val) => write(val, "grad"),
            Angle::Radian(val) => write(val, "rad"),
            Angle::Turn(val) => write(val, "turn"),
        }
    }
}

impl From<Angle> for Radians<CSSFloat> {
    #[inline]
    fn from(a: Angle) -> Self {
        Radians::new(a.radians())
    }
}
