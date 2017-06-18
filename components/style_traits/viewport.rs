/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helper types for the `@viewport` rule.

use {CSSPixel, PinchZoomFactor, ParseError};
use cssparser::{Parser, ToCss, ParseError as CssParseError, BasicParseError};
use euclid::TypedSize2D;
use std::ascii::AsciiExt;
use std::fmt;

define_css_keyword_enum!(UserZoom:
                         "zoom" => Zoom,
                         "fixed" => Fixed);

define_css_keyword_enum!(Orientation:
                         "auto" => Auto,
                         "portrait" => Portrait,
                         "landscape" => Landscape);

/// A trait used to query whether this value has viewport units.
pub trait HasViewportPercentage {
    /// Returns true if this value has viewport units.
    fn has_viewport_percentage(&self) -> bool;
}

/// A macro used to implement HasViewportPercentage trait
/// for a given type that may never contain viewport units.
#[macro_export]
macro_rules! no_viewport_percentage {
    ($($name: ident),+) => {
        $(impl $crate::HasViewportPercentage for $name {
            #[inline]
            fn has_viewport_percentage(&self) -> bool {
                false
            }
        })+
    };
}

no_viewport_percentage!(bool, f32);

impl<T: HasViewportPercentage> HasViewportPercentage for Box<T> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        (**self).has_viewport_percentage()
    }
}

impl<T: HasViewportPercentage> HasViewportPercentage for Option<T> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.as_ref().map_or(false, T::has_viewport_percentage)
    }
}

impl<T: HasViewportPercentage, U> HasViewportPercentage for TypedSize2D<T, U> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.width.has_viewport_percentage() || self.height.has_viewport_percentage()
    }
}

impl<T: HasViewportPercentage> HasViewportPercentage for Vec<T> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool {
        self.iter().any(T::has_viewport_percentage)
    }
}

/// A set of viewport descriptors:
///
/// https://drafts.csswg.org/css-device-adapt/#viewport-desc
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize, HeapSizeOf))]
pub struct ViewportConstraints {
    /// Width and height:
    ///  * https://drafts.csswg.org/css-device-adapt/#width-desc
    ///  * https://drafts.csswg.org/css-device-adapt/#height-desc
    pub size: TypedSize2D<f32, CSSPixel>,
    /// https://drafts.csswg.org/css-device-adapt/#zoom-desc
    pub initial_zoom: PinchZoomFactor,
    /// https://drafts.csswg.org/css-device-adapt/#min-max-width-desc
    pub min_zoom: Option<PinchZoomFactor>,
    /// https://drafts.csswg.org/css-device-adapt/#min-max-width-desc
    pub max_zoom: Option<PinchZoomFactor>,
    /// https://drafts.csswg.org/css-device-adapt/#user-zoom-desc
    pub user_zoom: UserZoom,
    /// https://drafts.csswg.org/css-device-adapt/#orientation-desc
    pub orientation: Orientation
}

impl ToCss for ViewportConstraints {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        write!(dest, "@viewport {{")?;
        write!(dest, " width: {}px;", self.size.width)?;
        write!(dest, " height: {}px;", self.size.height)?;
        write!(dest, " zoom: {};", self.initial_zoom.get())?;
        if let Some(min_zoom) = self.min_zoom {
            write!(dest, " min-zoom: {};", min_zoom.get())?;
        }
        if let Some(max_zoom) = self.max_zoom {
            write!(dest, " max-zoom: {};", max_zoom.get())?;
        }
        write!(dest, " user-zoom: ")?; self.user_zoom.to_css(dest)?;
        write!(dest, "; orientation: ")?; self.orientation.to_css(dest)?;
        write!(dest, "; }}")
    }
}

/// https://drafts.csswg.org/css-device-adapt/#descdef-viewport-zoom
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Zoom {
    /// A number value.
    Number(f32),
    /// A percentage value.
    Percentage(f32),
    /// The `auto` keyword.
    Auto,
}

impl ToCss for Zoom {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            Zoom::Number(number) => write!(dest, "{}", number),
            Zoom::Percentage(percentage) => write!(dest, "{}%", percentage * 100.),
            Zoom::Auto => write!(dest, "auto")
        }
    }
}

impl Zoom {
    /// Parse a zoom value per:
    ///
    /// https://drafts.csswg.org/css-device-adapt/#descdef-viewport-zoom
    pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Zoom, ParseError<'i>> {
        use PARSING_MODE_DEFAULT;
        use cssparser::Token;
        use values::specified::AllowedLengthType::NonNegative;

        match input.next()? {
            // TODO: This parse() method should take ParserContext as an
            // argument, and pass ParsingMode owned by the ParserContext to
            // is_ok() instead of using PARSING_MODE_DEFAULT directly.
            // In order to do so, we might want to move these stuff into style::stylesheets::viewport_rule.
            Token::Percentage { unit_value, .. } if NonNegative.is_ok(PARSING_MODE_DEFAULT, unit_value) => {
                Ok(Zoom::Percentage(unit_value))
            }
            Token::Number { value, .. } if NonNegative.is_ok(PARSING_MODE_DEFAULT, value) => {
                Ok(Zoom::Number(value))
            }
            Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") => {
                Ok(Zoom::Auto)
            }
            t => Err(CssParseError::Basic(BasicParseError::UnexpectedToken(t)))
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
