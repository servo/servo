/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to backgrounds.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
use style_traits::ParseError;
use values::generics::background::BackgroundSize as GenericBackgroundSize;
use values::specified::length::NonNegativeLengthOrPercentageOrAuto;

/// A specified value for the `background-size` property.
pub type BackgroundSize = GenericBackgroundSize<NonNegativeLengthOrPercentageOrAuto>;

impl Parse for BackgroundSize {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(width) = input.try(|i| NonNegativeLengthOrPercentageOrAuto::parse(context, i)) {
            let height = input
                .try(|i| NonNegativeLengthOrPercentageOrAuto::parse(context, i))
                .unwrap_or(NonNegativeLengthOrPercentageOrAuto::auto());
            return Ok(GenericBackgroundSize::Explicit { width, height });
        }
        Ok(try_match_ident_ignore_ascii_case! { input,
            "cover" => GenericBackgroundSize::Cover,
            "contain" => GenericBackgroundSize::Contain,
        })
    }
}

impl BackgroundSize {
    /// Returns `auto auto`.
    pub fn auto() -> Self {
        GenericBackgroundSize::Explicit {
            width: NonNegativeLengthOrPercentageOrAuto::auto(),
            height: NonNegativeLengthOrPercentageOrAuto::auto(),
        }
    }
}

/// One of the keywords for `background-repeat`.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq,
         SpecifiedValueInfo, ToComputedValue, ToCss)]
#[allow(missing_docs)]
pub enum BackgroundRepeatKeyword {
    Repeat,
    Space,
    Round,
    NoRepeat,
}

/// The specified value for the `background-repeat` property.
///
/// https://drafts.csswg.org/css-backgrounds/#the-background-repeat
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToCss)]
pub enum BackgroundRepeat {
    /// `repeat-x`
    RepeatX,
    /// `repeat-y`
    RepeatY,
    /// `[repeat | space | round | no-repeat]{1,2}`
    Keywords(BackgroundRepeatKeyword, Option<BackgroundRepeatKeyword>),
}

impl BackgroundRepeat {
    /// Returns the `repeat` value.
    #[inline]
    pub fn repeat() -> Self {
        BackgroundRepeat::Keywords(BackgroundRepeatKeyword::Repeat, None)
    }
}

impl Parse for BackgroundRepeat {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let ident = input.expect_ident_cloned()?;

        match_ignore_ascii_case! { &ident,
            "repeat-x" => return Ok(BackgroundRepeat::RepeatX),
            "repeat-y" => return Ok(BackgroundRepeat::RepeatY),
            _ => {},
        }

        let horizontal = match BackgroundRepeatKeyword::from_ident(&ident) {
            Ok(h) => h,
            Err(()) => {
                return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone())));
            },
        };

        let vertical = input.try(BackgroundRepeatKeyword::parse).ok();
        Ok(BackgroundRepeat::Keywords(horizontal, vertical))
    }
}
