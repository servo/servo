/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use cssparser::Color as CSSColor;
use std::f32::consts::PI;
use std::fmt;
use style_traits::ToCss;
use values::computed::{Angle, Context, Length, LengthOrPercentage, ToComputedValue};
use values::computed::position::Position;
use values::specified::{self, HorizontalDirection, SizeKeyword, VerticalDirection};
use values::specified::url::SpecifiedUrl;


impl ToComputedValue for specified::Image {
    type ComputedValue = Image;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Image {
        match *self {
            specified::Image::Url(ref url_value) => {
                Image::Url(url_value.clone())
            },
            specified::Image::Gradient(ref gradient) => {
                Image::Gradient(gradient.to_computed_value(context))
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &Image) -> Self {
        match *computed {
            Image::Url(ref url_value) => {
                specified::Image::Url(url_value.clone())
            },
            Image::Gradient(ref linear_gradient) => {
                specified::Image::Gradient(
                    ToComputedValue::from_computed_value(linear_gradient)
                )
            }
        }
    }
}

/// Computed values for an image according to CSS-IMAGES.
/// https://drafts.csswg.org/css-images/#image-values
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Image {
    Url(SpecifiedUrl),
    Gradient(Gradient),
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Image::Url(ref url) => url.to_css(f),
            Image::Gradient(ref grad) => {
                if grad.repeating {
                    let _ = write!(f, "repeating-");
                }
                match grad.gradient_kind {
                    GradientKind::Linear(_) => write!(f, "linear-gradient({:?})", grad),
                    GradientKind::Radial(_, _) => write!(f, "radial-gradient({:?})", grad),
                }
            },
        }
    }
}

impl ToCss for Image {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Image::Url(ref url) => url.to_css(dest),
            Image::Gradient(ref gradient) => gradient.to_css(dest)
        }
    }
}

/// Computed values for a CSS gradient.
/// https://drafts.csswg.org/css-images/#gradients
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Gradient {
    /// The color stops.
    pub stops: Vec<ColorStop>,
    /// True if this is a repeating gradient.
    pub repeating: bool,
    /// Gradient kind can be linear or radial.
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

impl fmt::Debug for Gradient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.gradient_kind {
            GradientKind::Linear(angle_or_corner) => {
                let _ = write!(f, "{:?}", angle_or_corner);
            },
            GradientKind::Radial(ref shape, position) => {
                let _ = write!(f, "{:?} at {:?}", shape, position);
            },
        }

        for stop in &self.stops {
            let _ = write!(f, ", {:?}", stop);
        }
        Ok(())
    }
}

impl ToComputedValue for specified::Gradient {
    type ComputedValue = Gradient;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Gradient {
        let specified::Gradient {
            ref stops,
            repeating,
            ref gradient_kind
        } = *self;
        Gradient {
            stops: stops.iter().map(|s| s.to_computed_value(context)).collect(),
            repeating: repeating,
            gradient_kind: gradient_kind.to_computed_value(context),
        }
    }
    #[inline]
    fn from_computed_value(computed: &Gradient) -> Self {
        let Gradient {
            ref stops,
            repeating,
            ref gradient_kind
        } = *computed;
        specified::Gradient {
            stops: stops.iter().map(ToComputedValue::from_computed_value).collect(),
            repeating: repeating,
            gradient_kind: ToComputedValue::from_computed_value(gradient_kind),
        }
    }
}

/// Computed values for CSS linear or radial gradients.
/// https://drafts.csswg.org/css-images/#gradients
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum GradientKind {
    Linear(AngleOrCorner),
    Radial(EndingShape, Position),
}

