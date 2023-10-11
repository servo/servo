/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animated types for CSS colors.

use crate::values::animated::{Animate, Procedure, ToAnimatedZero};
use crate::values::computed::Percentage;
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::color::{
    GenericColor, GenericColorMix, ColorInterpolationMethod, ColorSpace, HueInterpolationMethod,
};
use euclid::default::{Transform3D, Vector3D};
use std::f32::consts::PI;

/// An animated RGBA color.
///
/// Unlike in computed values, each component value may exceed the
/// range `[0.0, 1.0]`.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToAnimatedZero, ToAnimatedValue)]
#[repr(C)]
pub struct AnimatedRGBA {
    /// The red component.
    pub red: f32,
    /// The green component.
    pub green: f32,
    /// The blue component.
    pub blue: f32,
    /// The alpha component.
    pub alpha: f32,
}

use self::AnimatedRGBA as RGBA;

const RAD_PER_DEG: f32 = PI / 180.0;
const DEG_PER_RAD: f32 = 180.0 / PI;

impl RGBA {
    /// Returns a transparent color.
    #[inline]
    pub fn transparent() -> Self {
        Self::new(0., 0., 0., 0.)
    }

    /// Returns a new color.
    #[inline]
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        RGBA {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl Animate for RGBA {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let (left_weight, right_weight) = procedure.weights();
        Ok(Color::mix(
            &ColorInterpolationMethod::srgb(),
            self,
            left_weight as f32,
            other,
            right_weight as f32,
            /* normalize_weights = */ false,
        ))
    }
}

impl ComputeSquaredDistance for RGBA {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let start = [
            self.alpha,
            self.red * self.alpha,
            self.green * self.alpha,
            self.blue * self.alpha,
        ];
        let end = [
            other.alpha,
            other.red * other.alpha,
            other.green * other.alpha,
            other.blue * other.alpha,
        ];
        start
            .iter()
            .zip(&end)
            .map(|(this, other)| this.compute_squared_distance(other))
            .sum()
    }
}

/// An animated value for `<color>`.
pub type Color = GenericColor<RGBA, Percentage>;

/// An animated value for `<color-mix>`.
pub type ColorMix = GenericColorMix<Color, Percentage>;

impl Color {
    fn to_rgba(&self, current_color: RGBA) -> RGBA {
        let mut clone = self.clone();
        clone.simplify(Some(&current_color));
        *clone.as_numeric().unwrap()
    }

    /// Mix two colors into one.
    pub fn mix(
        interpolation: &ColorInterpolationMethod,
        left_color: &RGBA,
        mut left_weight: f32,
        right_color: &RGBA,
        mut right_weight: f32,
        normalize_weights: bool,
    ) -> RGBA {
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

        let mix_function = match interpolation.space {
            ColorSpace::Srgb => Self::mix_in::<RGBA>,
            ColorSpace::LinearSrgb => Self::mix_in::<LinearRGBA>,
            ColorSpace::Xyz => Self::mix_in::<XYZD65A>,
            ColorSpace::XyzD50 => Self::mix_in::<XYZD50A>,
            ColorSpace::Lab => Self::mix_in::<LABA>,
            ColorSpace::Hwb => Self::mix_in::<HWBA>,
            ColorSpace::Hsl => Self::mix_in::<HSLA>,
            ColorSpace::Lch => Self::mix_in::<LCHA>,
        };
        mix_function(
            left_color,
            left_weight,
            right_color,
            right_weight,
            interpolation.hue,
            alpha_multiplier,
        )
    }

    fn mix_in<S>(
        left_color: &RGBA,
        left_weight: f32,
        right_color: &RGBA,
        right_weight: f32,
        hue_interpolation: HueInterpolationMethod,
        alpha_multiplier: f32,
    ) -> RGBA
    where
        S: ModelledColor,
    {
        let left = S::from(*left_color);
        let right = S::from(*right_color);

        let color = S::lerp(
            &left,
            left_weight,
            &right,
            right_weight,
            hue_interpolation,
        );
        let mut rgba = RGBA::from(color.into());
        if alpha_multiplier != 1.0 {
            rgba.alpha *= alpha_multiplier;
        }

        rgba
    }
}

