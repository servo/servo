/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use crate::values::computed::position::Position;
use crate::values::computed::url::ComputedImageUrl;
#[cfg(feature = "gecko")]
use crate::values::computed::NumberOrPercentage;
use crate::values::computed::{Angle, Color, Context};
use crate::values::computed::{
    AngleOrPercentage, LengthPercentage, NonNegativeLength, NonNegativeLengthPercentage, ToComputedValue,
};
use crate::values::generics::image::{self as generic, GradientCompatMode};
use crate::values::specified::image::LineDirection as SpecifiedLineDirection;
use crate::values::specified::position::{HorizontalPositionKeyword, VerticalPositionKeyword};
use std::f32::consts::PI;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// Computed values for an image according to CSS-IMAGES.
/// <https://drafts.csswg.org/css-images/#image-values>
pub type Image = generic::GenericImage<Gradient, MozImageRect, ComputedImageUrl>;

/// Computed values for a CSS gradient.
/// <https://drafts.csswg.org/css-images/#gradients>
pub type Gradient = generic::GenericGradient<
    LineDirection,
    LengthPercentage,
    NonNegativeLength,
    NonNegativeLengthPercentage,
    Position,
    Angle,
    AngleOrPercentage,
    Color,
>;

/// A computed radial gradient ending shape.
pub type EndingShape = generic::GenericEndingShape<NonNegativeLength, NonNegativeLengthPercentage>;

/// A computed gradient line direction.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToResolvedValue)]
#[repr(C, u8)]
pub enum LineDirection {
    /// An angle.
    Angle(Angle),
    /// A horizontal direction.
    Horizontal(HorizontalPositionKeyword),
    /// A vertical direction.
    Vertical(VerticalPositionKeyword),
    /// A corner.
    Corner(HorizontalPositionKeyword, VerticalPositionKeyword),
}

/// Computed values for `-moz-image-rect(...)`.
#[cfg(feature = "gecko")]
pub type MozImageRect = generic::GenericMozImageRect<NumberOrPercentage, ComputedImageUrl>;

/// Empty enum on non-gecko
#[cfg(not(feature = "gecko"))]
pub type MozImageRect = crate::values::specified::image::MozImageRect;

impl generic::LineDirection for LineDirection {
    fn points_downwards(&self, compat_mode: GradientCompatMode) -> bool {
        match *self {
            LineDirection::Angle(angle) => angle.radians() == PI,
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
            LineDirection::Angle(ref angle) => angle.to_css(dest),
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
