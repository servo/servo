/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for font stuff.

use crate::parser::{Parse, ParserContext};
use crate::One;
use byteorder::{BigEndian, ReadBytesExt};
use cssparser::Parser;
use std::fmt::{self, Write};
use std::io::Cursor;
use style_traits::{CssWriter, ParseError};
use style_traits::{StyleParseErrorKind, ToCss};

/// https://drafts.csswg.org/css-fonts-4/#feature-tag-value
#[derive(
    Clone,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
pub struct FeatureTagValue<Integer> {
    /// A four-character tag, packed into a u32 (one byte per character).
    pub tag: FontTag,
    /// The actual value.
    pub value: Integer,
}

impl<Integer> ToCss for FeatureTagValue<Integer>
where
    Integer: One + ToCss + PartialEq,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.tag.to_css(dest)?;
        // Don't serialize the default value.
        if !self.value.is_one() {
            dest.write_char(' ')?;
            self.value.to_css(dest)?;
        }

        Ok(())
    }
}

/// Variation setting for a single feature, see:
///
/// https://drafts.csswg.org/css-fonts-4/#font-variation-settings-def
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub struct VariationValue<Number> {
    /// A four-character tag, packed into a u32 (one byte per character).
    #[animation(constant)]
    pub tag: FontTag,
    /// The actual value.
    pub value: Number,
}

/// A value both for font-variation-settings and font-feature-settings.
#[derive(
    Clone,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[css(comma)]
pub struct FontSettings<T>(#[css(if_empty = "normal", iterable)] pub Box<[T]>);

impl<T> FontSettings<T> {
    /// Default value of font settings as `normal`.
    #[inline]
    pub fn normal() -> Self {
        FontSettings(vec![].into_boxed_slice())
    }
}

impl<T: Parse> Parse for FontSettings<T> {
    /// https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-feature-settings
    /// https://drafts.csswg.org/css-fonts-4/#font-variation-settings-def
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input
            .try_parse(|i| i.expect_ident_matching("normal"))
            .is_ok()
        {
            return Ok(Self::normal());
        }

        Ok(FontSettings(
            input
                .parse_comma_separated(|i| T::parse(context, i))?
                .into_boxed_slice(),
        ))
    }
}

/// A font four-character tag, represented as a u32 for convenience.
///
/// See:
///   https://drafts.csswg.org/css-fonts-4/#font-variation-settings-def
///   https://drafts.csswg.org/css-fonts-4/#descdef-font-face-font-feature-settings
///
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
pub struct FontTag(pub u32);

impl ToCss for FontTag {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        use byteorder::ByteOrder;
        use std::str;

        let mut raw = [0u8; 4];
        BigEndian::write_u32(&mut raw, self.0);
        str::from_utf8(&raw).unwrap_or_default().to_css(dest)
    }
}

impl Parse for FontTag {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let tag = input.expect_string()?;

        // allowed strings of length 4 containing chars: <U+20, U+7E>
        if tag.len() != 4 || tag.as_bytes().iter().any(|c| *c < b' ' || *c > b'~') {
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        let mut raw = Cursor::new(tag.as_bytes());
        Ok(FontTag(raw.read_u32::<BigEndian>().unwrap()))
    }
}

/// A generic value for the `font-style` property.
///
/// https://drafts.csswg.org/css-fonts-4/#font-style-prop
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Hash,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToResolvedValue,
    ToShmem,
)]
pub enum FontStyle<Angle> {
    #[animation(error)]
    Normal,
    #[animation(error)]
    Italic,
    #[value_info(starts_with_keyword)]
    Oblique(Angle),
}

/// A generic value for the `font-size-adjust` property.
///
/// https://www.w3.org/TR/css-fonts-4/#font-size-adjust-prop
/// https://github.com/w3c/csswg-drafts/issues/6160
/// https://github.com/w3c/csswg-drafts/issues/6288
#[allow(missing_docs)]
#[repr(u8)]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    Hash,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
pub enum GenericFontSizeAdjust<Number> {
    #[animation(error)]
    None,
    // 'ex-height' is the implied basis, so the keyword can be omitted
    ExHeight(Number),
    #[value_info(starts_with_keyword)]
    CapHeight(Number),
    #[value_info(starts_with_keyword)]
    ChWidth(Number),
    #[value_info(starts_with_keyword)]
    IcWidth(Number),
    #[value_info(starts_with_keyword)]
    IcHeight(Number),
}

impl<Number: ToCss> ToCss for GenericFontSizeAdjust<Number> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        let (prefix, value) = match self {
            Self::None => return dest.write_str("none"),
            Self::ExHeight(v) => ("", v),
            Self::CapHeight(v) => ("cap-height ", v),
            Self::ChWidth(v) => ("ch-width ", v),
            Self::IcWidth(v) => ("ic-width ", v),
            Self::IcHeight(v) => ("ic-height ", v),
        };

        dest.write_str(prefix)?;
        value.to_css(dest)
    }
}
