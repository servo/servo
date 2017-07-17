/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values that are related to transformations.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseError;
use style_traits::{ParseError, StyleParseError};
use values::computed::{Context, LengthOrPercentage as ComputedLengthOrPercentage};
use values::computed::{Percentage as ComputedPercentage, ToComputedValue};
use values::computed::transform::TimingFunction as ComputedTimingFunction;
use values::generics::transform::{StepPosition, TimingFunction as GenericTimingFunction};
use values::generics::transform::{TimingKeyword, TransformOrigin as GenericTransformOrigin};
use values::specified::{Integer, Number};
use values::specified::length::{Length, LengthOrPercentage};
use values::specified::position::{Side, X, Y};

/// The specified value of a CSS `<transform-origin>`
pub type TransformOrigin = GenericTransformOrigin<OriginComponent<X>, OriginComponent<Y>, Length>;

/// The specified value of a component of a CSS `<transform-origin>`.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToCss)]
pub enum OriginComponent<S> {
    /// `center`
    Center,
    /// `<lop>`
    Length(LengthOrPercentage),
    /// `<side>`
    Side(S),
}

/// A specified timing function.
pub type TimingFunction = GenericTimingFunction<Integer, Number>;

impl Parse for TransformOrigin {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let parse_depth = |input: &mut Parser| {
            input.try(|i| Length::parse(context, i)).unwrap_or(Length::from_px(0.))
        };
        match input.try(|i| OriginComponent::parse(context, i)) {
            Ok(x_origin @ OriginComponent::Center) => {
                if let Ok(y_origin) = input.try(|i| OriginComponent::parse(context, i)) {
                    let depth = parse_depth(input);
                    return Ok(Self::new(x_origin, y_origin, depth));
                }
                let y_origin = OriginComponent::Center;
                if let Ok(x_keyword) = input.try(X::parse) {
                    let x_origin = OriginComponent::Side(x_keyword);
                    let depth = parse_depth(input);
                    return Ok(Self::new(x_origin, y_origin, depth));
                }
                let depth = Length::from_px(0.);
                return Ok(Self::new(x_origin, y_origin, depth));
            },
            Ok(x_origin) => {
                if let Ok(y_origin) = input.try(|i| OriginComponent::parse(context, i)) {
                    let depth = parse_depth(input);
                    return Ok(Self::new(x_origin, y_origin, depth));
                }
                let y_origin = OriginComponent::Center;
                let depth = Length::from_px(0.);
                return Ok(Self::new(x_origin, y_origin, depth));
            },
            Err(_) => {},
        }
        let y_keyword = Y::parse(input)?;
        let y_origin = OriginComponent::Side(y_keyword);
        if let Ok(x_keyword) = input.try(X::parse) {
            let x_origin = OriginComponent::Side(x_keyword);
            let depth = parse_depth(input);
            return Ok(Self::new(x_origin, y_origin, depth));
        }
        if input.try(|i| i.expect_ident_matching("center")).is_ok() {
            let x_origin = OriginComponent::Center;
            let depth = parse_depth(input);
            return Ok(Self::new(x_origin, y_origin, depth));
        }
        let x_origin = OriginComponent::Center;
        let depth = Length::from_px(0.);
        Ok(Self::new(x_origin, y_origin, depth))
    }
}

impl<S> Parse for OriginComponent<S>
    where S: Parse,
{
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("center")).is_ok() {
            return Ok(OriginComponent::Center);
        }
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse(context, i)) {
            return Ok(OriginComponent::Length(lop));
        }
        let keyword = S::parse(context, input)?;
        Ok(OriginComponent::Side(keyword))
    }
}

impl<S> ToComputedValue for OriginComponent<S>
    where S: Side,
{
    type ComputedValue = ComputedLengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            OriginComponent::Center => {
                ComputedLengthOrPercentage::Percentage(ComputedPercentage(0.5))
            },
            OriginComponent::Length(ref length) => {
                length.to_computed_value(context)
            },
            OriginComponent::Side(ref keyword) => {
                let p = ComputedPercentage(if keyword.is_start() { 0. } else { 1. });
                ComputedLengthOrPercentage::Percentage(p)
            },
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        OriginComponent::Length(ToComputedValue::from_computed_value(computed))
    }
}

impl<S> OriginComponent<S> {
    /// `0%`
    pub fn zero() -> Self {
        OriginComponent::Length(LengthOrPercentage::Percentage(ComputedPercentage::zero()))
    }
}

#[cfg(feature = "gecko")]
#[inline]
fn allow_frames_timing() -> bool {
    use gecko_bindings::bindings;
    unsafe { bindings::Gecko_IsFramesTimingEnabled() }
}

#[cfg(feature = "servo")]
#[inline]
fn allow_frames_timing() -> bool { true }

impl Parse for TimingFunction {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(keyword) = input.try(TimingKeyword::parse) {
            return Ok(GenericTimingFunction::Keyword(keyword));
        }
        if let Ok(ident) = input.try(|i| i.expect_ident()) {
            let position = match_ignore_ascii_case! { &ident,
                "step-start" => StepPosition::Start,
                "step-end" => StepPosition::End,
                _ => return Err(SelectorParseError::UnexpectedIdent(ident.clone()).into()),
            };
            return Ok(GenericTimingFunction::Steps(Integer::new(1), position));
        }
        let function = input.expect_function()?;
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
                        return Err(StyleParseError::UnspecifiedError.into());
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
                "frames" => {
                    if allow_frames_timing() {
                        let frames = Integer::parse_with_minimum(context, i, 2)?;
                        Ok(GenericTimingFunction::Frames(frames))
                    } else {
                        Err(())
                    }
                },
                _ => Err(()),
            }).map_err(|()| StyleParseError::UnexpectedFunction(function).into())
        })
    }
}

impl ToComputedValue for TimingFunction {
    type ComputedValue = ComputedTimingFunction;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            GenericTimingFunction::Keyword(keyword) => {
                GenericTimingFunction::Keyword(keyword)
            },
            GenericTimingFunction::CubicBezier { x1, y1, x2, y2 } => {
                GenericTimingFunction::CubicBezier {
                    x1: x1.to_computed_value(context),
                    y1: y1.to_computed_value(context),
                    x2: x2.to_computed_value(context),
                    y2: y2.to_computed_value(context),
                }
            },
            GenericTimingFunction::Steps(steps, position) => {
                GenericTimingFunction::Steps(
                    steps.to_computed_value(context) as u32,
                    position,
                )
            },
            GenericTimingFunction::Frames(frames) => {
                GenericTimingFunction::Frames(
                    frames.to_computed_value(context) as u32,
                )
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            GenericTimingFunction::Keyword(keyword) => {
                GenericTimingFunction::Keyword(keyword)
            },
            GenericTimingFunction::CubicBezier { ref x1, ref y1, ref x2, ref y2 } => {
                GenericTimingFunction::CubicBezier {
                    x1: Number::from_computed_value(x1),
                    y1: Number::from_computed_value(y1),
                    x2: Number::from_computed_value(x2),
                    y2: Number::from_computed_value(y2),
                }
            },
            GenericTimingFunction::Steps(steps, position) => {
                GenericTimingFunction::Steps(
                    Integer::from_computed_value(&(steps as i32)),
                    position,
                )
            },
            GenericTimingFunction::Frames(frames) => {
                GenericTimingFunction::Frames(
                    Integer::from_computed_value(&(frames as i32)),
                )
            },
        }
    }
}
