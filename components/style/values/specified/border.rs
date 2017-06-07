/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to borders.

use app_units::Au;
use cssparser::Parser;
use parser::{Parse, ParserContext};
use values::computed::{Context, ToComputedValue};
use values::generics::border::BorderCornerRadius as GenericBorderCornerRadius;
use values::generics::border::BorderImageSideWidth as GenericBorderImageSideWidth;
use values::generics::border::BorderImageSlice as GenericBorderImageSlice;
use values::generics::border::BorderRadius as GenericBorderRadius;
use values::generics::rect::Rect;
use values::specified::{AllowQuirks, Number, NumberOrPercentage};
use values::specified::length::{Length, LengthOrPercentage};

/// A specified value for a single side of the `border-width` property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToCss)]
pub enum BorderSideWidth {
    /// `thin`
    Thin,
    /// `medium`
    Medium,
    /// `thick`
    Thick,
    /// `<length>`
    Length(Length),
}

/// A specified value for the `border-image-width` property.
pub type BorderImageWidth = Rect<BorderImageSideWidth>;

/// A specified value for a single side of a `border-image-width` property.
pub type BorderImageSideWidth = GenericBorderImageSideWidth<LengthOrPercentage, Number>;

/// A specified value for the `border-image-slice` property.
pub type BorderImageSlice = GenericBorderImageSlice<NumberOrPercentage>;

/// A specified value for the `border-radius` property.
pub type BorderRadius = GenericBorderRadius<LengthOrPercentage>;

/// A specified value for the `border-*-radius` longhand properties.
pub type BorderCornerRadius = GenericBorderCornerRadius<LengthOrPercentage>;

impl BorderSideWidth {
    /// Parses, with quirks.
    pub fn parse_quirky(
        context: &ParserContext,
        input: &mut Parser,
        allow_quirks: AllowQuirks)
        -> Result<Self, ()>
    {
        if let Ok(length) = input.try(|i| Length::parse_non_negative_quirky(context, i, allow_quirks)) {
            return Ok(BorderSideWidth::Length(length));
        }
        match_ignore_ascii_case! { &input.expect_ident()?,
            "thin" => Ok(BorderSideWidth::Thin),
            "medium" => Ok(BorderSideWidth::Medium),
            "thick" => Ok(BorderSideWidth::Thick),
            _ => Err(())
        }
    }
}

impl Parse for BorderSideWidth {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl ToComputedValue for BorderSideWidth {
    type ComputedValue = Au;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        // We choose the pixel length of the keyword values the same as both spec and gecko.
        // Spec: https://drafts.csswg.org/css-backgrounds-3/#line-width
        // Gecko: https://bugzilla.mozilla.org/show_bug.cgi?id=1312155#c0
        match *self {
            BorderSideWidth::Thin => Length::from_px(1.).to_computed_value(context),
            BorderSideWidth::Medium => Length::from_px(3.).to_computed_value(context),
            BorderSideWidth::Thick => Length::from_px(5.).to_computed_value(context),
            BorderSideWidth::Length(ref length) => length.to_computed_value(context)
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        BorderSideWidth::Length(ToComputedValue::from_computed_value(computed))
    }
}

impl BorderImageSideWidth {
    /// Returns `1`.
    #[inline]
    pub fn one() -> Self {
        GenericBorderImageSideWidth::Number(Number::new(1.))
    }
}

impl Parse for BorderImageSideWidth {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(GenericBorderImageSideWidth::Auto);
        }

        if let Ok(len) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
            return Ok(GenericBorderImageSideWidth::Length(len));
        }

        let num = Number::parse_non_negative(context, input)?;
        Ok(GenericBorderImageSideWidth::Number(num))
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

impl Parse for BorderRadius {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let widths = Rect::parse_with(context, input, LengthOrPercentage::parse_non_negative)?;
        let heights = if input.try(|i| i.expect_delim('/')).is_ok() {
            Rect::parse_with(context, input, LengthOrPercentage::parse_non_negative)?
        } else {
            widths.clone()
        };

        Ok(GenericBorderRadius {
            top_left: BorderCornerRadius::new(widths.0, heights.0),
            top_right: BorderCornerRadius::new(widths.1, heights.1),
            bottom_right: BorderCornerRadius::new(widths.2, heights.2),
            bottom_left: BorderCornerRadius::new(widths.3, heights.3),
        })
    }
}

impl Parse for BorderCornerRadius {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let first = LengthOrPercentage::parse_non_negative(context, input)?;
        let second = input
            .try(|i| LengthOrPercentage::parse_non_negative(context, i))
            .unwrap_or_else(|()| first.clone());
        Ok(Self::new(first, second))
    }
}
