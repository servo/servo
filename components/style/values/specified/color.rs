/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Non-standard CSS color values

use cssparser::{Color as CSSParserColor, Parser, Token};
use itoa;
use parser::{Parse, ParserContext};
use std::io::Write;
use values::specified::AllowQuirks;

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

/// Extension trait for the color type.
pub trait ColorExt: Sized {
    /// Parses a color, with quirks.
    fn parse_quirky(context: &ParserContext,
                    input: &mut Parser,
                    allow_quirks: AllowQuirks)
                    -> Result<Self, ()>;
}

impl ColorExt for Color {
    fn parse_quirky(context: &ParserContext,
                    input: &mut Parser,
                    allow_quirks: AllowQuirks)
                    -> Result<Self, ()> {
        if let Ok(color) = input.try(|i| Parse::parse(context, i)) {
            return Ok(color);
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
                return CSSParserColor::parse_hash(ident.as_bytes()).map(From::from);
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
        let mut written = 6 - total;
        written += itoa::write(&mut serialization[written..], value).unwrap();
        if let Some(dimension) = dimension {
            written += (&mut serialization[written..]).write(dimension.as_bytes()).unwrap();
        }
        debug_assert!(written == 6);
        CSSParserColor::parse_hash(&serialization).map(From::from)
    }
}
