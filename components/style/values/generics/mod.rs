/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types that share their serialization implementations
//! for both specified and computed values.

use counter_style::{Symbols, parse_counter_style_name};
use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt::{self, Write};
use style_traits::{Comma, CssWriter, OneOrMoreSeparated, ParseError};
use style_traits::{StyleParseErrorKind, ToCss};
use super::CustomIdent;

pub mod background;
pub mod basic_shape;
pub mod border;
#[path = "box.rs"]
pub mod box_;
pub mod effects;
pub mod flex;
#[cfg(feature = "gecko")]
pub mod gecko;
pub mod grid;
pub mod image;
pub mod position;
pub mod rect;
pub mod size;
pub mod svg;
pub mod text;
pub mod transform;

// https://drafts.csswg.org/css-counter-styles/#typedef-symbols-type
define_css_keyword_enum! { SymbolsType:
    "cyclic" => Cyclic,
    "numeric" => Numeric,
    "alphabetic" => Alphabetic,
    "symbolic" => Symbolic,
    "fixed" => Fixed,
}
add_impls_for_keyword_enum!(SymbolsType);

#[cfg(feature = "gecko")]
impl SymbolsType {
    /// Convert symbols type to their corresponding Gecko values.
    pub fn to_gecko_keyword(self) -> u8 {
        use gecko_bindings::structs;
        match self {
            SymbolsType::Cyclic => structs::NS_STYLE_COUNTER_SYSTEM_CYCLIC as u8,
            SymbolsType::Numeric => structs::NS_STYLE_COUNTER_SYSTEM_NUMERIC as u8,
            SymbolsType::Alphabetic => structs::NS_STYLE_COUNTER_SYSTEM_ALPHABETIC as u8,
            SymbolsType::Symbolic => structs::NS_STYLE_COUNTER_SYSTEM_SYMBOLIC as u8,
            SymbolsType::Fixed => structs::NS_STYLE_COUNTER_SYSTEM_FIXED as u8,
        }
    }

    /// Convert Gecko value to symbol type.
    pub fn from_gecko_keyword(gecko_value: u32) -> SymbolsType {
        use gecko_bindings::structs;
        match gecko_value {
            structs::NS_STYLE_COUNTER_SYSTEM_CYCLIC => SymbolsType::Cyclic,
            structs::NS_STYLE_COUNTER_SYSTEM_NUMERIC => SymbolsType::Numeric,
            structs::NS_STYLE_COUNTER_SYSTEM_ALPHABETIC => SymbolsType::Alphabetic,
            structs::NS_STYLE_COUNTER_SYSTEM_SYMBOLIC => SymbolsType::Symbolic,
            structs::NS_STYLE_COUNTER_SYSTEM_FIXED => SymbolsType::Fixed,
            x => panic!("Unexpected value for symbol type {}", x)
        }
    }
}

/// <https://drafts.csswg.org/css-counter-styles/#typedef-counter-style>
///
/// Since wherever <counter-style> is used, 'none' is a valid value as
/// well, we combine them into one type to make code simpler.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, Eq, PartialEq, ToComputedValue, ToCss)]
pub enum CounterStyleOrNone {
    /// `none`
    None,
    /// `<counter-style-name>`
    Name(CustomIdent),
    /// `symbols()`
    #[css(function)]
    Symbols(SymbolsType, Symbols),
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


impl Parse for CounterStyleOrNone {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(name) = input.try(|i| parse_counter_style_name(i)) {
            return Ok(CounterStyleOrNone::Name(name));
        }
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(CounterStyleOrNone::None);
        }
        if input.try(|i| i.expect_function_matching("symbols")).is_ok() {
            return input.parse_nested_block(|input| {
                let symbols_type = input.try(|i| SymbolsType::parse(i))
                    .unwrap_or(SymbolsType::Symbolic);
                let symbols = Symbols::parse(context, input)?;
                // There must be at least two symbols for alphabetic or
                // numeric system.
                if (symbols_type == SymbolsType::Alphabetic ||
                    symbols_type == SymbolsType::Numeric) && symbols.0.len() < 2 {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                // Identifier is not allowed in symbols() function.
                if symbols.0.iter().any(|sym| !sym.is_allowed_in_symbols()) {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                Ok(CounterStyleOrNone::Symbols(symbols_type, symbols))
            });
        }
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

/// A settings tag, defined by a four-character tag and a setting value
///
/// For font-feature-settings, this is a tag and an integer,
/// for font-variation-settings this is a tag and a float
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct FontSettingTag<T> {
    /// A four-character tag, packed into a u32 (one byte per character)
    pub tag: u32,
    /// The value
    pub value: T,
}

impl<T> OneOrMoreSeparated for FontSettingTag<T> {
    type S = Comma;
}

impl<T: ToCss> ToCss for FontSettingTag<T> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        use byteorder::{BigEndian, ByteOrder};
        use std::str;

        let mut raw = [0u8; 4];
        BigEndian::write_u32(&mut raw, self.tag);
        str::from_utf8(&raw).unwrap_or_default().to_css(dest)?;

        self.value.to_css(dest)
    }
}

impl<T: Parse> Parse for FontSettingTag<T> {
    /// <https://www.w3.org/TR/css-fonts-3/#propdef-font-feature-settings>
    /// <https://drafts.csswg.org/css-fonts-4/#low-level-font-variation->
    /// settings-control-the-font-variation-settings-property
    /// <string> [ on | off | <integer> ]
    /// <string> <number>
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        use byteorder::{ReadBytesExt, BigEndian};
        use std::io::Cursor;

        let u_tag;
        {
            let location = input.current_source_location();
            let tag = input.expect_string()?;

            // allowed strings of length 4 containing chars: <U+20, U+7E>
            if tag.len() != 4 ||
               tag.chars().any(|c| c < ' ' || c > '~')
            {
                return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            }

            let mut raw = Cursor::new(tag.as_bytes());
            u_tag = raw.read_u32::<BigEndian>().unwrap();
        }

        Ok(FontSettingTag { tag: u_tag, value: T::parse(context, input)? })
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
///
/// Do not use this type anywhere except within FontSettings
/// because it serializes with the preceding space
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct FontSettingTagInt(pub u32);

/// A number value to be used for font-variation-settings
///
/// Do not use this type anywhere except within FontSettings
/// because it serializes with the preceding space
#[cfg_attr(feature = "gecko", derive(Animate, ComputeSquaredDistance))]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct FontSettingTagFloat(pub f32);

impl ToCss for FontSettingTagInt {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match self.0 {
            1 => Ok(()),
            0 => dest.write_str(" off"),
            x => {
                dest.write_char(' ')?;
                x.to_css(dest)
            }
        }
    }
}

impl Parse for FontSettingTagInt {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
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
        input.expect_number().map(FontSettingTagFloat).map_err(|e| e.into())
    }
}

impl ToCss for FontSettingTagFloat {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str(" ")?;
        self.0.to_css(dest)
    }
}

/// A wrapper of Non-negative values.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf)]
#[derive(PartialEq, PartialOrd, ToAnimatedZero, ToComputedValue, ToCss)]
pub struct NonNegative<T>(pub T);

/// A wrapper of greater-than-or-equal-to-one values.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf)]
#[derive(PartialEq, PartialOrd, ToAnimatedZero, ToComputedValue, ToCss)]
pub struct GreaterThanOrEqualToOne<T>(pub T);
