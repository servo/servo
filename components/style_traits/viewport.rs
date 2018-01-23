/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helper types for the `@viewport` rule.

use {CSSPixel, CssWriter, ParseError, PinchZoomFactor, ToCss};
use cssparser::Parser;
use euclid::TypedSize2D;
#[allow(unused_imports)] use std::ascii::AsciiExt;
use std::fmt::{self, Write};

define_css_keyword_enum!(UserZoom:
                         "zoom" => Zoom,
                         "fixed" => Fixed);

define_css_keyword_enum!(Orientation:
                         "auto" => Auto,
                         "portrait" => Portrait,
                         "landscape" => Landscape);

/// A set of viewport descriptors:
///
/// <https://drafts.csswg.org/css-device-adapt/#viewport-desc>
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize, MallocSizeOf))]
pub struct ViewportConstraints {
    /// Width and height:
    ///  * https://drafts.csswg.org/css-device-adapt/#width-desc
    ///  * https://drafts.csswg.org/css-device-adapt/#height-desc
    pub size: TypedSize2D<f32, CSSPixel>,
    /// <https://drafts.csswg.org/css-device-adapt/#zoom-desc>
    pub initial_zoom: PinchZoomFactor,
    /// <https://drafts.csswg.org/css-device-adapt/#min-max-width-desc>
    pub min_zoom: Option<PinchZoomFactor>,
    /// <https://drafts.csswg.org/css-device-adapt/#min-max-width-desc>
    pub max_zoom: Option<PinchZoomFactor>,
    /// <https://drafts.csswg.org/css-device-adapt/#user-zoom-desc>
    pub user_zoom: UserZoom,
    /// <https://drafts.csswg.org/css-device-adapt/#orientation-desc>
    pub orientation: Orientation
}

impl ToCss for ViewportConstraints {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("@viewport { width: ")?;
        self.size.width.to_css(dest)?;

        dest.write_str("px; height: ")?;
        self.size.height.to_css(dest)?;

        dest.write_str("px; zoom: ")?;
        self.initial_zoom.get().to_css(dest)?;

        if let Some(min_zoom) = self.min_zoom {
            dest.write_str("; min-zoom: ")?;
            min_zoom.get().to_css(dest)?;
        }

        if let Some(max_zoom) = self.max_zoom {
            dest.write_str("; max-zoom: ")?;
            max_zoom.get().to_css(dest)?;
        }

        dest.write_str("; user-zoom: ")?;
        self.user_zoom.to_css(dest)?;

        dest.write_str("; orientation: ")?;
        self.orientation.to_css(dest)?;
        dest.write_str("; }")
    }
}

/// <https://drafts.csswg.org/css-device-adapt/#descdef-viewport-zoom>
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub enum Zoom {
    /// A number value.
    Number(f32),
    /// A percentage value.
    Percentage(f32),
    /// The `auto` keyword.
    Auto,
}

impl ToCss for Zoom {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            Zoom::Number(number) => number.to_css(dest),
            Zoom::Auto => dest.write_str("auto"),
            Zoom::Percentage(percentage) => {
                (percentage * 100.).to_css(dest)?;
                dest.write_char('%')
            }
        }
    }
}

impl Zoom {
    /// Parse a zoom value per:
    ///
    /// <https://drafts.csswg.org/css-device-adapt/#descdef-viewport-zoom>
    pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Zoom, ParseError<'i>> {
        use ParsingMode;
        use cssparser::Token;
        use values::specified::AllowedNumericType::NonNegative;

        let location = input.current_source_location();
        match *input.next()? {
            // TODO: This parse() method should take ParserContext as an
            // argument, and pass ParsingMode owned by the ParserContext to
            // is_ok() instead of using ParsingMode::DEFAULT directly.
            // In order to do so, we might want to move these stuff into style::stylesheets::viewport_rule.
            Token::Percentage { unit_value, .. } if NonNegative.is_ok(ParsingMode::DEFAULT, unit_value) => {
                Ok(Zoom::Percentage(unit_value))
            }
            Token::Number { value, .. } if NonNegative.is_ok(ParsingMode::DEFAULT, value) => {
                Ok(Zoom::Number(value))
            }
            Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") => {
                Ok(Zoom::Auto)
            }
            ref t => Err(location.new_unexpected_token_error(t.clone()))
        }
    }

    /// Get this zoom value as a float value. Returns `None` if the value is the
    /// `auto` keyword.
    #[inline]
    pub fn to_f32(&self) -> Option<f32> {
        match *self {
            Zoom::Number(number) => Some(number as f32),
            Zoom::Percentage(percentage) => Some(percentage as f32),
            Zoom::Auto => None
        }
    }
}
