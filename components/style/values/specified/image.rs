/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use crate::custom_properties::SpecifiedValue;
use crate::parser::{Parse, ParserContext};
use crate::values::generics::image::PaintWorklet;
use crate::values::generics::image::{
    self as generic, Circle, Ellipse, GradientCompatMode, ShapeExtent,
};
use crate::values::generics::position::Position as GenericPosition;
use crate::values::generics::NonNegative;
use crate::values::specified::position::{HorizontalPositionKeyword, VerticalPositionKeyword};
use crate::values::specified::position::{Position, PositionComponent, Side};
use crate::values::specified::url::SpecifiedImageUrl;
use crate::values::specified::{
    Angle, AngleOrPercentage, Color, Length, LengthPercentage, NonNegativeLength, NonNegativeLengthPercentage,
};
use crate::values::specified::{Number, NumberOrPercentage, Percentage};
use crate::Atom;
use cssparser::{Delimiter, Parser, Token};
use selectors::parser::SelectorParseErrorKind;
#[cfg(feature = "servo")]
use servo_url::ServoUrl;
use std::cmp::Ordering;
use std::fmt::{self, Write};
use style_traits::{CssType, CssWriter, KeywordsCollectFn, ParseError};
use style_traits::{SpecifiedValueInfo, StyleParseErrorKind, ToCss};

/// Specified values for an image according to CSS-IMAGES.
/// <https://drafts.csswg.org/css-images/#image-values>
pub type Image = generic::Image<Gradient, MozImageRect, SpecifiedImageUrl>;

/// Specified values for a CSS gradient.
/// <https://drafts.csswg.org/css-images/#gradients>
pub type Gradient = generic::Gradient<
    LineDirection,
    LengthPercentage,
    NonNegativeLength,
    NonNegativeLengthPercentage,
    Position,
    Angle,
    AngleOrPercentage,
    Color,
>;

type LengthPercentageItemList = crate::OwnedSlice<generic::GradientItem<Color, LengthPercentage>>;

impl SpecifiedValueInfo for Gradient {
    const SUPPORTED_TYPES: u8 = CssType::GRADIENT;

    fn collect_completion_keywords(f: KeywordsCollectFn) {
        // This list here should keep sync with that in Gradient::parse.
        f(&[
            "linear-gradient",
            "-webkit-linear-gradient",
            "-moz-linear-gradient",
            "repeating-linear-gradient",
            "-webkit-repeating-linear-gradient",
            "-moz-repeating-linear-gradient",
            "radial-gradient",
            "-webkit-radial-gradient",
            "-moz-radial-gradient",
            "repeating-radial-gradient",
            "-webkit-repeating-radial-gradient",
            "-moz-repeating-radial-gradient",
            "-webkit-gradient",
        ]);

        if static_prefs::pref!("layout.css.conic-gradient.enabled") {
            f(&[
                "conic-gradient",
                "repeating-conic-gradient",
            ]);
        }
    }
}

/// A specified gradient line direction.
///
/// FIXME(emilio): This should be generic over Angle.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub enum LineDirection {
    /// An angular direction.
    Angle(Angle),
    /// A horizontal direction.
    Horizontal(HorizontalPositionKeyword),
    /// A vertical direction.
    Vertical(VerticalPositionKeyword),
    /// A direction towards a corner of a box.
    Corner(HorizontalPositionKeyword, VerticalPositionKeyword),
}

/// A specified ending shape.
pub type EndingShape = generic::EndingShape<NonNegativeLength, NonNegativeLengthPercentage>;

/// Specified values for `moz-image-rect`
/// -moz-image-rect(<uri>, top, right, bottom, left);
#[cfg(all(feature = "gecko", not(feature = "cbindgen")))]
pub type MozImageRect = generic::GenericMozImageRect<NumberOrPercentage, SpecifiedImageUrl>;

#[cfg(not(feature = "gecko"))]
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
/// Empty enum on non-Gecko
pub enum MozImageRect {}

