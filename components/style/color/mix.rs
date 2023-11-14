/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Color mixing/interpolation.

use super::{AbsoluteColor, ColorComponents, ColorSpace};
use crate::parser::{Parse, ParserContext};
use crate::values::animated::color::AnimatedRGBA as RGBA;
use cssparser::Parser;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

/// A hue-interpolation-method as defined in [1].
///
/// [1]: https://drafts.csswg.org/css-color-4/#typedef-hue-interpolation-method
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum HueInterpolationMethod {
    /// https://drafts.csswg.org/css-color-4/#shorter
    Shorter,
    /// https://drafts.csswg.org/css-color-4/#longer
    Longer,
    /// https://drafts.csswg.org/css-color-4/#increasing
    Increasing,
    /// https://drafts.csswg.org/css-color-4/#decreasing
    Decreasing,
    /// https://drafts.csswg.org/css-color-4/#specified
    Specified,
}

/// https://drafts.csswg.org/css-color-4/#color-interpolation-method
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    ToShmem,
    ToAnimatedValue,
    ToComputedValue,
    ToResolvedValue,
)]
#[repr(C)]
pub struct ColorInterpolationMethod {
    /// The color-space the interpolation should be done in.
    pub space: ColorSpace,
    /// The hue interpolation method.
    pub hue: HueInterpolationMethod,
}

impl ColorInterpolationMethod {
    /// Returns the srgb interpolation method.
    pub fn srgb() -> Self {
        Self {
            space: ColorSpace::Srgb,
            hue: HueInterpolationMethod::Shorter,
        }
    }
}

impl Parse for ColorInterpolationMethod {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_ident_matching("in")?;
        let space = ColorSpace::parse(input)?;
        // https://drafts.csswg.org/css-color-4/#hue-interpolation
        //     Unless otherwise specified, if no specific hue interpolation
        //     algorithm is selected by the host syntax, the default is shorter.
        let hue = if space.is_polar() {
            input
                .try_parse(|input| -> Result<_, ParseError<'i>> {
                    let hue = HueInterpolationMethod::parse(input)?;
                    input.expect_ident_matching("hue")?;
                    Ok(hue)
                })
                .unwrap_or(HueInterpolationMethod::Shorter)
        } else {
            HueInterpolationMethod::Shorter
        };
        Ok(Self { space, hue })
    }
}

impl ToCss for ColorInterpolationMethod {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("in ")?;
        self.space.to_css(dest)?;
        if self.hue != HueInterpolationMethod::Shorter {
            dest.write_char(' ')?;
            self.hue.to_css(dest)?;
            dest.write_str(" hue")?;
        }
        Ok(())
    }
}

/// A color modelled in a specific color space (such as sRGB or CIE XYZ).
///
/// For now, colors modelled in other spaces need to be convertible to and from
/// `RGBA` because we use sRGB for displaying colors.
trait ModelledColor: Clone + Copy + From<RGBA> + Into<RGBA> {
    /// Linearly interpolate between the left and right colors.
    ///
    /// The HueInterpolationMethod parameter is only for color spaces where the hue is
    /// represented as an angle (e.g., CIE LCH).
    fn lerp(
        left_bg: &Self,
        left_weight: f32,
        right_bg: &Self,
        right_weight: f32,
        hue_interpolation: HueInterpolationMethod,
    ) -> Self;
}

/// Mix two colors into one.
pub fn mix(
    interpolation: &ColorInterpolationMethod,
    left_color: &AbsoluteColor,
    mut left_weight: f32,
    right_color: &AbsoluteColor,
    mut right_weight: f32,
    normalize_weights: bool,
) -> AbsoluteColor {
    // https://drafts.csswg.org/css-color-5/#color-mix-percent-norm
    let mut alpha_multiplier = 1.0;
    if normalize_weights {
        let sum = left_weight + right_weight;
        if sum != 1.0 {
            let scale = 1.0 / sum;
            left_weight *= scale;
            right_weight *= scale;
            if sum < 1.0 {
                alpha_multiplier = sum;
            }
        }
    }

    mix_in(
        interpolation.space,
        left_color,
        left_weight,
        right_color,
        right_weight,
        interpolation.hue,
        alpha_multiplier,
    )
}