impl Animate for Color {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let (left_weight, right_weight) = procedure.weights();
        let mut color = Color::ColorMix(Box::new(ColorMix {
            interpolation: ColorInterpolationMethod::srgb(),
            left: self.clone(),
            left_percentage: Percentage(left_weight as f32),
            right: other.clone(),
            right_percentage: Percentage(right_weight as f32),
            // See https://github.com/w3c/csswg-drafts/issues/7324
            normalize_weights: false,
        }));
        color.simplify(None);
        Ok(color)
    }
}

impl ComputeSquaredDistance for Color {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let current_color = RGBA::transparent();
        self.to_rgba(current_color)
            .compute_squared_distance(&other.to_rgba(current_color))
    }
}

impl ToAnimatedZero for Color {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(Color::rgba(RGBA::transparent()))
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

macro_rules! impl_lerp {
    ($ty:ident, $hue_index:expr) => {
        // These ensure the transmutes below are sound.
        const_assert_eq!(std::mem::size_of::<$ty>(), std::mem::size_of::<f32>() * 4);
        const_assert_eq!(std::mem::align_of::<$ty>(), std::mem::align_of::<f32>());
        impl ModelledColor for $ty {
            fn lerp(
                left: &Self,
                left_weight: f32,
                right: &Self,
                right_weight: f32,
                hue_interpolation: HueInterpolationMethod,
            ) -> Self {
                use std::mem::transmute;
                unsafe {
                    transmute::<[f32; 4], Self>(interpolate_premultiplied(
                        transmute::<&Self, &[f32; 4]>(left),
                        left_weight,
                        transmute::<&Self, &[f32; 4]>(right),
                        right_weight,
                        $hue_index,
                        hue_interpolation,
                    ))
                }
            }
        }
    };
}

impl_lerp!(RGBA, None);

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct LinearRGBA {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

impl_lerp!(LinearRGBA, None);

/// An animated XYZ D65 colour.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct XYZD65A {
    x: f32,
    y: f32,
    z: f32,
    alpha: f32,
}

impl_lerp!(XYZD65A, None);

/// An animated XYZ D50 colour.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct XYZD50A {
    x: f32,
    y: f32,
    z: f32,
    alpha: f32,
}

impl_lerp!(XYZD50A, None);

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct LABA {
    lightness: f32,
    a: f32,
    b: f32,
    alpha: f32,
}

impl_lerp!(LABA, None);

/// An animated LCHA colour.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct LCHA {
    lightness: f32,
    chroma: f32,
    hue: f32,
    alpha: f32,
}

impl_lerp!(LCHA, Some(2));

/// An animated hwb() color.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct HWBA {
    hue: f32,
    white: f32,
    black: f32,
    alpha: f32,
}

impl_lerp!(HWBA, Some(0));

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct HSLA {
    hue: f32,
    sat: f32,
    light: f32,
    alpha: f32,
}

impl_lerp!(HSLA, Some(0));

// https://drafts.csswg.org/css-color/#rgb-to-hsl
//
// We also return min/max for the hwb conversion.
fn rgb_to_hsl(rgba: RGBA) -> (HSLA, f32, f32) {
    let RGBA {
        red,
        green,
        blue,
        alpha,
    } = rgba;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let mut hue = std::f32::NAN;
    let mut sat = 0.;
    let light = (min + max) / 2.;
    let d = max - min;

    if d != 0. {
        sat = if light == 0.0 || light == 1.0 {
            0.
        } else {
            (max - light) / light.min(1. - light)
        };

        if max == red {
            hue = (green - blue) / d + if green < blue { 6. } else { 0. }
        } else if max == green {
            hue = (blue - red) / d + 2.;
        } else {
            hue = (red - green) / d + 4.;
        }

        hue *= 60.;
    }

    (
        HSLA {
            hue,
            sat,
            light,
            alpha,
        },
        min,
        max,
    )
}

