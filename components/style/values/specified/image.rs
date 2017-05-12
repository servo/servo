/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use Atom;
use cssparser::{Parser, Token};
use parser::{Parse, ParserContext};
#[cfg(feature = "servo")]
use servo_url::ServoUrl;
use std::f32::consts::PI;
use std::fmt;
use style_traits::ToCss;
use values::{Either, None_};
use values::generics::image::{Circle, CompatMode, Ellipse, ColorStop as GenericColorStop};
use values::generics::image::{EndingShape as GenericEndingShape, Gradient as GenericGradient};
use values::generics::image::{GradientItem as GenericGradientItem, GradientKind as GenericGradientKind};
use values::generics::image::{Image as GenericImage, ImageRect as GenericImageRect};
use values::generics::image::{LineDirection as GenericsLineDirection, ShapeExtent};
use values::specified::{Angle, CSSColor, Length, LengthOrPercentage, NumberOrPercentage, Percentage};
use values::specified::position::{Position, X, Y};
use values::specified::url::SpecifiedUrl;

/// A specified image layer.
pub type ImageLayer = Either<None_, Image>;

/// Specified values for an image according to CSS-IMAGES.
/// https://drafts.csswg.org/css-images/#image-values
pub type Image = GenericImage<Gradient, ImageRect>;

/// Specified values for a CSS gradient.
/// https://drafts.csswg.org/css-images/#gradients
pub type Gradient = GenericGradient<
    LineDirection,
    Length,
    LengthOrPercentage,
    Position,
    CSSColor,
>;

/// A specified gradient kind.
pub type GradientKind = GenericGradientKind<
    LineDirection,
    Length,
    LengthOrPercentage,
    Position,
>;

/// A specified gradient line direction.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LineDirection {
    /// An angular direction.
    Angle(Angle),
    /// A horizontal direction.
    Horizontal(X),
    /// A vertical direction.
    Vertical(Y),
    /// A direction towards a corner of a box.
    Corner(X, Y),
}

/// A specified ending shape.
pub type EndingShape = GenericEndingShape<Length, LengthOrPercentage>;

/// A specified gradient item.
pub type GradientItem = GenericGradientItem<CSSColor, LengthOrPercentage>;

/// A computed color stop.
pub type ColorStop = GenericColorStop<CSSColor, LengthOrPercentage>;

/// Specified values for `moz-image-rect`
/// -moz-image-rect(<uri>, top, right, bottom, left);
pub type ImageRect = GenericImageRect<NumberOrPercentage>;

impl Parse for Image {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Image, ()> {
        if let Ok(url) = input.try(|input| SpecifiedUrl::parse(context, input)) {
            return Ok(GenericImage::Url(url));
        }
        if let Ok(gradient) = input.try(|input| Gradient::parse_function(context, input)) {
            return Ok(GenericImage::Gradient(gradient));
        }
        if let Ok(image_rect) = input.try(|input| ImageRect::parse(context, input)) {
            return Ok(GenericImage::Rect(image_rect));
        }

        Ok(GenericImage::Element(Image::parse_element(input)?))
    }
}

impl Image {
    /// Creates an already specified image value from an already resolved URL
    /// for insertion in the cascade.
    #[cfg(feature = "servo")]
    pub fn for_cascade(url: ServoUrl) -> Self {
        GenericImage::Url(SpecifiedUrl::for_cascade(url))
    }

    /// Parses a `-moz-element(# <element-id>)`.
    fn parse_element(input: &mut Parser) -> Result<Atom, ()> {
        input.try(|i| i.expect_function_matching("-moz-element"))?;
        input.parse_nested_block(|i| {
            match i.next()? {
                Token::IDHash(id) => Ok(Atom::from(id)),
                _ => Err(()),
            }
        })
    }
}

