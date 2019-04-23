/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to backgrounds.

use crate::parser::{Parse, ParserContext};
use crate::values::generics::background::BackgroundSize as GenericBackgroundSize;
use crate::values::specified::length::{
    NonNegativeLengthPercentage, NonNegativeLengthPercentageOrAuto,
};
use cssparser::Parser;
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

/// A specified value for the `background-size` property.
pub type BackgroundSize = GenericBackgroundSize<NonNegativeLengthPercentage>;

impl Parse for BackgroundSize {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(width) = input.try(|i| NonNegativeLengthPercentageOrAuto::parse(context, i)) {
            let height = input
                .try(|i| NonNegativeLengthPercentageOrAuto::parse(context, i))
                .unwrap_or(NonNegativeLengthPercentageOrAuto::auto());
            return Ok(GenericBackgroundSize::ExplicitSize { width, height });
        }
        Ok(try_match_ident_ignore_ascii_case! { input,
            "cover" => GenericBackgroundSize::Cover,
            "contain" => GenericBackgroundSize::Contain,
        })
    }
}

/// One of the keywords for `background-repeat`.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[allow(missing_docs)]
#[value_info(other_values = "repeat-x,repeat-y")]
pub enum BackgroundRepeatKeyword {
    Repeat,
    Space,
    Round,
    NoRepeat,
}

/// The value of the `background-repeat` property, with `repeat-x` / `repeat-y`
/// represented as the combination of `no-repeat` and `repeat` in the opposite
/// axes.
///
/// https://drafts.csswg.org/css-backgrounds/#the-background-repeat
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
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

impl Parse for BackgroundRepeat {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let ident = input.expect_ident_cloned()?;

        match_ignore_ascii_case! { &ident,
            "repeat-x" => {
                return Ok(BackgroundRepeat(BackgroundRepeatKeyword::Repeat, BackgroundRepeatKeyword::NoRepeat));
            },
            "repeat-y" => {
                return Ok(BackgroundRepeat(BackgroundRepeatKeyword::NoRepeat, BackgroundRepeatKeyword::Repeat));
            },
            _ => {},
        }

        let horizontal = match BackgroundRepeatKeyword::from_ident(&ident) {
            Ok(h) => h,
            Err(()) => {
                return Err(
                    input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone()))
                );
            },
        };

        let vertical = input.try(BackgroundRepeatKeyword::parse).ok();
        Ok(BackgroundRepeat(horizontal, vertical.unwrap_or(horizontal)))
    }
}
