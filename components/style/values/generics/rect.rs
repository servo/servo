/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values that are composed of four sides.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::ToCss;

/// A CSS value made of four components, where its `ToCss` impl will try to
/// serialize as few components as possible, like for example in `border-width`.
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Rect<T>(pub T, pub T, pub T, pub T);

impl<T> Rect<T> {
    /// Returns a new `Rect<T>` value.
    pub fn new(first: T, second: T, third: T, fourth: T) -> Self {
        Rect(first, second, third, fourth)
    }
}

impl<T> Rect<T>
    where T: Clone
{
    /// Parses a new `Rect<T>` value with the given parse function.
    pub fn parse_with<Parse>(
        context: &ParserContext,
        input: &mut Parser,
        parse: Parse)
        -> Result<Self, ()>
        where Parse: Fn(&ParserContext, &mut Parser) -> Result<T, ()>
    {
        let first = parse(context, input)?;
        let second = if let Ok(second) = input.try(|i| parse(context, i)) { second } else {
            // <first>
            return Ok(Self::new(first.clone(), first.clone(), first.clone(), first));
        };
        let third = if let Ok(third) = input.try(|i| parse(context, i)) { third } else {
            // <first> <second>
            return Ok(Self::new(first.clone(), second.clone(), first, second));
        };
        let fourth = if let Ok(fourth) = input.try(|i| parse(context, i)) { fourth } else {
            // <first> <second> <third>
            return Ok(Self::new(first, second.clone(), third, second));
        };
        // <first> <second> <third> <fourth>
        Ok(Self::new(first, second, third, fourth))
    }
}

impl<T> From<T> for Rect<T>
    where T: Clone
{
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value.clone(), value.clone(), value.clone(), value)
    }
}

impl<T> Parse for Rect<T>
    where T: Clone + Parse
{
    #[inline]
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        Self::parse_with(context, input, T::parse)
    }
}

impl<T> ToCss for Rect<T>
    where T: PartialEq + ToCss
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        self.0.to_css(dest)?;
        let same_vertical = self.0 == self.2;
        let same_horizontal = self.1 == self.3;
        if same_vertical && same_horizontal && self.0 == self.1 {
            return Ok(());
        }
        dest.write_str(" ")?;
        self.1.to_css(dest)?;
        if same_vertical && same_horizontal {
            return Ok(());
        }
        dest.write_str(" ")?;
        self.2.to_css(dest)?;
        if same_horizontal {
            return Ok(());
        }
        dest.write_str(" ")?;
        self.3.to_css(dest)
    }
}
