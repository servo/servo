/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS Easing functions.
use crate::parser::{Parse, ParserContext};
use crate::piecewise_linear::{PiecewiseLinearFunction, PiecewiseLinearFunctionBuildParameters};
use crate::values::computed::easing::TimingFunction as ComputedTimingFunction;
use crate::values::computed::{Context, ToComputedValue};
use crate::values::generics::easing::TimingFunction as GenericTimingFunction;
use crate::values::generics::easing::{StepPosition, TimingKeyword};
use crate::values::specified::{Integer, Number, Percentage};
use cssparser::{Delimiter, Parser, Token};
use selectors::parser::SelectorParseErrorKind;
use std::iter::FromIterator;
use style_traits::{ParseError, StyleParseErrorKind};

/// An entry for linear easing function.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct LinearStop {
    /// Output of the function at the given point.
    pub output: Number,
    /// Playback progress at which this output is given.
    #[css(skip_if = "Option::is_none")]
    pub input: Option<Percentage>,
}

/// A list of specified linear stops.
#[derive(Clone, Default, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
#[css(comma)]
pub struct LinearStops {
    #[css(iterable)]
    entries: crate::OwnedSlice<LinearStop>,
}

impl LinearStops {
    fn new(list: crate::OwnedSlice<LinearStop>) -> Self {
        LinearStops { entries: list }
    }
}

/// A specified timing function.
pub type TimingFunction = GenericTimingFunction<Integer, Number, LinearStops>;

#[cfg(feature = "gecko")]
fn linear_timing_function_enabled() -> bool {
    static_prefs::pref!("layout.css.linear-easing-function.enabled")
}

#[cfg(feature = "servo")]
fn linear_timing_function_enabled() -> bool {
    false
}

impl Parse for TimingFunction {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(keyword) = input.try_parse(TimingKeyword::parse) {
            return Ok(GenericTimingFunction::Keyword(keyword));
        }
        if let Ok(ident) = input.try_parse(|i| i.expect_ident_cloned()) {
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
            match_ignore_ascii_case! { &function,
                "cubic-bezier" => Self::parse_cubic_bezier(context, i),
                "steps" => Self::parse_steps(context, i),
                "linear" => Self::parse_linear_function(context, i),
                _ => Err(location.new_custom_error(StyleParseErrorKind::UnexpectedFunction(function.clone()))),
            }
        })
    }
}

impl TimingFunction {
    fn parse_cubic_bezier<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let x1 = Number::parse(context, input)?;
        input.expect_comma()?;
        let y1 = Number::parse(context, input)?;
        input.expect_comma()?;
        let x2 = Number::parse(context, input)?;
        input.expect_comma()?;
        let y2 = Number::parse(context, input)?;

        if x1.get() < 0.0 || x1.get() > 1.0 || x2.get() < 0.0 || x2.get() > 1.0 {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(GenericTimingFunction::CubicBezier { x1, y1, x2, y2 })
    }

    fn parse_steps<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let steps = Integer::parse_positive(context, input)?;
        let position = input
            .try_parse(|i| {
                i.expect_comma()?;
                StepPosition::parse(context, i)
            })
            .unwrap_or(StepPosition::End);

        // jump-none accepts a positive integer greater than 1.
        // FIXME(emilio): The spec asks us to avoid rejecting it at parse
        // time except until computed value time.
        //
        // It's not totally clear it's worth it though, and no other browser
        // does this.
        if position == StepPosition::JumpNone && steps.value() <= 1 {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(GenericTimingFunction::Steps(steps, position))
    }

    fn parse_linear_function<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if !linear_timing_function_enabled() {
            return Err(input.new_custom_error(StyleParseErrorKind::ExperimentalProperty));
        }
        let mut result = vec![];
        // Closely follows `parse_comma_separated`, but can generate multiple entries for one comma-separated entry.
        loop {
            input.parse_until_before(Delimiter::Comma, |i| {
                let mut input_start = i.try_parse(|i| Percentage::parse(context, i)).ok();
                let mut input_end = i.try_parse(|i| Percentage::parse(context, i)).ok();

                let output = Number::parse(context, i)?;
                if input_start.is_none() {
                    debug_assert!(input_end.is_none(), "Input end parsed without input start?");
                    input_start = i.try_parse(|i| Percentage::parse(context, i)).ok();
                    input_end = i.try_parse(|i| Percentage::parse(context, i)).ok();
                }
                result.push(LinearStop {
                    output,
                    input: input_start.into(),
                });
                if input_end.is_some() {
                    debug_assert!(
                        input_start.is_some(),
                        "Input end valid but not input start?"
                    );
                    result.push(LinearStop {
                        output,
                        input: input_end.into(),
                    });
                }

                Ok(())
            })?;

            match input.next() {
                Err(_) => break,
                Ok(&Token::Comma) => continue,
                Ok(_) => unreachable!(),
            }
        }
        if result.len() < 2 {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(GenericTimingFunction::LinearFunction(LinearStops::new(
            crate::OwnedSlice::from(result),
        )))
    }
}

impl LinearStop {
    /// Convert this type to entries that can be used to build PiecewiseLinearFunction.
    pub fn to_piecewise_linear_build_parameters(
        x: &LinearStop,
    ) -> PiecewiseLinearFunctionBuildParameters {
        (x.output.get(), x.input.map(|x| x.get()))
    }
}

// We need this for converting the specified TimingFunction into computed TimingFunction without
// Context (for some FFIs in glue.rs). In fact, we don't really need Context to get the computed
// value of TimingFunction.
impl TimingFunction {
    /// Generate the ComputedTimingFunction without Context.
    pub fn to_computed_value_without_context(&self) -> ComputedTimingFunction {
        match &self {
            GenericTimingFunction::Steps(steps, pos) => {
                GenericTimingFunction::Steps(steps.value(), *pos)
            },
            GenericTimingFunction::CubicBezier { x1, y1, x2, y2 } => {
                GenericTimingFunction::CubicBezier {
                    x1: x1.get(),
                    y1: y1.get(),
                    x2: x2.get(),
                    y2: y2.get(),
                }
            },
            GenericTimingFunction::Keyword(keyword) => GenericTimingFunction::Keyword(*keyword),
            GenericTimingFunction::LinearFunction(steps) => {
                GenericTimingFunction::LinearFunction(PiecewiseLinearFunction::from_iter(
                    steps
                        .entries
                        .iter()
                        .map(|e| LinearStop::to_piecewise_linear_build_parameters(e)),
                ))
            },
        }
    }
}

impl ToComputedValue for TimingFunction {
    type ComputedValue = ComputedTimingFunction;
    fn to_computed_value(&self, _: &Context) -> Self::ComputedValue {
        self.to_computed_value_without_context()
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match &computed {
            ComputedTimingFunction::Steps(steps, pos) => Self::Steps(Integer::new(*steps), *pos),
            ComputedTimingFunction::CubicBezier { x1, y1, x2, y2 } => Self::CubicBezier {
                x1: Number::new(*x1),
                y1: Number::new(*y1),
                x2: Number::new(*x2),
                y2: Number::new(*y2),
            },
            ComputedTimingFunction::Keyword(keyword) => GenericTimingFunction::Keyword(*keyword),
            ComputedTimingFunction::LinearFunction(function) => {
                GenericTimingFunction::LinearFunction(LinearStops {
                    entries: crate::OwnedSlice::from_iter(function.iter().map(|e| LinearStop {
                        output: Number::new(e.y),
                        input: Some(Percentage::new(e.x)).into(),
                    })),
                })
            },
        }
    }
}
