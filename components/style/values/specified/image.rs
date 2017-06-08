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
use std::cmp::Ordering;
use std::f32::consts::PI;
use std::fmt;
use style_traits::ToCss;
use values::{Either, None_};
use values::generics::image::{Circle, CompatMode, Ellipse, ColorStop as GenericColorStop};
use values::generics::image::{EndingShape as GenericEndingShape, Gradient as GenericGradient};
use values::generics::image::{GradientItem as GenericGradientItem, GradientKind as GenericGradientKind};
use values::generics::image::{Image as GenericImage, ImageRect as GenericImageRect};
use values::generics::image::{LineDirection as GenericsLineDirection, ShapeExtent};
use values::generics::image::PaintWorklet;
use values::generics::position::Position as GenericPosition;
use values::specified::{Angle, Color, Length, LengthOrPercentage};
use values::specified::{Number, NumberOrPercentage, Percentage, RGBAColor};
use values::specified::position::{Position, PositionComponent, Side, X, Y};
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
    RGBAColor,
>;

/// A specified gradient kind.
pub type GradientKind = GenericGradientKind<
    LineDirection,
    Length,
    LengthOrPercentage,
    Position,
>;

/// A specified gradient line direction.
#[derive(Clone, Debug, HasViewportPercentage, PartialEq)]
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
pub type GradientItem = GenericGradientItem<RGBAColor, LengthOrPercentage>;

/// A computed color stop.
pub type ColorStop = GenericColorStop<RGBAColor, LengthOrPercentage>;

/// Specified values for `moz-image-rect`
/// -moz-image-rect(<uri>, top, right, bottom, left);
pub type ImageRect = GenericImageRect<NumberOrPercentage>;

