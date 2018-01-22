/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic type for CSS properties that are composed by two dimensions.

use cssparser::Parser;
use euclid::Size2D;
use parser::ParserContext;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};
use values::animated::ToAnimatedValue;

/// A generic size, for `border-*-radius` longhand properties, or
/// `border-spacing`.
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug)]
#[derive(MallocSizeOf, PartialEq, ToComputedValue)]
pub struct Size<L>(pub Size2D<L>);

impl<L> Size<L> {
    #[inline]
    /// Create a new `Size` for an area of given width and height.
    pub fn new(width: L, height: L) -> Size<L> {
        Size(Size2D::new(width, height))
    }

    /// Returns the width component.
    pub fn width(&self) -> &L {
        &self.0.width
    }

    /// Returns the height component.
    pub fn height(&self) -> &L {
        &self.0.height
    }

    /// Parse a `Size` with a given parsing function.
    pub fn parse_with<'i, 't, F>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        parse_one: F,
    ) -> Result<Self, ParseError<'i>>
    where
        L: Clone,
        F: Fn(&ParserContext, &mut Parser<'i, 't>) -> Result<L, ParseError<'i>>
    {
        let first = parse_one(context, input)?;
        let second = input
            .try(|i| parse_one(context, i))
            .unwrap_or_else(|_| first.clone());
        Ok(Self::new(first, second))
    }
}

impl<L> ToCss for Size<L>
where L:
    ToCss + PartialEq,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.0.width.to_css(dest)?;

        if self.0.height != self.0.width {
            dest.write_str(" ")?;
            self.0.height.to_css(dest)?;
        }

        Ok(())
    }
}

impl<L> ToAnimatedValue for Size<L>
where L:
    ToAnimatedValue,
{
    type AnimatedValue = Size<L::AnimatedValue>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        Size(Size2D::new(
            self.0.width.to_animated_value(),
            self.0.height.to_animated_value(),
        ))
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        Size(Size2D::new(
            L::from_animated_value(animated.0.width),
            L::from_animated_value(animated.0.height),
        ))
    }
}
