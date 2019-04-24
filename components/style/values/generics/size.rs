/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic type for CSS properties that are composed by two dimensions.

use crate::parser::ParserContext;
use crate::Zero;
use cssparser::Parser;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

/// A generic size, for `border-*-radius` longhand properties, or
/// `border-spacing`.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToAnimatedValue,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[allow(missing_docs)]
#[repr(C)]
pub struct Size2D<L> {
    pub width: L,
    pub height: L,
}

impl<L> Size2D<L> {
    #[inline]
    /// Create a new `Size2D` for an area of given width and height.
    pub fn new(width: L, height: L) -> Self {
        Self { width, height }
    }

    /// Returns the width component.
    pub fn width(&self) -> &L {
        &self.width
    }

    /// Returns the height component.
    pub fn height(&self) -> &L {
        &self.height
    }

    /// Parse a `Size2D` with a given parsing function.
    pub fn parse_with<'i, 't, F>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        parse_one: F,
    ) -> Result<Self, ParseError<'i>>
    where
        L: Clone,
        F: Fn(&ParserContext, &mut Parser<'i, 't>) -> Result<L, ParseError<'i>>,
    {
        let first = parse_one(context, input)?;
        let second = input
            .try(|i| parse_one(context, i))
            .unwrap_or_else(|_| first.clone());
        Ok(Self::new(first, second))
    }
}

impl<L> ToCss for Size2D<L>
where
    L: ToCss + PartialEq,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.width.to_css(dest)?;

        if self.height != self.width {
            dest.write_str(" ")?;
            self.height.to_css(dest)?;
        }

        Ok(())
    }
}

impl<L: Zero> Zero for Size2D<L> {
    fn zero() -> Self {
        Self::new(L::zero(), L::zero())
    }

    fn is_zero(&self) -> bool {
        self.width.is_zero() && self.height.is_zero()
    }
}
