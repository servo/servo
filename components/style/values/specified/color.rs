/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified color values.

use cssparser::{Color as CSSParserColor, Parser, RGBA, Token, BasicParseError, BasicParseErrorKind};
#[cfg(feature = "gecko")]
use gecko_bindings::structs::nscolor;
use itoa;
use parser::{ParserContext, Parse};
#[cfg(feature = "gecko")]
use properties::longhands::system_colors::SystemColor;
use std::fmt;
use std::io::Write;
use style_traits::{ToCss, ParseError, StyleParseErrorKind, ValueParseErrorKind};
use super::AllowQuirks;
use values::computed::{Color as ComputedColor, Context, ToComputedValue};

/// Specified color value
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Color {
    /// The 'currentColor' keyword
    CurrentColor,
    /// A specific RGBA color
    Numeric {
        /// Parsed RGBA color
        parsed: RGBA,
        /// Authored representation
        authored: Option<Box<str>>,
    },
    /// A complex color value from computed value
    Complex(ComputedColor),

    /// A system color
    #[cfg(feature = "gecko")]
    System(SystemColor),
    /// A special color keyword value used in Gecko
    #[cfg(feature = "gecko")]
    Special(gecko::SpecialColorKeyword),
    /// Quirksmode-only rule for inheriting color from the body
    #[cfg(feature = "gecko")]
    InheritFromBodyQuirk,
}


#[cfg(feature = "gecko")]
mod gecko {
    use style_traits::ToCss;

    define_css_keyword_enum! { SpecialColorKeyword:
        "-moz-default-color" => MozDefaultColor,
        "-moz-default-background-color" => MozDefaultBackgroundColor,
        "-moz-hyperlinktext" => MozHyperlinktext,
        "-moz-activehyperlinktext" => MozActiveHyperlinktext,
        "-moz-visitedhyperlinktext" => MozVisitedHyperlinktext,
    }
}

impl From<RGBA> for Color {
    fn from(value: RGBA) -> Self {
        Color::rgba(value)
    }
}

impl Parse for Color {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Color::parse_color(input)
    }
}

impl ToCss for Color {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Color::CurrentColor => CSSParserColor::CurrentColor.to_css(dest),
            Color::Numeric { authored: Some(ref authored), .. } => dest.write_str(authored),
            Color::Numeric { parsed: ref rgba, .. } => rgba.to_css(dest),
            Color::Complex(_) => Ok(()),
            #[cfg(feature = "gecko")]
            Color::System(system) => system.to_css(dest),
            #[cfg(feature = "gecko")]
            Color::Special(special) => special.to_css(dest),
            #[cfg(feature = "gecko")]
            Color::InheritFromBodyQuirk => Ok(()),
        }
    }
}

/// A wrapper of cssparser::Color::parse_hash.
///
/// That function should never return CurrentColor, so it makes no sense
/// to handle a cssparser::Color here. This should really be done in
/// cssparser directly rather than here.
fn parse_hash_color(value: &[u8]) -> Result<RGBA, ()> {
    CSSParserColor::parse_hash(value).map(|color| {
        match color {
            CSSParserColor::RGBA(rgba) => rgba,
            CSSParserColor::CurrentColor =>
                unreachable!("parse_hash should never return currentcolor"),
        }
    })
}

impl Color {
    /// Returns currentcolor value.
    #[inline]
    pub fn currentcolor() -> Color {
        Color::CurrentColor
    }

    /// Returns transparent value.
    #[inline]
    pub fn transparent() -> Color {
        // We should probably set authored to "transparent", but maybe it doesn't matter.
        Color::rgba(RGBA::transparent())
    }

    /// Returns a numeric RGBA color value.
    #[inline]
    pub fn rgba(rgba: RGBA) -> Self {
        Color::Numeric {
            parsed: rgba,
            authored: None,
        }
    }

