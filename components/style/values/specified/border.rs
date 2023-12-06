/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to borders.

use crate::parser::{Parse, ParserContext};
use crate::values::computed::{Context, ToComputedValue};
use crate::values::generics::border::BorderCornerRadius as GenericBorderCornerRadius;
use crate::values::generics::border::BorderImageSideWidth as GenericBorderImageSideWidth;
use crate::values::generics::border::BorderImageSlice as GenericBorderImageSlice;
use crate::values::generics::border::BorderRadius as GenericBorderRadius;
use crate::values::generics::border::BorderSpacing as GenericBorderSpacing;
use crate::values::generics::rect::Rect;
use crate::values::generics::size::Size2D;
use crate::values::specified::length::{Length, NonNegativeLength, NonNegativeLengthPercentage};
use crate::values::specified::{AllowQuirks, NonNegativeNumber, NonNegativeNumberOrPercentage};
use crate::values::specified::Color;
use crate::Zero;
use app_units::Au;
use cssparser::Parser;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss, values::SequenceWriter};

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

/// https://drafts.csswg.org/css-backgrounds-3/#typedef-line-width
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum LineWidth {
    /// `thin`
    Thin,
    /// `medium`
    Medium,
    /// `thick`
    Thick,
    /// `<length>`
    Length(NonNegativeLength),
}

impl LineWidth {
    /// Returns the `0px` value.
    #[inline]
    pub fn zero() -> Self {
        Self::Length(NonNegativeLength::zero())
    }

    fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(length) =
            input.try_parse(|i| NonNegativeLength::parse_quirky(context, i, allow_quirks))
        {
            return Ok(Self::Length(length));
        }
        Ok(try_match_ident_ignore_ascii_case! { input,
            "thin" => Self::Thin,
            "medium" => Self::Medium,
            "thick" => Self::Thick,
        })
    }
}

impl Parse for LineWidth {
    fn parse<'i>(
        context: &ParserContext,
        input: &mut Parser<'i, '_>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl ToComputedValue for LineWidth {
    type ComputedValue = app_units::Au;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            // https://drafts.csswg.org/css-backgrounds-3/#line-width
            Self::Thin => Au::from_px(1),
            Self::Medium => Au::from_px(3),
            Self::Thick => Au::from_px(5),
            Self::Length(ref length) => Au::from_f32_px(length.to_computed_value(context).px()),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Self::Length(NonNegativeLength::from_px(computed.to_f32_px()))
    }
}

/// A specified value for a single side of the `border-width` property. The difference between this
/// and LineWidth is whether we snap to device pixels or not.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct BorderSideWidth(LineWidth);

impl BorderSideWidth {
    /// Returns the `medium` value.
    pub fn medium() -> Self {
        Self(LineWidth::Medium)
    }

    /// Returns a bare px value from the argument.
    pub fn from_px(px: f32) -> Self {
        Self(LineWidth::Length(Length::from_px(px).into()))
    }

    /// Parses, with quirks.
    pub fn parse_quirky<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_quirks: AllowQuirks,
    ) -> Result<Self, ParseError<'i>> {
        Ok(Self(LineWidth::parse_quirky(context, input, allow_quirks)?))
    }
}

impl Parse for BorderSideWidth {
    fn parse<'i>(
        context: &ParserContext,
        input: &mut Parser<'i, '_>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl ToComputedValue for BorderSideWidth {
    type ComputedValue = app_units::Au;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        let width = self.0.to_computed_value(context);
        // Round `width` down to the nearest device pixel, but any non-zero value that would round
        // down to zero is clamped to 1 device pixel.
        if width == Au(0) {
            return width;
        }

        let au_per_dev_px = context.device().app_units_per_device_pixel();
        std::cmp::max(
            Au(au_per_dev_px),
            Au(width.0 / au_per_dev_px * au_per_dev_px),
        )
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Self(LineWidth::from_computed_value(computed))
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
        let mut fill = input.try_parse(|i| i.expect_ident_matching("fill")).is_ok();
        let offsets = Rect::parse_with(context, input, NonNegativeNumberOrPercentage::parse)?;
        if !fill {
            fill = input.try_parse(|i| i.expect_ident_matching("fill")).is_ok();
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
        let heights = if input.try_parse(|i| i.expect_delim('/')).is_ok() {
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
            dest.write_char(' ')?;
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
        let vertical = input.try_parse(BorderImageRepeatKeyword::parse).ok();
        Ok(BorderImageRepeat(
            horizontal,
            vertical.unwrap_or(horizontal),
        ))
    }
}

/// Serializes a border shorthand value composed of width/style/color.
pub fn serialize_directional_border<W>(
    dest: &mut CssWriter<W>,
    width: &BorderSideWidth,
    style: &BorderStyle,
    color: &Color,
) -> fmt::Result
where
    W: Write,
{
    let has_style = *style != BorderStyle::None;
    let has_color = *color != Color::CurrentColor;
    let has_width = *width != BorderSideWidth::medium();
    if !has_style && !has_color && !has_width {
        return width.to_css(dest)
    }
    let mut writer = SequenceWriter::new(dest, " ");
    if has_width {
        writer.item(width)?;
    }
    if has_style {
        writer.item(style)?;
    }
    if has_color {
        writer.item(color)?;
    }
    Ok(())
}
