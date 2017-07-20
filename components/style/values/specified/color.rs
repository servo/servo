/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified color values.

use cssparser::{Color as CSSParserColor, Parser, RGBA, Token, BasicParseError};
#[cfg(feature = "gecko")]
use gecko_bindings::structs::nscolor;
use itoa;
use parser::{ParserContext, Parse};
#[cfg(feature = "gecko")]
use properties::longhands::color::SystemColor;
use std::fmt;
use std::io::Write;
use style_traits::{ToCss, ParseError, StyleParseError};
use super::AllowQuirks;
use values::computed::{Color as ComputedColor, Context, ToComputedValue};

/// Specified color value
#[derive(Clone, PartialEq, Debug)]
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

no_viewport_percentage!(Color);

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
        // Currently we only store authored value for color keywords,
        // because all browsers serialize those values as keywords for
        // specified value.
        let start_position = input.position();
        let authored = match input.next() {
            Ok(Token::Ident(s)) => Some(s.to_lowercase().into_boxed_str()),
            _ => None,
        };
        input.reset(start_position);
        if let Ok(value) = input.try(CSSParserColor::parse) {
            Ok(match value {
                CSSParserColor::CurrentColor => Color::CurrentColor,
                CSSParserColor::RGBA(rgba) => Color::Numeric {
                    parsed: rgba,
                    authored: authored,
                },
            })
        } else {
            #[cfg(feature = "gecko")] {
                if let Ok(system) = input.try(SystemColor::parse) {
                    Ok(Color::System(system))
                } else {
                    gecko::SpecialColorKeyword::parse(input).map(Color::Special)
                }
            }
            #[cfg(not(feature = "gecko"))] {
                Err(StyleParseError::UnspecifiedError.into())
            }
        }
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
        input.try(|i| Self::parse(context, i)).or_else(|_| {
            if !allow_quirks.allowed(context.quirks_mode) {
                return Err(StyleParseError::UnspecifiedError.into());
            }
            Color::parse_quirky_color(input).map(|rgba| Color::rgba(rgba))
        })
    }

    /// Parse a <quirky-color> value.
    ///
    /// https://quirks.spec.whatwg.org/#the-hashless-hex-color-quirk
    fn parse_quirky_color<'i, 't>(input: &mut Parser<'i, 't>) -> Result<RGBA, ParseError<'i>> {
        let (value, unit) = match input.next()? {
            Token::Number { int_value: Some(integer), .. } => {
                (integer, None)
            },
            Token::Dimension { int_value: Some(integer), unit, .. } => {
                (integer, Some(unit))
            },
            Token::Ident(ident) => {
                if ident.len() != 3 && ident.len() != 6 {
                    return Err(StyleParseError::UnspecifiedError.into());
                }
                return parse_hash_color(ident.as_bytes())
                    .map_err(|()| StyleParseError::UnspecifiedError.into());
            }
            t => {
                return Err(BasicParseError::UnexpectedToken(t).into());
            },
        };
        if value < 0 {
            return Err(StyleParseError::UnspecifiedError.into());
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
            return Err(StyleParseError::UnspecifiedError.into())
        };
        let total = length + unit.as_ref().map_or(0, |d| d.len());
        if total > 6 {
            return Err(StyleParseError::UnspecifiedError.into());
        }
        let mut serialization = [b'0'; 6];
        let space_padding = 6 - total;
        let mut written = space_padding;
        written += itoa::write(&mut serialization[written..], value).unwrap();
        if let Some(unit) = unit {
            written += (&mut serialization[written..]).write(unit.as_bytes()).unwrap();
        }
        debug_assert!(written == 6);
        parse_hash_color(&serialization).map_err(|()| StyleParseError::UnspecifiedError.into())
    }

    /// Returns false if the color is completely transparent, and
    /// true otherwise.
    pub fn is_non_transparent(&self) -> bool {
        match *self {
            Color::Numeric { ref parsed, .. } => parsed.alpha != 0,
            _ => true,
        }
    }
}

#[cfg(feature = "gecko")]
fn convert_nscolor_to_computedcolor(color: nscolor) -> ComputedColor {
    use gecko::values::convert_nscolor_to_rgba;
    ComputedColor::rgba(convert_nscolor_to_rgba(color))
}

impl ToComputedValue for Color {
    type ComputedValue = ComputedColor;

    fn to_computed_value(&self, _context: &Context) -> ComputedColor {
        match *self {
            Color::CurrentColor => ComputedColor::currentcolor(),
            Color::Numeric { ref parsed, .. } => ComputedColor::rgba(*parsed),
            Color::Complex(ref complex) => *complex,
            #[cfg(feature = "gecko")]
            Color::System(system) =>
                convert_nscolor_to_computedcolor(system.to_computed_value(_context)),
            #[cfg(feature = "gecko")]
            Color::Special(special) => {
                use self::gecko::SpecialColorKeyword as Keyword;
                let pres_context = _context.device().pres_context();
                convert_nscolor_to_computedcolor(match special {
                    Keyword::MozDefaultColor => pres_context.mDefaultColor,
                    Keyword::MozDefaultBackgroundColor => pres_context.mBackgroundColor,
                    Keyword::MozHyperlinktext => pres_context.mLinkColor,
                    Keyword::MozActiveHyperlinktext => pres_context.mActiveLinkColor,
                    Keyword::MozVisitedHyperlinktext => pres_context.mVisitedLinkColor,
                })
            }
            #[cfg(feature = "gecko")]
            Color::InheritFromBodyQuirk => {
                use dom::TElement;
                use gecko::wrapper::GeckoElement;
                use gecko_bindings::bindings::Gecko_GetBody;
                let pres_context = _context.device().pres_context();
                let body = unsafe { Gecko_GetBody(pres_context) }.map(GeckoElement);
                let data = body.as_ref().and_then(|wrap| wrap.borrow_data());
                if let Some(data) = data {
                    ComputedColor::rgba(data.styles.primary()
                                            .get_color()
                                            .clone_color())
                } else {
                    convert_nscolor_to_computedcolor(pres_context.mDefaultColor)
                }
            },
        }
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
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct RGBAColor(pub Color);

no_viewport_percentage!(RGBAColor);

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
