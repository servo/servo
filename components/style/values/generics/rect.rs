/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values that are composed of four sides.

use crate::parser::{Parse, ParserContext};
use cssparser::Parser;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

/// A CSS value made of four components, where its `ToCss` impl will try to
/// serialize as few components as possible, like for example in `border-width`.
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
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct Rect<T>(pub T, pub T, pub T, pub T);

impl<T> Rect<T> {
    /// Returns a new `Rect<T>` value.
    pub fn new(first: T, second: T, third: T, fourth: T) -> Self {
        Rect(first, second, third, fourth)
    }
}

impl<T> Rect<T>
where
    T: Clone,
{
    /// Returns a rect with all the values equal to `v`.
    pub fn all(v: T) -> Self {
        Rect::new(v.clone(), v.clone(), v.clone(), v)
    }

    /// Parses a new `Rect<T>` value with the given parse function.
    pub fn parse_with<'i, 't, Parse>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        parse: Parse,
    ) -> Result<Self, ParseError<'i>>
    where
        Parse: Fn(&ParserContext, &mut Parser<'i, 't>) -> Result<T, ParseError<'i>>,
    {
        let first = parse(context, input)?;
        let second = if let Ok(second) = input.try(|i| parse(context, i)) {
            second
        } else {
            // <first>
            return Ok(Self::new(
                first.clone(),
                first.clone(),
                first.clone(),
                first,
            ));
        };
        let third = if let Ok(third) = input.try(|i| parse(context, i)) {
            third
        } else {
            // <first> <second>
            return Ok(Self::new(first.clone(), second.clone(), first, second));
        };
        let fourth = if let Ok(fourth) = input.try(|i| parse(context, i)) {
            fourth
        } else {
            // <first> <second> <third>
            return Ok(Self::new(first, second.clone(), third, second));
        };
        // <first> <second> <third> <fourth>
        Ok(Self::new(first, second, third, fourth))
    }
}

impl<T> Parse for Rect<T>
where
    T: Clone + Parse,
{
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_with(context, input, T::parse)
    }
}

impl<T> ToCss for Rect<T>
where
    T: PartialEq + ToCss,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
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