impl Parse for Image {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Image, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(generic::Image::None);
        }
        if let Ok(url) = input.try(|input| SpecifiedImageUrl::parse(context, input)) {
            return Ok(generic::Image::Url(url));
        }
        if let Ok(gradient) = input.try(|i| Gradient::parse(context, i)) {
            return Ok(generic::Image::Gradient(Box::new(gradient)));
        }
        #[cfg(feature = "servo-layout-2013")]
        {
            if let Ok(paint_worklet) = input.try(|i| PaintWorklet::parse(context, i)) {
                return Ok(generic::Image::PaintWorklet(paint_worklet));
            }
        }
        #[cfg(feature = "gecko")]
        {
            if let Ok(image_rect) = input.try(|input| MozImageRect::parse(context, input)) {
                return Ok(generic::Image::Rect(Box::new(image_rect)));
            }
            Ok(generic::Image::Element(Image::parse_element(input)?))
        }
        #[cfg(not(feature = "gecko"))]
        Err(input.new_error_for_next_token())
    }
}

impl Image {
    /// Creates an already specified image value from an already resolved URL
    /// for insertion in the cascade.
    #[cfg(feature = "servo")]
    pub fn for_cascade(url: ServoUrl) -> Self {
        use crate::values::CssUrl;
        generic::Image::Url(CssUrl::for_cascade(url))
    }

    /// Parses a `-moz-element(# <element-id>)`.
    #[cfg(feature = "gecko")]
    fn parse_element<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Atom, ParseError<'i>> {
        input.try(|i| i.expect_function_matching("-moz-element"))?;
        let location = input.current_source_location();
        input.parse_nested_block(|i| match *i.next()? {
            Token::IDHash(ref id) => Ok(Atom::from(id.as_ref())),
            ref t => Err(location.new_unexpected_token_error(t.clone())),
        })
    }

    /// Provides an alternate method for parsing that associates the URL with
    /// anonymous CORS headers.
    ///
    /// FIXME(emilio): It'd be nicer for this to pass a `CorsMode` parameter to
    /// a shared function instead.
    pub fn parse_with_cors_anonymous<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Image, ParseError<'i>> {
        if let Ok(url) =
            input.try(|input| SpecifiedImageUrl::parse_with_cors_anonymous(context, input))
        {
            return Ok(generic::Image::Url(url));
        }
        Self::parse(context, input)
    }
}

impl Parse for Gradient {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        enum Shape {
            Linear,
            Radial,
            Conic,
        }

        let func = input.expect_function()?;
        let (shape, repeating, compat_mode) = match_ignore_ascii_case! { &func,
            "linear-gradient" => {
                (Shape::Linear, false, GradientCompatMode::Modern)
            },
            "-webkit-linear-gradient" => {
                (Shape::Linear, false, GradientCompatMode::WebKit)
            },
            #[cfg(feature = "gecko")]
            "-moz-linear-gradient" => {
                (Shape::Linear, false, GradientCompatMode::Moz)
            },
            "repeating-linear-gradient" => {
                (Shape::Linear, true, GradientCompatMode::Modern)
            },
            "-webkit-repeating-linear-gradient" => {
                (Shape::Linear, true, GradientCompatMode::WebKit)
            },
            #[cfg(feature = "gecko")]
            "-moz-repeating-linear-gradient" => {
                (Shape::Linear, true, GradientCompatMode::Moz)
            },
            "radial-gradient" => {
                (Shape::Radial, false, GradientCompatMode::Modern)
            },
            "-webkit-radial-gradient" => {
                (Shape::Radial, false, GradientCompatMode::WebKit)
            },
            #[cfg(feature = "gecko")]
            "-moz-radial-gradient" => {
                (Shape::Radial, false, GradientCompatMode::Moz)
            },
            "repeating-radial-gradient" => {
                (Shape::Radial, true, GradientCompatMode::Modern)
            },
            "-webkit-repeating-radial-gradient" => {
                (Shape::Radial, true, GradientCompatMode::WebKit)
            },
            #[cfg(feature = "gecko")]
            "-moz-repeating-radial-gradient" => {
                (Shape::Radial, true, GradientCompatMode::Moz)
            },
            "conic-gradient" if static_prefs::pref!("layout.css.conic-gradient.enabled") => {
                (Shape::Conic, false, GradientCompatMode::Modern)
            },
            "repeating-conic-gradient" if static_prefs::pref!("layout.css.conic-gradient.enabled") => {
                (Shape::Conic, true, GradientCompatMode::Modern)
            },
            "-webkit-gradient" => {
                return input.parse_nested_block(|i| {
                    Self::parse_webkit_gradient_argument(context, i)
                });
            },
            _ => {
                let func = func.clone();
                return Err(input.new_custom_error(StyleParseErrorKind::UnexpectedFunction(func)));
            }
        };

