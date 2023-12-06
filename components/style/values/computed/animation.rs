/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed values for properties related to animations and transitions

use crate::values::computed::{Context, LengthPercentage, ToComputedValue};
use crate::values::generics::animation as generics;
use crate::values::specified::animation as specified;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

pub use crate::values::specified::animation::{
    AnimationName, ScrollAxis, ScrollTimelineName, TransitionProperty,
};

/// A computed value for the `animation-iteration-count` property.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToResolvedValue, ToShmem)]
#[repr(C)]
pub struct AnimationIterationCount(pub f32);

impl ToComputedValue for specified::AnimationIterationCount {
    type ComputedValue = AnimationIterationCount;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        AnimationIterationCount(match *self {
            specified::AnimationIterationCount::Number(n) => n.to_computed_value(context).0,
            specified::AnimationIterationCount::Infinite => f32::INFINITY,
        })
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        use crate::values::specified::NonNegativeNumber;
        if computed.0.is_infinite() {
            specified::AnimationIterationCount::Infinite
        } else {
            specified::AnimationIterationCount::Number(NonNegativeNumber::new(computed.0))
        }
    }
}

impl AnimationIterationCount {
    /// Returns the value `1.0`.
    #[inline]
    pub fn one() -> Self {
        Self(1.0)
    }
}

impl ToCss for AnimationIterationCount {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.0.is_infinite() {
            dest.write_str("infinite")
        } else {
            self.0.to_css(dest)
        }
    }
}

/// A computed value for the `animation-timeline` property.
pub type AnimationTimeline = generics::GenericAnimationTimeline<LengthPercentage>;

/// A computed value for the `view-timeline-inset` property.
pub type ViewTimelineInset = generics::GenericViewTimelineInset<LengthPercentage>;