impl Parse for Image {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Image, ()> {
        #[cfg(feature = "gecko")]
        {
          if let Ok(mut url) = input.try(|input| SpecifiedUrl::parse(context, input)) {
              url.build_image_value();
              return Ok(GenericImage::Url(url));
          }
        }
        #[cfg(feature = "servo")]
        {
          if let Ok(url) = input.try(|input| SpecifiedUrl::parse(context, input)) {
              return Ok(GenericImage::Url(url));
          }
        }
        if let Ok(gradient) = input.try(|i| Gradient::parse(context, i)) {
            return Ok(GenericImage::Gradient(gradient));
        }
        #[cfg(feature = "servo")]
        {
            if let Ok(paint_worklet) = input.try(|i| PaintWorklet::parse(context, i)) {
                return Ok(GenericImage::PaintWorklet(paint_worklet));
            }
        }
        #[cfg(feature = "gecko")]
        {
            if let Ok(mut image_rect) = input.try(|input| ImageRect::parse(context, input)) {
                image_rect.url.build_image_value();
                return Ok(GenericImage::Rect(image_rect));
            }
        }
        #[cfg(feature = "servo")]
        {
            if let Ok(image_rect) = input.try(|input| ImageRect::parse(context, input)) {
                return Ok(GenericImage::Rect(image_rect));
            }
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

impl Parse for Gradient {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
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
            "-webkit-gradient" => {
                return input.parse_nested_block(|i| Self::parse_webkit_gradient_argument(context, i));
            },
            _ => { return Err(()); }
        };

        let (kind, items) = input.parse_nested_block(|i| {
            let shape = match shape {
                Shape::Linear => GradientKind::parse_linear(context, i, compat_mode)?,
                Shape::Radial => GradientKind::parse_radial(context, i, compat_mode)?,
            };
            let items = GradientItem::parse_comma_separated(context, i)?;
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
}

impl Gradient {
    fn parse_webkit_gradient_argument(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        type Point = GenericPosition<Component<X>, Component<Y>>;

        #[derive(Clone, Copy)]
        enum Component<S> {
            Center,
            Number(NumberOrPercentage),
            Side(S),
        }

        impl LineDirection {
            fn from_points(first: Point, second: Point) -> Self {
                let h_ord = first.horizontal.partial_cmp(&second.horizontal);
                let v_ord = first.vertical.partial_cmp(&second.vertical);
                let (h, v) = match (h_ord, v_ord) {
                    (Some(h), Some(v)) => (h, v),
                    _ => return LineDirection::Vertical(Y::Bottom),
                };
                match (h, v) {
                    (Ordering::Less, Ordering::Less) => {
                        LineDirection::Corner(X::Right, Y::Bottom)
                    },
                    (Ordering::Less, Ordering::Equal) => {
                        LineDirection::Horizontal(X::Right)
                    },
                    (Ordering::Less, Ordering::Greater) => {
                        LineDirection::Corner(X::Right, Y::Top)
                    },
                    (Ordering::Equal, Ordering::Greater) => {
                        LineDirection::Vertical(Y::Top)
                    },
                    (Ordering::Equal, Ordering::Equal) |
                    (Ordering::Equal, Ordering::Less) => {
                        LineDirection::Vertical(Y::Bottom)
                    },
                    (Ordering::Greater, Ordering::Less) => {
                        LineDirection::Corner(X::Left, Y::Bottom)
                    },
                    (Ordering::Greater, Ordering::Equal) => {
                        LineDirection::Horizontal(X::Left)
                    },
                    (Ordering::Greater, Ordering::Greater) => {
                        LineDirection::Corner(X::Left, Y::Top)
                    },
                }
            }
        }

        impl From<Point> for Position {
            fn from(point: Point) -> Self {
                Self::new(point.horizontal.into(), point.vertical.into())
            }
        }

        impl Parse for Point {
            fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
                input.try(|i| {
                    let x = Component::parse(context, i)?;
                    let y = Component::parse(context, i)?;

                    Ok(Self::new(x, y))
                })
            }
        }

        impl<S: Side> From<Component<S>> for NumberOrPercentage {
            fn from(component: Component<S>) -> Self {
                match component {
                    Component::Center => NumberOrPercentage::Percentage(Percentage(0.5)),
                    Component::Number(number) => number,
                    Component::Side(side) => {
                        let p = Percentage(if side.is_start() { 0. } else { 1. });
                        NumberOrPercentage::Percentage(p)
                    },
                }
            }
        }

        impl<S: Side> From<Component<S>> for PositionComponent<S> {
            fn from(component: Component<S>) -> Self {
                match component {
                    Component::Center => {
                        PositionComponent::Center
                    },
                    Component::Number(NumberOrPercentage::Number(number)) => {
                        PositionComponent::Length(Length::from_px(number.value).into())
                    },
                    Component::Number(NumberOrPercentage::Percentage(p)) => {
                        PositionComponent::Length(p.into())
                    },
                    Component::Side(side) => {
                        PositionComponent::Side(side, None)
                    },
                }
            }
        }

        impl<S: Copy + Side> Component<S> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                match (NumberOrPercentage::from(*self), NumberOrPercentage::from(*other)) {
                    (NumberOrPercentage::Percentage(a), NumberOrPercentage::Percentage(b)) => {
                        a.0.partial_cmp(&b.0)
                    },
                    (NumberOrPercentage::Number(a), NumberOrPercentage::Number(b)) => {
                        a.value.partial_cmp(&b.value)
                    },
                    (_, _) => {
                        None
                    }
                }
            }
        }

        impl<S: Parse> Parse for Component<S> {
            fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
                if let Ok(side) = input.try(|i| S::parse(context, i)) {
                    return Ok(Component::Side(side));
                }
                if let Ok(number) = input.try(|i| NumberOrPercentage::parse(context, i)) {
                    return Ok(Component::Number(number));
                }
                input.try(|i| i.expect_ident_matching("center"))?;
                Ok(Component::Center)
            }
        }

        let ident = input.expect_ident()?;
        input.expect_comma()?;

        let (kind, reverse_stops) = match_ignore_ascii_case! { &ident,
            "linear" => {
                let first = Point::parse(context, input)?;
                input.expect_comma()?;
                let second = Point::parse(context, input)?;

                let direction = LineDirection::from_points(first, second);
                let kind = GenericGradientKind::Linear(direction);

                (kind, false)
            },
            "radial" => {
                let first_point = Point::parse(context, input)?;
                input.expect_comma()?;
                let first_radius = Number::parse(context, input)?;
                input.expect_comma()?;
                let second_point = Point::parse(context, input)?;
                input.expect_comma()?;
                let second_radius = Number::parse(context, input)?;

                let (reverse_stops, point, radius) = if second_radius.value >= first_radius.value {
                    (false, second_point, second_radius)
                } else {
                    (true, first_point, first_radius)
                };

                let shape = GenericEndingShape::Circle(Circle::Radius(Length::from_px(radius.value)));
                let position = point.into();
                let kind = GenericGradientKind::Radial(shape, position);

                (kind, reverse_stops)
            },
            _ => return Err(()),
        };