impl ToComputedValue for specified::GradientKind {
    type ComputedValue = GradientKind;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> GradientKind {
        match *self {
            specified::GradientKind::Linear(angle_or_corner) => {
                GradientKind::Linear(angle_or_corner.to_computed_value(context))
            },
            specified::GradientKind::Radial(ref shape, position) => {
                GradientKind::Radial(shape.to_computed_value(context),
                                     position.to_computed_value(context))
            },
        }
    }
    #[inline]
    fn from_computed_value(computed: &GradientKind) -> Self {
        match *computed {
            GradientKind::Linear(angle_or_corner) => {
                specified::GradientKind::Linear(ToComputedValue::from_computed_value(&angle_or_corner))
            },
            GradientKind::Radial(ref shape, position) => {
                specified::GradientKind::Radial(ToComputedValue::from_computed_value(shape),
                                                ToComputedValue::from_computed_value(&position))
            },
        }
    }
}

/// Computed values for one color stop in a linear gradient.
/// https://drafts.csswg.org/css-images/#typedef-color-stop-list
#[derive(Clone, PartialEq, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ColorStop {
    /// The color of this stop.
    pub color: CSSColor,

    /// The position of this stop. If not specified, this stop is placed halfway between the
    /// point that precedes it and the point that follows it per CSS-IMAGES § 3.4.
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

impl fmt::Debug for ColorStop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _ = write!(f, "{:?}", self.color);
        self.position.map(|pos| {
            let _ = write!(f, " {:?}", pos);
        });
        Ok(())
    }
}

impl ToComputedValue for specified::ColorStop {
    type ComputedValue = ColorStop;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> ColorStop {
        ColorStop {
            color: self.color.parsed,
            position: match self.position {
                None => None,
                Some(value) => Some(value.to_computed_value(context)),
            },
        }
    }
    #[inline]
    fn from_computed_value(computed: &ColorStop) -> Self {
        specified::ColorStop {
            color: ToComputedValue::from_computed_value(&computed.color),
            position: match computed.position {
                None => None,
                Some(value) => Some(ToComputedValue::from_computed_value(&value)),
            },
        }
    }
}

/// Computed values for EndingShape
/// https://drafts.csswg.org/css-images/#valdef-radial-gradient-ending-shape
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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

impl fmt::Debug for EndingShape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EndingShape::Circle(ref length) => {
                let _ = write!(f, "circle {:?}", length);
            },
            EndingShape::Ellipse(ref length) => {
                let _ = write!(f, "ellipse {:?}", length);
            }
        }
        Ok(())
    }
}

impl ToComputedValue for specified::GradientEndingShape {
    type ComputedValue = EndingShape;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> EndingShape {
        match *self {
            specified::GradientEndingShape::Circle(ref length) => {
                EndingShape::Circle(length.to_computed_value(context))
            },
            specified::GradientEndingShape::Ellipse(ref length) => {
                EndingShape::Ellipse(length.to_computed_value(context))
            },
        }
    }
    #[inline]
    fn from_computed_value(computed: &EndingShape) -> Self {
        match *computed {
            EndingShape::Circle(ref length) => {
                specified::GradientEndingShape::Circle(ToComputedValue::from_computed_value(length))
            },
            EndingShape::Ellipse(ref length) => {
                specified::GradientEndingShape::Ellipse(ToComputedValue::from_computed_value(length))
            },
        }
    }
}

/// https://drafts.csswg.org/css-images/#valdef-radial-gradient-size
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrKeyword {
    Length(Length),
    Keyword(SizeKeyword),
}

impl ToCss for LengthOrKeyword {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrKeyword::Length(ref length) => length.to_css(dest),
            LengthOrKeyword::Keyword(keyword) => keyword.to_css(dest),
        }
    }
}

impl fmt::Debug for LengthOrKeyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LengthOrKeyword::Length(ref length) => {
                let _ = write!(f, "{:?}", length);
            },
            LengthOrKeyword::Keyword(keyword) => {
                let _ = write!(f, "{:?}", keyword);
            },
        }
        Ok(())
    }
}

