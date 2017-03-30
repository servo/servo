/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use cssparser::Parser;
use parser::{Parse, ParserContext};
#[cfg(feature = "servo")]
use servo_url::ServoUrl;
use std::fmt;
use style_traits::ToCss;
use values::specified::{Angle, CSSColor, Length, LengthOrPercentage, NumberOrPercentage};
use values::specified::position::Position;
use values::specified::url::SpecifiedUrl;

/// Specified values for an image according to CSS-IMAGES.
/// https://drafts.csswg.org/css-images/#image-values
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Image {
    /// A `<url()>` image.
    Url(SpecifiedUrl),
    /// A `<gradient>` image.
    Gradient(Gradient),
    /// A `-moz-image-rect` image
    ImageRect(ImageRect),
}

impl ToCss for Image {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Image::Url(ref url_value) => url_value.to_css(dest),
            Image::Gradient(ref gradient) => gradient.to_css(dest),
            Image::ImageRect(ref image_rect) => image_rect.to_css(dest),
        }
    }
}

impl Image {
    #[allow(missing_docs)]
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<Image, ()> {
        if let Ok(url) = input.try(|input| SpecifiedUrl::parse(context, input)) {
            return Ok(Image::Url(url));
        }
        if let Ok(gradient) = input.try(|input| Gradient::parse_function(context, input)) {
            return Ok(Image::Gradient(gradient));
        }

        Ok(Image::ImageRect(ImageRect::parse(context, input)?))
    }

    /// Creates an already specified image value from an already resolved URL
    /// for insertion in the cascade.
    #[cfg(feature = "servo")]
    pub fn for_cascade(url: ServoUrl) -> Self {
        Image::Url(SpecifiedUrl::for_cascade(url))
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
        let mut skipcomma = false;
        match self.gradient_kind {
            GradientKind::Linear(angle_or_corner) => {
                try!(dest.write_str("linear-gradient("));
                try!(angle_or_corner.to_css(dest));
                if angle_or_corner == AngleOrCorner::None {
                    skipcomma = true;
                }
            },
            GradientKind::Radial(ref shape, ref position) => {
                try!(dest.write_str("radial-gradient("));
                try!(shape.to_css(dest));
                try!(dest.write_str(" at "));
                try!(position.to_css(dest));
            },
        }
        for stop in &self.stops {
            if !skipcomma {
                try!(dest.write_str(", "));
            } else {
                skipcomma = false;
            }
            try!(stop.to_css(dest));
        }
        dest.write_str(")")
    }
}

impl Gradient {
    /// Parses a gradient from the given arguments.
    pub fn parse_function(context: &ParserContext, input: &mut Parser) -> Result<Gradient, ()> {
        let mut repeating = false;
        let (gradient_kind, stops) = match_ignore_ascii_case! { &try!(input.expect_function()),
            "linear-gradient" => {
                try!(input.parse_nested_block(|input| {
                        let kind = try!(GradientKind::parse_linear(context, input));
                        let stops = try!(input.parse_comma_separated(|i| ColorStop::parse(context, i)));
                        Ok((kind, stops))
                    })
                )
            },
            "repeating-linear-gradient" => {
                repeating = true;
                try!(input.parse_nested_block(|input| {
                        let kind = try!(GradientKind::parse_linear(context, input));
                        let stops = try!(input.parse_comma_separated(|i| ColorStop::parse(context, i)));
                        Ok((kind, stops))
                    })
                )
            },
            "radial-gradient" => {
                try!(input.parse_nested_block(|input| {
                        let kind = try!(GradientKind::parse_radial(context, input));
                        let stops = try!(input.parse_comma_separated(|i| ColorStop::parse(context, i)));
                        Ok((kind, stops))
                    })
                )
            },
            "repeating-radial-gradient" => {
                repeating = true;
                try!(input.parse_nested_block(|input| {
                        let kind = try!(GradientKind::parse_radial(context, input));
                        let stops = try!(input.parse_comma_separated(|i| ColorStop::parse(context, i)));
                        Ok((kind, stops))
                    })
                )
            },
            _ => { return Err(()); }
        };

        // https://drafts.csswg.org/css-images/#typedef-color-stop-list
        if stops.len() < 2 {
            return Err(())
        }

        Ok(Gradient {
            stops: stops,
            repeating: repeating,
            gradient_kind: gradient_kind,
        })
    }
}