        Ok(input.parse_nested_block(|i| {
            Ok(match shape {
                Shape::Linear => Self::parse_linear(context, i, repeating, compat_mode)?,
                Shape::Radial => Self::parse_radial(context, i, repeating, compat_mode)?,
                Shape::Conic => Self::parse_conic(context, i, repeating)?,
            })
        })?)
    }
}

impl Gradient {
    fn parse_webkit_gradient_argument<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        use crate::values::specified::position::{
            HorizontalPositionKeyword as X, VerticalPositionKeyword as Y,
        };
        type Point = GenericPosition<Component<X>, Component<Y>>;

        #[derive(Clone, Copy, Parse)]
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
                    (Ordering::Less, Ordering::Less) => LineDirection::Corner(X::Right, Y::Bottom),
                    (Ordering::Less, Ordering::Equal) => LineDirection::Horizontal(X::Right),
                    (Ordering::Less, Ordering::Greater) => LineDirection::Corner(X::Right, Y::Top),
                    (Ordering::Equal, Ordering::Greater) => LineDirection::Vertical(Y::Top),
                    (Ordering::Equal, Ordering::Equal) | (Ordering::Equal, Ordering::Less) => {
                        LineDirection::Vertical(Y::Bottom)
                    },
                    (Ordering::Greater, Ordering::Less) => {
                        LineDirection::Corner(X::Left, Y::Bottom)
                    },
                    (Ordering::Greater, Ordering::Equal) => LineDirection::Horizontal(X::Left),
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
            fn parse<'i, 't>(
                context: &ParserContext,
                input: &mut Parser<'i, 't>,
            ) -> Result<Self, ParseError<'i>> {
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
                        let p = if side.is_start() {
                            Percentage::zero()
                        } else {
                            Percentage::hundred()
                        };
                        NumberOrPercentage::Percentage(p)
                    },
                }
            }
        }

        impl<S: Side> From<Component<S>> for PositionComponent<S> {
            fn from(component: Component<S>) -> Self {
                match component {
                    Component::Center => PositionComponent::Center,
                    Component::Number(NumberOrPercentage::Number(number)) => {
                        PositionComponent::Length(Length::from_px(number.value).into())
                    },
                    Component::Number(NumberOrPercentage::Percentage(p)) => {
                        PositionComponent::Length(p.into())
                    },
                    Component::Side(side) => PositionComponent::Side(side, None),
                }
            }
        }

        impl<S: Copy + Side> Component<S> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                match (
                    NumberOrPercentage::from(*self),
                    NumberOrPercentage::from(*other),
                ) {
                    (NumberOrPercentage::Percentage(a), NumberOrPercentage::Percentage(b)) => {
                        a.get().partial_cmp(&b.get())
                    },
                    (NumberOrPercentage::Number(a), NumberOrPercentage::Number(b)) => {
                        a.value.partial_cmp(&b.value)
                    },
                    (_, _) => None,
                }
            }
        }

        let ident = input.expect_ident_cloned()?;
        input.expect_comma()?;

        Ok(match_ignore_ascii_case! { &ident,
            "linear" => {
                let first = Point::parse(context, input)?;
                input.expect_comma()?;
                let second = Point::parse(context, input)?;

                let direction = LineDirection::from_points(first, second);
                let items = Gradient::parse_webkit_gradient_stops(context, input, false)?;

                generic::Gradient::Linear {
                    direction,
                    items,
                    repeating: false,
                    compat_mode: GradientCompatMode::Modern,
                }
            },
            "radial" => {
                let first_point = Point::parse(context, input)?;
                input.expect_comma()?;
                let first_radius = Number::parse_non_negative(context, input)?;
                input.expect_comma()?;
                let second_point = Point::parse(context, input)?;
                input.expect_comma()?;
                let second_radius = Number::parse_non_negative(context, input)?;

                let (reverse_stops, point, radius) = if second_radius.value >= first_radius.value {
                    (false, second_point, second_radius)
                } else {
                    (true, first_point, first_radius)
                };

                let rad = Circle::Radius(NonNegative(Length::from_px(radius.value)));
                let shape = generic::EndingShape::Circle(rad);
                let position: Position = point.into();
                let items = Gradient::parse_webkit_gradient_stops(context, input, reverse_stops)?;

                generic::Gradient::Radial {
                    shape,
                    position,
                    items,
                    repeating: false,
                    compat_mode: GradientCompatMode::Modern,
                }
            },
            _ => {
                let e = SelectorParseErrorKind::UnexpectedIdent(ident.clone());
                return Err(input.new_custom_error(e));
            },
        })
    }

    fn parse_webkit_gradient_stops<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        reverse_stops: bool,
    ) -> Result<LengthPercentageItemList, ParseError<'i>> {
        let mut items = input
            .try(|i| {
                i.expect_comma()?;
                i.parse_comma_separated(|i| {
                    let function = i.expect_function()?.clone();
                    let (color, mut p) = i.parse_nested_block(|i| {
                        let p = match_ignore_ascii_case! { &function,
                            "color-stop" => {
                                let p = NumberOrPercentage::parse(context, i)?.to_percentage();
                                i.expect_comma()?;
                                p
                            },
                            "from" => Percentage::zero(),
                            "to" => Percentage::hundred(),
                            _ => {
                                return Err(i.new_custom_error(
                                    StyleParseErrorKind::UnexpectedFunction(function.clone())
                                ))
                            },
                        };
                        let color = Color::parse(context, i)?;
                        if color == Color::CurrentColor {
                            return Err(i.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                        }
                        Ok((color.into(), p))
                    })?;
                    if reverse_stops {
                        p.reverse();
                    }
                    Ok(generic::GradientItem::ComplexColorStop {
                        color,
                        position: p.into(),
                    })
                })
            })
            .unwrap_or(vec![]);

        if items.is_empty() {
            items = vec![
                generic::GradientItem::ComplexColorStop {
                    color: Color::transparent().into(),
                    position: Percentage::zero().into(),
                },
                generic::GradientItem::ComplexColorStop {
                    color: Color::transparent().into(),
                    position: Percentage::hundred().into(),
                },
            ];
        } else if items.len() == 1 {
            let first = items[0].clone();
            items.push(first);
        } else {
            items.sort_by(|a, b| {
                match (a, b) {
                    (
                        &generic::GradientItem::ComplexColorStop {
                            position: ref a_position,
                            ..
                        },
                        &generic::GradientItem::ComplexColorStop {
                            position: ref b_position,
                            ..
                        },
                    ) => match (a_position, b_position) {
                        (&LengthPercentage::Percentage(a), &LengthPercentage::Percentage(b)) => {
                            return a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal);
                        },
                        _ => {},
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
        Ok(items.into())
    }

    /// Not used for -webkit-gradient syntax and conic-gradient
    fn parse_stops<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<LengthPercentageItemList, ParseError<'i>> {
        let items = generic::GradientItem::parse_comma_separated(context, input, LengthPercentage::parse)?;
        if items.len() < 2 {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(items)
    }

    /// Parses a linear gradient.
    /// GradientCompatMode can change during `-moz-` prefixed gradient parsing if it come across a `to` keyword.
    fn parse_linear<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        repeating: bool,
        mut compat_mode: GradientCompatMode,
    ) -> Result<Self, ParseError<'i>> {
        let direction =
            if let Ok(d) = input.try(|i| LineDirection::parse(context, i, &mut compat_mode)) {
                input.expect_comma()?;
                d
            } else {
                match compat_mode {
                    GradientCompatMode::Modern => {
                        LineDirection::Vertical(VerticalPositionKeyword::Bottom)
                    },
                    _ => LineDirection::Vertical(VerticalPositionKeyword::Top),
                }
            };
        let items = Gradient::parse_stops(context, input)?;

        Ok(Gradient::Linear {
            direction,
            items,
            repeating,
            compat_mode,
        })
    }

    /// Parses a radial gradient.
    fn parse_radial<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        repeating: bool,
        compat_mode: GradientCompatMode,
    ) -> Result<Self, ParseError<'i>> {
        let (shape, position) = match compat_mode {
            GradientCompatMode::Modern => {
                let shape = input.try(|i| EndingShape::parse(context, i, compat_mode));
                let position = input.try(|i| {
                    i.expect_ident_matching("at")?;
                    Position::parse(context, i)
                });
                (shape, position.ok())
            },
            _ => {
                let position = input.try(|i| Position::parse(context, i));
                let shape = input.try(|i| {
                    if position.is_ok() {
                        i.expect_comma()?;
                    }
                    EndingShape::parse(context, i, compat_mode)
                });
                (shape, position.ok())
            },
        };

        if shape.is_ok() || position.is_some() {
            input.expect_comma()?;
        }

        let shape = shape.unwrap_or({
            generic::EndingShape::Ellipse(Ellipse::Extent(ShapeExtent::FarthestCorner))
        });

        let position = position.unwrap_or(Position::center());

        let items = Gradient::parse_stops(context, input)?;

        Ok(Gradient::Radial {
            shape,
            position,
            items,
            repeating,
            compat_mode,
        })
    }
    fn parse_conic<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        repeating: bool,
    ) -> Result<Self, ParseError<'i>> {
        let angle = input.try(|i| {
            i.expect_ident_matching("from")?;
            // Spec allows unitless zero start angles
            // https://drafts.csswg.org/css-images-4/#valdef-conic-gradient-angle
            Angle::parse_with_unitless(context, i)
        });
        let position = input.try(|i| {
            i.expect_ident_matching("at")?;
            Position::parse(context, i)
        });
        if angle.is_ok() || position.is_ok() {
            input.expect_comma()?;
        }

        let angle = angle.unwrap_or(Angle::zero());
        let position = position.unwrap_or(Position::center());
        let items = generic::GradientItem::parse_comma_separated(context, input, AngleOrPercentage::parse_with_unitless)?;

        if items.len() < 2 {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(Gradient::Conic {
            angle,
            position,
            items,
            repeating,
        })
    }
}

