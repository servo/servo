/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified color values.

use cssparser::{self, Parser, Token};
use itoa;
use parser::{ParserContext, Parse};
use std::fmt;
use std::io::Write;
use style_traits::ToCss;
use super::AllowQuirks;

#[cfg(not(feature = "gecko"))] pub use self::servo::Color;
#[cfg(feature = "gecko")] pub use self::gecko::Color;

#[cfg(not(feature = "gecko"))]
mod servo {
    pub use cssparser::Color;
    use cssparser::Parser;
    use parser::{Parse, ParserContext};

    impl Parse for Color {
        fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
            Color::parse(input)
        }
    }
}

#[cfg(feature = "gecko")]
mod gecko {
    use cssparser::{Color as CSSParserColor, Parser, RGBA};
    use parser::{Parse, ParserContext};
    use properties::longhands::color::SystemColor;
    use std::fmt;
    use style_traits::ToCss;

    /// Color value including non-standard -moz prefixed values.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum Color {
        /// The 'currentColor' keyword
        CurrentColor,
        /// A specific RGBA color
        RGBA(RGBA),
        /// A system color
        System(SystemColor),
        /// -moz-default-color
        MozDefaultColor,
        /// -moz-default-background-color
        MozDefaultBackgroundColor,
        /// -moz-hyperlinktext
        MozHyperlinktext,
        /// -moz-activehyperlinktext
        MozActiveHyperlinktext,
        /// -moz-visitedhyperlinktext
        MozVisitedHyperlinktext,
        /// Quirksmode-only rule for inheriting color from the body
        InheritFromBodyQuirk,
    }

    no_viewport_percentage!(Color);

    impl From<CSSParserColor> for Color {
        fn from(value: CSSParserColor) -> Self {
            match value {
                CSSParserColor::CurrentColor => Color::CurrentColor,
                CSSParserColor::RGBA(x) => Color::RGBA(x),
            }
        }
    }

    impl Parse for Color {
        fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
            if let Ok(value) = input.try(CSSParserColor::parse) {
                Ok(value.into())
            } else if let Ok(system) = input.try(SystemColor::parse) {
                Ok(Color::System(system))
            } else {
                let ident = input.expect_ident()?;
                match_ignore_ascii_case! { &ident,
                    "-moz-default-color" => Ok(Color::MozDefaultColor),
                    "-moz-default-background-color" => Ok(Color::MozDefaultBackgroundColor),
                    "-moz-hyperlinktext" => Ok(Color::MozHyperlinktext),
                    "-moz-activehyperlinktext" => Ok(Color::MozActiveHyperlinktext),
                    "-moz-visitedhyperlinktext" => Ok(Color::MozVisitedHyperlinktext),
                    _ => Err(())
                }
            }
        }
    }

    impl ToCss for Color {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                // Standard values:
                Color::CurrentColor => CSSParserColor::CurrentColor.to_css(dest),
                Color::RGBA(rgba) => rgba.to_css(dest),
                Color::System(system) => system.to_css(dest),

                // Non-standard values:
                Color::MozDefaultColor => dest.write_str("-moz-default-color"),
                Color::MozDefaultBackgroundColor => dest.write_str("-moz-default-background-color"),
                Color::MozHyperlinktext => dest.write_str("-moz-hyperlinktext"),
                Color::MozActiveHyperlinktext => dest.write_str("-moz-activehyperlinktext"),
                Color::MozVisitedHyperlinktext => dest.write_str("-moz-visitedhyperlinktext"),
                Color::InheritFromBodyQuirk => Ok(()),
            }
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
