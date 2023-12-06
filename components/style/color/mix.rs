/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Color mixing/interpolation.

use super::{AbsoluteColor, ColorComponents, ColorFlags, ColorSpace};
use crate::parser::{Parse, ParserContext};
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
    pub const fn srgb() -> Self {
        Self {
            space: ColorSpace::Srgb,
            hue: HueInterpolationMethod::Shorter,
        }
    }

    /// Return the oklab interpolation method used for default color
    /// interpolcation.
    pub const fn oklab() -> Self {
        Self {
            space: ColorSpace::Oklab,
            hue: HueInterpolationMethod::Shorter,
        }
    }

    /// Decides the best method for interpolating between the given colors.
    /// https://drafts.csswg.org/css-color-4/#interpolation-space
    pub fn best_interpolation_between(left: &AbsoluteColor, right: &AbsoluteColor) -> Self {
        // The preferred color space to use for interpolating colors is Oklab.
        // However, if either of the colors are in legacy rgb(), hsl() or hwb(),
        // then interpolation is done in sRGB.
        if !left.is_legacy_color() || !right.is_legacy_color() {
            Self::oklab()
        } else {
            Self::srgb()
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

/// Mix two colors into one.
pub fn mix(
    interpolation: ColorInterpolationMethod,
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

/// What the outcome of each component should be in a mix result.
#[derive(Clone, Copy)]
#[repr(u8)]
enum ComponentMixOutcome {
    /// Mix the left and right sides to give the result.
    Mix,
    /// Carry the left side forward to the result.
    UseLeft,
    /// Carry the right side forward to the result.
    UseRight,
    /// The resulting component should also be none.
    None,
}

impl ComponentMixOutcome {
    fn from_colors(
        left: &AbsoluteColor,
        right: &AbsoluteColor,
        flags_to_check: ColorFlags,
    ) -> Self {
        match (
            left.flags.contains(flags_to_check),
            right.flags.contains(flags_to_check),
        ) {
            (true, true) => Self::None,
            (true, false) => Self::UseRight,
            (false, true) => Self::UseLeft,
            (false, false) => Self::Mix,
        }
    }
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
    let outcomes = [
        ComponentMixOutcome::from_colors(left_color, right_color, ColorFlags::C1_IS_NONE),
        ComponentMixOutcome::from_colors(left_color, right_color, ColorFlags::C2_IS_NONE),
        ComponentMixOutcome::from_colors(left_color, right_color, ColorFlags::C3_IS_NONE),
        ComponentMixOutcome::from_colors(left_color, right_color, ColorFlags::ALPHA_IS_NONE),
    ];

    // Convert both colors into the interpolation color space.
    let left = left_color.to_color_space(color_space);
    let left = left.raw_components();

    let right = right_color.to_color_space(color_space);
    let right = right.raw_components();

    let (result, result_flags) = interpolate_premultiplied(
        &left,
        left_weight,
        &right,
        right_weight,
        color_space.hue_index(),
        hue_interpolation,
        &outcomes,
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

    let mut result = AbsoluteColor::new(
        color_space,
        ColorComponents(result[0], result[1], result[2]),
        alpha,
    );

    result.flags = result_flags;
    // If both sides are legacy RGB, then the result stays in legacy RGB.
    if !left_color.is_legacy_color() || !right_color.is_legacy_color() {
        result.flags.insert(ColorFlags::AS_COLOR_FUNCTION);
    }

    result
}

fn interpolate_premultiplied_component(
    left: f32,
    left_weight: f32,
    left_alpha: f32,
    right: f32,
    right_weight: f32,
    right_alpha: f32,
) -> f32 {
    left * left_weight * left_alpha + right * right_weight * right_alpha
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

struct InterpolatedAlpha {
    /// The adjusted left alpha value.
    left: f32,
    /// The adjusted right alpha value.
    right: f32,
    /// The interpolated alpha value.
    interpolated: f32,
    /// Whether the alpha component should be `none`.
    is_none: bool,
}

fn interpolate_alpha(
    left: f32,
    left_weight: f32,
    right: f32,
    right_weight: f32,
    outcome: ComponentMixOutcome,
) -> InterpolatedAlpha {
    // <https://drafts.csswg.org/css-color-4/#interpolation-missing>
    let mut result = match outcome {
        ComponentMixOutcome::Mix => {
            let interpolated = left * left_weight + right * right_weight;
            InterpolatedAlpha {
                left,
                right,
                interpolated,
                is_none: false,
            }
        },
        ComponentMixOutcome::UseLeft => InterpolatedAlpha {
            left,
            right: left,
            interpolated: left,
            is_none: false,
        },
        ComponentMixOutcome::UseRight => InterpolatedAlpha {
            left: right,
            right,
            interpolated: right,
            is_none: false,
        },
        ComponentMixOutcome::None => InterpolatedAlpha {
            left: 1.0,
            right: 1.0,
            interpolated: 0.0,
            is_none: true,
        },
    };

    // Clip all alpha values to [0.0..1.0].
    result.left = result.left.clamp(0.0, 1.0);
    result.right = result.right.clamp(0.0, 1.0);
    result.interpolated = result.interpolated.clamp(0.0, 1.0);

    result
}

fn interpolate_premultiplied(
    left: &[f32; 4],
    left_weight: f32,
    right: &[f32; 4],
    right_weight: f32,
    hue_index: Option<usize>,
    hue_interpolation: HueInterpolationMethod,
    outcomes: &[ComponentMixOutcome; 4],
) -> ([f32; 4], ColorFlags) {
    let alpha = interpolate_alpha(left[3], left_weight, right[3], right_weight, outcomes[3]);
    let mut flags = if alpha.is_none {
        ColorFlags::ALPHA_IS_NONE
    } else {
        ColorFlags::empty()
    };

    let mut result = [0.; 4];

    for i in 0..3 {
        match outcomes[i] {
            ComponentMixOutcome::Mix => {
                let is_hue = hue_index == Some(i);
                result[i] = if is_hue {
                    normalize_hue(interpolate_hue(
                        left[i],
                        left_weight,
                        right[i],
                        right_weight,
                        hue_interpolation,
                    ))
                } else {
                    let interpolated = interpolate_premultiplied_component(
                        left[i],
                        left_weight,
                        alpha.left,
                        right[i],
                        right_weight,
                        alpha.right,
                    );

                    if alpha.interpolated == 0.0 {
                        interpolated
                    } else {
                        interpolated / alpha.interpolated
                    }
                };
            },
            ComponentMixOutcome::UseLeft => result[i] = left[i],
            ComponentMixOutcome::UseRight => result[i] = right[i],
            ComponentMixOutcome::None => {
                result[i] = 0.0;
                match i {
                    0 => flags.insert(ColorFlags::C1_IS_NONE),
                    1 => flags.insert(ColorFlags::C2_IS_NONE),
                    2 => flags.insert(ColorFlags::C3_IS_NONE),
                    _ => unreachable!(),
                }
            },
        }
    }
    result[3] = alpha.interpolated;

    (result, flags)
}
