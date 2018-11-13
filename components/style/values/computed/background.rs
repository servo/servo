/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to backgrounds.

use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};
use values::computed::{Context, ToComputedValue};
use values::computed::length::NonNegativeLengthOrPercentageOrAuto;
use values::generics::background::BackgroundSize as GenericBackgroundSize;
use values::specified::background::BackgroundRepeat as SpecifiedBackgroundRepeat;
use values::specified::background::BackgroundRepeatKeyword;

/// A computed value for the `background-size` property.
pub type BackgroundSize = GenericBackgroundSize<NonNegativeLengthOrPercentageOrAuto>;

impl BackgroundSize {
    /// Returns `auto auto`.
    pub fn auto() -> Self {
        GenericBackgroundSize::Explicit {
            width: NonNegativeLengthOrPercentageOrAuto::auto(),
            height: NonNegativeLengthOrPercentageOrAuto::auto(),
        }
    }
}

/// The computed value of the `background-repeat` property:
///
/// https://drafts.csswg.org/css-backgrounds/#the-background-repeat
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct BackgroundRepeat(pub BackgroundRepeatKeyword, pub BackgroundRepeatKeyword);

impl BackgroundRepeat {
    /// Returns the `repeat repeat` value.
    pub fn repeat() -> Self {
        BackgroundRepeat(
            BackgroundRepeatKeyword::Repeat,
            BackgroundRepeatKeyword::Repeat,
        )
    }
}

impl ToCss for BackgroundRepeat {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match (self.0, self.1) {
            (BackgroundRepeatKeyword::Repeat, BackgroundRepeatKeyword::NoRepeat) => {
                dest.write_str("repeat-x")
            },
            (BackgroundRepeatKeyword::NoRepeat, BackgroundRepeatKeyword::Repeat) => {
                dest.write_str("repeat-y")
            },
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
            SpecifiedBackgroundRepeat::RepeatX => BackgroundRepeat(
                BackgroundRepeatKeyword::Repeat,
                BackgroundRepeatKeyword::NoRepeat,
            ),
            SpecifiedBackgroundRepeat::RepeatY => BackgroundRepeat(
                BackgroundRepeatKeyword::NoRepeat,
                BackgroundRepeatKeyword::Repeat,
            ),
            SpecifiedBackgroundRepeat::Keywords(horizontal, vertical) => {
                BackgroundRepeat(horizontal, vertical.unwrap_or(horizontal))
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        // FIXME(emilio): Why can't this just be:
        //   SpecifiedBackgroundRepeat::Keywords(computed.0, computed.1)
        match (computed.0, computed.1) {
            (BackgroundRepeatKeyword::Repeat, BackgroundRepeatKeyword::NoRepeat) => {
                SpecifiedBackgroundRepeat::RepeatX
            },
            (BackgroundRepeatKeyword::NoRepeat, BackgroundRepeatKeyword::Repeat) => {
                SpecifiedBackgroundRepeat::RepeatY
            },
            (horizontal, vertical) => {
                SpecifiedBackgroundRepeat::Keywords(horizontal, Some(vertical))
            },
        }
    }
}
