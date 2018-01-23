/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to backgrounds.

use properties::animated_properties::RepeatableListAnimatable;
use properties::longhands::background_size::computed_value::T as BackgroundSizeList;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};
use values::animated::{ToAnimatedValue, ToAnimatedZero};
use values::computed::{Context, ToComputedValue};
use values::computed::length::LengthOrPercentageOrAuto;
use values::generics::background::BackgroundSize as GenericBackgroundSize;
use values::specified::background::{BackgroundRepeat as SpecifiedBackgroundRepeat, RepeatKeyword};

/// A computed value for the `background-size` property.
pub type BackgroundSize = GenericBackgroundSize<LengthOrPercentageOrAuto>;

impl BackgroundSize {
    /// Returns `auto auto`.
    pub fn auto() -> Self {
        GenericBackgroundSize::Explicit {
            width: LengthOrPercentageOrAuto::Auto,
            height: LengthOrPercentageOrAuto::Auto,
        }
    }
}

impl RepeatableListAnimatable for BackgroundSize {}

impl ToAnimatedZero for BackgroundSize {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
}

impl ToAnimatedValue for BackgroundSize {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        use values::computed::{Length, Percentage};
        let clamp_animated_value = |value: LengthOrPercentageOrAuto| -> LengthOrPercentageOrAuto {
            match value {
                LengthOrPercentageOrAuto::Length(len) => {
                    LengthOrPercentageOrAuto::Length(Length::new(len.px().max(0.)))
                },
                LengthOrPercentageOrAuto::Percentage(percent) => {
                    LengthOrPercentageOrAuto::Percentage(Percentage(percent.0.max(0.)))
                },
                _ => value
            }
        };
        match animated {
            GenericBackgroundSize::Explicit { width, height } => {
                GenericBackgroundSize::Explicit {
                    width: clamp_animated_value(width),
                    height: clamp_animated_value(height)
                }
            },
            _ => animated
        }
    }
}

impl ToAnimatedValue for BackgroundSizeList {
    type AnimatedValue = Self;

    #[inline]
    fn to_animated_value(self) -> Self {
        self
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        BackgroundSizeList(ToAnimatedValue::from_animated_value(animated.0))
    }
}

/// The computed value of the `background-repeat` property:
///
/// https://drafts.csswg.org/css-backgrounds/#the-background-repeat
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct BackgroundRepeat(pub RepeatKeyword, pub RepeatKeyword);

impl BackgroundRepeat {
    /// Returns the `repeat repeat` value.
    pub fn repeat() -> Self {
        BackgroundRepeat(RepeatKeyword::Repeat, RepeatKeyword::Repeat)
    }
}

impl ToCss for BackgroundRepeat {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match (self.0, self.1) {
            (RepeatKeyword::Repeat, RepeatKeyword::NoRepeat) => dest.write_str("repeat-x"),
            (RepeatKeyword::NoRepeat, RepeatKeyword::Repeat) => dest.write_str("repeat-y"),
            (horizontal, vertical) => {
                horizontal.to_css(dest)?;
                if horizontal != vertical {
                    dest.write_str(" ")?;
                    vertical.to_css(dest)?;
                }
                Ok(())
            },
        }
    }
}

impl ToComputedValue for SpecifiedBackgroundRepeat {
    type ComputedValue = BackgroundRepeat;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> Self::ComputedValue {
        match *self {
            SpecifiedBackgroundRepeat::RepeatX => {
                BackgroundRepeat(RepeatKeyword::Repeat, RepeatKeyword::NoRepeat)
            }
            SpecifiedBackgroundRepeat::RepeatY => {
                BackgroundRepeat(RepeatKeyword::NoRepeat, RepeatKeyword::Repeat)
            }
            SpecifiedBackgroundRepeat::Keywords(horizontal, vertical) => {
                BackgroundRepeat(horizontal, vertical.unwrap_or(horizontal))
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        // FIXME(emilio): Why can't this just be:
        //   SpecifiedBackgroundRepeat::Keywords(computed.0, computed.1)
        match (computed.0, computed.1) {
            (RepeatKeyword::Repeat, RepeatKeyword::NoRepeat) => {
                SpecifiedBackgroundRepeat::RepeatX
            }
            (RepeatKeyword::NoRepeat, RepeatKeyword::Repeat) => {
                SpecifiedBackgroundRepeat::RepeatY
            }
            (horizontal, vertical) => {
                SpecifiedBackgroundRepeat::Keywords(horizontal, Some(vertical))
            }
        }
    }
}
