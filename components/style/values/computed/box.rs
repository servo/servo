/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for box properties.

use values::computed::Number;
use values::computed::length::LengthOrPercentage;
use values::generics::box_::AnimationIterationCount as GenericAnimationIterationCount;
use values::generics::box_::VerticalAlign as GenericVerticalAlign;

pub use values::specified::box_::{AnimationName, Display, OverflowClipBox};
pub use values::specified::box_::{OverscrollBehavior, ScrollSnapType, TouchAction, WillChange};

/// A computed value for the `vertical-align` property.
pub type VerticalAlign = GenericVerticalAlign<LengthOrPercentage>;

/// A computed value for the `animation-iteration-count` property.
pub type AnimationIterationCount = GenericAnimationIterationCount<Number>;

impl AnimationIterationCount {
    /// Returns the value `1.0`.
    #[inline]
    pub fn one() -> Self {
        GenericAnimationIterationCount::Number(1.0)
    }
}
