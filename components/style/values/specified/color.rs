/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified color values.

use cssparser::{self, Color as CSSParserColor, Parser, RGBA, Token};
use itoa;
use parser::{ParserContext, Parse};
#[cfg(feature = "gecko")]
use properties::longhands::color::SystemColor;
use std::fmt;
use std::io::Write;
use style_traits::ToCss;
use super::AllowQuirks;
use values::computed::{Context, ToComputedValue};

/// Specified color value
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Color {
    /// The 'currentColor' keyword
    CurrentColor,
    /// A specific RGBA color
    RGBA(RGBA),

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

impl From<CSSParserColor> for Color {
    fn from(value: CSSParserColor) -> Self {
        match value {
            CSSParserColor::CurrentColor => Color::CurrentColor,
            CSSParserColor::RGBA(x) => Color::RGBA(x),
        }
    }
}

impl From<RGBA> for Color {
    fn from(value: RGBA) -> Self {
        Color::RGBA(value)
    }
}

impl Parse for Color {
    fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(value) = input.try(CSSParserColor::parse) {
            Ok(value.into())
        } else {
            #[cfg(feature = "gecko")] {
                if let Ok(system) = input.try(SystemColor::parse) {
                    Ok(Color::System(system))
                } else {
                    gecko::SpecialColorKeyword::parse(input).map(Color::Special)
                }
            }
            #[cfg(not(feature = "gecko"))] {
                Err(())
            }
        }
    }
}

impl ToCss for Color {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Color::CurrentColor => CSSParserColor::CurrentColor.to_css(dest),
            Color::RGBA(rgba) => rgba.to_css(dest),
            #[cfg(feature = "gecko")]
            Color::System(system) => system.to_css(dest),
            #[cfg(feature = "gecko")]
            Color::Special(special) => special.to_css(dest),
            #[cfg(feature = "gecko")]
            Color::InheritFromBodyQuirk => Ok(()),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct CSSColor {
    pub parsed: Color,
    pub authored: Option<Box<str>>,
}

impl Parse for CSSColor {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        Self::parse_quirky(context, input, AllowQuirks::No)
    }
}

impl CSSColor {
    /// Parse a color, with quirks.
    ///
    /// https://quirks.spec.whatwg.org/#the-hashless-hex-color-quirk
    pub fn parse_quirky(context: &ParserContext,
                        input: &mut Parser,
                        allow_quirks: AllowQuirks)
                        -> Result<Self, ()> {
        let start_position = input.position();
        let authored = match input.next() {
            Ok(Token::Ident(s)) => Some(s.into_owned().into_boxed_str()),
            _ => None,
        };
        input.reset(start_position);
        if let Ok(parsed) = input.try(|i| Parse::parse(context, i)) {
            return Ok(CSSColor {
                parsed: parsed,
                authored: authored,
            });
        }
        if !allow_quirks.allowed(context.quirks_mode) {
            return Err(());
        }
        let (number, dimension) = match input.next()? {
            Token::Number(number) => {
                (number, None)
            },
            Token::Dimension(number, dimension) => {
                (number, Some(dimension))
            },
            Token::Ident(ident) => {
                if ident.len() != 3 && ident.len() != 6 {
                    return Err(());
                }
                return cssparser::Color::parse_hash(ident.as_bytes()).map(|color| {
                    Self {
                        parsed: color.into(),
                        authored: None
                    }
                });
            }
            _ => {
                return Err(());
            },
        };
        let value = number.int_value.ok_or(())?;
        if value < 0 {
            return Err(());
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
            return Err(())
        };
        let total = length + dimension.as_ref().map_or(0, |d| d.len());
        if total > 6 {
            return Err(());
        }
        let mut serialization = [b'0'; 6];
        let space_padding = 6 - total;
        let mut written = space_padding;
        written += itoa::write(&mut serialization[written..], value).unwrap();
        if let Some(dimension) = dimension {
            written += (&mut serialization[written..]).write(dimension.as_bytes()).unwrap();
        }
        debug_assert!(written == 6);
        Ok(CSSColor {
            parsed: cssparser::Color::parse_hash(&serialization).map(From::from)?,
            authored: None,
        })
    }

