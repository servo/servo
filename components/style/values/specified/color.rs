/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Non-standard CSS color values

use cssparser::{Color, Parser, RGBA};
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::ToCss;
use values::HasViewportPercentage;

/// Color value including non-standard -moz prefixed values.
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
pub enum MozColor {
    /// The 'currentColor' keyword
    CurrentColor,
    /// A specific RGBA color
    RGBA(RGBA),

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
}

no_viewport_percentage!(MozColor);

impl Parse for MozColor {
    fn parse(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(value) = input.try(Color::parse) {
            match value {
                Color::CurrentColor => Ok(MozColor::CurrentColor),
                Color::RGBA(x) => Ok(MozColor::RGBA(x)),
            }
        } else {
            let ident = input.expect_ident()?;
            match_ignore_ascii_case! { &ident,
                "-moz-default-color" => Ok(MozColor::MozDefaultColor),
                "-moz-default-background-color" => Ok(MozColor::MozDefaultBackgroundColor),
                "-moz-hyperlinktext" => Ok(MozColor::MozHyperlinktext),
                "-moz-activehyperlinktext" => Ok(MozColor::MozActiveHyperlinktext),
                "-moz-visitedhyperlinktext" => Ok(MozColor::MozVisitedHyperlinktext),
                _ => Err(())
            }
        }
    }
}

impl ToCss for MozColor {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            // Standard values:
            MozColor::CurrentColor => Color::CurrentColor.to_css(dest),
            MozColor::RGBA(rgba) => rgba.to_css(dest),

            // Non-standard values:
            MozColor::MozDefaultColor => dest.write_str("-moz-default-color"),
            MozColor::MozDefaultBackgroundColor => dest.write_str("-moz-default-background-color"),
            MozColor::MozHyperlinktext => dest.write_str("-moz-hyperlinktext"),
            MozColor::MozActiveHyperlinktext => dest.write_str("-moz-activehyperlinktext"),
            MozColor::MozVisitedHyperlinktext => dest.write_str("-moz-visitedhyperlinktext"),
        }
    }
}
