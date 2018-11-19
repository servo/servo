/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS Easing functions.

use crate::parser::{Parse, ParserContext};
use crate::values::computed::easing::TimingFunction as ComputedTimingFunction;
use crate::values::generics::easing::TimingFunction as GenericTimingFunction;
use crate::values::generics::easing::{StepPosition, TimingKeyword};
use crate::values::specified::{Integer, Number};
use cssparser::Parser;
use selectors::parser::SelectorParseErrorKind;
use style_traits::{ParseError, StyleParseErrorKind};

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
                        StepPosition::parse(context, i)
                    }).unwrap_or(StepPosition::End);

                    // jump-none accepts a positive integer greater than 1.
                    // FIXME(emilio): The spec asks us to avoid rejecting it at parse
                    // time except until computed value time.
                    //
                    // It's not totally clear it's worth it though, and no other browser
                    // does this.
                    if position == StepPosition::JumpNone && 2 > steps.value() {
                        return Err(i.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                    }
                    Ok(GenericTimingFunction::Steps(steps, position))
                },
                _ => Err(()),
            })
            .map_err(|()| {
                location.new_custom_error(StyleParseErrorKind::UnexpectedFunction(function.clone()))
            })
        })
    }
}

// We need this for converting the specified TimingFunction into computed TimingFunction without
// Context (for some FFIs in glue.rs). In fact, we don't really need Context to get the computed
// value of TimingFunction.
impl TimingFunction {
    /// Generate the ComputedTimingFunction without Context.
    pub fn to_computed_value_without_context(&self) -> ComputedTimingFunction {
        match *self {
            GenericTimingFunction::Steps(steps, pos) => {
                GenericTimingFunction::Steps(steps.value(), pos)
            },
            GenericTimingFunction::CubicBezier { x1, y1, x2, y2 } => {
                GenericTimingFunction::CubicBezier {
                    x1: x1.get(),
                    y1: y1.get(),
                    x2: x2.get(),
                    y2: y2.get(),
                }
            },
            GenericTimingFunction::Keyword(keyword) => GenericTimingFunction::Keyword(keyword),
        }
    }
}