impl generic::LineDirection for LineDirection {
    fn points_downwards(&self, compat_mode: GradientCompatMode) -> bool {
        match *self {
            LineDirection::Angle(ref angle) => angle.degrees() == 180.0,
            LineDirection::Vertical(VerticalPositionKeyword::Bottom) => {
                compat_mode == GradientCompatMode::Modern
            },
            LineDirection::Vertical(VerticalPositionKeyword::Top) => {
                compat_mode != GradientCompatMode::Modern
            },
            _ => false,
        }
    }

    fn to_css<W>(&self, dest: &mut CssWriter<W>, compat_mode: GradientCompatMode) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            LineDirection::Angle(angle) => angle.to_css(dest),
            LineDirection::Horizontal(x) => {
                if compat_mode == GradientCompatMode::Modern {
                    dest.write_str("to ")?;
                }
                x.to_css(dest)
            },
            LineDirection::Vertical(y) => {
                if compat_mode == GradientCompatMode::Modern {
                    dest.write_str("to ")?;
                }
                y.to_css(dest)
            },
            LineDirection::Corner(x, y) => {
                if compat_mode == GradientCompatMode::Modern {
                    dest.write_str("to ")?;
                }
                x.to_css(dest)?;
                dest.write_str(" ")?;
                y.to_css(dest)
            },
        }
    }
}

