/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use Atom;
use cssparser::{Parser, Token, BasicParseError};
use custom_properties::SpecifiedValue;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseError;
#[cfg(feature = "servo")]
use servo_url::ServoUrl;
use std::cmp::Ordering;
use std::f32::consts::PI;
use std::fmt;
use style_traits::{ToCss, ParseError, StyleParseError};
use values::{Either, None_};
#[cfg(feature = "gecko")]
use values::computed::{Context, Position as ComputedPosition, ToComputedValue};
use values::generics::image::{Circle, CompatMode, Ellipse, ColorStop as GenericColorStop};
use values::generics::image::{EndingShape as GenericEndingShape, Gradient as GenericGradient};
use values::generics::image::{GradientItem as GenericGradientItem, GradientKind as GenericGradientKind};
use values::generics::image::{Image as GenericImage, LineDirection as GenericsLineDirection};
use values::generics::image::{MozImageRect as GenericMozImageRect, ShapeExtent};
use values::generics::image::PaintWorklet;
use values::generics::position::Position as GenericPosition;
use values::specified::{Angle, Color, Length, LengthOrPercentage};
use values::specified::{Number, NumberOrPercentage, Percentage, RGBAColor};
use values::specified::position::{LegacyPosition, Position, PositionComponent, Side, X, Y};
use values::specified::url::SpecifiedUrl;

/// A specified image layer.
pub type ImageLayer = Either<None_, Image>;

/// Specified values for an image according to CSS-IMAGES.
/// https://drafts.csswg.org/css-images/#image-values
pub type Image = GenericImage<Gradient, MozImageRect>;

/// Specified values for a CSS gradient.
/// https://drafts.csswg.org/css-images/#gradients
#[cfg(not(feature = "gecko"))]
pub type Gradient = GenericGradient<
    LineDirection,
    Length,
    LengthOrPercentage,
    Position,
    RGBAColor,
    Angle,
>;

/// Specified values for a CSS gradient.
/// https://drafts.csswg.org/css-images/#gradients
#[cfg(feature = "gecko")]
pub type Gradient = GenericGradient<
    LineDirection,
    Length,
    LengthOrPercentage,
    GradientPosition,
    RGBAColor,
    Angle,
>;

/// A specified gradient kind.
#[cfg(not(feature = "gecko"))]
pub type GradientKind = GenericGradientKind<
    LineDirection,
    Length,
    LengthOrPercentage,
    Position,
    Angle,
>;

/// A specified gradient kind.
#[cfg(feature = "gecko")]
pub type GradientKind = GenericGradientKind<
    LineDirection,
    Length,
    LengthOrPercentage,
    GradientPosition,
    Angle,
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
    /// A Position and an Angle for legacy `-moz-` prefixed gradient.
    /// `-moz-` prefixed linear gradient can contain both a position and an angle but it
    /// uses legacy syntax for position. That means we can't specify both keyword and
    /// length for each horizontal/vertical components.
    #[cfg(feature = "gecko")]
    MozPosition(Option<LegacyPosition>, Option<Angle>),
}

/// A binary enum to hold either Position or LegacyPosition.
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToCss)]
#[cfg(feature = "gecko")]
pub enum GradientPosition {
    /// 1, 2, 3, 4-valued <position>.
    Modern(Position),
    /// 1, 2-valued <position>.
    Legacy(LegacyPosition),
}

/// A specified ending shape.
pub type EndingShape = GenericEndingShape<Length, LengthOrPercentage>;

/// A specified gradient item.
pub type GradientItem = GenericGradientItem<RGBAColor, LengthOrPercentage>;

/// A computed color stop.
pub type ColorStop = GenericColorStop<RGBAColor, LengthOrPercentage>;

/// Specified values for `moz-image-rect`
/// -moz-image-rect(<uri>, top, right, bottom, left);
pub type MozImageRect = GenericMozImageRect<NumberOrPercentage>;

