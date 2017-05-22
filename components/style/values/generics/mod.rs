/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types that share their serialization implementations
//! for both specified and computed values.

use counter_style::parse_counter_style_name;
use cssparser::Parser;
use euclid::size::Size2D;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::{HasViewportPercentage, ToCss};
use super::CustomIdent;

pub use self::basic_shape::serialize_radius_values;

pub mod basic_shape;
pub mod grid;
pub mod image;
pub mod position;

#[derive(Clone, Debug, PartialEq, ToComputedValue)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A type for representing CSS `width` and `height` values.
pub struct BorderRadiusSize<L>(pub Size2D<L>);

impl<L> HasViewportPercentage for BorderRadiusSize<L> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool { false }
}

impl<L: Clone> From<L> for BorderRadiusSize<L> {
    fn from(other: L) -> Self {
        Self::new(other.clone(), other)
    }
}

impl<L> BorderRadiusSize<L> {
    #[inline]
    /// Create a new `BorderRadiusSize` for an area of given width and height.
    pub fn new(width: L, height: L) -> BorderRadiusSize<L> {
        BorderRadiusSize(Size2D::new(width, height))
    }
}

impl<L: Clone> BorderRadiusSize<L> {
    #[inline]
    /// Create a new `BorderRadiusSize` for a circle of given radius.
    pub fn circle(radius: L) -> BorderRadiusSize<L> {
        BorderRadiusSize(Size2D::new(radius.clone(), radius))
    }
}

impl<L: ToCss> ToCss for BorderRadiusSize<L> {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.width.to_css(dest)?;
        dest.write_str(" ")?;
        self.0.height.to_css(dest)
    }
}

/// https://drafts.csswg.org/css-counter-styles/#typedef-counter-style
///
/// Since wherever <counter-style> is used, 'none' is a valid value as
/// well, we combine them into one type to make code simpler.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CounterStyleOrNone {
    /// none
    None_,
    /// <counter-style-name>
    Name(CustomIdent),
}

impl CounterStyleOrNone {
    /// disc value
    pub fn disc() -> Self {
        CounterStyleOrNone::Name(CustomIdent(atom!("disc")))
    }

    /// decimal value
    pub fn decimal() -> Self {
        CounterStyleOrNone::Name(CustomIdent(atom!("decimal")))
    }
}

no_viewport_percentage!(CounterStyleOrNone);

impl Parse for CounterStyleOrNone {
    fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        input.try(|input| {
            parse_counter_style_name(input).map(CounterStyleOrNone::Name)
        }).or_else(|_| {
            input.expect_ident_matching("none").map(|_| CounterStyleOrNone::None_)
        })
    }
}

impl ToCss for CounterStyleOrNone {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self {
            &CounterStyleOrNone::None_ => dest.write_str("none"),
            &CounterStyleOrNone::Name(ref name) => name.to_css(dest),
        }
    }
}