impl LineDirection {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        compat_mode: &mut GradientCompatMode,
    ) -> Result<Self, ParseError<'i>> {
        // Gradients allow unitless zero angles as an exception, see:
        // https://github.com/w3c/csswg-drafts/issues/1162
        if let Ok(angle) = input.try(|i| Angle::parse_with_unitless(context, i)) {
            return Ok(LineDirection::Angle(angle));
        }

        input.try(|i| {
            let to_ident = i.try(|i| i.expect_ident_matching("to"));
            match *compat_mode {
                // `to` keyword is mandatory in modern syntax.
                GradientCompatMode::Modern => to_ident?,
                // Fall back to Modern compatibility mode in case there is a `to` keyword.
                // According to Gecko, `-moz-linear-gradient(to ...)` should serialize like
                // `linear-gradient(to ...)`.
                GradientCompatMode::Moz if to_ident.is_ok() => {
                    *compat_mode = GradientCompatMode::Modern
                },
                // There is no `to` keyword in webkit prefixed syntax. If it's consumed,
                // parsing should throw an error.
                GradientCompatMode::WebKit if to_ident.is_ok() => {
                    return Err(
                        i.new_custom_error(SelectorParseErrorKind::UnexpectedIdent("to".into()))
                    );
                },
                _ => {},
            }

            if let Ok(x) = i.try(HorizontalPositionKeyword::parse) {
                if let Ok(y) = i.try(VerticalPositionKeyword::parse) {
                    return Ok(LineDirection::Corner(x, y));
                }
                return Ok(LineDirection::Horizontal(x));
            }
            let y = VerticalPositionKeyword::parse(i)?;
            if let Ok(x) = i.try(HorizontalPositionKeyword::parse) {
                return Ok(LineDirection::Corner(x, y));
            }
            Ok(LineDirection::Vertical(y))
        })
    }
}

