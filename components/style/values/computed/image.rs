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
use values::generics::image::{CompatMode, ColorStop as GenericColorStop};
use values::generics::image::{Gradient as GenericGradient, GradientItem as GenericGradientItem};
use values::generics::image::{Image as GenericImage, ImageRect as GenericImageRect};
use values::computed::{Angle, Context, Length, LengthOrPercentage, NumberOrPercentage, ToComputedValue};
use values::computed::position::Position;
use values::specified;
use values::specified::position::{X, Y};

pub use values::specified::SizeKeyword;

/// Computed values for an image according to CSS-IMAGES.
/// https://drafts.csswg.org/css-images/#image-values
pub type Image = GenericImage<Gradient, NumberOrPercentage>;

/// Computed values for a CSS gradient.
/// https://drafts.csswg.org/css-images/#gradients
pub type Gradient = GenericGradient<GradientKind, CSSColor, LengthOrPercentage>;

/// A computed gradient item.
pub type GradientItem = GenericGradientItem<CSSColor, LengthOrPercentage>;

/// A computed color stop.
pub type ColorStop = GenericColorStop<CSSColor, LengthOrPercentage>;

/// Computed values for ImageRect.
pub type ImageRect = GenericImageRect<NumberOrPercentage>;

impl ToCss for Gradient {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.compat_mode == CompatMode::WebKit {
            try!(dest.write_str("-webkit-"));
        }
        if self.repeating {
            try!(dest.write_str("repeating-"));
        }
        match self.kind {
            GradientKind::Linear(angle_or_corner) => {
                try!(dest.write_str("linear-gradient("));
                try!(angle_or_corner.to_css(dest, self.compat_mode));
            },
            GradientKind::Radial(ref shape, position) => {
                try!(dest.write_str("radial-gradient("));
                try!(shape.to_css(dest));
                try!(dest.write_str(" at "));
                try!(position.to_css(dest));
            },
        }
        for item in &self.items {
            try!(dest.write_str(", "));
            try!(item.to_css(dest));
        }
        try!(dest.write_str(")"));
        Ok(())
    }
}

impl fmt::Debug for Gradient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            GradientKind::Linear(angle_or_corner) => {
                let _ = write!(f, "{:?}", angle_or_corner);
            },
            GradientKind::Radial(ref shape, position) => {
                let _ = write!(f, "{:?} at {:?}", shape, position);
            },
        }

        for item in &self.items {
            let _ = write!(f, ", {:?}", item);
        }
        Ok(())
    }
}

/// Computed values for CSS linear or radial gradients.
/// https://drafts.csswg.org/css-images/#gradients
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
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
            specified::GradientKind::Radial(ref shape, ref position) => {
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

/// Computed values for EndingShape
/// https://drafts.csswg.org/css-images/#valdef-radial-gradient-ending-shape
#[derive(Clone, PartialEq)]
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
#[allow(missing_docs)]
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
            specified::LengthOrKeyword::Length(ref length) => {
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
#[allow(missing_docs)]
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
            specified::LengthOrPercentageOrKeyword::LengthOrPercentage(ref first_len, ref second_len) => {
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
#[allow(missing_docs)]
pub enum AngleOrCorner {
    Angle(Angle),
    Corner(X, Y)
}

impl ToComputedValue for specified::AngleOrCorner {
    type ComputedValue = AngleOrCorner;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> AngleOrCorner {
        match *self {
            specified::AngleOrCorner::None => {
                AngleOrCorner::Angle(Angle::from_radians(PI))
            },
            specified::AngleOrCorner::Angle(angle) => {
                AngleOrCorner::Angle(angle.to_computed_value(context))
            },
            specified::AngleOrCorner::Corner(horizontal, vertical) => {
                match (horizontal, vertical) {
                    (None, Some(Y::Top)) => {
                        AngleOrCorner::Angle(Angle::from_radians(0.0))
                    },
                    (Some(X::Right), None) => {
                        AngleOrCorner::Angle(Angle::from_radians(PI * 0.5))
                    },
                    (None, Some(Y::Bottom)) => {
                        AngleOrCorner::Angle(Angle::from_radians(PI))
                    },
                    (Some(X::Left), None) => {
                        AngleOrCorner::Angle(Angle::from_radians(PI * 1.5))
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
            AngleOrCorner::Angle(ref angle) => {
                specified::AngleOrCorner::Angle(specified::Angle::from_computed_value(angle))
            },
            AngleOrCorner::Corner(horizontal, vertical) => {
                specified::AngleOrCorner::Corner(Some(horizontal), Some(vertical))
            }
        }
    }
}

impl AngleOrCorner {
    fn to_css<W>(&self, dest: &mut W, mode: CompatMode) -> fmt::Result where W: fmt::Write {
        match *self {
            AngleOrCorner::Angle(angle) => angle.to_css(dest),
            AngleOrCorner::Corner(horizontal, vertical) => {
                if mode == CompatMode::Modern {
                    try!(dest.write_str("to "));
                }
                try!(horizontal.to_css(dest));
                try!(dest.write_str(" "));
                try!(vertical.to_css(dest));
                Ok(())
            }
        }
    }
}

/// Computed values for none | <image> | <mask-source>.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct LayerImage(pub Option<Image>);

impl ToCss for LayerImage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self.0 {
            None => dest.write_str("none"),
            Some(ref image) => image.to_css(dest),
        }
    }
}
