/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use crate::values::computed::percentage::Percentage;
use crate::values::computed::position::Position;
use crate::values::computed::url::ComputedImageUrl;
#[cfg(feature = "gecko")]
use crate::values::computed::NumberOrPercentage;
use crate::values::computed::{Angle, Color, Context};
use crate::values::computed::{
    AngleOrPercentage, LengthPercentage, NonNegativeLength, NonNegativeLengthPercentage,
    Resolution, ToComputedValue,
};
use crate::values::generics::image::{self as generic, GradientCompatMode};
use crate::values::specified::image as specified;
use crate::values::specified::position::{HorizontalPositionKeyword, VerticalPositionKeyword};
use std::f32::consts::PI;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// Computed values for an image according to CSS-IMAGES.
/// <https://drafts.csswg.org/css-images/#image-values>
pub type Image =
    generic::GenericImage<Gradient, MozImageRect, ComputedImageUrl, Color, Percentage, Resolution>;

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

/// Computed values for CSS cross-fade
/// <https://drafts.csswg.org/css-images-4/#cross-fade-function>
pub type CrossFade = generic::CrossFade<Image, Color, Percentage>;
/// A computed percentage or nothing.
pub type PercentOrNone = generic::PercentOrNone<Percentage>;

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

/// The computed value for an `image-set()` image.
pub type ImageSet = generic::GenericImageSet<Image, Resolution>;

impl ToComputedValue for specified::ImageSet {
    type ComputedValue = ImageSet;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        let items = self.items.to_computed_value(context);
        let dpr = context.device().device_pixel_ratio().get();

        // If no item have a supported MIME type, the behavior is undefined by the standard
        // By default, we select the first item
        let mut supported_image = false;
        let mut selected_index = 0;
        let mut selected_resolution = items[0].resolution.dppx();

        for (i, item) in items.iter().enumerate() {

            // If the MIME type is not supported, we discard the ImageSetItem
            if item.has_mime_type && !context.device().is_supported_mime_type(&item.mime_type) {
                continue;
            }

            let candidate_resolution = item.resolution.dppx();

            // https://drafts.csswg.org/css-images-4/#image-set-notation:
            //
            //     Make a UA-specific choice of which to load, based on whatever
            //     criteria deemed relevant (such as the resolution of the
            //     display, connection speed, etc).
            //
            // For now, select the lowest resolution greater than display
            // density, otherwise the greatest resolution available
            let better_candidate = || {
                if selected_resolution < dpr && candidate_resolution > selected_resolution {
                    return true;
                }
                if candidate_resolution < selected_resolution && candidate_resolution >= dpr {
                    return true;
                }
                false
            };

            // The first item with a supported MIME type is obviously the current best candidate
            if !supported_image || better_candidate() {
                supported_image = true;
                selected_index = i;
                selected_resolution = candidate_resolution;
            }
        }

        ImageSet {
            selected_index,
            items,
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Self {
            selected_index: 0,
            items: ToComputedValue::from_computed_value(&computed.items),
        }
    }
}

/// Computed values for `-moz-image-rect(...)`.
#[cfg(feature = "gecko")]
pub type MozImageRect = generic::GenericMozImageRect<NumberOrPercentage, ComputedImageUrl>;

/// Empty enum on non-gecko
#[cfg(not(feature = "gecko"))]
pub type MozImageRect = specified::MozImageRect;

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

impl ToComputedValue for specified::LineDirection {
    type ComputedValue = LineDirection;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            specified::LineDirection::Angle(ref angle) => {
                LineDirection::Angle(angle.to_computed_value(context))
            },
            specified::LineDirection::Horizontal(x) => LineDirection::Horizontal(x),
            specified::LineDirection::Vertical(y) => LineDirection::Vertical(y),
            specified::LineDirection::Corner(x, y) => LineDirection::Corner(x, y),
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            LineDirection::Angle(ref angle) => {
                specified::LineDirection::Angle(ToComputedValue::from_computed_value(angle))
            },
            LineDirection::Horizontal(x) => specified::LineDirection::Horizontal(x),
            LineDirection::Vertical(y) => specified::LineDirection::Vertical(y),
            LineDirection::Corner(x, y) => specified::LineDirection::Corner(x, y),
        }
    }
}
