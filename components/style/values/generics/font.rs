/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for font stuff.

use std::fmt::{self, Write};
use cssparser::Parser;
use parser::{Parse, ParserContext};
use style_traits::{Comma, CssWriter, OneOrMoreSeparated, ParseError, ToCss, StyleParseErrorKind};
use values::specified::font::FontTag;

/// A settings tag, defined by a four-character tag and a setting value
///
/// For font-feature-settings, this is a tag and an integer, for
/// font-variation-settings this is a tag and a float.
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct FontSettingTag<T> {
    /// A four-character tag, packed into a u32 (one byte per character)
    pub tag: FontTag,
    /// The actual value.
    pub value: T,
}

impl<T> OneOrMoreSeparated for FontSettingTag<T> {
    type S = Comma;
}

impl<T: Parse> Parse for FontSettingTag<T> {
    /// <https://www.w3.org/TR/css-fonts-3/#propdef-font-feature-settings>
    /// <https://drafts.csswg.org/css-fonts-4/#low-level-font-variation->
    /// settings-control-the-font-variation-settings-property
    /// <string> [ on | off | <integer> ]
    /// <string> <number>
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let tag = FontTag::parse(context, input)?;
        let value = T::parse(context, input)?;

        Ok(Self { tag, value })
    }
}

/// A font settings value for font-variation-settings or font-feature-settings
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub enum FontSettings<T> {
    /// No settings (default)
    Normal,
    /// Set of settings
    Tag(Vec<FontSettingTag<T>>)
}

impl <T> FontSettings<T> {
    #[inline]
    /// Default value of font settings as `normal`
    pub fn normal() -> Self {
        FontSettings::Normal
    }
}

impl<T: Parse> Parse for FontSettings<T> {
    /// <https://www.w3.org/TR/css-fonts-3/#propdef-font-feature-settings>
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("normal")).is_ok() {
            return Ok(FontSettings::Normal);
        }
        Vec::parse(context, input).map(FontSettings::Tag)
    }
}

/// An integer that can also parse "on" and "off",
/// for font-feature-settings
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct FontSettingTagInt(pub u32);

/// A number value to be used for font-variation-settings.
///
/// FIXME(emilio): The spec only says <integer>, so we should be able to reuse
/// the other code:
///
/// https://drafts.csswg.org/css-fonts-4/#propdef-font-variation-settings
#[cfg_attr(feature = "gecko", derive(Animate, ComputeSquaredDistance))]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct FontSettingTagFloat(pub f32);

impl ToCss for FontSettingTagInt {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match self.0 {
            1 => Ok(()),
            0 => dest.write_str("off"),
            x => x.to_css(dest),
        }
    }
}

impl Parse for FontSettingTagInt {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // FIXME(emilio): This doesn't handle calc properly.
        if let Ok(value) = input.try(|input| input.expect_integer()) {
            // handle integer, throw if it is negative
            if value >= 0 {
                Ok(FontSettingTagInt(value as u32))
            } else {
                Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            }
        } else if let Ok(_) = input.try(|input| input.expect_ident_matching("on")) {
            // on is an alias for '1'
            Ok(FontSettingTagInt(1))
        } else if let Ok(_) = input.try(|input| input.expect_ident_matching("off")) {
            // off is an alias for '0'
            Ok(FontSettingTagInt(0))
        } else {
            // empty value is an alias for '1'
            Ok(FontSettingTagInt(1))
        }
    }
}

impl Parse for FontSettingTagFloat {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // FIXME(emilio): Should handle calc() using Number::parse.
        //
        // Also why is this not in font.rs?
        input.expect_number().map(FontSettingTagFloat).map_err(|e| e.into())
    }
}
