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
use values::generics::image::{LineDirection as GenericLineDirection, MozImageRect as GenericMozImageRect};
use values::specified::image::{Gradient as SpecifiedGradient, LineDirection as SpecifiedLineDirection};
use values::specified::image::{GradientKind as SpecifiedGradientKind};
use values::specified::position::{X, Y};

/// A computed image layer.
pub type ImageLayer = Either<None_, Image>;

/// Computed values for an image according to CSS-IMAGES.
/// https://drafts.csswg.org/css-images/#image-values
pub type Image = GenericImage<Gradient, MozImageRect>;

/// Computed values for a CSS gradient.
/// https://drafts.csswg.org/css-images/#gradients
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
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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
pub type MozImageRect = GenericMozImageRect<NumberOrPercentage>;

impl GenericLineDirection for LineDirection {
    fn points_downwards(&self) -> bool {
        match *self {
            LineDirection::Angle(angle) => angle.radians() == PI,
            LineDirection::Vertical(Y::Bottom) => true,
            LineDirection::Corner(..) => false,
            #[cfg(feature = "gecko")]
            LineDirection::MozPosition(_, _) => false,
            _ => false,
        }
    }

    fn to_css<W>(&self, dest: &mut W, compat_mode: CompatMode) -> fmt::Result
        where W: fmt::Write
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

impl ToComputedValue for SpecifiedGradient {
    type ComputedValue = Gradient;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        Self::ComputedValue {
            kind: self.kind.to_computed_value(context),
            items: self.items.to_computed_value(context),
            repeating: self.repeating,
            compat_mode: self.compat_mode
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Self {
            kind: SpecifiedGradientKind::from_computed_value(&computed.kind),
            items: ToComputedValue::from_computed_value(&computed.items),
            repeating: computed.repeating,
            compat_mode: computed.compat_mode
        }
    }
}

impl ToComputedValue for SpecifiedGradientKind {
    type ComputedValue = GradientKind;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match self {
            &GenericGradientKind::Linear(ref line_direction) => {
                GenericGradientKind::Linear(line_direction.to_computed_value(context))
            },
            &GenericGradientKind::Radial(ref ending_shape, ref position, ref angle) => {
                GenericGradientKind::Radial(ending_shape.to_computed_value(context),
                                            position.to_computed_value(context),
                                            angle.map(|angle| angle.to_computed_value(context)))
            }
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            GenericGradientKind::Linear(line_direction) => {
                GenericGradientKind::Linear(SpecifiedLineDirection::from_computed_value(&line_direction))
            },
            GenericGradientKind::Radial(ending_shape, position, angle) => {
                GenericGradientKind::Radial(ToComputedValue::from_computed_value(&ending_shape),
                                            ToComputedValue::from_computed_value(&position),
                                            angle.map(|angle| ToComputedValue::from_computed_value(&angle)))
            }
        }
    }
}
