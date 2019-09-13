/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values in SVG

use crate::parser::{Parse, ParserContext};
use cssparser::Parser;
use style_traits::ParseError;

/// The fallback of an SVG paint server value.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericSVGPaintFallback<C> {
    /// The `none` keyword.
    None,
    /// A magic value that represents no fallback specified and serializes to
    /// the empty string.
    #[css(skip)]
    Unset,
    /// A color.
    Color(C),
}

pub use self::GenericSVGPaintFallback as SVGPaintFallback;

/// An SVG paint value
///
/// <https://www.w3.org/TR/SVG2/painting.html#SpecifyingPaint>
#[animation(no_bound(Url))]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericSVGPaint<Color, Url> {
    /// The paint source.
    pub kind: GenericSVGPaintKind<Color, Url>,
    /// The fallback color.
    pub fallback: GenericSVGPaintFallback<Color>,
}

pub use self::GenericSVGPaint as SVGPaint;

impl<C, U> Default for SVGPaint<C, U> {
    fn default() -> Self {
        Self {
            kind: SVGPaintKind::None,
            fallback: SVGPaintFallback::Unset,
        }
    }
}

/// An SVG paint value without the fallback.
///
/// Whereas the spec only allows PaintServer to have a fallback, Gecko lets the
/// context properties have a fallback as well.
#[animation(no_bound(U))]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericSVGPaintKind<C, U> {
    /// `none`
    #[animation(error)]
    None,
    /// `<color>`
    Color(C),
    /// `url(...)`
    #[animation(error)]
    PaintServer(U),
    /// `context-fill`
    ContextFill,
    /// `context-stroke`
    ContextStroke,
}

pub use self::GenericSVGPaintKind as SVGPaintKind;

impl<C: Parse, U: Parse> Parse for SVGPaint<C, U> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let kind = SVGPaintKind::parse(context, input)?;
        if matches!(kind, SVGPaintKind::None | SVGPaintKind::Color(..)) {
            return Ok(SVGPaint {
                kind,
                fallback: SVGPaintFallback::Unset,
            });
        }
        let fallback = input
            .try(|i| SVGPaintFallback::parse(context, i))
            .unwrap_or(SVGPaintFallback::Unset);
        Ok(SVGPaint { kind, fallback })
    }
}

/// An SVG length value supports `context-value` in addition to length.
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
    ToResolvedValue,
    ToShmem,
)]
pub enum SVGLength<L> {
    /// `<length> | <percentage> | <number>`
    LengthPercentage(L),
    /// `context-value`
    #[animation(error)]
    ContextValue,
}

/// Generic value for stroke-dasharray.
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericSVGStrokeDashArray<L> {
    /// `[ <length> | <percentage> | <number> ]#`
    #[css(comma)]
    Values(#[css(if_empty = "none", iterable)] crate::OwnedSlice<L>),
    /// `context-value`
    ContextValue,
}

pub use self::GenericSVGStrokeDashArray as SVGStrokeDashArray;

/// An SVG opacity value accepts `context-{fill,stroke}-opacity` in
/// addition to opacity value.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericSVGOpacity<OpacityType> {
    /// `<opacity-value>`
    Opacity(OpacityType),
    /// `context-fill-opacity`
    #[animation(error)]
    ContextFillOpacity,
    /// `context-stroke-opacity`
    #[animation(error)]
    ContextStrokeOpacity,
}

pub use self::GenericSVGOpacity as SVGOpacity;
