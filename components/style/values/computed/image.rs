/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use crate::values::computed::position::Position;
use crate::values::computed::url::ComputedImageUrl;
use crate::values::computed::{Angle, Color, Context};
use crate::values::computed::{Length, LengthPercentage, NumberOrPercentage, ToComputedValue};
use crate::values::generics::image::{self as generic, CompatMode};
use crate::values::specified::image::LineDirection as SpecifiedLineDirection;
use crate::values::specified::position::{X, Y};
use crate::values::{Either, None_};
use std::f32::consts::PI;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// A computed image layer.
pub type ImageLayer = Either<None_, Image>;

/// Computed values for an image according to CSS-IMAGES.
/// <https://drafts.csswg.org/css-images/#image-values>
pub type Image = generic::Image<Gradient, MozImageRect, ComputedImageUrl>;

/// Computed values for a CSS gradient.
/// <https://drafts.csswg.org/css-images/#gradients>
pub type Gradient =
    generic::Gradient<LineDirection, Length, LengthPercentage, Position, Color>;

/// A computed gradient kind.
pub type GradientKind =
    generic::GradientKind<LineDirection, Length, LengthPercentage, Position>;

/// A computed gradient line direction.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToResolvedValue)]
pub enum LineDirection {
    /// An angle.
    Angle(Angle),
    /// A horizontal direction.
    Horizontal(X),
    /// A vertical direction.
    Vertical(Y),
    /// A corner.
    Corner(X, Y),
}

/// A computed radial gradient ending shape.
pub type EndingShape = generic::EndingShape<Length, LengthPercentage>;

/// A computed gradient item.
pub type GradientItem = generic::GenericGradientItem<Color, LengthPercentage>;

/// A computed color stop.
pub type ColorStop = generic::ColorStop<Color, LengthPercentage>;

/// Computed values for `-moz-image-rect(...)`.
pub type MozImageRect = generic::MozImageRect<NumberOrPercentage, ComputedImageUrl>;

impl generic::LineDirection for LineDirection {
    fn points_downwards(&self, compat_mode: CompatMode) -> bool {
        match *self {
            LineDirection::Angle(angle) => angle.radians() == PI,
            LineDirection::Vertical(Y::Bottom) if compat_mode == CompatMode::Modern => true,
            LineDirection::Vertical(Y::Top) if compat_mode != CompatMode::Modern => true,
            _ => false,
        }
    }

    fn to_css<W>(&self, dest: &mut CssWriter<W>, compat_mode: CompatMode) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            LineDirection::Angle(ref angle) => angle.to_css(dest),
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
            SpecifiedLineDirection::Horizontal(x) => LineDirection::Horizontal(x),
            SpecifiedLineDirection::Vertical(y) => LineDirection::Vertical(y),
            SpecifiedLineDirection::Corner(x, y) => LineDirection::Corner(x, y),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            LineDirection::Angle(ref angle) => {
                SpecifiedLineDirection::Angle(ToComputedValue::from_computed_value(angle))
            },
            LineDirection::Horizontal(x) => SpecifiedLineDirection::Horizontal(x),
            LineDirection::Vertical(y) => SpecifiedLineDirection::Vertical(y),
            LineDirection::Corner(x, y) => SpecifiedLineDirection::Corner(x, y),
        }
    }
}
