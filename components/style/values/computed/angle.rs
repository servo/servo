/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed angles.

use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::CSSFloat;
use crate::Zero;
use std::f64::consts::PI;
use std::fmt::{self, Write};
use std::{f32, f64};
use style_traits::{CssWriter, ToCss};

/// A computed angle in degrees.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Add,
    Animate,
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    PartialOrd,
    ToAnimatedZero,
    ToResolvedValue,
)]
#[repr(C)]
pub struct Angle(CSSFloat);

impl ToCss for Angle {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.degrees().to_css(dest)?;
        dest.write_str("deg")
    }
}

const RAD_PER_DEG: f64 = PI / 180.0;

impl Angle {
    /// Creates a computed `Angle` value from a radian amount.
    pub fn from_radians(radians: CSSFloat) -> Self {
        Angle(radians / RAD_PER_DEG as f32)
    }

    /// Creates a computed `Angle` value from a degrees amount.
    #[inline]
    pub fn from_degrees(degrees: CSSFloat) -> Self {
        Angle(degrees)
    }

    /// Returns the amount of radians this angle represents.
    #[inline]
    pub fn radians(&self) -> CSSFloat {
        self.radians64().min(f32::MAX as f64).max(f32::MIN as f64) as f32
    }

    /// Returns the amount of radians this angle represents as a `f64`.
    ///
    /// Gecko stores angles as singles, but does this computation using doubles.
    ///
    /// This is significant enough to mess up rounding to the nearest
    /// quarter-turn for 225 degrees, for example.
    #[inline]
    pub fn radians64(&self) -> f64 {
        self.0 as f64 * RAD_PER_DEG
    }

    /// Return the value in degrees.
    #[inline]
    pub fn degrees(&self) -> CSSFloat {
        self.0
    }
}

impl Zero for Angle {
    #[inline]
    fn zero() -> Self {
        Angle(0.0)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0 == 0.
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
