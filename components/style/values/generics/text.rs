/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for text properties.

use crate::parser::ParserContext;
use crate::values::animated::{Animate, Procedure, ToAnimatedZero};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use app_units::Au;
use cssparser::Parser;
use style_traits::ParseError;

/// A generic value for the `initial-letter` property.
#[derive(
    Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToComputedValue, ToCss,
)]
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
#[derive(
    Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToComputedValue, ToCss,
)]
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
        parse: F,
    ) -> Result<Self, ParseError<'i>>
    where
        F: FnOnce(&ParserContext, &mut Parser<'i, 't>) -> Result<Value, ParseError<'i>>,
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

impl<Value> Animate for Spacing<Value>
where
    Value: Animate + From<Au>,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if let (&Spacing::Normal, &Spacing::Normal) = (self, other) {
            return Ok(Spacing::Normal);
        }
        let zero = Value::from(Au(0));
        let this = self.value().unwrap_or(&zero);
        let other = other.value().unwrap_or(&zero);
        Ok(Spacing::Value(this.animate(other, procedure)?))
    }
}

impl<V> ComputeSquaredDistance for Spacing<V>
where
    V: ComputeSquaredDistance + From<Au>,
{
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let zero = V::from(Au(0));
        let this = self.value().unwrap_or(&zero);
        let other = other.value().unwrap_or(&zero);
        this.compute_squared_distance(other)
    }
}

impl<V> ToAnimatedZero for Spacing<V> {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}

/// A generic value for the `line-height` property.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToCss,
)]
pub enum LineHeight<Number, LengthPercentage> {
    /// `normal`
    Normal,
    /// `-moz-block-height`
    #[cfg(feature = "gecko")]
    MozBlockHeight,
    /// `<number>`
    Number(Number),
    /// `<length-or-percentage>`
    Length(LengthPercentage),
}

impl<N, L> ToAnimatedZero for LineHeight<N, L> {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}

impl<N, L> LineHeight<N, L> {
    /// Returns `normal`.
    #[inline]
    pub fn normal() -> Self {
        LineHeight::Normal
    }
}

/// A generic value for the `-moz-tab-size` property.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
)]
pub enum MozTabSize<Number, Length> {
    /// A number.
    Number(Number),
    /// A length.
    Length(Length),
}
