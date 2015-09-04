/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ToCss};
use euclid::scale_factor::ScaleFactor;
use euclid::size::TypedSize2D;
use std::ascii::AsciiExt;
use std::fmt;
use util::geometry::{PagePx, ViewportPx};
use values::specified::AllowedNumericType;

define_css_keyword_enum!(UserZoom:
                         "zoom" => Zoom,
                         "fixed" => Fixed);

define_css_keyword_enum!(Orientation:
                         "auto" => Auto,
                         "portrait" => Portrait,
                         "landscape" => Landscape);


#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ViewportConstraints {
    pub size: TypedSize2D<ViewportPx, f32>,

    pub initial_zoom: ScaleFactor<PagePx, ViewportPx, f32>,
    pub min_zoom: Option<ScaleFactor<PagePx, ViewportPx, f32>>,
    pub max_zoom: Option<ScaleFactor<PagePx, ViewportPx, f32>>,

    pub user_zoom: UserZoom,
    pub orientation: Orientation
}

impl ToCss for ViewportConstraints {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        try!(write!(dest, "@viewport {{"));
        try!(write!(dest, " width: {}px;", self.size.width.get()));
        try!(write!(dest, " height: {}px;", self.size.height.get()));
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

/// Zoom is a number | percentage | auto
/// See http://dev.w3.org/csswg/css-device-adapt/#descdef-viewport-zoom
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Zoom {
    Number(f32),
    Percentage(f32),
    Auto,
}

impl ToCss for Zoom {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        match *self {
            Zoom::Number(number) => write!(dest, "{}", number),
            Zoom::Percentage(percentage) => write!(dest, "{}%", percentage * 100.),
            Zoom::Auto => write!(dest, "auto")
        }
    }
}

impl Zoom {
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

    #[inline]
    pub fn to_f32(&self) -> Option<f32> {
        match *self {
            Zoom::Number(number) => Some(number as f32),
            Zoom::Percentage(percentage) => Some(percentage as f32),
            Zoom::Auto => None
        }
    }
}