        let mut items = input.try(|i| {
            i.expect_comma()?;
            i.parse_comma_separated(|i| {
                let function = i.expect_function()?;
                let (color, mut p) = i.parse_nested_block(|i| {
                    let p = match_ignore_ascii_case! { &function,
                        "color-stop" => {
                            let p = match NumberOrPercentage::parse(context, i)? {
                                NumberOrPercentage::Number(number) => number.value,
                                NumberOrPercentage::Percentage(p) => p.0,
                            };
                            i.expect_comma()?;
                            p
                        },
                        "from" => 0.,
                        "to" => 1.,
                        _ => return Err(()),
                    };
                    let color = Color::parse(context, i)?;
                    if color == Color::CurrentColor {
                        return Err(());
                    }
                    Ok((color.into(), p))
                })?;
                if reverse_stops {
                    p = 1. - p;
                }
                Ok(GenericGradientItem::ColorStop(GenericColorStop {
                    color: color,
                    position: Some(LengthOrPercentage::Percentage(Percentage(p))),
                }))
            })
        }).unwrap_or(vec![]);

        if items.is_empty() {
            items = vec![
                GenericGradientItem::ColorStop(GenericColorStop {
                    color: Color::transparent().into(),
                    position: Some(Percentage(0.).into()),
                }),
                GenericGradientItem::ColorStop(GenericColorStop {
                    color: Color::transparent().into(),
                    position: Some(Percentage(1.).into()),
                }),
            ];
        } else if items.len() == 1 {
            let first = items[0].clone();
            items.push(first);
        } else {
            items.sort_by(|a, b| {
                match (a, b) {
                    (&GenericGradientItem::ColorStop(ref a), &GenericGradientItem::ColorStop(ref b)) => {
                        match (&a.position, &b.position) {
                            (&Some(LengthOrPercentage::Percentage(a)), &Some(LengthOrPercentage::Percentage(b))) => {
                                let ordering = a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal);
                                if ordering != Ordering::Equal {
                                    return ordering;
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
                if reverse_stops {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
        }

        Ok(GenericGradient {
            kind: kind,
            items: items,
            repeating: false,
            compat_mode: CompatMode::Modern,
        })
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

impl GradientItem {
    fn parse_comma_separated(context: &ParserContext, input: &mut Parser) -> Result<Vec<Self>, ()> {
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

impl Parse for ColorStop {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        Ok(ColorStop {
            color: try!(RGBAColor::parse(context, input)),
            position: input.try(|i| LengthOrPercentage::parse(context, i)).ok(),
        })
    }
}

impl Parse for PaintWorklet {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        input.expect_function_matching("paint")?;
        input.parse_nested_block(|i| {
            let name = i.expect_ident()?;
            Ok(PaintWorklet {
                name: Atom::from(name),
            })
        })
    }
}

impl Parse for ImageRect {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        input.try(|i| i.expect_function_matching("-moz-image-rect"))?;
        input.parse_nested_block(|i| {
            let string = i.expect_url_or_string()?;
            let url = SpecifiedUrl::parse_from_string(string, context)?;
            i.expect_comma()?;
            let top = NumberOrPercentage::parse_non_negative(context, i)?;
            i.expect_comma()?;
            let right = NumberOrPercentage::parse_non_negative(context, i)?;
            i.expect_comma()?;
            let bottom = NumberOrPercentage::parse_non_negative(context, i)?;
            i.expect_comma()?;
            let left = NumberOrPercentage::parse_non_negative(context, i)?;

            Ok(ImageRect {
                url: url,
                top: top,
                right: right,
                bottom: bottom,
                left: left,
            })
        })
    }
}