impl From<RGBA> for HSLA {
    fn from(rgba: RGBA) -> Self {
        rgb_to_hsl(rgba).0
    }
}

impl From<HSLA> for RGBA {
    fn from(hsla: HSLA) -> Self {
        // cssparser expects hue in the 0..1 range.
        let hue_normalized = normalize_hue(hsla.hue) / 360.;
        let (r, g, b) = cssparser::hsl_to_rgb(hue_normalized, hsla.sat, hsla.light);
        RGBA::new(r, g, b, hsla.alpha)
    }
}

impl From<RGBA> for HWBA {
    // https://drafts.csswg.org/css-color/#rgb-to-hwb
    fn from(rgba: RGBA) -> Self {
        let (hsl, min, max) = rgb_to_hsl(rgba);
        Self {
            hue: hsl.hue,
            white: min,
            black: 1. - max,
            alpha: rgba.alpha,
        }
    }
}

impl From<HWBA> for RGBA {
    fn from(hwba: HWBA) -> Self {
        let hue_normalized = normalize_hue(hwba.hue) / 360.;
        let (r, g, b) = cssparser::hwb_to_rgb(hue_normalized, hwba.white, hwba.black);
        RGBA::new(r, g, b, hwba.alpha)
    }
}

impl From<RGBA> for LinearRGBA {
    fn from(rgba: RGBA) -> Self {
        fn linearize(value: f32) -> f32 {
            let sign = if value < 0. { -1. } else { 1. };
            let abs = value.abs();
            if abs < 0.04045 {
                return value / 12.92;
            }

            sign * ((abs + 0.055) / 1.055).powf(2.4)
        }
        Self {
            red: linearize(rgba.red),
            green: linearize(rgba.green),
            blue: linearize(rgba.blue),
            alpha: rgba.alpha,
        }
    }
}

impl From<LinearRGBA> for RGBA {
    fn from(lrgba: LinearRGBA) -> Self {
        fn delinearize(value: f32) -> f32 {
            let sign = if value < 0. { -1. } else { 1. };
            let abs = value.abs();

            if abs > 0.0031308 {
                sign * (1.055 * abs.powf(1. / 2.4) - 0.055)
            } else {
                12.92 * value
            }
        }
        Self {
            red: delinearize(lrgba.red),
            green: delinearize(lrgba.green),
            blue: delinearize(lrgba.blue),
            alpha: lrgba.alpha,
        }
    }
}

impl From<XYZD65A> for XYZD50A {
    fn from(d65: XYZD65A) -> Self {
        // https://drafts.csswg.org/css-color-4/#color-conversion-code
        #[cfg_attr(rustfmt, rustfmt_skip)]
        const BRADFORD: Transform3D<f32> = Transform3D::new(
             1.0479298208405488,    0.029627815688159344, -0.009243058152591178,  0.,
             0.022946793341019088,  0.990434484573249,     0.015055144896577895,  0.,
            -0.05019222954313557,  -0.01707382502938514,   0.7518742899580008,    0.,
             0.,                    0.,                    0.,                    1.,
        );
        let d50 = BRADFORD.transform_vector3d(Vector3D::new(d65.x, d65.y, d65.z));
        Self {
            x: d50.x,
            y: d50.y,
            z: d50.z,
            alpha: d65.alpha,
        }
    }
}

