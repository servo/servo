/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values that are related to transformations.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::ToCss;
use values::computed::{LengthOrPercentage as ComputedLengthOrPercentage, Context, ToComputedValue};
use values::generics::transform::TransformOrigin as GenericTransformOrigin;
use values::specified::length::{Length, LengthOrPercentage};
use values::specified::position::{Side, X, Y};

/// The specified value of a CSS `<transform-origin>`
pub type TransformOrigin = GenericTransformOrigin<OriginComponent<X>, OriginComponent<Y>, Length>;

/// The specified value of a component of a CSS `<transform-origin>`.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq)]
pub enum OriginComponent<S> {
    /// `center`
    Center,
    /// `<lop>`
    Length(LengthOrPercentage),
    /// `<side>`
    Side(S),
}

impl Parse for TransformOrigin {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
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
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
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

impl<S: ToCss> ToCss for OriginComponent<S>
    where S: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            OriginComponent::Center => dest.write_str("center"),
            OriginComponent::Length(ref lop) => lop.to_css(dest),
            OriginComponent::Side(ref keyword) => keyword.to_css(dest),
        }
    }
}

impl<S> ToComputedValue for OriginComponent<S>
    where S: Side,
{
    type ComputedValue = ComputedLengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            OriginComponent::Center => {
                ComputedLengthOrPercentage::Percentage(0.5)
            },
            OriginComponent::Length(ref length) => {
                length.to_computed_value(context)
            },
            OriginComponent::Side(ref keyword) => {
                let p = if keyword.is_start() { 0. } else { 1. };
                ComputedLengthOrPercentage::Percentage(p)
            },
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        OriginComponent::Length(ToComputedValue::from_computed_value(computed))
    }
}