impl Parse for Image {
    #[cfg_attr(not(feature = "gecko"), allow(unused_mut))]
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Image, ParseError<'i>> {
        if let Ok(mut url) = input.try(|input| SpecifiedUrl::parse(context, input)) {
            #[cfg(feature = "gecko")]
            {
                url.build_image_value();
            }
            return Ok(GenericImage::Url(url));
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
        if let Ok(mut image_rect) = input.try(|input| MozImageRect::parse(context, input)) {
            #[cfg(feature = "gecko")]
            {
                image_rect.url.build_image_value();
            }
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
    fn parse_element<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Atom, ParseError<'i>> {
        input.try(|i| i.expect_function_matching("-moz-element"))?;
        input.parse_nested_block(|i| {
            match *i.next()? {
                Token::IDHash(ref id) => Ok(Atom::from(id.as_ref())),
                ref t => Err(BasicParseError::UnexpectedToken(t.clone()).into()),
            }
        })
    }
}

impl Parse for Gradient {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        enum Shape {
            Linear,
            Radial,
        }

        // FIXME: remove clone() when lifetimes are non-lexical
        let func = input.expect_function()?.clone();
        let result = match_ignore_ascii_case! { &func,
            "linear-gradient" => {
                Some((Shape::Linear, false, CompatMode::Modern))
            },
            "-webkit-linear-gradient" => {
                Some((Shape::Linear, false, CompatMode::WebKit))
            },
            #[cfg(feature = "gecko")]
            "-moz-linear-gradient" => {
                Some((Shape::Linear, false, CompatMode::Moz))
            },
            "repeating-linear-gradient" => {
                Some((Shape::Linear, true, CompatMode::Modern))
            },
            "-webkit-repeating-linear-gradient" => {
                Some((Shape::Linear, true, CompatMode::WebKit))
            },
            #[cfg(feature = "gecko")]
            "-moz-repeating-linear-gradient" => {
                Some((Shape::Linear, true, CompatMode::Moz))
            },
            "radial-gradient" => {
                Some((Shape::Radial, false, CompatMode::Modern))
            },
            "-webkit-radial-gradient" => {
                Some((Shape::Radial, false, CompatMode::WebKit))
            }
            #[cfg(feature = "gecko")]
            "-moz-radial-gradient" => {
                Some((Shape::Radial, false, CompatMode::Moz))
            },
            "repeating-radial-gradient" => {
                Some((Shape::Radial, true, CompatMode::Modern))
            },
            "-webkit-repeating-radial-gradient" => {
                Some((Shape::Radial, true, CompatMode::WebKit))
            },
            #[cfg(feature = "gecko")]
            "-moz-repeating-radial-gradient" => {
                Some((Shape::Radial, true, CompatMode::Moz))
            },
            "-webkit-gradient" => {
                return input.parse_nested_block(|i| Self::parse_webkit_gradient_argument(context, i));
            },
            _ => None,
        };

        let (shape, repeating, mut compat_mode) = match result {
            Some(result) => result,
            None => return Err(StyleParseError::UnexpectedFunction(func.clone()).into()),
        };

        let (kind, items) = input.parse_nested_block(|i| {
            let shape = match shape {
                Shape::Linear => GradientKind::parse_linear(context, i, &mut compat_mode)?,
                Shape::Radial => GradientKind::parse_radial(context, i, &mut compat_mode)?,
            };
            let items = GradientItem::parse_comma_separated(context, i)?;
            Ok((shape, items))
        })?;

        if items.len() < 2 {
            return Err(StyleParseError::UnspecifiedError.into());
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
    fn parse_webkit_gradient_argument<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                              -> Result<Self, ParseError<'i>> {
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
            fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                             -> Result<Self, ParseError<'i>> {
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
                    Component::Center => NumberOrPercentage::Percentage(Percentage::new(0.5)),
                    Component::Number(number) => number,
                    Component::Side(side) => {
                        let p = if side.is_start() { Percentage::zero() } else { Percentage::hundred() };
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
                        a.get().partial_cmp(&b.get())
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
            fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                             -> Result<Self, ParseError<'i>> {
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

        let ident = input.expect_ident_cloned()?;
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
                let position: Position = point.into();

                #[cfg(feature = "gecko")]
                {
                    let kind = GenericGradientKind::Radial(shape, GradientPosition::Modern(position), None);
                    (kind, reverse_stops)
                }

                #[cfg(not(feature = "gecko"))]
                {
                    let kind = GenericGradientKind::Radial(shape, position, None);
                    (kind, reverse_stops)
                }
            },
            _ => return Err(SelectorParseError::UnexpectedIdent(ident.clone()).into()),
        };