impl From<XYZD50A> for XYZD65A {
    fn from(d50: XYZD50A) -> Self {
        // https://drafts.csswg.org/css-color-4/#color-conversion-code
        #[cfg_attr(rustfmt, rustfmt_skip)]
        const BRADFORD_INVERSE: Transform3D<f32> = Transform3D::new(
            0.9554734527042182,   -0.028369706963208136,  0.012314001688319899,  0.,
           -0.023098536874261423,  1.0099954580058226,   -0.020507696433477912,  0.,
            0.0632593086610217,    0.021041398966943008,  1.3303659366080753,    0.,
            0.,                    0.,                    0.,                    1.,
        );
        let d65 = BRADFORD_INVERSE.transform_vector3d(Vector3D::new(d50.x, d50.y, d50.z));
        Self {
            x: d65.x,
            y: d65.y,
            z: d65.z,
            alpha: d50.alpha,
        }
    }
}

impl From<LinearRGBA> for XYZD65A {
    fn from(lrgba: LinearRGBA) -> Self {
        // https://drafts.csswg.org/css-color-4/#color-conversion-code
        #[cfg_attr(rustfmt, rustfmt_skip)]
        const LSRGB_TO_XYZ: Transform3D<f32> = Transform3D::new(
            0.41239079926595934,  0.21263900587151027,  0.01933081871559182,  0.,
            0.357584339383878,    0.715168678767756,    0.11919477979462598,  0.,
            0.1804807884018343,   0.07219231536073371,  0.9505321522496607,   0.,
            0.,                   0.,                   0.,                   1.,
        );
        let linear_rgb = Vector3D::new(lrgba.red, lrgba.green, lrgba.blue);
        let xyz = LSRGB_TO_XYZ.transform_vector3d(linear_rgb);
        Self {
            x: xyz.x,
            y: xyz.y,
            z: xyz.z,
            alpha: lrgba.alpha,
        }
    }
}

impl From<XYZD65A> for LinearRGBA {
    fn from(d65: XYZD65A) -> Self {
        // https://drafts.csswg.org/css-color-4/#color-conversion-code
        #[cfg_attr(rustfmt, rustfmt_skip)]
        const XYZ_TO_LSRGB: Transform3D<f32> = Transform3D::new(
             3.2409699419045226,  -0.9692436362808796,   0.05563007969699366,  0.,
            -1.537383177570094,    1.8759675015077202,  -0.20397695888897652,  0.,
            -0.4986107602930034,   0.04155505740717559,  1.0569715142428786,   0.,
             0.,                   0.,                   0.,                   1.,
        );

        let xyz = Vector3D::new(d65.x, d65.y, d65.z);
        let rgb = XYZ_TO_LSRGB.transform_vector3d(xyz);
        Self {
            red: rgb.x,
            green: rgb.y,
            blue: rgb.z,
            alpha: d65.alpha,
        }
    }
}

impl From<XYZD65A> for RGBA {
    fn from(d65: XYZD65A) -> Self {
        Self::from(LinearRGBA::from(d65))
    }
}

impl From<RGBA> for XYZD65A {
    /// Convert an RGBA colour to XYZ as specified in [1].
    ///
    /// [1]: https://drafts.csswg.org/css-color/#rgb-to-lab
    fn from(rgba: RGBA) -> Self {
        Self::from(LinearRGBA::from(rgba))
    }
}

impl From<XYZD50A> for LABA {
    /// Convert an XYZ colour to LAB as specified in [1] and [2].
    ///
    /// [1]: https://drafts.csswg.org/css-color/#rgb-to-lab
    /// [2]: https://drafts.csswg.org/css-color/#color-conversion-code
    fn from(xyza: XYZD50A) -> Self {
        const WHITE: [f32; 3] = [0.96422, 1., 0.82521];

        fn compute_f(value: f32) -> f32 {
            const EPSILON: f32 = 216. / 24389.;
            const KAPPA: f32 = 24389. / 27.;

            if value > EPSILON {
                value.cbrt()
            } else {
                (KAPPA * value + 16.) / 116.
            }
        }

        // 4. Convert D50-adapted XYZ to Lab.
        let f = [
            compute_f(xyza.x / WHITE[0]),
            compute_f(xyza.y / WHITE[1]),
            compute_f(xyza.z / WHITE[2]),
        ];

        let lightness = 116. * f[1] - 16.;
        let a = 500. * (f[0] - f[1]);
        let b = 200. * (f[1] - f[2]);

        LABA {
            lightness,
            a,
            b,
            alpha: xyza.alpha,
        }
    }
}

