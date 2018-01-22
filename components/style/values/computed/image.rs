/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use cssparser::RGBA;
use std::f32::consts::PI;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};
use values::{Either, None_};
use values::computed::{Angle, ComputedUrl, Context, Length, LengthOrPercentage, NumberOrPercentage, ToComputedValue};
#[cfg(feature = "gecko")]
use values::computed::Percentage;
use values::computed::position::Position;
use values::generics::image::{CompatMode, ColorStop as GenericColorStop, EndingShape as GenericEndingShape};
use values::generics::image::{Gradient as GenericGradient, GradientItem as GenericGradientItem};
use values::generics::image::{Image as GenericImage, GradientKind as GenericGradientKind};
use values::generics::image::{LineDirection as GenericLineDirection, MozImageRect as GenericMozImageRect};
use values::specified::image::LineDirection as SpecifiedLineDirection;
use values::specified::position::{X, Y};

/// A computed image layer.
pub type ImageLayer = Either<None_, Image>;

/// Computed values for an image according to CSS-IMAGES.
/// <https://drafts.csswg.org/css-images/#image-values>
pub type Image = GenericImage<Gradient, MozImageRect, ComputedUrl>;

/// Computed values for a CSS gradient.
/// <https://drafts.csswg.org/css-images/#gradients>
pub type Gradient = GenericGradient<
    LineDirection,
    Length,
    LengthOrPercentage,
    Position,
    RGBA,
    Angle,
>;

/// A computed gradient kind.
pub type GradientKind = GenericGradientKind<
    LineDirection,
    Length,
    LengthOrPercentage,
    Position,
    Angle,
>;

/// A computed gradient line direction.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq)]
pub enum LineDirection {
    /// An angle.
    Angle(Angle),
    /// A horizontal direction.
    Horizontal(X),
    /// A vertical direction.
    Vertical(Y),
    /// A corner.
    Corner(X, Y),
    /// A Position and an Angle for legacy `-moz-` prefixed gradient.
    #[cfg(feature = "gecko")]
    MozPosition(Option<Position>, Option<Angle>),
}

/// A computed radial gradient ending shape.
pub type EndingShape = GenericEndingShape<Length, LengthOrPercentage>;

/// A computed gradient item.
pub type GradientItem = GenericGradientItem<RGBA, LengthOrPercentage>;

/// A computed color stop.
pub type ColorStop = GenericColorStop<RGBA, LengthOrPercentage>;

/// Computed values for `-moz-image-rect(...)`.
pub type MozImageRect = GenericMozImageRect<NumberOrPercentage, ComputedUrl>;

impl GenericLineDirection for LineDirection {
    fn points_downwards(&self, compat_mode: CompatMode) -> bool {
        match *self {
            LineDirection::Angle(angle) => angle.radians() == PI,
            LineDirection::Vertical(Y::Bottom)
                if compat_mode == CompatMode::Modern => true,
            LineDirection::Vertical(Y::Top)
                if compat_mode != CompatMode::Modern => true,
            LineDirection::Corner(..) => false,
            #[cfg(feature = "gecko")]
            LineDirection::MozPosition(Some(Position {
                horizontal: LengthOrPercentage::Percentage(Percentage(x)),
                vertical: LengthOrPercentage::Percentage(Percentage(y)),
            }), None) => {
                // `50% 0%` is the default value for line direction.
                x == 0.5 && y == 0.0
            },
            _ => false,
        }
    }

    fn to_css<W>(
        &self,
        dest: &mut CssWriter<W>,
        compat_mode: CompatMode,
    ) -> fmt::Result
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
            #[cfg(feature = "gecko")]
            LineDirection::MozPosition(position, angle) => {
                let mut need_space = false;
                if let Some(position) = position {
                    position.to_css(dest)?;
                    need_space = true;
                }
                if let Some(angle) = angle {
                    if need_space {
                        dest.write_str(" ")?;
                    }
                    angle.to_css(dest)?;
                }
                Ok(())
            }
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
            SpecifiedLineDirection::Horizontal(x) => {
                LineDirection::Horizontal(x)
            },
            SpecifiedLineDirection::Vertical(y) => {
                LineDirection::Vertical(y)
            },
            SpecifiedLineDirection::Corner(x, y) => {
                LineDirection::Corner(x, y)
            },
            #[cfg(feature = "gecko")]
            SpecifiedLineDirection::MozPosition(ref position, ref angle) => {
                LineDirection::MozPosition(position.to_computed_value(context),
                                           angle.to_computed_value(context))
            },
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            LineDirection::Angle(ref angle) => {
                SpecifiedLineDirection::Angle(ToComputedValue::from_computed_value(angle))
            },
            LineDirection::Horizontal(x) => {
                SpecifiedLineDirection::Horizontal(x)
            },
            LineDirection::Vertical(y) => {
                SpecifiedLineDirection::Vertical(y)
            },
            LineDirection::Corner(x, y) => {
                SpecifiedLineDirection::Corner(x, y)
            },
            #[cfg(feature = "gecko")]
            LineDirection::MozPosition(ref position, ref angle) => {
                SpecifiedLineDirection::MozPosition(ToComputedValue::from_computed_value(position),
                                                    ToComputedValue::from_computed_value(angle))
            },
        }
    }
}