        let mut items = input.try(|i| {
            i.expect_comma()?;
            i.parse_comma_separated(|i| {
                let function = i.expect_function()?.clone();
                let (color, mut p) = i.parse_nested_block(|i| {
                    let p = match_ignore_ascii_case! { &function,
                        "color-stop" => {
                            let p = match NumberOrPercentage::parse(context, i)? {
                                NumberOrPercentage::Number(number) => Percentage::new(number.value),
                                NumberOrPercentage::Percentage(p) => p,
                            };
                            i.expect_comma()?;
                            p
                        },
                        "from" => Percentage::zero(),
                        "to" => Percentage::hundred(),
                        _ => return Err(StyleParseError::UnexpectedFunction(function.clone()).into()),
                    };
                    let color = Color::parse(context, i)?;
                    if color == Color::CurrentColor {
                        return Err(StyleParseError::UnspecifiedError.into());
                    }
                    Ok((color.into(), p))
                })?;
                if reverse_stops {
                    p.reverse();
                }
                Ok(GenericGradientItem::ColorStop(GenericColorStop {
                    color: color,
                    position: Some(p.into()),
                }))
            })
        }).unwrap_or(vec![]);

        if items.is_empty() {
            items = vec![
                GenericGradientItem::ColorStop(GenericColorStop {
                    color: Color::transparent().into(),
                    position: Some(Percentage::zero().into()),
                }),
                GenericGradientItem::ColorStop(GenericColorStop {
                    color: Color::transparent().into(),
                    position: Some(Percentage::hundred().into()),
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
                                return a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal);
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
    /// Parses a linear gradient.
    /// CompatMode can change during `-moz-` prefixed gradient parsing if it come across a `to` keyword.
    fn parse_linear<'i, 't>(context: &ParserContext,
                            input: &mut Parser<'i, 't>,
                            compat_mode: &mut CompatMode)
                            -> Result<Self, ParseError<'i>> {
        let direction = if let Ok(d) = input.try(|i| LineDirection::parse(context, i, compat_mode)) {
            input.expect_comma()?;
            d
        } else {
            LineDirection::Vertical(Y::Bottom)
        };
        Ok(GenericGradientKind::Linear(direction))
    }

    fn parse_radial<'i, 't>(context: &ParserContext,
                            input: &mut Parser<'i, 't>,
                            compat_mode: &mut CompatMode)
                            -> Result<Self, ParseError<'i>> {
        let (shape, position, angle, moz_position) = match *compat_mode {
            CompatMode::Modern => {
                let shape = input.try(|i| EndingShape::parse(context, i, *compat_mode));
                let position = input.try(|i| {
                    i.expect_ident_matching("at")?;
                    Position::parse(context, i)
                });
                (shape, position.ok(), None, None)
            },
            CompatMode::WebKit => {
                let position = input.try(|i| Position::parse(context, i));
                let shape = input.try(|i| {
                    if position.is_ok() {
                        i.expect_comma()?;
                    }
                    EndingShape::parse(context, i, *compat_mode)
                });
                (shape, position.ok(), None, None)
            },
            // The syntax of `-moz-` prefixed radial gradient is:
            // -moz-radial-gradient(
            //   [ [ <position> || <angle> ]?  [ ellipse | [ <length> | <percentage> ]{2} ] , |
            //     [ <position> || <angle> ]?  [ [ circle | ellipse ] | <extent-keyword> ] , |
            //   ]?
            //   <color-stop> [ , <color-stop> ]+
            // )
            // where <extent-keyword> = closest-corner | closest-side | farthest-corner | farthest-side |
            //                          cover | contain
            // and <color-stop>     = <color> [ <percentage> | <length> ]?
            CompatMode::Moz => {
                let mut position = input.try(|i| LegacyPosition::parse(context, i));
                let angle = input.try(|i| Angle::parse(context, i)).ok();
                if position.is_err() {
                    position = input.try(|i| LegacyPosition::parse(context, i));
                }

                let shape = input.try(|i| {
                    if position.is_ok() || angle.is_some() {
                        i.expect_comma()?;
                    }
                    EndingShape::parse(context, i, *compat_mode)
                });

                (shape, None, angle, position.ok())
            }
        };

        if shape.is_ok() || position.is_some() || angle.is_some() || moz_position.is_some() {
            input.expect_comma()?;
        }

        let shape = shape.unwrap_or({
            GenericEndingShape::Ellipse(Ellipse::Extent(ShapeExtent::FarthestCorner))
        });

        #[cfg(feature = "gecko")]
        {
            if *compat_mode == CompatMode::Moz {
                // If this form can be represented in Modern mode, then convert the compat_mode to Modern.
                if angle.is_none() {
                    *compat_mode = CompatMode::Modern;
                }
                let position = moz_position.unwrap_or(LegacyPosition::center());
                return Ok(GenericGradientKind::Radial(shape, GradientPosition::Legacy(position), angle));
            }
        }

        let position = position.unwrap_or(Position::center());
        #[cfg(feature = "gecko")]
        {
            return Ok(GenericGradientKind::Radial(shape, GradientPosition::Modern(position), angle));
        }
        #[cfg(not(feature = "gecko"))]
        {
            return Ok(GenericGradientKind::Radial(shape, position, angle));
        }
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
            },
            #[cfg(feature = "gecko")]
            LineDirection::MozPosition(ref position, ref angle) => {
                let mut need_space = false;
                if let Some(ref position) = *position {
                    position.to_css(dest)?;
                    need_space = true;
                }
                if let Some(ref angle) = *angle {
                    if need_space {
                        dest.write_str(" ")?;
                    }
                    angle.to_css(dest)?;
                }
                Ok(())
            },
        }
    }
}

