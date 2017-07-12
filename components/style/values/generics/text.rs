/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for text properties.

use app_units::Au;
use cssparser::Parser;
use parser::ParserContext;
use properties::animated_properties::Animatable;
use style_traits::ParseError;

/// A generic value for the `initial-letter` property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue, ToCss)]
pub enum InitialLetter<Number, Integer> {
    /// `normal`
    Normal,
    /// `<number> <integer>?`
    Specified(Number, Option<Integer>),
}

impl<N, I> InitialLetter<N, I> {
    /// Returns `normal`.
    #[inline]
    pub fn normal() -> Self {
        InitialLetter::Normal
    }
}

/// A generic spacing value for the `letter-spacing` and `word-spacing` properties.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue, ToCss)]
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
    pub fn parse_with<'i, 't, F>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        parse: F)
        -> Result<Self, ParseError<'i>>
        where F: FnOnce(&ParserContext, &mut Parser<'i, 't>) -> Result<Value, ParseError<'i>>
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

/// A generic value for the `line-height` property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToCss)]
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
