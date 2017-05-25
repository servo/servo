/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to borders.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use values::generics::border::BorderImageSlice as GenericBorderImageSlice;
use values::generics::border::BorderImageWidthSide as GenericBorderImageWidthSide;
use values::generics::rect::Rect;
use values::specified::{Number, NumberOrPercentage};
use values::specified::length::LengthOrPercentage;

/// A specified value for the `border-image-width` property.
pub type BorderImageWidth = Rect<BorderImageWidthSide>;

/// A specified value for a single side of a `border-image-width` property.
pub type BorderImageWidthSide = GenericBorderImageWidthSide<LengthOrPercentage, Number>;

/// A specified value for the `border-image-slice` property.
pub type BorderImageSlice = GenericBorderImageSlice<NumberOrPercentage>;

impl BorderImageWidthSide {
    /// Returns `1`.
    #[inline]
    pub fn one() -> Self {
        GenericBorderImageWidthSide::Number(Number::new(1.))
    }
}

impl Parse for BorderImageWidthSide {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(GenericBorderImageWidthSide::Auto);
        }

        if let Ok(len) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
            return Ok(GenericBorderImageWidthSide::Length(len));
        }

        let num = Number::parse_non_negative(context, input)?;
        Ok(GenericBorderImageWidthSide::Number(num))
    }
}

impl Parse for BorderImageSlice {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let mut fill = input.try(|i| i.expect_ident_matching("fill")).is_ok();
        let offsets = Rect::parse_with(context, input, NumberOrPercentage::parse_non_negative)?;
        if !fill {
            fill = input.try(|i| i.expect_ident_matching("fill")).is_ok();
        }
        Ok(GenericBorderImageSlice {
            offsets: offsets,
            fill: fill,
        })
    }
}
