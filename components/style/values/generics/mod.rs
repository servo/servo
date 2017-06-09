/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types that share their serialization implementations
//! for both specified and computed values.

use counter_style::{Symbols, parse_counter_style_name};
use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::{OneOrMoreCommaSeparated, ToCss};
use super::CustomIdent;
use values::specified::url::SpecifiedUrl;

pub mod background;
pub mod basic_shape;
pub mod border;
#[cfg(feature = "gecko")]
pub mod gecko;
pub mod grid;
pub mod image;
pub mod position;
pub mod rect;
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
}

/// https://drafts.csswg.org/css-counter-styles/#typedef-counter-style
///
/// Since wherever <counter-style> is used, 'none' is a valid value as
/// well, we combine them into one type to make code simpler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CounterStyleOrNone {
    /// none
    None_,
    /// <counter-style-name>
    Name(CustomIdent),
    /// symbols()
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

no_viewport_percentage!(CounterStyleOrNone);

impl Parse for CounterStyleOrNone {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(name) = input.try(|i| parse_counter_style_name(i)) {
            return Ok(CounterStyleOrNone::Name(name));
        }
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(CounterStyleOrNone::None_);
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
                    return Err(());
                }
                // Identifier is not allowed in symbols() function.
                if symbols.0.iter().any(|sym| !sym.is_allowed_in_symbols()) {
                    return Err(());
                }
                Ok(CounterStyleOrNone::Symbols(symbols_type, symbols))
            });
        }
        Err(())
    }
}

impl ToCss for CounterStyleOrNone {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self {
            &CounterStyleOrNone::None_ => dest.write_str("none"),
            &CounterStyleOrNone::Name(ref name) => name.to_css(dest),
            &CounterStyleOrNone::Symbols(ref symbols_type, ref symbols) => {
                dest.write_str("symbols(")?;
                symbols_type.to_css(dest)?;
                dest.write_str(" ")?;
                symbols.to_css(dest)?;
                dest.write_str(")")
            }
        }
    }
}

/// A settings tag, defined by a four-character tag and a setting value
///
/// For font-feature-settings, this is a tag and an integer,
/// for font-variation-settings this is a tag and a float
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct FontSettingTag<T> {
    /// A four-character tag, packed into a u32 (one byte per character)
    pub tag: u32,
    /// The value
    pub value: T,
}

impl<T> OneOrMoreCommaSeparated for FontSettingTag<T> {}

impl<T: ToCss> ToCss for FontSettingTag<T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use byteorder::{BigEndian, ByteOrder};
        use std::str;

        let mut raw = [0u8; 4];
        BigEndian::write_u32(&mut raw, self.tag);
        str::from_utf8(&raw).unwrap_or_default().to_css(dest)?;

        self.value.to_css(dest)
    }
}

impl<T: Parse> Parse for FontSettingTag<T> {
    /// https://www.w3.org/TR/css-fonts-3/#propdef-font-feature-settings
    /// https://drafts.csswg.org/css-fonts-4/#low-level-font-variation-
    /// settings-control-the-font-variation-settings-property
    /// <string> [ on | off | <integer> ]
    /// <string> <number>
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        use byteorder::{ReadBytesExt, BigEndian};
        use std::io::Cursor;

        let tag = try!(input.expect_string());

        // allowed strings of length 4 containing chars: <U+20, U+7E>
        if tag.len() != 4 ||
           tag.chars().any(|c| c < ' ' || c > '~')
        {
            return Err(())
        }

        let mut raw = Cursor::new(tag.as_bytes());
        let u_tag = raw.read_u32::<BigEndian>().unwrap();

        Ok(FontSettingTag { tag: u_tag, value: T::parse(context, input)? })
    }
}


/// A font settings value for font-variation-settings or font-feature-settings
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, Eq, PartialEq, ToCss)]
pub enum FontSettings<T> {
    /// No settings (default)
    Normal,
    /// Set of settings
    Tag(Vec<FontSettingTag<T>>)
}

impl<T: Parse> Parse for FontSettings<T> {
    /// https://www.w3.org/TR/css-fonts-3/#propdef-font-feature-settings
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct FontSettingTagInt(pub u32);
/// A number value to be used for font-variation-settings
///
/// Do not use this type anywhere except within FontSettings
/// because it serializes with the preceding space
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct FontSettingTagFloat(pub f32);

impl ToCss for FontSettingTagInt {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self.0 {
            1 => Ok(()),
            0 => dest.write_str(" off"),
            x => write!(dest, " {}", x)
        }
    }
}