impl LineDirection {
    fn parse<'i, 't>(context: &ParserContext,
                     input: &mut Parser<'i, 't>,
                     compat_mode: &mut CompatMode)
                     -> Result<Self, ParseError<'i>> {
        let mut _angle = if *compat_mode == CompatMode::Moz {
            input.try(|i| Angle::parse(context, i)).ok()
        } else {
            if let Ok(angle) = input.try(|i| Angle::parse_with_unitless(context, i)) {
                return Ok(LineDirection::Angle(angle));
            }
            None
        };

        input.try(|i| {
            let to_ident = i.try(|i| i.expect_ident_matching("to"));
            match *compat_mode {
                /// `to` keyword is mandatory in modern syntax.
                CompatMode::Modern => to_ident?,
                // Fall back to Modern compatibility mode in case there is a `to` keyword.
                // According to Gecko, `-moz-linear-gradient(to ...)` should serialize like
                // `linear-gradient(to ...)`.
                CompatMode::Moz if to_ident.is_ok() => *compat_mode = CompatMode::Modern,
                /// There is no `to` keyword in webkit prefixed syntax. If it's consumed,
                /// parsing should throw an error.
                CompatMode::WebKit if to_ident.is_ok() => {
                    return Err(SelectorParseError::UnexpectedIdent("to".into()).into())
                },
                _ => {},
            }

            #[cfg(feature = "gecko")]
            {
                // `-moz-` prefixed linear gradient can be both Angle and Position.
                if *compat_mode == CompatMode::Moz {
                    let position = i.try(|i| LegacyPosition::parse(context, i)).ok();
                    if _angle.is_none() {
                        _angle = i.try(|i| Angle::parse(context, i)).ok();
                    };

                    if _angle.is_none() && position.is_none() {
                        return Err(StyleParseError::UnspecifiedError.into());
                    }
                    return Ok(LineDirection::MozPosition(position, _angle));
                }
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

#[cfg(feature = "gecko")]
impl ToComputedValue for GradientPosition {
    type ComputedValue = ComputedPosition;

    fn to_computed_value(&self, context: &Context) -> ComputedPosition {
        match *self {
            GradientPosition::Modern(ref pos) => pos.to_computed_value(context),
            GradientPosition::Legacy(ref pos) => pos.to_computed_value(context),
        }
    }

    fn from_computed_value(computed: &ComputedPosition) -> Self {
        GradientPosition::Modern(ToComputedValue::from_computed_value(computed))
    }
}

impl EndingShape {
    fn parse<'i, 't>(context: &ParserContext,
                     input: &mut Parser<'i, 't>,
                     compat_mode: CompatMode)
                     -> Result<Self, ParseError<'i>> {
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
                let pair: Result<_, ParseError> = input.try(|i| {
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
        // -moz- prefixed radial gradient doesn't allow EndingShape's Length or LengthOrPercentage
        // to come before shape keyword. Otherwise it conflicts with <position>.
        if compat_mode != CompatMode::Moz {
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
    fn parse_with_compat_mode<'i, 't>(input: &mut Parser<'i, 't>,
                                      compat_mode: CompatMode)
                                      -> Result<Self, ParseError<'i>> {
        match Self::parse(input)? {
            ShapeExtent::Contain | ShapeExtent::Cover if compat_mode == CompatMode::Modern => {
                Err(StyleParseError::UnspecifiedError.into())
            },
            ShapeExtent::Contain => Ok(ShapeExtent::ClosestSide),
            ShapeExtent::Cover => Ok(ShapeExtent::FarthestCorner),
            keyword => Ok(keyword),
        }
    }
}

impl GradientItem {
    fn parse_comma_separated<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                     -> Result<Vec<Self>, ParseError<'i>> {
        let mut seen_stop = false;
        let items = input.parse_comma_separated(|input| {
            if seen_stop {
                if let Ok(hint) = input.try(|i| LengthOrPercentage::parse(context, i)) {
                    seen_stop = false;
                    return Ok(GenericGradientItem::InterpolationHint(hint));
                }
            }
            seen_stop = true;
            ColorStop::parse(context, input).map(GenericGradientItem::ColorStop)
        })?;
        if !seen_stop || items.len() < 2 {
            return Err(StyleParseError::UnspecifiedError.into());
        }
        Ok(items)
    }
}

impl Parse for ColorStop {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<Self, ParseError<'i>> {
        Ok(ColorStop {
            color: RGBAColor::parse(context, input)?,
            position: input.try(|i| LengthOrPercentage::parse(context, i)).ok(),
        })
    }
}

impl Parse for PaintWorklet {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("paint")?;
        input.parse_nested_block(|input| {
            let name = Atom::from(&**input.expect_ident()?);
            let arguments = input.try(|input| {
                input.expect_comma()?;
                input.parse_comma_separated(|input| Ok(*SpecifiedValue::parse(context, input)?))
            }).unwrap_or(vec![]);
            Ok(PaintWorklet {
                name: name,
                arguments: arguments,
            })
        })
    }
}

impl Parse for MozImageRect {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        input.try(|i| i.expect_function_matching("-moz-image-rect"))?;
        input.parse_nested_block(|i| {
            let string = i.expect_url_or_string()?;
            let url = SpecifiedUrl::parse_from_string(string.as_ref().to_owned(), context)?;
            i.expect_comma()?;
            let top = NumberOrPercentage::parse_non_negative(context, i)?;
            i.expect_comma()?;
            let right = NumberOrPercentage::parse_non_negative(context, i)?;
            i.expect_comma()?;
            let bottom = NumberOrPercentage::parse_non_negative(context, i)?;
            i.expect_comma()?;
            let left = NumberOrPercentage::parse_non_negative(context, i)?;

            Ok(MozImageRect {
                url: url,
                top: top,
                right: right,
                bottom: bottom,
                left: left,
            })
        })
    }
}
