/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use cssparser::{Parser, ToCss};
use parser::{Parse, ParserContext};
use std::f32::consts::PI;
use std::fmt;
use url::Url;
use values::computed::ToComputedValue;
use values::specified::{Angle, CSSColor, Length, LengthOrPercentage, UrlExtraData};
use values::specified::position::{Keyword, Position};

/// Specified values for an image according to CSS-IMAGES.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Image {
    Url(Url, UrlExtraData),
    Gradient(Gradient),
}

impl ToCss for Image {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use values::LocalToCss;
        match *self {
            Image::Url(ref url, ref _extra_data) => {
                url.to_css(dest)
            }
            Image::Gradient(ref gradient) => gradient.to_css(dest)
        }
    }
}

impl Image {
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<Image, ()> {
        if let Ok(url) = input.try(|input| input.expect_url()) {
            match UrlExtraData::make_from(context) {
                Some(extra_data) => {
                    Ok(Image::Url(context.parse_url(&url), extra_data))
                },
                None => {
                    // FIXME(heycam) should ensure we always have a principal, etc., when
                    // parsing style attributes and re-parsing due to CSS Variables.
                    println!("stylo: skipping declaration without ParserContextExtraData");
                    Err(())
                },
            }
        } else {
            Ok(Image::Gradient(try!(Gradient::parse_function(input))))
        }
    }
}

/// Specified values for a CSS gradient.
/// https://drafts.csswg.org/css-images/#gradients
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Gradient {
    /// The color stops.
    pub stops: Vec<ColorStop>,
    /// True if this is a repeating gradient.
    pub repeating: bool,
    /// Gradients can be linear or radial.
    pub gradient_kind: GradientKind,
}

impl ToCss for Gradient {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.repeating {
            try!(dest.write_str("repeating-"));
        }
        match self.gradient_kind {
            GradientKind::Linear(angle_or_corner) => {
                try!(dest.write_str("linear-gradient("));
                try!(angle_or_corner.to_css(dest));
            },
            GradientKind::Radial(ref shape, position) => {
                try!(dest.write_str("radial-gradient("));
                try!(shape.to_css(dest));
                try!(dest.write_str(" at "));
                try!(position.to_css(dest));
            },
        }
        for stop in &self.stops {
            try!(dest.write_str(", "));
            try!(stop.to_css(dest));
        }
        try!(dest.write_str(")"));
        Ok(())
    }
}

impl Gradient {
    /// Parses a gradient from the given arguments.
    pub fn parse_function(input: &mut Parser) -> Result<Gradient, ()> {
        let mut repeating = false;
        let gradient_kind = match_ignore_ascii_case! { try!(input.expect_function()),
            "linear-gradient" => {
                Ok(try!(input.parse_nested_block(|input| {
                        GradientKind::parse_linear(input)
                    })
                ))
            },
            "repeating-linear-gradient" => {
                repeating = true;
                Ok(try!(input.parse_nested_block(|input| {
                        GradientKind::parse_linear(input)
                    })
                ))
            },
            "radial-gradient" => {
                Ok(try!(input.parse_nested_block(|input| {
                        GradientKind::parse_radial(input)
                    })
                ))
            },
            "repeating-radial-gradient" => {
                repeating = true;
                Ok(try!(input.parse_nested_block(|input| {
                        GradientKind::parse_radial(input)
                    })
                ))
            },
            _ => Err(())
        };
        // Parse the color stops.
        let stops = try!(input.parse_comma_separated(parse_one_color_stop));
        if stops.len() < 2 {
            return Err(())
        }
        Ok(Gradient {
            stops: stops,
            repeating: repeating,
            gradient_kind: gradient_kind.unwrap(),
        })
    }
}

/// Specified values for CSS linear or radial gradients.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum GradientKind {
    Linear(AngleOrCorner),
    Radial(EndingShape, Position),
}

impl GradientKind {
    /// Parses a linear gradient kind from the given arguments.
    pub fn parse_linear(input: &mut Parser) -> Result<GradientKind, ()> {
        let angle_or_corner = if input.try(|input| input.expect_ident_matching("to")).is_ok() {
            let (horizontal, vertical) =
            if let Ok(value) = input.try(HorizontalDirection::parse) {
                (Some(value), input.try(VerticalDirection::parse).ok())
            } else {
                let value = try!(VerticalDirection::parse(input));
                (input.try(HorizontalDirection::parse).ok(), Some(value))
            };
            try!(input.expect_comma());
            match (horizontal, vertical) {
                (None, Some(VerticalDirection::Top)) => {
                    AngleOrCorner::Angle(Angle(0.0))
                },
                (Some(HorizontalDirection::Right), None) => {
                    AngleOrCorner::Angle(Angle(PI * 0.5))
                },
                (None, Some(VerticalDirection::Bottom)) => {
                    AngleOrCorner::Angle(Angle(PI))
                },
                (Some(HorizontalDirection::Left), None) => {
                    AngleOrCorner::Angle(Angle(PI * 1.5))
                },
                (Some(horizontal), Some(vertical)) => {
                    AngleOrCorner::Corner(horizontal, vertical)
                }
                (None, None) => unreachable!(),
            }
        } else if let Ok(angle) = input.try(Angle::parse) {
            try!(input.expect_comma());
            AngleOrCorner::Angle(angle)
        } else {
            AngleOrCorner::Angle(Angle(PI))
        };

        Ok(GradientKind::Linear(angle_or_corner))
    }