impl Gradient {
    /// Parses a gradient from the given arguments.
    pub fn parse_function(context: &ParserContext, input: &mut Parser) -> Result<Gradient, ()> {
        enum Shape {
            Linear,
            Radial,
        }

        let (shape, repeating, compat_mode) = match_ignore_ascii_case! { &try!(input.expect_function()),
            "linear-gradient" => {
                (Shape::Linear, false, CompatMode::Modern)
            },
            "-webkit-linear-gradient" => {
                (Shape::Linear, false, CompatMode::WebKit)
            },
            "repeating-linear-gradient" => {
                (Shape::Linear, true, CompatMode::Modern)
            },
            "-webkit-repeating-linear-gradient" => {
                (Shape::Linear, true, CompatMode::WebKit)
            },
            "radial-gradient" => {
                (Shape::Radial, false, CompatMode::Modern)
            },
            "-webkit-radial-gradient" => {
                (Shape::Radial, false, CompatMode::WebKit)
            },
            "repeating-radial-gradient" => {
                (Shape::Radial, true, CompatMode::Modern)
            },
            "-webkit-repeating-radial-gradient" => {
                (Shape::Radial, true, CompatMode::WebKit)
            },
            _ => { return Err(()); }
        };

        let (kind, items) = input.parse_nested_block(|i| {
            let shape = match shape {
                Shape::Linear => GradientKind::parse_linear(context, i, compat_mode)?,
                Shape::Radial => GradientKind::parse_radial(context, i, compat_mode)?,
            };
            let items = Gradient::parse_items(context, i)?;
            Ok((shape, items))
        })?;

        if items.len() < 2 {
            return Err(());
        }

        Ok(Gradient {
            items: items,
            repeating: repeating,
            kind: kind,
            compat_mode: compat_mode,
        })
    }

    fn parse_items(context: &ParserContext, input: &mut Parser) -> Result<Vec<GradientItem>, ()> {
        let mut seen_stop = false;
        let items = try!(input.parse_comma_separated(|input| {
            if seen_stop {
                if let Ok(hint) = input.try(|i| LengthOrPercentage::parse(context, i)) {
                    seen_stop = false;
                    return Ok(GenericGradientItem::InterpolationHint(hint));
                }
            }
            seen_stop = true;
            ColorStop::parse(context, input).map(GenericGradientItem::ColorStop)
        }));
        if !seen_stop || items.len() < 2 {
            return Err(());
        }
        Ok(items)
    }
}

impl GradientKind {
    fn parse_linear(context: &ParserContext,
                    input: &mut Parser,
                    compat_mode: CompatMode)
                    -> Result<Self, ()> {
        let direction = if let Ok(d) = input.try(|i| LineDirection::parse(context, i, compat_mode)) {
            input.expect_comma()?;
            d
        } else {
            LineDirection::Vertical(Y::Bottom)
        };
        Ok(GenericGradientKind::Linear(direction))
    }

    fn parse_radial(context: &ParserContext,
                    input: &mut Parser,
                    compat_mode: CompatMode)
                    -> Result<Self, ()> {
        let (shape, position) = if compat_mode == CompatMode::Modern {
            let shape = input.try(|i| EndingShape::parse(context, i, compat_mode));
            let position = input.try(|i| {
                i.expect_ident_matching("at")?;
                Position::parse(context, i)
            });
            (shape, position)
        } else {
            let position = input.try(|i| Position::parse(context, i));
            let shape = input.try(|i| {
                if position.is_ok() {
                    i.expect_comma()?;
                }
                EndingShape::parse(context, i, compat_mode)
            });
            (shape, position)
        };

        if shape.is_ok() || position.is_ok() {
            input.expect_comma()?;
        }

        let shape = shape.unwrap_or({
            GenericEndingShape::Ellipse(Ellipse::Extent(ShapeExtent::FarthestCorner))
        });
        let position = position.unwrap_or(Position::center());
        Ok(GenericGradientKind::Radial(shape, position))
    }
}

impl GenericsLineDirection for LineDirection {
    fn points_downwards(&self) -> bool {
        match *self {
            LineDirection::Angle(ref angle) => angle.radians() == PI,
            LineDirection::Vertical(Y::Bottom) => true,
            _ => false,
        }
    }

    fn to_css<W>(&self, dest: &mut W, compat_mode: CompatMode) -> fmt::Result
        where W: fmt::Write
    {
        match *self {
            LineDirection::Angle(angle) => {
                angle.to_css(dest)
            },
            LineDirection::Horizontal(x) => {
                if compat_mode == CompatMode::Modern {
                    dest.write_str("to ")?;
                }
                x.to_css(dest)
            },
            LineDirection::Vertical(y) => {
                if compat_mode == CompatMode::Modern {
                    dest.write_str("to ")?;
                }
                y.to_css(dest)
            },
            LineDirection::Corner(x, y) => {
                if compat_mode == CompatMode::Modern {
                    dest.write_str("to ")?;
                }
                x.to_css(dest)?;
                dest.write_str(" ")?;
                y.to_css(dest)
            }
        }
    }
}

impl LineDirection {
    fn parse(context: &ParserContext,
             input: &mut Parser,
             compat_mode: CompatMode)
             -> Result<Self, ()> {
        if let Ok(angle) = input.try(|i| Angle::parse_with_unitless(context, i)) {
            return Ok(LineDirection::Angle(angle));
        }
        input.try(|i| {
            if compat_mode == CompatMode::Modern {
                i.expect_ident_matching("to")?;
            }
            if let Ok(x) = i.try(X::parse) {
                if let Ok(y) = i.try(Y::parse) {
                    return Ok(LineDirection::Corner(x, y));
                }
                return Ok(LineDirection::Horizontal(x));
            }
            let y = Y::parse(i)?;
            if let Ok(x) = i.try(X::parse) {
                return Ok(LineDirection::Corner(x, y));
            }
            Ok(LineDirection::Vertical(y))
        })
    }
}