/// Specified values for CSS linear or radial gradients.
/// https://drafts.csswg.org/css-images/#gradients
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum GradientKind {
    /// A `<linear-gradient()>`:
    ///
    /// https://drafts.csswg.org/css-images/#funcdef-linear-gradient
    Linear(AngleOrCorner),

    /// A `<radial-gradient()>`:
    ///
    /// https://drafts.csswg.org/css-images/#radial-gradients
    Radial(EndingShape, Position),
}

impl GradientKind {
    /// Parses a linear gradient kind from the given arguments.
    pub fn parse_linear(context: &ParserContext, input: &mut Parser) -> Result<GradientKind, ()> {
        let angle_or_corner = try!(AngleOrCorner::parse(context, input));
        Ok(GradientKind::Linear(angle_or_corner))
    }

    /// Parses a radial gradient from the given arguments.
    pub fn parse_radial(context: &ParserContext, input: &mut Parser) -> Result<GradientKind, ()> {
        let mut needs_comma = true;

        // Ending shape and position can be in various order. Checks all probabilities.
        let (shape, position) = if let Ok(position) = input.try(|i| parse_position(context, i)) {
            // Handle just <position>
            (EndingShape::Ellipse(LengthOrPercentageOrKeyword::Keyword(SizeKeyword::FarthestCorner)), position)
        } else if let Ok((first, second)) = input.try(|i| parse_two_length(context, i)) {
            // Handle <LengthOrPercentage> <LengthOrPercentage> <shape>? <position>?
            let _ = input.try(|input| input.expect_ident_matching("ellipse"));
            (EndingShape::Ellipse(LengthOrPercentageOrKeyword::LengthOrPercentage(first, second)),
             input.try(|i| parse_position(context, i)).unwrap_or(Position::center()))
        } else if let Ok(length) = input.try(|i| Length::parse(context, i)) {
            // Handle <Length> <circle>? <position>?
            let _ = input.try(|input| input.expect_ident_matching("circle"));
            (EndingShape::Circle(LengthOrKeyword::Length(length)),
             input.try(|i| parse_position(context, i)).unwrap_or(Position::center()))
        } else if let Ok(keyword) = input.try(SizeKeyword::parse) {
            // Handle <keyword> <shape-keyword>? <position>?
            let shape = if input.try(|input| input.expect_ident_matching("circle")).is_ok() {
                EndingShape::Circle(LengthOrKeyword::Keyword(keyword))
            } else {
                let _ = input.try(|input| input.expect_ident_matching("ellipse"));
                EndingShape::Ellipse(LengthOrPercentageOrKeyword::Keyword(keyword))
            };
            (shape, input.try(|i| parse_position(context, i)).unwrap_or(Position::center()))
        } else {
            // Handle <shape-keyword> <length>? <position>?
            if input.try(|input| input.expect_ident_matching("ellipse")).is_ok() {
                // Handle <ellipse> <LengthOrPercentageOrKeyword>? <position>?
                let length = input.try(|i| LengthOrPercentageOrKeyword::parse(context, i))
                                  .unwrap_or(LengthOrPercentageOrKeyword::Keyword(SizeKeyword::FarthestCorner));
                (EndingShape::Ellipse(length),
                 input.try(|i| parse_position(context, i)).unwrap_or(Position::center()))
            } else if input.try(|input| input.expect_ident_matching("circle")).is_ok() {
                // Handle <ellipse> <LengthOrKeyword>? <position>?
                let length = input.try(|i| LengthOrKeyword::parse(context, i))
                                  .unwrap_or(LengthOrKeyword::Keyword(SizeKeyword::FarthestCorner));
                (EndingShape::Circle(length), input.try(|i| parse_position(context, i))
                                                   .unwrap_or(Position::center()))
            } else {
                // If there is no shape keyword, it should set to default.
                needs_comma = false;
                (EndingShape::Ellipse(LengthOrPercentageOrKeyword::Keyword(SizeKeyword::FarthestCorner)),
                 input.try(|i| parse_position(context, i)).unwrap_or(Position::center()))
            }
        };

        if needs_comma {
            try!(input.expect_comma());
        }

        Ok(GradientKind::Radial(shape, position))
    }
}