impl Parse for FontSettingTagInt {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(value) = input.try(|input| input.expect_integer()) {
            // handle integer, throw if it is negative
            if value >= 0 {
                Ok(FontSettingTagInt(value as u32))
            } else {
                Err(())
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
    fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        input.expect_number().map(FontSettingTagFloat)
    }
}

impl ToCss for FontSettingTagFloat {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str(" ")?;
        self.0.to_css(dest)
    }
}


/// An SVG paint value
///
/// https://www.w3.org/TR/SVG2/painting.html#SpecifyingPaint
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SVGPaint<ColorType> {
    /// The paint source
    pub kind: SVGPaintKind<ColorType>,
    /// The fallback color
    pub fallback: Option<ColorType>,
}

/// An SVG paint value without the fallback
///
/// Whereas the spec only allows PaintServer
/// to have a fallback, Gecko lets the context
/// properties have a fallback as well.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq, ToCss)]
pub enum SVGPaintKind<ColorType> {
    /// `none`
    None,
    /// `<color>`
    Color(ColorType),
    /// `url(...)`
    PaintServer(SpecifiedUrl),
    /// `context-fill`
    ContextFill,
    /// `context-stroke`
    ContextStroke,
}

impl<ColorType> SVGPaintKind<ColorType> {
    /// Convert to a value with a different kind of color
    pub fn convert<F, OtherColor>(&self, f: F) -> SVGPaintKind<OtherColor>
        where F: Fn(&ColorType) -> OtherColor {
            match *self {
                SVGPaintKind::None => SVGPaintKind::None,
                SVGPaintKind::ContextStroke => SVGPaintKind::ContextStroke,
                SVGPaintKind::ContextFill => SVGPaintKind::ContextFill,
                SVGPaintKind::Color(ref color) => {
                    SVGPaintKind::Color(f(color))
                }
                SVGPaintKind::PaintServer(ref server) => {
                    SVGPaintKind::PaintServer(server.clone())
                }
            }
    }
}

impl<ColorType> SVGPaint<ColorType> {
    /// Convert to a value with a different kind of color
    pub fn convert<F, OtherColor>(&self, f: F) -> SVGPaint<OtherColor>
        where F: Fn(&ColorType) -> OtherColor {
        SVGPaint {
            kind: self.kind.convert(&f),
            fallback: self.fallback.as_ref().map(|color| f(color))
        }
    }
}

impl<ColorType> SVGPaintKind<ColorType> {
    /// Parse a keyword value only
    fn parse_ident(input: &mut Parser) -> Result<Self, ()> {
        Ok(match_ignore_ascii_case! { &input.expect_ident()?,
            "none" => SVGPaintKind::None,
            "context-fill" => SVGPaintKind::ContextFill,
            "context-stroke" => SVGPaintKind::ContextStroke,
            _ => return Err(())
        })
    }
}

impl<ColorType: Parse> Parse for SVGPaint<ColorType> {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(url) = input.try(|i| SpecifiedUrl::parse(context, i)) {
            let fallback = input.try(|i| ColorType::parse(context, i));
            Ok(SVGPaint {
                kind: SVGPaintKind::PaintServer(url),
                fallback: fallback.ok(),
            })
        } else if let Ok(kind) = input.try(SVGPaintKind::parse_ident) {
            if let SVGPaintKind::None = kind {
                Ok(SVGPaint {
                    kind: kind,
                    fallback: None,
                })
            } else {
                let fallback = input.try(|i| ColorType::parse(context, i));
                Ok(SVGPaint {
                    kind: kind,
                    fallback: fallback.ok(),
                })
            }
        } else if let Ok(color) = input.try(|i| ColorType::parse(context, i)) {
            Ok(SVGPaint {
                kind: SVGPaintKind::Color(color),
                fallback: None,
            })
        } else {
            Err(())
        }
    }
}

impl<ColorType: ToCss> ToCss for SVGPaint<ColorType> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.kind.to_css(dest)?;
        if let Some(ref fallback) = self.fallback {
            fallback.to_css(dest)?;
        }
        Ok(())
    }
}