impl EndingShape {
    fn parse(context: &ParserContext,
             input: &mut Parser,
             compat_mode: CompatMode)
             -> Result<Self, ()> {
        if let Ok(extent) = input.try(|i| ShapeExtent::parse_with_compat_mode(i, compat_mode)) {
            if input.try(|i| i.expect_ident_matching("circle")).is_ok() {
                return Ok(GenericEndingShape::Circle(Circle::Extent(extent)));
            }
            let _ = input.try(|i| i.expect_ident_matching("ellipse"));
            return Ok(GenericEndingShape::Ellipse(Ellipse::Extent(extent)));
        }
        if input.try(|i| i.expect_ident_matching("circle")).is_ok() {
            if let Ok(extent) = input.try(|i| ShapeExtent::parse_with_compat_mode(i, compat_mode)) {
                return Ok(GenericEndingShape::Circle(Circle::Extent(extent)));
            }
            if compat_mode == CompatMode::Modern {
                if let Ok(length) = input.try(|i| Length::parse(context, i)) {
                    return Ok(GenericEndingShape::Circle(Circle::Radius(length)));
                }
            }
            return Ok(GenericEndingShape::Circle(Circle::Extent(ShapeExtent::FarthestCorner)));
        }
        if input.try(|i| i.expect_ident_matching("ellipse")).is_ok() {
            if let Ok(extent) = input.try(|i| ShapeExtent::parse_with_compat_mode(i, compat_mode)) {
                return Ok(GenericEndingShape::Ellipse(Ellipse::Extent(extent)));
            }
            if compat_mode == CompatMode::Modern {
                let pair: Result<_, ()> = input.try(|i| {
                    let x = LengthOrPercentage::parse(context, i)?;
                    let y = LengthOrPercentage::parse(context, i)?;
                    Ok((x, y))
                });
                if let Ok((x, y)) = pair {
                    return Ok(GenericEndingShape::Ellipse(Ellipse::Radii(x, y)));
                }
            }
            return Ok(GenericEndingShape::Ellipse(Ellipse::Extent(ShapeExtent::FarthestCorner)));
        }
        if let Ok(length) = input.try(|i| Length::parse(context, i)) {
            if let Ok(y) = input.try(|i| LengthOrPercentage::parse(context, i)) {
                if compat_mode == CompatMode::Modern {
                    let _ = input.try(|i| i.expect_ident_matching("ellipse"));
                }
                return Ok(GenericEndingShape::Ellipse(Ellipse::Radii(length.into(), y)));
            }
            if compat_mode == CompatMode::Modern {
                let y = input.try(|i| {
                    i.expect_ident_matching("ellipse")?;
                    LengthOrPercentage::parse(context, i)
                });
                if let Ok(y) = y {
                    return Ok(GenericEndingShape::Ellipse(Ellipse::Radii(length.into(), y)));
                }
                let _ = input.try(|i| i.expect_ident_matching("circle"));
            }
            return Ok(GenericEndingShape::Circle(Circle::Radius(length)));
        }
        input.try(|i| {
            let x = Percentage::parse(context, i)?;
            let y = if let Ok(y) = i.try(|i| LengthOrPercentage::parse(context, i)) {
                if compat_mode == CompatMode::Modern {
                    let _ = i.try(|i| i.expect_ident_matching("ellipse"));
                }
                y
            } else {
                if compat_mode == CompatMode::Modern {
                    i.expect_ident_matching("ellipse")?;
                }
                LengthOrPercentage::parse(context, i)?
            };
            Ok(GenericEndingShape::Ellipse(Ellipse::Radii(x.into(), y)))
        })
    }
}

impl ShapeExtent {
    fn parse_with_compat_mode(input: &mut Parser,
                              compat_mode: CompatMode)
                              -> Result<Self, ()> {
        match try!(Self::parse(input)) {
            ShapeExtent::Contain | ShapeExtent::Cover if compat_mode == CompatMode::Modern => Err(()),
            keyword => Ok(keyword),
        }
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

impl Parse for ColorStop {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        Ok(ColorStop {
            color: try!(CSSColor::parse(context, input)),
            position: input.try(|i| LengthOrPercentage::parse(context, i)).ok(),
        })
    }
}