    /// Returns false if the color is completely transparent, and
    /// true otherwise.
    pub fn is_non_transparent(&self) -> bool {
        match self.parsed {
            Color::RGBA(rgba) if rgba.alpha == 0 => false,
            _ => true,
        }
    }
}

no_viewport_percentage!(CSSColor);

impl ToCss for CSSColor {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self.authored {
            Some(ref s) => dest.write_str(s),
            None => self.parsed.to_css(dest),
        }
    }
}

impl From<Color> for CSSColor {
    fn from(color: Color) -> Self {
        CSSColor {
            parsed: color,
            authored: None,
        }
    }
}

impl CSSColor {
    #[inline]
    /// Returns currentcolor value.
    pub fn currentcolor() -> CSSColor {
        Color::CurrentColor.into()
    }

    #[inline]
    /// Returns transparent value.
    pub fn transparent() -> CSSColor {
        // We should probably set authored to "transparent", but maybe it doesn't matter.
        Color::RGBA(cssparser::RGBA::transparent()).into()
    }
}

impl ToComputedValue for Color {
    type ComputedValue = RGBA;

    fn to_computed_value(&self, context: &Context) -> RGBA {
        #[cfg(feature = "gecko")]
        use gecko::values::convert_nscolor_to_rgba as to_rgba;
        match *self {
            Color::RGBA(rgba) => rgba,
            Color::CurrentColor => context.inherited_style.get_color().clone_color(),
            #[cfg(feature = "gecko")]
            Color::System(system) => to_rgba(system.to_computed_value(context)),
            #[cfg(feature = "gecko")]
            Color::Special(special) => {
                use self::gecko::SpecialColorKeyword as Keyword;
                let pres_context = unsafe { &*context.device.pres_context };
                to_rgba(match special {
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
                let pres_context = unsafe { &*context.device.pres_context };
                let body = unsafe {
                    Gecko_GetBody(pres_context)
                };
                if let Some(body) = body {
                    let wrap = GeckoElement(body);
                    let borrow = wrap.borrow_data();
                    borrow.as_ref().unwrap()
                          .styles().primary.values()
                          .get_color()
                          .clone_color()
                } else {
                    to_rgba(pres_context.mDefaultColor)
                }
            },
        }
    }

    fn from_computed_value(computed: &RGBA) -> Self {
        Color::RGBA(*computed)
    }
}

impl ToComputedValue for CSSColor {
    type ComputedValue = CSSParserColor;

    #[inline]
    fn to_computed_value(&self, _context: &Context) -> CSSParserColor {
        match self.parsed {
            Color::RGBA(rgba) => CSSParserColor::RGBA(rgba),
            Color::CurrentColor => CSSParserColor::CurrentColor,
            // Resolve non-standard -moz keywords to RGBA:
            #[cfg(feature = "gecko")]
            non_standard => CSSParserColor::RGBA(non_standard.to_computed_value(_context)),
        }
    }

    #[inline]
    fn from_computed_value(computed: &CSSParserColor) -> Self {
        (match *computed {
            CSSParserColor::RGBA(rgba) => Color::RGBA(rgba),
            CSSParserColor::CurrentColor => Color::CurrentColor,
        }).into()
    }
}

/// Specified color value, but resolved to just RGBA for computed value
/// with value from color property at the same context.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct RGBAColor(pub CSSColor);

no_viewport_percentage!(RGBAColor);

impl Parse for RGBAColor {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        CSSColor::parse(context, input).map(RGBAColor)
    }
}

impl ToCss for RGBAColor {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

impl ToComputedValue for RGBAColor {
    type ComputedValue = RGBA;

    fn to_computed_value(&self, context: &Context) -> RGBA {
        match self.0.to_computed_value(context) {
            CSSParserColor::RGBA(rgba) => rgba,
            CSSParserColor::CurrentColor => context.style.get_color().clone_color(),
        }
    }

    fn from_computed_value(computed: &RGBA) -> Self {
        RGBAColor(CSSColor {
            parsed: Color::RGBA(*computed),
            authored: None,
        })
    }
}

impl From<Color> for RGBAColor {
    fn from(color: Color) -> RGBAColor {
        RGBAColor(CSSColor {
            parsed: color,
            authored: None,
        })
    }
}

impl From<CSSColor> for RGBAColor {
    fn from(color: CSSColor) -> RGBAColor {
        RGBAColor(color)
    }
}
