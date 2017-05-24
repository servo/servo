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
use style_traits::{HasViewportPercentage, OneOrMoreCommaSeparated, ToCss};
use super::CustomIdent;

pub use self::basic_shape::serialize_radius_values;

pub mod basic_shape;
pub mod border;
pub mod grid;
pub mod image;
pub mod position;
pub mod rect;

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
        use byteorder::{WriteBytesExt, BigEndian};
        use cssparser::serialize_string;
        use std::str;

        let mut raw: Vec<u8> = vec!();
        raw.write_u32::<BigEndian>(self.tag).unwrap();
        serialize_string(str::from_utf8(&raw).unwrap_or_default(), dest)?;

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
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum FontSettings<T> {
    /// No settings (default)
    Normal,
    /// Set of settings
    Tag(Vec<FontSettingTag<T>>)
}

impl<T: ToCss> ToCss for FontSettings<T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            FontSettings::Normal => dest.write_str("normal"),
            FontSettings::Tag(ref ftvs) => ftvs.to_css(dest)
        }
    }
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


