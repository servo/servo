/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use cssparser::RGBA;
use std::f32::consts::PI;
use std::fmt;
use style_traits::ToCss;
use values::{Either, None_};
use values::computed::{Angle, Context, Length, LengthOrPercentage, NumberOrPercentage, ToComputedValue};
use values::computed::position::Position;
use values::generics::image::{CompatMode, ColorStop as GenericColorStop, EndingShape as GenericEndingShape};
use values::generics::image::{Gradient as GenericGradient, GradientItem as GenericGradientItem};
use values::generics::image::{Image as GenericImage, GradientKind as GenericGradientKind};
use values::generics::image::{ImageRect as GenericImageRect, LineDirection as GenericLineDirection};
use values::specified::image::LineDirection as SpecifiedLineDirection;
use values::specified::position::{X, Y};

/// A computed image layer.
pub type ImageLayer = Either<None_, Image>;

/// Computed values for an image according to CSS-IMAGES.
/// https://drafts.csswg.org/css-images/#image-values
pub type Image = GenericImage<Gradient, ImageRect>;

/// Computed values for a CSS gradient.
/// https://drafts.csswg.org/css-images/#gradients
pub type Gradient = GenericGradient<
    LineDirection,
    Length,
    LengthOrPercentage,
    Position,
    RGBA,
>;

/// A computed gradient kind.
pub type GradientKind = GenericGradientKind<
    LineDirection,
    Length,
    LengthOrPercentage,
    Position,
>;

/// A computed gradient line direction.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LineDirection {
    /// An angle.
    Angle(Angle),
    /// A corner.
    Corner(X, Y),
}

/// A computed radial gradient ending shape.
pub type EndingShape = GenericEndingShape<Length, LengthOrPercentage>;

/// A computed gradient item.
pub type GradientItem = GenericGradientItem<RGBA, LengthOrPercentage>;

/// A computed color stop.
pub type ColorStop = GenericColorStop<RGBA, LengthOrPercentage>;

/// Computed values for ImageRect.
pub type ImageRect = GenericImageRect<NumberOrPercentage>;

impl GenericLineDirection for LineDirection {
    fn points_downwards(&self) -> bool {
        match *self {
            LineDirection::Angle(angle) => angle.radians() == PI,
            LineDirection::Corner(..) => false,
        }
    }

    fn to_css<W>(&self, dest: &mut W, compat_mode: CompatMode) -> fmt::Result
        where W: fmt::Write
    {
        match *self {
            LineDirection::Angle(ref angle) => angle.to_css(dest),
            LineDirection::Corner(x, y) => {
                if compat_mode == CompatMode::Modern {
                    dest.write_str("to ")?;
                }
                x.to_css(dest)?;
                dest.write_str(" ")?;
                y.to_css(dest)
            },
        }
    }
}

impl ToComputedValue for SpecifiedLineDirection {
    type ComputedValue = LineDirection;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            SpecifiedLineDirection::Angle(ref angle) => {
                LineDirection::Angle(angle.to_computed_value(context))
            },
            SpecifiedLineDirection::Horizontal(X::Left) => {
                LineDirection::Angle(Angle::Degree(270.))
            },
            SpecifiedLineDirection::Horizontal(X::Right) => {
                LineDirection::Angle(Angle::Degree(90.))
            },
            SpecifiedLineDirection::Vertical(Y::Top) => {
                LineDirection::Angle(Angle::Degree(0.))
            },
            SpecifiedLineDirection::Vertical(Y::Bottom) => {
                LineDirection::Angle(Angle::Degree(180.))
            },
            SpecifiedLineDirection::Corner(x, y) => {
                LineDirection::Corner(x, y)
            },
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            LineDirection::Angle(ref angle) => {
                SpecifiedLineDirection::Angle(ToComputedValue::from_computed_value(angle))
            },
            LineDirection::Corner(x, y) => {
                SpecifiedLineDirection::Corner(x, y)
            },
        }
    }
}