    /// Parse a color, with quirks.
    ///
    /// https://quirks.spec.whatwg.org/#the-hashless-hex-color-quirk
    pub fn parse_quirky<'i, 't>(context: &ParserContext,
                                input: &mut Parser<'i, 't>,
                                allow_quirks: AllowQuirks)
                                -> Result<Self, ParseError<'i>> {
        input.try(|i| Self::parse(context, i)).or_else(|e| {
            if !allow_quirks.allowed(context.quirks_mode) {
                return Err(e);
            }
            Color::parse_quirky_color(input)
                .map(|rgba| Color::rgba(rgba))
                .map_err(|_| e)
        })
    }

    /// Parse a <quirky-color> value.
    ///
    /// https://quirks.spec.whatwg.org/#the-hashless-hex-color-quirk
    fn parse_quirky_color<'i, 't>(input: &mut Parser<'i, 't>) -> Result<RGBA, ParseError<'i>> {
        let location = input.current_source_location();
        let (value, unit) = match *input.next()? {
            Token::Number { int_value: Some(integer), .. } => {
                (integer, None)
            },
            Token::Dimension { int_value: Some(integer), ref unit, .. } => {
                (integer, Some(unit))
            },
            Token::Ident(ref ident) => {
                if ident.len() != 3 && ident.len() != 6 {
                    return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
                return parse_hash_color(ident.as_bytes())
                    .map_err(|()| location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }
            ref t => {
                return Err(location.new_unexpected_token_error(t.clone()));
            },
        };
        if value < 0 {
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        let length = if value <= 9 {
            1
        } else if value <= 99 {
            2
        } else if value <= 999 {
            3
        } else if value <= 9999 {
            4
        } else if value <= 99999 {
            5
        } else if value <= 999999 {
            6
        } else {
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        };
        let total = length + unit.as_ref().map_or(0, |d| d.len());
        if total > 6 {
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        let mut serialization = [b'0'; 6];
        let space_padding = 6 - total;
        let mut written = space_padding;
        written += itoa::write(&mut serialization[written..], value).unwrap();
        if let Some(unit) = unit {
            written += (&mut serialization[written..]).write(unit.as_bytes()).unwrap();
        }
        debug_assert!(written == 6);
        parse_hash_color(&serialization).map_err(|()| {
            location.new_custom_error(StyleParseErrorKind::UnspecifiedError)
        })
    }

    /// Returns false if the color is completely transparent, and
    /// true otherwise.
    pub fn is_non_transparent(&self) -> bool {
        match *self {
            Color::Numeric { ref parsed, .. } => parsed.alpha != 0,
            _ => true,
        }
    }

    /// Parse a <color> value.
    pub fn parse_color<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        // Currently we only store authored value for color keywords,
        // because all browsers serialize those values as keywords for
        // specified value.
        let start = input.state();
        let authored = match input.next() {
            Ok(&Token::Ident(ref s)) => Some(s.to_lowercase().into_boxed_str()),
            _ => None,
        };
        input.reset(&start);
        match input.try(CSSParserColor::parse) {
            Ok(value) =>
                Ok(match value {
                    CSSParserColor::CurrentColor => Color::CurrentColor,
                    CSSParserColor::RGBA(rgba) => Color::Numeric {
                        parsed: rgba,
                        authored: authored,
                    },
                }),
            Err(e) => {
                #[cfg(feature = "gecko")] {
                    if let Ok(system) = input.try(SystemColor::parse) {
                        return Ok(Color::System(system));
                    } else if let Ok(c) = gecko::SpecialColorKeyword::parse(input) {
                        return Ok(Color::Special(c));
                    }
                }
                match e {
                    BasicParseError { kind: BasicParseErrorKind::UnexpectedToken(t), location } => {
                        Err(location.new_custom_error(
                            StyleParseErrorKind::ValueError(ValueParseErrorKind::InvalidColor(t))
                        ))
                    }
                    e => Err(e.into())
                }
            }
        }
    }
}

#[cfg(feature = "gecko")]
fn convert_nscolor_to_computedcolor(color: nscolor) -> ComputedColor {
    use gecko::values::convert_nscolor_to_rgba;
    ComputedColor::rgba(convert_nscolor_to_rgba(color))
}