    /// Parses a radial gradient from the given arguments.
    pub fn parse_radial(input: &mut Parser) -> Result<GradientKind, ()> {
        let shape = input.try(EndingShape::parse)
                         .unwrap_or(EndingShape::Circle(Size::Keyword(SizeKeyword::FarthestCorner)));

        let position = if input.try(|input| input.expect_ident_matching("at")).is_ok() {
            try!(Position::parse(input))
        } else {
            // TODO(canaltinova): Is it default value? I'm not sure.
            Position {
                horiz_keyword: Some(Keyword::Center),
                horiz_position: None,
                vert_keyword: Some(Keyword::Center),
                vert_position: None,
            }
        };

        Ok(GradientKind::Radial(shape, position))
    }
}

/// Specified values for an angle or a corner in a linear gradient.
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum AngleOrCorner {
    Angle(Angle),
    Corner(HorizontalDirection, VerticalDirection),
}

impl ToCss for AngleOrCorner {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            AngleOrCorner::Angle(angle) => angle.to_css(dest),
            AngleOrCorner::Corner(horizontal, vertical) => {
                try!(dest.write_str("to "));
                try!(horizontal.to_css(dest));
                try!(dest.write_str(" "));
                try!(vertical.to_css(dest));
                Ok(())
            }
        }
    }
}

/// Specified values for one color stop in a linear gradient.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ColorStop {
    /// The color of this stop.
    pub color: CSSColor,

    /// The position of this stop. If not specified, this stop is placed halfway between the
    /// point that precedes it and the point that follows it.
    pub position: Option<LengthOrPercentage>,
}

impl ToCss for ColorStop {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.color.to_css(dest));
        if let Some(position) = self.position {
            try!(dest.write_str(" "));
            try!(position.to_css(dest));
        }
        Ok(())
    }
}

define_css_keyword_enum!(HorizontalDirection: "left" => Left, "right" => Right);
define_css_keyword_enum!(VerticalDirection: "top" => Top, "bottom" => Bottom);

fn parse_one_color_stop(input: &mut Parser) -> Result<ColorStop, ()> {
    Ok(ColorStop {
        color: try!(CSSColor::parse(input)),
        position: input.try(LengthOrPercentage::parse).ok(),
    })
}

/// Determines whether the gradient's ending shape is a circle or an ellipse.
/// If <shape> is omitted, the ending shape defaults to a circle
/// if the <size> is a single <length>, and to an ellipse otherwise.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum EndingShape {
    Circle(Size<Length>),
    Ellipse(Size<LengthOrPercentage>, Size<LengthOrPercentage>),
}

impl Parse for EndingShape {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            "circle" => {
                let position = input.try(Size::parse).unwrap_or(
                    Size::Keyword(SizeKeyword::FarthestSide));
                Ok(EndingShape::Circle(position))
            },
            "ellipse" => {
              let (first_pos, second_pos) = if let Ok(first) = input.try(Size::parse) {
                    if let Ok(second) = input.try(Size::parse) {
                        (first, second)
                    } else {
                        (first, Size::Keyword(SizeKeyword::FarthestSide))
                    }
                } else {
                    (Size::Keyword(SizeKeyword::FarthestSide),
                     Size::Keyword(SizeKeyword::FarthestSide))
                };
                Ok(EndingShape::Ellipse(first_pos, second_pos))
            },
            _ => {
                // If 1 <length> is present, it defaults to circle, otherwise defaults to ellipse.
                if let Ok(first) = input.try(Size::parse) {
                    if let Ok(second) = input.try(Size::parse) {
                        Ok(EndingShape::Ellipse(first, second))
                    } else {
                        if let Size::Length(length) = first {
                            if let LengthOrPercentage::Length(len) = length {
                                Ok(EndingShape::Circle(Size::Length(len)))
                            } else {
                                Err(())
                            }
                        } else {
                            Err(())
                        }
                    }
                } else {
                    Err(())
                }
            }
        }
    }
}

impl ToCss for EndingShape {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            EndingShape::Circle(ref length) => {
                try!(dest.write_str("circle "));
                try!(length.to_css(dest));
            },
            EndingShape::Ellipse(ref first_len, ref second_len) => {
                try!(dest.write_str("ellipse "));
                try!(first_len.to_css(dest));
                try!(dest.write_str(" "));
                try!(second_len.to_css(dest));
            },
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Size<T: Clone + Parse + ToCss + ToComputedValue> {
    Length(T),
    Keyword(SizeKeyword),
}

impl<T: Clone + Parse + ToCss + ToComputedValue> Size<T> {
    pub fn parse(input: &mut Parser) -> Result<Size<T>, ()> {
        if let Ok(keyword) = input.try(SizeKeyword::parse) {
            Ok(Size::Keyword(keyword))
        } else {
            let length = input.try(<T>::parse).unwrap(); // TODO(canaltinova): It may not work
            Ok(Size::Length(length))
        }
    }
}

impl<T: Clone + Parse + ToCss + ToComputedValue> ToCss for Size<T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Size::Length(ref length) => length.to_css(dest),
            Size::Keyword(keyword) => keyword.to_css(dest),
        }
    }
}

define_css_keyword_enum!(SizeKeyword: "closest-side" => ClosestSide, "farthest-side" => FarthestSide,
                         "closest-corner" => ClosestCorner, "farthest-corner" => FarthestCorner);
