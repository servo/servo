/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS Easing functions.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
use style_traits::{ParseError, StyleParseErrorKind};
use values::computed::{Context, TimingFunction as ComputedTimingFunction, ToComputedValue};
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

impl ToComputedValue for TimingFunction {
    type ComputedValue = ComputedTimingFunction;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            GenericTimingFunction::Keyword(keyword) => GenericTimingFunction::Keyword(keyword),
            GenericTimingFunction::CubicBezier { x1, y1, x2, y2 } => {
                GenericTimingFunction::CubicBezier {
                    x1: x1.to_computed_value(context),
                    y1: y1.to_computed_value(context),
                    x2: x2.to_computed_value(context),
                    y2: y2.to_computed_value(context),
                }
            },
            GenericTimingFunction::Steps(steps, position) => {
                GenericTimingFunction::Steps(steps.to_computed_value(context) as u32, position)
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            GenericTimingFunction::Keyword(keyword) => GenericTimingFunction::Keyword(keyword),
            GenericTimingFunction::CubicBezier {
                ref x1,
                ref y1,
                ref x2,
                ref y2,
            } => GenericTimingFunction::CubicBezier {
                x1: Number::from_computed_value(x1),
                y1: Number::from_computed_value(y1),
                x2: Number::from_computed_value(x2),
                y2: Number::from_computed_value(y2),
            },
            GenericTimingFunction::Steps(steps, position) => GenericTimingFunction::Steps(
                Integer::from_computed_value(&(steps as i32)),
                position,
            ),
        }
    }
}
