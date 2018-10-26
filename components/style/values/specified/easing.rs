/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS Easing functions.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
use style_traits::{ParseError, StyleParseErrorKind};
use values::generics::easing::{StepPosition, TimingKeyword};
use values::generics::easing::TimingFunction as GenericTimingFunction;
use values::specified::{Integer, Number};

/// A specified timing function.
pub type TimingFunction = GenericTimingFunction<Integer, Number>;

impl Parse for TimingFunction {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(keyword) = input.try(TimingKeyword::parse) {
            return Ok(GenericTimingFunction::Keyword(keyword));
        }
        if let Ok(ident) = input.try(|i| i.expect_ident_cloned()) {
            let position = match_ignore_ascii_case! { &ident,
                "step-start" => StepPosition::Start,
                "step-end" => StepPosition::End,
                _ => {
                    return Err(input.new_custom_error(
                        SelectorParseErrorKind::UnexpectedIdent(ident.clone())
                    ));
                },
            };
            return Ok(GenericTimingFunction::Steps(Integer::new(1), position));
        }
        let location = input.current_source_location();
        let function = input.expect_function()?.clone();
        input.parse_nested_block(move |i| {
            (match_ignore_ascii_case! { &function,
                "cubic-bezier" => {
                    let x1 = Number::parse(context, i)?;
                    i.expect_comma()?;
                    let y1 = Number::parse(context, i)?;
                    i.expect_comma()?;
                    let x2 = Number::parse(context, i)?;
                    i.expect_comma()?;
                    let y2 = Number::parse(context, i)?;

                    if x1.get() < 0.0 || x1.get() > 1.0 || x2.get() < 0.0 || x2.get() > 1.0 {
                        return Err(i.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                    }

                    Ok(GenericTimingFunction::CubicBezier { x1, y1, x2, y2 })
                },
                "steps" => {
                    let steps = Integer::parse_positive(context, i)?;
                    let position = i.try(|i| {
                        i.expect_comma()?;
                        StepPosition::parse(i)
                    }).unwrap_or(StepPosition::End);
                    Ok(GenericTimingFunction::Steps(steps, position))
                },
                _ => Err(()),
            }).map_err(|()| {
                location.new_custom_error(
                    StyleParseErrorKind::UnexpectedFunction(function.clone())
                )
            })
        })
    }
}