impl ToComputedValue for specified::LengthOrKeyword {
    type ComputedValue = LengthOrKeyword;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> LengthOrKeyword {
        match *self {
            specified::LengthOrKeyword::Length(length) => {
                LengthOrKeyword::Length(length.to_computed_value(context))
            },
            specified::LengthOrKeyword::Keyword(keyword) => {
                LengthOrKeyword::Keyword(keyword)
            },
        }
    }
    #[inline]
    fn from_computed_value(computed: &LengthOrKeyword) -> Self {
        match *computed {
            LengthOrKeyword::Length(length) => {
                specified::LengthOrKeyword::Length(ToComputedValue::from_computed_value(&length))
            },
            LengthOrKeyword::Keyword(keyword) => {
                specified::LengthOrKeyword::Keyword(keyword)
            },
        }
    }
}

/// https://drafts.csswg.org/css-images/#valdef-radial-gradient-size
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrPercentageOrKeyword {
    LengthOrPercentage(LengthOrPercentage, LengthOrPercentage),
    Keyword(SizeKeyword),
}

impl ToCss for LengthOrPercentageOrKeyword {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrPercentageOrKeyword::LengthOrPercentage(ref first_len, second_len) => {
                try!(first_len.to_css(dest));
                try!(dest.write_str(" "));
                second_len.to_css(dest)
            },
            LengthOrPercentageOrKeyword::Keyword(keyword) => keyword.to_css(dest),
        }
    }
}

impl fmt::Debug for LengthOrPercentageOrKeyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LengthOrPercentageOrKeyword::LengthOrPercentage(ref first_len, second_len) => {
                let _ = write!(f, "{:?} {:?}", first_len, second_len);
            },
            LengthOrPercentageOrKeyword::Keyword(keyword) => {
                let _ = write!(f, "{:?}", keyword);
            },
        }
        Ok(())
    }
}

impl ToComputedValue for specified::LengthOrPercentageOrKeyword {
    type ComputedValue = LengthOrPercentageOrKeyword;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> LengthOrPercentageOrKeyword {
        match *self {
            specified::LengthOrPercentageOrKeyword::LengthOrPercentage(first_len, second_len) => {
                LengthOrPercentageOrKeyword::LengthOrPercentage(first_len.to_computed_value(context),
                                                                second_len.to_computed_value(context))
            },
            specified::LengthOrPercentageOrKeyword::Keyword(keyword) => {
                LengthOrPercentageOrKeyword::Keyword(keyword)
            },
        }
    }
    #[inline]
    fn from_computed_value(computed: &LengthOrPercentageOrKeyword) -> Self {
        match *computed {
            LengthOrPercentageOrKeyword::LengthOrPercentage(first_len, second_len) => {
                specified::LengthOrPercentageOrKeyword::LengthOrPercentage(
                    ToComputedValue::from_computed_value(&first_len),
                    ToComputedValue::from_computed_value(&second_len))
            },
            LengthOrPercentageOrKeyword::Keyword(keyword) => {
                specified::LengthOrPercentageOrKeyword::Keyword(keyword)
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum AngleOrCorner {
    Angle(Angle),
    Corner(HorizontalDirection, VerticalDirection)
}

impl ToComputedValue for specified::AngleOrCorner {
    type ComputedValue = AngleOrCorner;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> AngleOrCorner {
        match *self {
            specified::AngleOrCorner::None => {
                AngleOrCorner::Angle(Angle(PI))
            },
            specified::AngleOrCorner::Angle(angle) => {
                AngleOrCorner::Angle(angle)
            },
            specified::AngleOrCorner::Corner(horizontal, vertical) => {
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
                    },
                    (None, None) => {
                        unreachable!()
                    }
                }
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &AngleOrCorner) -> Self {
        match *computed {
            AngleOrCorner::Angle(angle) => {
                specified::AngleOrCorner::Angle(angle)
            },
            AngleOrCorner::Corner(horizontal, vertical) => {
                specified::AngleOrCorner::Corner(Some(horizontal), Some(vertical))
            }
        }
    }
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
