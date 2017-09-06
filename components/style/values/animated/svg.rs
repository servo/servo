/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated types for CSS values in SVG

use values::computed::{Number, Percentage};
use values::computed::length::CalcLengthOrPercentage;

/// Stroke-* value support unit less value, so servo interpolate length value as
/// number unlike computed value and specified value.
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, PartialEq, ToAnimatedZero)]
pub enum SvgLengthOrPercentageOrNumber {
    /// Real number value.
    Number(Number),
    /// Percentage value.
    Percentage(Percentage),
    /// Calc value, this type can hold percentage value. For present, percentage
    /// value store the Percentage type.(i.e. Servo doesn't use CalcLengthOrPercentage
    /// for storing the percentage value)
    /// TODO: We need to support interpolation of calc value.
    /// https://bugzilla.mozilla.org/show_bug.cgi?id=1386967
    #[animation(error)]
    Calc(CalcLengthOrPercentage),
}