/// Specified values for `moz-image-rect`
/// -moz-image-rect(<uri>, top, right, bottom, left);
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct ImageRect {
    pub url: SpecifiedUrl,
    pub top: NumberOrPercentage,
    pub bottom: NumberOrPercentage,
    pub right: NumberOrPercentage,
    pub left: NumberOrPercentage,
}

impl ToCss for ImageRect {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("-moz-image-rect(")?;
        self.url.to_css(dest)?;
        dest.write_str(", ")?;
        self.top.to_css(dest)?;
        dest.write_str(", ")?;
        self.right.to_css(dest)?;
        dest.write_str(", ")?;
        self.bottom.to_css(dest)?;
        dest.write_str(", ")?;
        self.left.to_css(dest)?;
        dest.write_str(")")
    }
}

impl Parse for ImageRect {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { &try!(input.expect_function()),
            "-moz-image-rect" => {
                input.parse_nested_block(|input| {
                    let url = SpecifiedUrl::parse(context, input)?;
                    input.expect_comma()?;
                    let top = NumberOrPercentage::parse(context, input)?;
                    input.expect_comma()?;
                    let right = NumberOrPercentage::parse(context, input)?;
                    input.expect_comma()?;
                    let bottom = NumberOrPercentage::parse(context, input)?;
                    input.expect_comma()?;
                    let left = NumberOrPercentage::parse(context, input)?;

                    Ok(ImageRect {
                        url: url,
                        top: top,
                        right: right,
                        bottom: bottom,
                        left: left,
                    })
                })
            }
            _ => Err(())
        }
    }
}

fn parse_two_length(context: &ParserContext, input: &mut Parser)
                    -> Result<(LengthOrPercentage, LengthOrPercentage), ()> {
    let first = try!(LengthOrPercentage::parse(context, input));
    let second = try!(LengthOrPercentage::parse(context, input));
    Ok((first, second))
}

fn parse_position(context: &ParserContext, input: &mut Parser) -> Result<Position, ()> {
    try!(input.expect_ident_matching("at"));
    input.try(|i| Position::parse(context, i))
}

/// Specified values for an angle or a corner in a linear gradient.
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum AngleOrCorner {
    Angle(Angle),
    Corner(Option<HorizontalDirection>, Option<VerticalDirection>),
    None,
}

impl ToCss for AngleOrCorner {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            AngleOrCorner::None => Ok(()),
            AngleOrCorner::Angle(angle) => angle.to_css(dest),
            AngleOrCorner::Corner(horizontal, vertical) => {
                try!(dest.write_str("to "));
                let mut horizontal_present = false;
                if let Some(horizontal) = horizontal {
                    try!(horizontal.to_css(dest));
                    horizontal_present = true;
                }
                if let Some(vertical) = vertical {
                    if horizontal_present {
                        try!(dest.write_str(" "));
                    }
                    try!(vertical.to_css(dest));
                }
                Ok(())
            }
        }
    }
}