fn mix_in(
    color_space: ColorSpace,
    left_color: &AbsoluteColor,
    left_weight: f32,
    right_color: &AbsoluteColor,
    right_weight: f32,
    hue_interpolation: HueInterpolationMethod,
    alpha_multiplier: f32,
) -> AbsoluteColor {
    // Convert both colors into the interpolation color space.
    let left = left_color.to_color_space(color_space);
    let left = left.raw_components();

    let right = right_color.to_color_space(color_space);
    let right = right.raw_components();

    let result = interpolate_premultiplied(
        &left,
        left_weight,
        &right,
        right_weight,
        color_space.hue_index(),
        hue_interpolation,
    );

    let alpha = if alpha_multiplier != 1.0 {
        result[3] * alpha_multiplier
    } else {
        result[3]
    };

    // FIXME: In rare cases we end up with 0.999995 in the alpha channel,
    //        so we reduce the precision to avoid serializing to
    //        rgba(?, ?, ?, 1).  This is not ideal, so we should look into
    //        ways to avoid it. Maybe pre-multiply all color components and
    //        then divide after calculations?
    let alpha = (alpha * 1000.0).round() / 1000.0;

    AbsoluteColor::new(
        color_space,
        ColorComponents(result[0], result[1], result[2]),
        alpha,
    )
}

fn interpolate_premultiplied_component(
    left: f32,
    left_weight: f32,
    left_alpha: f32,
    right: f32,
    right_weight: f32,
    right_alpha: f32,
    inverse_of_result_alpha: f32,
) -> f32 {
    (left * left_weight * left_alpha + right * right_weight * right_alpha) * inverse_of_result_alpha
}

// Normalize hue into [0, 360)
#[inline]
fn normalize_hue(v: f32) -> f32 {
    v - 360. * (v / 360.).floor()
}

fn adjust_hue(left: &mut f32, right: &mut f32, hue_interpolation: HueInterpolationMethod) {
    // Adjust the hue angle as per
    // https://drafts.csswg.org/css-color/#hue-interpolation.
    //
    // If both hue angles are NAN, they should be set to 0. Otherwise, if a
    // single hue angle is NAN, it should use the other hue angle.
    if left.is_nan() {
        if right.is_nan() {
            *left = 0.;
            *right = 0.;
        } else {
            *left = *right;
        }
    } else if right.is_nan() {
        *right = *left;
    }

    if hue_interpolation == HueInterpolationMethod::Specified {
        // Angles are not adjusted. They are interpolated like any other
        // component.
        return;
    }

    *left = normalize_hue(*left);
    *right = normalize_hue(*right);

    match hue_interpolation {
        // https://drafts.csswg.org/css-color/#shorter
        HueInterpolationMethod::Shorter => {
            let delta = *right - *left;

            if delta > 180. {
                *left += 360.;
            } else if delta < -180. {
                *right += 360.;
            }
        },
        // https://drafts.csswg.org/css-color/#longer
        HueInterpolationMethod::Longer => {
            let delta = *right - *left;
            if 0. < delta && delta < 180. {
                *left += 360.;
            } else if -180. < delta && delta < 0. {
                *right += 360.;
            }
        },
        // https://drafts.csswg.org/css-color/#increasing
        HueInterpolationMethod::Increasing => {
            if *right < *left {
                *right += 360.;
            }
        },
        // https://drafts.csswg.org/css-color/#decreasing
        HueInterpolationMethod::Decreasing => {
            if *left < *right {
                *left += 360.;
            }
        },
        HueInterpolationMethod::Specified => unreachable!("Handled above"),
    }
}

fn interpolate_hue(
    mut left: f32,
    left_weight: f32,
    mut right: f32,
    right_weight: f32,
    hue_interpolation: HueInterpolationMethod,
) -> f32 {
    adjust_hue(&mut left, &mut right, hue_interpolation);
    left * left_weight + right * right_weight
}

fn interpolate_premultiplied(
    left: &[f32; 4],
    left_weight: f32,
    right: &[f32; 4],
    right_weight: f32,
    hue_index: Option<usize>,
    hue_interpolation: HueInterpolationMethod,
) -> [f32; 4] {
    let left_alpha = left[3];
    let right_alpha = right[3];
    let result_alpha = (left_alpha * left_weight + right_alpha * right_weight).min(1.);
    let mut result = [0.; 4];
    if result_alpha <= 0. {
        return result;
    }

    let inverse_of_result_alpha = 1. / result_alpha;
    for i in 0..3 {
        let is_hue = hue_index == Some(i);
        result[i] = if is_hue {
            interpolate_hue(
                left[i],
                left_weight,
                right[i],
                right_weight,
                hue_interpolation,
            )
        } else {
            interpolate_premultiplied_component(
                left[i],
                left_weight,
                left_alpha,
                right[i],
                right_weight,
                right_alpha,
                inverse_of_result_alpha,
            )
        };
    }
    result[3] = result_alpha;

    result
}
