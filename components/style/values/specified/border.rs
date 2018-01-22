/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to borders.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
use style_traits::ParseError;
use values::computed::{self, Context, ToComputedValue};
use values::generics::border::BorderCornerRadius as GenericBorderCornerRadius;
use values::generics::border::BorderImageSideWidth as GenericBorderImageSideWidth;
use values::generics::border::BorderImageSlice as GenericBorderImageSlice;
use values::generics::border::BorderRadius as GenericBorderRadius;
use values::generics::border::BorderSpacing as GenericBorderSpacing;
use values::generics::rect::Rect;
use values::generics::size::Size;
use values::specified::{AllowQuirks, Number, NumberOrPercentage};
use values::specified::length::{Length, LengthOrPercentage, NonNegativeLength};

/// A specified value for a single side of the `border-width` property.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
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

/// A specified value for the `border-spacing` longhand properties.
pub type BorderSpacing = GenericBorderSpacing<NonNegativeLength>;

impl BorderSideWidth {
    /// Parses, with quirks.
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks)
        -> Result<Self, ParseError<'i>>
    {
        if let Ok(length) = input.try(|i| Length::parse_non_negative_quirky(context, i, allow_quirks)) {
            return Ok(BorderSideWidth::Length(length));
        }
        try_match_ident_ignore_ascii_case! { input,
            "thin" => Ok(BorderSideWidth::Thin),
            "medium" => Ok(BorderSideWidth::Medium),
            "thick" => Ok(BorderSideWidth::Thick),
        }
    }
}

impl Parse for BorderSideWidth {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl ToComputedValue for BorderSideWidth {
    type ComputedValue = computed::NonNegativeLength;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        // We choose the pixel length of the keyword values the same as both spec and gecko.
        // Spec: https://drafts.csswg.org/css-backgrounds-3/#line-width
        // Gecko: https://bugzilla.mozilla.org/show_bug.cgi?id=1312155#c0
        match *self {
            BorderSideWidth::Thin => Length::from_px(1.).to_computed_value(context),
            BorderSideWidth::Medium => Length::from_px(3.).to_computed_value(context),
            BorderSideWidth::Thick => Length::from_px(5.).to_computed_value(context),
            BorderSideWidth::Length(ref length) => length.to_computed_value(context),
        }.into()
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        BorderSideWidth::Length(ToComputedValue::from_computed_value(&computed.0))
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
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
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
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
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
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
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
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        Size::parse_with(context, input, LengthOrPercentage::parse_non_negative)
            .map(GenericBorderCornerRadius)
    }
}

impl Parse for BorderSpacing {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        Size::parse_with(context, input, |context, input| {
            Length::parse_non_negative_quirky(context, input, AllowQuirks::Yes).map(From::from)
        }).map(GenericBorderSpacing)
    }
}

define_css_keyword_enum! { RepeatKeyword:
    "stretch" => Stretch,
    "repeat" => Repeat,
    "round" => Round,
    "space" => Space
}

/// The specified value for the `border-image-repeat` property.
///
/// https://drafts.csswg.org/css-backgrounds/#the-border-image-repeat
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub struct BorderImageRepeat(pub RepeatKeyword, pub Option<RepeatKeyword>);

impl BorderImageRepeat {
    /// Returns the `repeat` value.
    #[inline]
    pub fn repeat() -> Self {
        BorderImageRepeat(RepeatKeyword::Repeat, None)
    }
}

impl Parse for BorderImageRepeat {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let ident = input.expect_ident_cloned()?;
        let horizontal = match RepeatKeyword::from_ident(&ident) {
            Ok(h) => h,
            Err(()) => {
                return Err(input.new_custom_error(
                    SelectorParseErrorKind::UnexpectedIdent(ident.clone())
                ));
            }
        };

        let vertical = input.try(RepeatKeyword::parse).ok();
        Ok(BorderImageRepeat(horizontal, vertical))
    }
}