impl Parse for AngleOrCorner {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if input.try(|input| input.expect_ident_matching("to")).is_ok() {
            let (horizontal, vertical) =
            if let Ok(value) = input.try(HorizontalDirection::parse) {
                (Some(value), input.try(VerticalDirection::parse).ok())
            } else {
                let value = try!(VerticalDirection::parse(input));
                (input.try(HorizontalDirection::parse).ok(), Some(value))
            };
            try!(input.expect_comma());
            Ok(AngleOrCorner::Corner(horizontal, vertical))
        } else if let Ok(angle) = input.try(|i| Angle::parse(context, i)) {
            try!(input.expect_comma());
            Ok(AngleOrCorner::Angle(angle))
        } else {
            Ok(AngleOrCorner::None)
        }
    }
}

/// Specified values for one color stop in a linear gradient.
/// https://drafts.csswg.org/css-images/#typedef-color-stop-list
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
        if let Some(ref position) = self.position {
            try!(dest.write_str(" "));
            try!(position.to_css(dest));
        }
        Ok(())
    }
}

define_css_keyword_enum!(HorizontalDirection: "left" => Left, "right" => Right);
define_css_keyword_enum!(VerticalDirection: "top" => Top, "bottom" => Bottom);

impl Parse for ColorStop {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        Ok(ColorStop {
            color: try!(CSSColor::parse(context, input)),
            position: input.try(|i| LengthOrPercentage::parse(context, i)).ok(),
        })
    }
}

/// Determines whether the gradient's ending shape is a circle or an ellipse.
/// If <shape> is omitted, the ending shape defaults to a circle
/// if the <size> is a single <length>, and to an ellipse otherwise.
/// https://drafts.csswg.org/css-images/#valdef-radial-gradient-ending-shape
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum EndingShape {
    Circle(LengthOrKeyword),
    Ellipse(LengthOrPercentageOrKeyword),
}

impl ToCss for EndingShape {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            EndingShape::Circle(ref length) => {
                try!(dest.write_str("circle "));
                try!(length.to_css(dest));
            },
            EndingShape::Ellipse(ref length) => {
                try!(dest.write_str("ellipse "));
                try!(length.to_css(dest));
            },
        }
        Ok(())
    }
}

/// https://drafts.csswg.org/css-images/#valdef-radial-gradient-size
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum LengthOrKeyword {
    Length(Length),
    Keyword(SizeKeyword),
}

impl Parse for LengthOrKeyword {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(keyword) = input.try(SizeKeyword::parse) {
            Ok(LengthOrKeyword::Keyword(keyword))
        } else {
            Ok(LengthOrKeyword::Length(try!(Length::parse(context, input))))
        }
    }
}

impl ToCss for LengthOrKeyword {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrKeyword::Length(ref length) => length.to_css(dest),
            LengthOrKeyword::Keyword(keyword) => keyword.to_css(dest),
        }
    }
}

/// https://drafts.csswg.org/css-images/#valdef-radial-gradient-size
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum LengthOrPercentageOrKeyword {
    LengthOrPercentage(LengthOrPercentage, LengthOrPercentage),
    Keyword(SizeKeyword),
}


impl Parse for LengthOrPercentageOrKeyword {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(keyword) = input.try(SizeKeyword::parse) {
            Ok(LengthOrPercentageOrKeyword::Keyword(keyword))
        } else {
            Ok(LengthOrPercentageOrKeyword::LengthOrPercentage(
                try!(LengthOrPercentage::parse(context, input)),
                try!(LengthOrPercentage::parse(context, input))))
        }
    }
}

impl ToCss for LengthOrPercentageOrKeyword {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrPercentageOrKeyword::LengthOrPercentage(ref first_len, ref second_len) => {
                try!(first_len.to_css(dest));
                try!(dest.write_str(" "));
                second_len.to_css(dest)
            },
            LengthOrPercentageOrKeyword::Keyword(keyword) => keyword.to_css(dest),
        }
    }
}

/// https://drafts.csswg.org/css-images/#typedef-extent-keyword
define_css_keyword_enum!(SizeKeyword: "closest-side" => ClosestSide, "farthest-side" => FarthestSide,
                         "closest-corner" => ClosestCorner, "farthest-corner" => FarthestCorner);