impl Color {
    /// Converts this Color into a ComputedColor.
    ///
    /// If `context` is `None`, and the specified color requires data from
    /// the context to resolve, then `None` is returned.
    pub fn to_computed_color(
        &self,
        _context: Option<&Context>,
    ) -> Option<ComputedColor> {
        match *self {
            Color::CurrentColor => {
                Some(ComputedColor::currentcolor())
            }
            Color::Numeric { ref parsed, .. } => {
                Some(ComputedColor::rgba(*parsed))
            }
            Color::Complex(ref complex) => {
                Some(*complex)
            }
            #[cfg(feature = "gecko")]
            Color::System(system) => {
                _context.map(|context| {
                    convert_nscolor_to_computedcolor(
                        system.to_computed_value(context)
                    )
                })
            }
            #[cfg(feature = "gecko")]
            Color::Special(special) => {
                use self::gecko::SpecialColorKeyword as Keyword;
                _context.map(|context| {
                    let pres_context = context.device().pres_context();
                    convert_nscolor_to_computedcolor(match special {
                        Keyword::MozDefaultColor => pres_context.mDefaultColor,
                        Keyword::MozDefaultBackgroundColor => pres_context.mBackgroundColor,
                        Keyword::MozHyperlinktext => pres_context.mLinkColor,
                        Keyword::MozActiveHyperlinktext => pres_context.mActiveLinkColor,
                        Keyword::MozVisitedHyperlinktext => pres_context.mVisitedLinkColor,
                    })
                })
            }
            #[cfg(feature = "gecko")]
            Color::InheritFromBodyQuirk => {
                _context.map(|context| {
                    ComputedColor::rgba(context.device().body_text_color())
                })
            },
        }
    }
}

impl ToComputedValue for Color {
    type ComputedValue = ComputedColor;

    fn to_computed_value(&self, context: &Context) -> ComputedColor {
        let result = self.to_computed_color(Some(context)).unwrap();
        if result.foreground_ratio != 0 {
            if let Some(longhand) = context.for_non_inherited_property {
                if longhand.stores_complex_colors_lossily() {
                    context.rule_cache_conditions.borrow_mut().set_uncacheable();
                }
            }
        }
        result
    }

    fn from_computed_value(computed: &ComputedColor) -> Self {
        if computed.is_numeric() {
            Color::rgba(computed.color)
        } else if computed.is_currentcolor() {
            Color::currentcolor()
        } else {
            Color::Complex(*computed)
        }
    }
}

/// Specified color value, but resolved to just RGBA for computed value
/// with value from color property at the same context.
#[derive(Clone, Debug, PartialEq, ToCss)]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct RGBAColor(pub Color);


impl Parse for RGBAColor {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        Color::parse(context, input).map(RGBAColor)
    }
}

impl ToComputedValue for RGBAColor {
    type ComputedValue = RGBA;

    fn to_computed_value(&self, context: &Context) -> RGBA {
        self.0.to_computed_value(context)
            .to_rgba(context.style().get_color().clone_color())
    }

    fn from_computed_value(computed: &RGBA) -> Self {
        RGBAColor(Color::rgba(*computed))
    }
}

impl From<Color> for RGBAColor {
    fn from(color: Color) -> RGBAColor {
        RGBAColor(color)
    }
}

/// Specified value for the "color" property, which resolves the `currentcolor`
/// keyword to the parent color instead of self's color.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq, ToCss)]
pub struct ColorPropertyValue(pub Color);

impl ToComputedValue for ColorPropertyValue {
    type ComputedValue = RGBA;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> RGBA {
        self.0.to_computed_value(context)
            .to_rgba(context.builder.get_parent_color().clone_color())
    }

    #[inline]
    fn from_computed_value(computed: &RGBA) -> Self {
        ColorPropertyValue(Color::rgba(*computed).into())
    }
}

impl Parse for ColorPropertyValue {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Color::parse_quirky(context, input, AllowQuirks::Yes).map(ColorPropertyValue)
    }
}
