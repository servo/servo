/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to borders.

use crate::parser::{Parse, ParserContext};
use crate::values::computed::{self, Context, ToComputedValue};
use crate::values::generics::border::BorderCornerRadius as GenericBorderCornerRadius;
use crate::values::generics::border::BorderImageSideWidth as GenericBorderImageSideWidth;
use crate::values::generics::border::BorderImageSlice as GenericBorderImageSlice;
use crate::values::generics::border::BorderRadius as GenericBorderRadius;
use crate::values::generics::border::BorderSpacing as GenericBorderSpacing;
use crate::values::generics::rect::Rect;
use crate::values::generics::size::Size2D;
use crate::values::specified::length::{NonNegativeLength, NonNegativeLengthPercentage};
use crate::values::specified::{AllowQuirks, NonNegativeNumber, NonNegativeNumberOrPercentage};
use crate::Zero;
use cssparser::Parser;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

/// A specified value for a single side of a `border-style` property.
///
/// The order here corresponds to the integer values from the border conflict
/// resolution rules in CSS 2.1 § 17.6.2.1. Higher values override lower values.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    FromPrimitive,
    MallocSizeOf,
    Ord,
    Parse,
    PartialEq,
    PartialOrd,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum BorderStyle {
    Hidden,
    None,
    Inset,
    Groove,
    Outset,
    Ridge,
    Dotted,
    Dashed,
    Solid,
    Double,
}

impl BorderStyle {
    /// Whether this border style is either none or hidden.
    #[inline]
    pub fn none_or_hidden(&self) -> bool {
        matches!(*self, BorderStyle::None | BorderStyle::Hidden)
    }
}

/// A specified value for a single side of the `border-width` property.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum BorderSideWidth {
    /// `thin`
    Thin,
    /// `medium`
    Medium,
    /// `thick`
    Thick,
    /// `<length>`
    Length(NonNegativeLength),
}

/// A specified value for the `border-image-width` property.
pub type BorderImageWidth = Rect<BorderImageSideWidth>;

/// A specified value for a single side of a `border-image-width` property.
pub type BorderImageSideWidth =
    GenericBorderImageSideWidth<NonNegativeLengthPercentage, NonNegativeNumber>;

/// A specified value for the `border-image-slice` property.
pub type BorderImageSlice = GenericBorderImageSlice<NonNegativeNumberOrPercentage>;

/// A specified value for the `border-radius` property.
pub type BorderRadius = GenericBorderRadius<NonNegativeLengthPercentage>;

/// A specified value for the `border-*-radius` longhand properties.
pub type BorderCornerRadius = GenericBorderCornerRadius<NonNegativeLengthPercentage>;

/// A specified value for the `border-spacing` longhand properties.
pub type BorderSpacing = GenericBorderSpacing<NonNegativeLength>;

impl BorderImageSlice {
    /// Returns the `100%` value.
    #[inline]
    pub fn hundred_percent() -> Self {
        GenericBorderImageSlice {
            offsets: Rect::all(NonNegativeNumberOrPercentage::hundred_percent()),
            fill: false,
        }
    }
}

impl BorderSideWidth {
    /// Returns the `0px` value.
    #[inline]
    pub fn zero() -> Self {
        BorderSideWidth::Length(NonNegativeLength::zero())
    }

    /// Parses, with quirks.
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(length) = input.try(|i| NonNegativeLength::parse_quirky(context, i, allow_quirks))
        {
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
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
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
            BorderSideWidth::Thin => NonNegativeLength::from_px(1.).to_computed_value(context),
            BorderSideWidth::Medium => NonNegativeLength::from_px(3.).to_computed_value(context),
            BorderSideWidth::Thick => NonNegativeLength::from_px(5.).to_computed_value(context),
            BorderSideWidth::Length(ref length) => length.to_computed_value(context),
        }
        .into()
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
        GenericBorderImageSideWidth::Number(NonNegativeNumber::new(1.))
    }
}

impl Parse for BorderImageSlice {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut fill = input.try(|i| i.expect_ident_matching("fill")).is_ok();
        let offsets = Rect::parse_with(context, input, NonNegativeNumberOrPercentage::parse)?;
        if !fill {
            fill = input.try(|i| i.expect_ident_matching("fill")).is_ok();
        }
        Ok(GenericBorderImageSlice { offsets, fill })
    }
}

impl Parse for BorderRadius {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let widths = Rect::parse_with(context, input, NonNegativeLengthPercentage::parse)?;
        let heights = if input.try(|i| i.expect_delim('/')).is_ok() {
            Rect::parse_with(context, input, NonNegativeLengthPercentage::parse)?
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
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Size2D::parse_with(context, input, NonNegativeLengthPercentage::parse)
            .map(GenericBorderCornerRadius)
    }
}

impl Parse for BorderSpacing {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Size2D::parse_with(context, input, |context, input| {
            NonNegativeLength::parse_quirky(context, input, AllowQuirks::Yes)
        })
        .map(GenericBorderSpacing)
    }
}

/// A single border-image-repeat keyword.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum BorderImageRepeatKeyword {
    Stretch,
    Repeat,
    Round,
    Space,
}

/// The specified value for the `border-image-repeat` property.
///
/// https://drafts.csswg.org/css-backgrounds/#the-border-image-repeat
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
pub struct BorderImageRepeat(pub BorderImageRepeatKeyword, pub BorderImageRepeatKeyword);

impl ToCss for BorderImageRepeat {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.0.to_css(dest)?;
        if self.0 != self.1 {
            dest.write_str(" ")?;
            self.1.to_css(dest)?;
        }
        Ok(())
    }
}

impl BorderImageRepeat {
    /// Returns the `stretch` value.
    #[inline]
    pub fn stretch() -> Self {
        BorderImageRepeat(
            BorderImageRepeatKeyword::Stretch,
            BorderImageRepeatKeyword::Stretch,
        )
    }
}

impl Parse for BorderImageRepeat {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let horizontal = BorderImageRepeatKeyword::parse(input)?;
        let vertical = input.try(BorderImageRepeatKeyword::parse).ok();
        Ok(BorderImageRepeat(
            horizontal,
            vertical.unwrap_or(horizontal),
        ))
    }
}
