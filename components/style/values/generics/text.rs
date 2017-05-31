/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for text properties.

use app_units::Au;
use cssparser::Parser;
use parser::ParserContext;
use properties::animated_properties::Animatable;
use std::fmt;
use style_traits::ToCss;

/// A generic spacing value for the `letter-spacing` and `word-spacing` properties.alloc
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
pub enum Spacing<Value> {
    /// `normal`
    Normal,
    /// `<value>`
    Value(Value),
}

impl<Value> Spacing<Value> {
    /// Returns `normal`.
    #[inline]
    pub fn normal() -> Self {
        Spacing::Normal
    }

    /// Parses.
    #[inline]
    pub fn parse_with<F>(
        context: &ParserContext,
        input: &mut Parser,
        parse: F)
        -> Result<Self, ()>
        where F: FnOnce(&ParserContext, &mut Parser) -> Result<Value, ()>
    {
        if input.try(|i| i.expect_ident_matching("normal")).is_ok() {
            return Ok(Spacing::Normal);
        }
        parse(context, input).map(Spacing::Value)
    }

    /// Returns the spacing value, if not `normal`.
    #[inline]
    pub fn value(&self) -> Option<&Value> {
        match *self {
            Spacing::Normal => None,
            Spacing::Value(ref value) => Some(value),
        }
    }
}

impl<Value> Animatable for Spacing<Value>
    where Value: Animatable + From<Au>,
{
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        if let (&Spacing::Normal, &Spacing::Normal) = (self, other) {
            return Ok(Spacing::Normal);
        }
        let zero = Value::from(Au(0));
        let this = self.value().unwrap_or(&zero);
        let other = other.value().unwrap_or(&zero);
        this.add_weighted(other, self_portion, other_portion).map(Spacing::Value)
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        let zero = Value::from(Au(0));
        let this = self.value().unwrap_or(&zero);
        let other = other.value().unwrap_or(&zero);
        this.compute_distance(other)
    }
}

impl<Value> ToCss for Spacing<Value>
    where Value: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        match *self {
            Spacing::Normal => dest.write_str("normal"),
            Spacing::Value(ref value) => value.to_css(dest),
        }
    }
}

/// A generic value for the `line-height` property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq)]
pub enum LineHeight<Number, LengthOrPercentage> {
    /// `normal`
    Normal,
    /// `-moz-block-height`
    #[cfg(feature = "gecko")]
    MozBlockHeight,
    /// `<number>`
    Number(Number),
    /// `<length-or-percentage>`
    Length(LengthOrPercentage),
}

impl<N, L> LineHeight<N, L> {
    /// Returns `normal`.
    #[inline]
    pub fn normal() -> Self {
        LineHeight::Normal
    }
}

impl<N, L> ToCss for LineHeight<N, L>
    where N: ToCss, L: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            LineHeight::Normal => dest.write_str("normal"),
            #[cfg(feature = "gecko")]
            LineHeight::MozBlockHeight => dest.write_str("-moz-block-height"),
            LineHeight::Number(ref number) => number.to_css(dest),
            LineHeight::Length(ref value) => value.to_css(dest),
        }
    }
}
