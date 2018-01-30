/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified values for inherited box

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::f64::consts::PI;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
use values::computed;
use values::computed::{Context, Orientation, ToComputedValue};
use values::specified::Angle;

/// The specified value of the `image-orientation` property.
/// https://drafts.csswg.org/css-images/#propdef-image-orientation
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq)]
pub struct ImageOrientation {
    /// The angle specified, if any
    pub angle: Option<Angle>,

    /// Whether or not "flip" was specified
    pub flipped: bool
}

impl ToCss for ImageOrientation {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if let Some(angle) = self.angle {
            angle.to_css(dest)?;
            if self.flipped {
                dest.write_str(" flip")
            } else {
                Ok(())
            }
        } else {
            if self.flipped {
                dest.write_str("flip")
            } else {
                dest.write_str("from-image")
            }
        }
    }
}

const TWO_PI: f64 = 2.0 * PI;

// According to CSS Content Module Level 3:
// The computed value of the property is calculated by rounding the specified angle
// to the nearest quarter-turn, rounding away from 0, then moduloing the value by 1 turn.
// This mirrors the Gecko implementation in
// nsStyleImageOrientation::CreateAsAngleAndFlip.
#[inline]
fn orientation_of_angle(angle: &computed::Angle) -> Orientation {
    // Note that `angle` can be negative.
    let mut rounded_angle = angle.radians64() % TWO_PI;
    if rounded_angle < 0.0 {
        // This computation introduces rounding error. Gecko previously
        // didn't handle the negative case correctly; by branching we can
        // match Gecko's behavior when it was correct.
        rounded_angle += TWO_PI;
    }
    if rounded_angle < 0.25 * PI {
        return Orientation::Angle0
    }
    if rounded_angle < 0.75 * PI {
        return Orientation::Angle90
    }
    if rounded_angle < 1.25 * PI {
        return Orientation::Angle180
    }
    if rounded_angle < 1.75 * PI {
        return Orientation::Angle270
    }
    Orientation::Angle0
}

impl ToComputedValue for ImageOrientation {
    type ComputedValue = computed::ImageOrientation;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> computed::ImageOrientation {
        if let Some(ref angle) = self.angle {
            let angle = angle.to_computed_value(context);
            let orientation = orientation_of_angle(&angle);
            computed::ImageOrientation::AngleWithFlipped(orientation, self.flipped)
        } else {
            if self.flipped {
                computed::ImageOrientation::zero()
            } else {
                computed::ImageOrientation::FromImage
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &computed::ImageOrientation) -> Self {
        match *computed {
            computed::ImageOrientation::FromImage => {
                ImageOrientation {
                    angle: None,
                    flipped: false
                }
            },

            computed::ImageOrientation::AngleWithFlipped(ref orientation, flipped) => {
                ImageOrientation {
                    angle: Some(orientation.angle()),
                    flipped: flipped,
                }
            }
        }
    }
}

impl Parse for ImageOrientation {
    // from-image | <angle> | [<angle>? flip]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("from-image")).is_ok() {
            // Handle from-image
            Ok(ImageOrientation { angle: None, flipped: false })
        } else if input.try(|input| input.expect_ident_matching("flip")).is_ok() {
            // Handle flip
            Ok(ImageOrientation { angle: Some(Angle::zero()), flipped: true })
        } else {
            // Handle <angle> | <angle> flip
            let angle = input.try(|input| Angle::parse(context, input)).ok();
            if angle.is_none() {
                return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
            }

            let flipped = input.try(|input| input.expect_ident_matching("flip")).is_ok();
            Ok(ImageOrientation { angle: angle, flipped: flipped })
        }
    }
}
