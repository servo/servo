/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values that are composed of four sides.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::ToCss;

/// A CSS value made of four sides: top, right, bottom, and left.
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Rect<T> {
    /// Top
    pub top: T,
    /// Right.
    pub right: T,
    /// Bottom.
    pub bottom: T,
    /// Left.
    pub left: T,
}

impl<T> Rect<T> {
    /// Returns a new `Rect<T>` value.
    pub fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Rect {
            top: top,
            right: right,
            bottom: bottom,
            left: left,
        }
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
        let top = parse(context, input)?;
        let right = if let Ok(right) = input.try(|i| parse(context, i)) { right } else {
            // <top>
            return Ok(Self::new(top.clone(), top.clone(), top.clone(), top));
        };
        let bottom = if let Ok(bottom) = input.try(|i| parse(context, i)) { bottom } else {
            // <top> <right>
            return Ok(Self::new(top.clone(), right.clone(), top, right));
        };
        let left = if let Ok(left) = input.try(|i| parse(context, i)) { left } else {
            // <top> <right> <bottom>
            return Ok(Self::new(top, right.clone(), bottom, right));
        };
        // <top> <right> <bottom> <left>
        Ok(Self::new(top, right, bottom, left))
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
        self.top.to_css(dest)?;
        let same_vertical = self.top == self.bottom;
        let same_horizontal = self.right == self.left;
        if same_vertical && same_horizontal && self.top == self.right {
            return Ok(());
        }
        dest.write_str(" ")?;
        self.right.to_css(dest)?;
        if same_vertical && same_horizontal {
            return Ok(());
        }
        dest.write_str(" ")?;
        self.bottom.to_css(dest)?;
        if same_horizontal {
            return Ok(());
        }
        dest.write_str(" ")?;
        self.left.to_css(dest)
    }
}