impl From<LABA> for LCHA {
    /// Convert a LAB color to LCH as specified in [1].
    ///
    /// [1]: https://drafts.csswg.org/css-color/#color-conversion-code
    fn from(laba: LABA) -> Self {
        let hue = laba.b.atan2(laba.a) * DEG_PER_RAD;
        let chroma = (laba.a * laba.a + laba.b * laba.b).sqrt();
        LCHA {
            lightness: laba.lightness,
            chroma,
            hue,
            alpha: laba.alpha,
        }
    }
}

impl From<LCHA> for LABA {
    /// Convert a LCH color to LAB as specified in [1].
    ///
    /// [1]: https://drafts.csswg.org/css-color/#color-conversion-code
    fn from(lcha: LCHA) -> Self {
        let hue_radians = lcha.hue * RAD_PER_DEG;
        let a = lcha.chroma * hue_radians.cos();
        let b = lcha.chroma * hue_radians.sin();
        LABA {
            lightness: lcha.lightness,
            a,
            b,
            alpha: lcha.alpha,
        }
    }
}

impl From<LABA> for XYZD50A {
    /// Convert a CIELAB color to XYZ as specified in [1] and [2].
    ///
    /// [1]: https://drafts.csswg.org/css-color/#lab-to-predefined
    /// [2]: https://drafts.csswg.org/css-color/#color-conversion-code
    fn from(laba: LABA) -> Self {
        // 1. Convert LAB to (D50-adapated) XYZ.
        const KAPPA: f32 = 24389. / 27.;
        const EPSILON: f32 = 216. / 24389.;
        const WHITE: [f32; 3] = [0.96422, 1., 0.82521];

        let f1 = (laba.lightness + 16f32) / 116f32;
        let f0 = (laba.a / 500.) + f1;
        let f2 = f1 - laba.b / 200.;

        let x = if f0.powf(3.) > EPSILON {
            f0.powf(3.)
        } else {
            (116. * f0 - 16.) / KAPPA
        };
        let y = if laba.lightness > KAPPA * EPSILON {
            ((laba.lightness + 16.) / 116.).powf(3.)
        } else {
            laba.lightness / KAPPA
        };
        let z = if f2.powf(3.) > EPSILON {
            f2.powf(3.)
        } else {
            (116. * f2 - 16.) / KAPPA
        };

        Self {
            x: x * WHITE[0],
            y: y * WHITE[1],
            z: z * WHITE[2],
            alpha: laba.alpha,
        }
    }
}

impl From<XYZD50A> for RGBA {
    fn from(d50: XYZD50A) -> Self {
        Self::from(XYZD65A::from(d50))
    }
}

impl From<RGBA> for XYZD50A {
    fn from(rgba: RGBA) -> Self {
        Self::from(XYZD65A::from(rgba))
    }
}

impl From<RGBA> for LABA {
    fn from(rgba: RGBA) -> Self {
        Self::from(XYZD50A::from(rgba))
    }
}

impl From<LABA> for RGBA {
    fn from(laba: LABA) -> Self {
        Self::from(XYZD50A::from(laba))
    }
}

impl From<RGBA> for LCHA {
    fn from(rgba: RGBA) -> Self {
        Self::from(LABA::from(rgba))
    }
}

impl From<LCHA> for RGBA {
    fn from(lcha: LCHA) -> Self {
        Self::from(LABA::from(lcha))
    }
}
