/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Helper types for the `@viewport` rule.

use {PagePx, ViewportPx};
use cssparser::{Parser, ToCss};
use euclid::scale_factor::ScaleFactor;
use euclid::size::TypedSize2D;
use std::ascii::AsciiExt;
use std::fmt;
use values::specified::AllowedNumericType;

define_css_keyword_enum!(UserZoom:
                         "zoom" => Zoom,
                         "fixed" => Fixed);

define_css_keyword_enum!(Orientation:
                         "auto" => Auto,
                         "portrait" => Portrait,
                         "landscape" => Landscape);


/// A set of viewport descriptors:
///
/// https://drafts.csswg.org/css-device-adapt/#viewport-desc
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize, HeapSizeOf))]
pub struct ViewportConstraints {
    /// Width and height:
    ///  * https://drafts.csswg.org/css-device-adapt/#width-desc
    ///  * https://drafts.csswg.org/css-device-adapt/#height-desc
    pub size: TypedSize2D<f32, ViewportPx>,
    /// https://drafts.csswg.org/css-device-adapt/#zoom-desc
    pub initial_zoom: ScaleFactor<f32, PagePx, ViewportPx>,
    /// https://drafts.csswg.org/css-device-adapt/#min-max-width-desc
    pub min_zoom: Option<ScaleFactor<f32, PagePx, ViewportPx>>,
    /// https://drafts.csswg.org/css-device-adapt/#min-max-width-desc
    pub max_zoom: Option<ScaleFactor<f32, PagePx, ViewportPx>>,
    /// https://drafts.csswg.org/css-device-adapt/#user-zoom-desc
    pub user_zoom: UserZoom,
    /// https://drafts.csswg.org/css-device-adapt/#orientation-desc
    pub orientation: Orientation
}

impl ToCss for ViewportConstraints {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        try!(write!(dest, "@viewport {{"));
        try!(write!(dest, " width: {}px;", self.size.width));
        try!(write!(dest, " height: {}px;", self.size.height));
        try!(write!(dest, " zoom: {};", self.initial_zoom.get()));
        if let Some(min_zoom) = self.min_zoom {
            try!(write!(dest, " min-zoom: {};", min_zoom.get()));
        }
        if let Some(max_zoom) = self.max_zoom {
            try!(write!(dest, " max-zoom: {};", max_zoom.get()));
        }
        try!(write!(dest, " user-zoom: ")); try!(self.user_zoom.to_css(dest));
        try!(write!(dest, "; orientation: ")); try!(self.orientation.to_css(dest));
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
    pub fn parse(input: &mut Parser) -> Result<Zoom, ()> {
        use cssparser::Token;

        match try!(input.next()) {
            Token::Percentage(ref value) if AllowedNumericType::NonNegative.is_ok(value.unit_value) =>
                Ok(Zoom::Percentage(value.unit_value)),
            Token::Number(ref value) if AllowedNumericType::NonNegative.is_ok(value.value) =>
                Ok(Zoom::Number(value.value)),
            Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") =>
                Ok(Zoom::Auto),
            _ => Err(())
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