impl EndingShape {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        compat_mode: GradientCompatMode,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(extent) = input.try(|i| ShapeExtent::parse_with_compat_mode(i, compat_mode)) {
            if input.try(|i| i.expect_ident_matching("circle")).is_ok() {
                return Ok(generic::EndingShape::Circle(Circle::Extent(extent)));
            }
            let _ = input.try(|i| i.expect_ident_matching("ellipse"));
            return Ok(generic::EndingShape::Ellipse(Ellipse::Extent(extent)));
        }
        if input.try(|i| i.expect_ident_matching("circle")).is_ok() {
            if let Ok(extent) = input.try(|i| ShapeExtent::parse_with_compat_mode(i, compat_mode)) {
                return Ok(generic::EndingShape::Circle(Circle::Extent(extent)));
            }
            if compat_mode == GradientCompatMode::Modern {
                if let Ok(length) = input.try(|i| NonNegativeLength::parse(context, i)) {
                    return Ok(generic::EndingShape::Circle(Circle::Radius(length)));
                }
            }
            return Ok(generic::EndingShape::Circle(Circle::Extent(
                ShapeExtent::FarthestCorner,
            )));
        }
        if input.try(|i| i.expect_ident_matching("ellipse")).is_ok() {
            if let Ok(extent) = input.try(|i| ShapeExtent::parse_with_compat_mode(i, compat_mode)) {
                return Ok(generic::EndingShape::Ellipse(Ellipse::Extent(extent)));
            }
            if compat_mode == GradientCompatMode::Modern {
                let pair: Result<_, ParseError> = input.try(|i| {
                    let x = NonNegativeLengthPercentage::parse(context, i)?;
                    let y = NonNegativeLengthPercentage::parse(context, i)?;
                    Ok((x, y))
                });
                if let Ok((x, y)) = pair {
                    return Ok(generic::EndingShape::Ellipse(Ellipse::Radii(x, y)));
                }
            }
            return Ok(generic::EndingShape::Ellipse(Ellipse::Extent(
                ShapeExtent::FarthestCorner,
            )));
        }
        if let Ok(length) = input.try(|i| NonNegativeLength::parse(context, i)) {
            if let Ok(y) = input.try(|i| NonNegativeLengthPercentage::parse(context, i)) {
                if compat_mode == GradientCompatMode::Modern {
                    let _ = input.try(|i| i.expect_ident_matching("ellipse"));
                }
                return Ok(generic::EndingShape::Ellipse(Ellipse::Radii(
                    NonNegative(LengthPercentage::from(length.0)),
                    y,
                )));
            }
            if compat_mode == GradientCompatMode::Modern {
                let y = input.try(|i| {
                    i.expect_ident_matching("ellipse")?;
                    NonNegativeLengthPercentage::parse(context, i)
                });
                if let Ok(y) = y {
                    return Ok(generic::EndingShape::Ellipse(Ellipse::Radii(
                        NonNegative(LengthPercentage::from(length.0)),
                        y,
                    )));
                }
                let _ = input.try(|i| i.expect_ident_matching("circle"));
            }

            return Ok(generic::EndingShape::Circle(Circle::Radius(length)));
        }
        input.try(|i| {
            let x = Percentage::parse_non_negative(context, i)?;
            let y = if let Ok(y) = i.try(|i| NonNegativeLengthPercentage::parse(context, i)) {
                if compat_mode == GradientCompatMode::Modern {
                    let _ = i.try(|i| i.expect_ident_matching("ellipse"));
                }
                y
            } else {
                if compat_mode == GradientCompatMode::Modern {
                    i.expect_ident_matching("ellipse")?;
                }
                NonNegativeLengthPercentage::parse(context, i)?
            };
            Ok(generic::EndingShape::Ellipse(Ellipse::Radii(
                NonNegative(LengthPercentage::from(x)),
                y,
            )))
        })
    }
}

