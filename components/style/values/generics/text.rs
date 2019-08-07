/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for text properties.

use crate::parser::ParserContext;
use crate::values::animated::ToAnimatedZero;
use cssparser::Parser;
use style_traits::ParseError;

/// A generic value for the `initial-letter` property.
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
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
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
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
}

#[cfg(feature = "gecko")]
fn line_height_moz_block_height_enabled(context: &ParserContext) -> bool {
    context.in_ua_sheet() ||
        static_prefs::pref!("layout.css.line-height-moz-block-height.content.enabled")
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
    ToShmem,
    ToResolvedValue,
    Parse,
)]
#[repr(C, u8)]
pub enum GenericLineHeight<N, L> {
    /// `normal`
    Normal,
    /// `-moz-block-height`
    #[cfg(feature = "gecko")]
    #[parse(condition = "line_height_moz_block_height_enabled")]
    MozBlockHeight,
    /// `<number>`
    Number(N),
    /// `<length-percentage>`
    Length(L),
}

pub use self::GenericLineHeight as LineHeight;

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
