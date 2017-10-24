/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for box properties.

use values::animated::ToAnimatedZero;

/// A generic value for the `vertical-align` property.
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq)]
#[derive(ToComputedValue, ToCss)]
pub enum VerticalAlign<LengthOrPercentage> {
    /// `baseline`
    Baseline,
    /// `sub`
    Sub,
    /// `super`
    Super,
    /// `top`
    Top,
    /// `text-top`
    TextTop,
    /// `middle`
    Middle,
    /// `bottom`
    Bottom,
    /// `text-bottom`
    TextBottom,
    /// `-moz-middle-with-baseline`
    #[cfg(feature = "gecko")]
    MozMiddleWithBaseline,
    /// `<length-percentage>`
    Length(LengthOrPercentage),
}

impl<L> VerticalAlign<L> {
    /// Returns `baseline`.
    #[inline]
    pub fn baseline() -> Self {
        VerticalAlign::Baseline
    }
}

impl<L> ToAnimatedZero for VerticalAlign<L> {
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}

/// https://drafts.csswg.org/css-animations/#animation-iteration-count
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub enum AnimationIterationCount<Number> {
    /// A `<number>` value.
    Number(Number),
    /// The `infinite` keyword.
    Infinite,
}