impl ShapeExtent {
    fn parse_with_compat_mode<'i, 't>(
        input: &mut Parser<'i, 't>,
        compat_mode: GradientCompatMode,
    ) -> Result<Self, ParseError<'i>> {
        match Self::parse(input)? {
            ShapeExtent::Contain | ShapeExtent::Cover
                if compat_mode == GradientCompatMode::Modern =>
            {
                Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            },
            ShapeExtent::Contain => Ok(ShapeExtent::ClosestSide),
            ShapeExtent::Cover => Ok(ShapeExtent::FarthestCorner),
            keyword => Ok(keyword),
        }
    }
}

impl<T> generic::GradientItem<Color, T> {
    fn parse_comma_separated<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        parse_position: impl for<'i1, 't1> Fn(&ParserContext, &mut Parser<'i1, 't1>) -> Result<T, ParseError<'i1>> + Copy,
    ) -> Result<crate::OwnedSlice<Self>, ParseError<'i>> {
        let mut items = Vec::new();
        let mut seen_stop = false;

        loop {
            input.parse_until_before(Delimiter::Comma, |input| {
                if seen_stop {
                    if let Ok(hint) = input.try(|i| parse_position(context, i)) {
                        seen_stop = false;
                        items.push(generic::GradientItem::InterpolationHint(hint));
                        return Ok(());
                    }
                }

                let stop = generic::ColorStop::parse(context, input, parse_position)?;

                if let Ok(multi_position) = input.try(|i| parse_position(context, i)) {
                    let stop_color = stop.color.clone();
                    items.push(stop.into_item());
                    items.push(
                        generic::ColorStop {
                            color: stop_color,
                            position: Some(multi_position),
                        }
                        .into_item(),
                    );
                } else {
                    items.push(stop.into_item());
                }

                seen_stop = true;
                Ok(())
            })?;

            match input.next() {
                Err(_) => break,
                Ok(&Token::Comma) => continue,
                Ok(_) => unreachable!(),
            }
        }

        if !seen_stop || items.len() < 2 {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(items.into())
    }
}

impl<T> generic::ColorStop<Color, T> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        parse_position: impl for<'i1, 't1> Fn(&ParserContext, &mut Parser<'i1, 't1>) -> Result<T, ParseError<'i1>>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(generic::ColorStop {
            color: Color::parse(context, input)?,
            position: input.try(|i| parse_position(context, i)).ok(),
        })
    }
}

impl Parse for PaintWorklet {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("paint")?;
        input.parse_nested_block(|input| {
            let name = Atom::from(&**input.expect_ident()?);
            let arguments = input
                .try(|input| {
                    input.expect_comma()?;
                    input.parse_comma_separated(|input| SpecifiedValue::parse(input))
                })
                .unwrap_or(vec![]);
            Ok(PaintWorklet { name, arguments })
        })
    }
}

impl Parse for MozImageRect {
    #[cfg(not(feature = "gecko"))]
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Err(input.new_error_for_next_token())
    }

    #[cfg(feature = "gecko")]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.try(|i| i.expect_function_matching("-moz-image-rect"))?;
        input.parse_nested_block(|i| {
            let string = i.expect_url_or_string()?;
            let url = SpecifiedImageUrl::parse_from_string(
                string.as_ref().to_owned(),
                context,
                crate::stylesheets::CorsMode::None,
            );
            i.expect_comma()?;
            let top = NumberOrPercentage::parse_non_negative(context, i)?;
            i.expect_comma()?;
            let right = NumberOrPercentage::parse_non_negative(context, i)?;
            i.expect_comma()?;
            let bottom = NumberOrPercentage::parse_non_negative(context, i)?;
            i.expect_comma()?;
            let left = NumberOrPercentage::parse_non_negative(context, i)?;
            Ok(MozImageRect {
                url,
                top,
                right,
                bottom,
                left,
            })
        })
    }
}
