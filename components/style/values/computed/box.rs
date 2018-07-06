/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for box properties.

use values::computed::{Context, Number, ToComputedValue};
use values::computed::length::{LengthOrPercentage, NonNegativeLength};
use values::generics::box_::AnimationIterationCount as GenericAnimationIterationCount;
use values::generics::box_::Perspective as GenericPerspective;
use values::generics::box_::VerticalAlign as GenericVerticalAlign;

pub use values::specified::box_::{AnimationName, Contain, Display, OverflowClipBox};
pub use values::specified::box_::Float as SpecifiedFloat;
pub use values::specified::box_::{OverscrollBehavior, ScrollSnapType, TouchAction, TransitionProperty, WillChange};

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

/// A computed value for the `perspective` property.
pub type Perspective = GenericPerspective<NonNegativeLength>;

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToCss)]
/// A computed value for the `float` property.
pub enum Float {
    Left,
    Right,
    None
}

impl ToComputedValue for SpecifiedFloat {
    type ComputedValue = Float;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        let ltr = context.style().writing_mode.is_bidi_ltr();
        // https://drafts.csswg.org/css-logical-props/#float-clear
        match *self {
            SpecifiedFloat::InlineStart => {
                context.rule_cache_conditions.borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
                if ltr {
                    Float::Left
                } else {
                    Float::Right
                }
            },
            SpecifiedFloat::InlineEnd => {
                context.rule_cache_conditions.borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
                if ltr {
                    Float::Right
                } else {
                    Float::Left
                }
            },
            SpecifiedFloat::Left => Float::Left,
            SpecifiedFloat::Right => Float::Right,
            SpecifiedFloat::None => Float::None
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> SpecifiedFloat {
        match *computed {
            Float::Left => SpecifiedFloat::Left,
            Float::Right => SpecifiedFloat::Right,
            Float::None => SpecifiedFloat::None
        }
    }
}
