/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animated types for CSS colors.

use crate::values::animated::{Animate, Procedure, ToAnimatedZero};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::color::{Color as GenericColor, ComplexColorRatios};
use crate::values::specified::color::{ColorSpaceKind, HueAdjuster};
use euclid::default::{Transform3D, Vector3D};

/// An animated RGBA color.
///
/// Unlike in computed values, each component value may exceed the
/// range `[0.0, 1.0]`.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToAnimatedZero)]
pub struct RGBA {
    /// The red component.
    pub red: f32,
    /// The green component.
    pub green: f32,
    /// The blue component.
    pub blue: f32,
    /// The alpha component.
    pub alpha: f32,
}

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

    /// Returns whether or not the colour is in gamut for sRGB.
    pub fn in_gamut(&self) -> bool {
        0. <= self.red &&
            self.red <= 1. &&
            0. <= self.green &&
            self.green <= 1. &&
            0. <= self.blue &&
            self.blue <= 1.
    }

    /// Returns the colour with coordinates clamped to the sRGB range.
    pub fn clamp(&self) -> Self {
        Self {
            red: self.red.max(0.).min(1.),
            green: self.green.max(0.).min(1.),
            blue: self.blue.max(0.).min(1.),
            alpha: self.alpha,
        }
    }
}

impl Animate for RGBA {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let mut alpha = self.alpha.animate(&other.alpha, procedure)?;
        if alpha <= 0. {
            // Ideally we should return color value that only alpha component is
            // 0, but this is what current gecko does.
            return Ok(RGBA::transparent());
        }

        alpha = alpha.min(1.);
        let red = (self.red * self.alpha).animate(&(other.red * other.alpha), procedure)?;
        let green = (self.green * self.alpha).animate(&(other.green * other.alpha), procedure)?;
        let blue = (self.blue * self.alpha).animate(&(other.blue * other.alpha), procedure)?;
        let inv = 1. / alpha;
        Ok(RGBA::new(red * inv, green * inv, blue * inv, alpha))
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
pub type Color = GenericColor<RGBA>;

impl Color {
    fn effective_intermediate_rgba(&self) -> RGBA {
        if self.ratios.bg == 0. {
            return RGBA::transparent();
        }

        if self.ratios.bg == 1. {
            return self.color;
        }

        RGBA {
            alpha: self.color.alpha * self.ratios.bg,
            ..self.color
        }
    }

    /// Mix two colors into one.
    pub fn mix(
        color_space: ColorSpaceKind,
        left_color: &Color,
        left_weight: f32,
        right_color: &Color,
        right_weight: f32,
        hue_adjuster: HueAdjuster,
    ) -> Self {
        match color_space {
            ColorSpaceKind::Srgb => Self::mix_in::<RGBA>(
                left_color,
                left_weight,
                right_color,
                right_weight,
                hue_adjuster,
            ),
            ColorSpaceKind::Xyz => Self::mix_in::<XYZA>(
                left_color,
                left_weight,
                right_color,
                right_weight,
                hue_adjuster,
            ),
            ColorSpaceKind::Lab => Self::mix_in::<LABA>(
                left_color,
                left_weight,
                right_color,
                right_weight,
                hue_adjuster,
            ),
            ColorSpaceKind::Lch => Self::mix_in::<LCHA>(
                left_color,
                left_weight,
                right_color,
                right_weight,
                hue_adjuster,
            ),
        }
    }

    fn mix_in<S>(
        left_color: &Color,
        left_weight: f32,
        right_color: &Color,
        right_weight: f32,
        hue_adjuster: HueAdjuster,
    ) -> Self
    where
        S: ModelledColor,
    {
        let left_bg = S::from(left_color.scaled_rgba());
        let right_bg = S::from(right_color.scaled_rgba());

        let color = S::lerp(left_bg, left_weight, right_bg, right_weight, hue_adjuster);
        let rgba: RGBA = color.into();
        let rgba = if !rgba.in_gamut() {
            // TODO: Better gamut mapping.
            rgba.clamp()
        } else {
            rgba
        };

        let fg = left_color.ratios.fg * left_weight + right_color.ratios.fg * right_weight;
        Self::new(rgba, ComplexColorRatios { bg: 1., fg })
    }

    fn scaled_rgba(&self) -> RGBA {
        if self.ratios.bg == 0. {
            return RGBA::transparent();
        }

        if self.ratios.bg == 1. {
            return self.color;
        }

        RGBA {
            red: self.color.red * self.ratios.bg,
            green: self.color.green * self.ratios.bg,
            blue: self.color.blue * self.ratios.bg,
            alpha: self.color.alpha * self.ratios.bg,
        }
    }
}

impl Animate for Color {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let self_numeric = self.is_numeric();
        let other_numeric = other.is_numeric();

        if self_numeric && other_numeric {
            return Ok(Self::rgba(self.color.animate(&other.color, procedure)?));
        }

        let self_currentcolor = self.is_currentcolor();
        let other_currentcolor = other.is_currentcolor();

        if self_currentcolor && other_currentcolor {
            let (self_weight, other_weight) = procedure.weights();
            return Ok(Self::new(
                RGBA::transparent(),
                ComplexColorRatios {
                    bg: 0.,
                    fg: (self_weight + other_weight) as f32,
                },
            ));
        }

        // FIXME(emilio): Without these special cases tests fail, looks fairly
        // sketchy!
        if (self_currentcolor && other_numeric) || (self_numeric && other_currentcolor) {
            let (self_weight, other_weight) = procedure.weights();
            return Ok(if self_numeric {
                Self::new(
                    self.color,
                    ComplexColorRatios {
                        bg: self_weight as f32,
                        fg: other_weight as f32,
                    },
                )
            } else {
                Self::new(
                    other.color,
                    ComplexColorRatios {
                        bg: other_weight as f32,
                        fg: self_weight as f32,
                    },
                )
            });
        }

        // Compute the "scaled" contribution for `color`.
        // Each `Color`, represents a complex combination of foreground color and
        // background color where fg and bg represent the overall
        // contributions. ie:
        //
        //    color = { bg * mColor, fg * foreground }
        //          =   { bg_color , fg_color }
        //          =     bg_color + fg_color
        //
        // where `foreground` is `currentcolor`, and `bg_color`,
        // `fg_color` are the scaled background and foreground
        // contributions.
        //
        // Each operation, lerp, addition, or accumulate, can be
        // represented as a scaled-addition each complex color. ie:
        //
        //    p * col1 + q * col2
        //
        // where p = (1 - a), q = a for lerp(a), p = 1, q = 1 for
        // addition, etc.
        //
        // Therefore:
        //
        //    col1 op col2
        //    = p * col1 + q * col2
        //    = p * { bg_color1, fg_color1 } + q * { bg_color2, fg_color2 }
        //    = p * (bg_color1 + fg_color1) + q * (bg_color2 + fg_color2)
        //    = p * bg_color1 + p * fg_color1 + q * bg_color2 + p * fg_color2
        //    = (p * bg_color1 + q * bg_color2) + (p * fg_color1 + q * fg_color2)
        //    = (bg_color1 op bg_color2) + (fg_color1 op fg_color2)
        //
        // fg_color1 op fg_color2 is equivalent to (fg1 op fg2) * foreground,
        // so the final color is:
        //
        //    = { bg_color, fg_color }
        //    = { 1 * (bg_color1 op bg_color2), (fg1 op fg2) * foreground }
        //
        // To perform the operation on two complex colors, we need to
        // generate the scaled contributions of each background color
        // component.
        let bg_color1 = self.scaled_rgba();
        let bg_color2 = other.scaled_rgba();

        // Perform bg_color1 op bg_color2
        let bg_color = bg_color1.animate(&bg_color2, procedure)?;

        // Calculate the final foreground color ratios; perform
        // animation on effective fg ratios.
        let fg = self.ratios.fg.animate(&other.ratios.fg, procedure)?;

        Ok(Self::new(bg_color, ComplexColorRatios { bg: 1., fg }))
    }
}

impl ComputeSquaredDistance for Color {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // All comments from the Animate impl also apply here.
        let self_numeric = self.is_numeric();
        let other_numeric = other.is_numeric();

        if self_numeric && other_numeric {
            return self.color.compute_squared_distance(&other.color);
        }

        let self_currentcolor = self.is_currentcolor();
        let other_currentcolor = other.is_currentcolor();
        if self_currentcolor && other_currentcolor {
            return Ok(SquaredDistance::from_sqrt(0.));
        }

        if (self_currentcolor && other_numeric) || (self_numeric && other_currentcolor) {
            let color = if self_numeric {
                &self.color
            } else {
                &other.color
            };
            // `computed_squared_distance` is symmetric.
            return Ok(color.compute_squared_distance(&RGBA::transparent())? +
                SquaredDistance::from_sqrt(1.));
        }

        let self_color = self.effective_intermediate_rgba();
        let other_color = other.effective_intermediate_rgba();
        let self_ratios = self.ratios;
        let other_ratios = other.ratios;

        Ok(self_color.compute_squared_distance(&other_color)? +
            self_ratios.bg.compute_squared_distance(&other_ratios.bg)? +
            self_ratios.fg.compute_squared_distance(&other_ratios.fg)?)
    }
}

impl ToAnimatedZero for Color {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(RGBA::transparent().into())
    }
}

/// A color modelled in a specific color space (such as sRGB or CIE XYZ).
///
/// For now, colors modelled in other spaces need to be convertible to and from
/// `RGBA` because we use sRGB for displaying colors.
trait ModelledColor: Clone + Copy + From<RGBA> + Into<RGBA> {
    /// Linearly interpolate between the left and right colors.
    ///
    /// The HueAdjuster parameter is only for color spaces where the hue is
    /// represented as an angle (e.g., CIE LCH).
    fn lerp(
        left_bg: Self,
        left_weight: f32,
        right_bg: Self,
        right_weight: f32,
        hue_adjuster: HueAdjuster,
    ) -> Self;
}

impl ModelledColor for RGBA {
    fn lerp(
        left_bg: Self,
        left_weight: f32,
        right_bg: Self,
        right_weight: f32,
        _: HueAdjuster,
    ) -> Self {
        // Interpolation with alpha, as per
        // https://drafts.csswg.org/css-color/#interpolation-alpha.
        let mut red = 0.;
        let mut green = 0.;
        let mut blue = 0.;

        // sRGB is a rectangular othogonal color space, so all component values
        // are multiplied by the alpha value.
        for &(bg, weight) in &[(left_bg, left_weight), (right_bg, right_weight)] {
            red += bg.red * bg.alpha * weight;
            green += bg.green * bg.alpha * weight;
            blue += bg.blue * bg.alpha * weight;
        }

        let alpha = (left_bg.alpha * left_weight + right_bg.alpha * right_weight).min(1.);
        if alpha <= 0. {
            RGBA::transparent()
        } else {
            let inv = 1. / alpha;
            RGBA::new(red * inv, green * inv, blue * inv, alpha)
        }
    }
}

/// An animated XYZA colour.
#[derive(Clone, Copy, Debug)]
pub struct XYZA {
    /// The x component.
    pub x: f32,
    /// The y component.
    pub y: f32,
    /// The z component.
    pub z: f32,
    /// The alpha component.
    pub alpha: f32,
}

impl XYZA {
    /// Returns a transparent color.
    #[inline]
    pub fn transparent() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
            alpha: 0.,
        }
    }
}

impl ModelledColor for XYZA {
    fn lerp(
        left_bg: Self,
        left_weight: f32,
        right_bg: Self,
        right_weight: f32,
        _: HueAdjuster,
    ) -> Self {
        // Interpolation with alpha, as per
        // https://drafts.csswg.org/css-color/#interpolation-alpha.
        let mut x = 0.;
        let mut y = 0.;
        let mut z = 0.;

        // CIE XYZ is a rectangular othogonal color space, so all component
        // values are multiplied by the alpha value.
        for &(bg, weight) in &[(left_bg, left_weight), (right_bg, right_weight)] {
            x += bg.x * bg.alpha * weight;
            y += bg.y * bg.alpha * weight;
            z += bg.z * bg.alpha * weight;
        }

        let alpha = (left_bg.alpha * left_weight + right_bg.alpha * right_weight).min(1.);
        if alpha <= 0. {
            Self::transparent()
        } else {
            let inv = 1. / alpha;
            Self {
                x: x * inv,
                y: y * inv,
                z: z * inv,
                alpha,
            }
        }
    }
}

/// An animated LABA colour.
#[derive(Clone, Copy, Debug)]
pub struct LABA {
    /// The lightness component.
    pub lightness: f32,
    /// The a component.
    pub a: f32,
    /// The b component.
    pub b: f32,
    /// The alpha component.
    pub alpha: f32,
}

impl LABA {
    /// Returns a transparent color.
    #[inline]
    pub fn transparent() -> Self {
        Self {
            lightness: 0.,
            a: 0.,
            b: 0.,
            alpha: 0.,
        }
    }
}

impl ModelledColor for LABA {
    fn lerp(
        left_bg: Self,
        left_weight: f32,
        right_bg: Self,
        right_weight: f32,
        _: HueAdjuster,
    ) -> Self {
        // Interpolation with alpha, as per
        // https://drafts.csswg.org/css-color/#interpolation-alpha.
        let mut lightness = 0.;
        let mut a = 0.;
        let mut b = 0.;

        // CIE LAB is a rectangular othogonal color space, so all component
        // values are multiplied by the alpha value.
        for &(bg, weight) in &[(left_bg, left_weight), (right_bg, right_weight)] {
            lightness += bg.lightness * bg.alpha * weight;
            a += bg.a * bg.alpha * weight;
            b += bg.b * bg.alpha * weight;
        }

        let alpha = (left_bg.alpha * left_weight + right_bg.alpha * right_weight).min(1.);
        if alpha <= 0. {
            Self::transparent()
        } else {
            let inv = 1. / alpha;
            Self {
                lightness: lightness * inv,
                a: a * inv,
                b: b * inv,
                alpha,
            }
        }
    }
}

/// An animated LCHA colour.
#[derive(Clone, Copy, Debug)]
pub struct LCHA {
    /// The lightness component.
    pub lightness: f32,
    /// The chroma component.
    pub chroma: f32,
    /// The hua component.
    pub hue: f32,
    /// The alpha component.
    pub alpha: f32,
}

impl LCHA {
    /// Returns a transparent color.
    #[inline]
    pub fn transparent() -> Self {
        Self {
            lightness: 0.,
            chroma: 0.,
            hue: 0.,
            alpha: 0.,
        }
    }
}

impl LCHA {
    fn adjust(left_bg: Self, right_bg: Self, hue_adjuster: HueAdjuster) -> (Self, Self) {
        use std::f32::consts::{PI, TAU};

        let mut left_bg = left_bg;
        let mut right_bg = right_bg;

        // Adjust the hue angle as per
        // https://drafts.csswg.org/css-color/#hue-interpolation.
        //
        // If both hue angles are NAN, they should be set to 0. Otherwise, if a
        // single hue angle is NAN, it should use the other hue angle.
        if left_bg.hue.is_nan() || right_bg.hue.is_nan() {
            if left_bg.hue.is_nan() && right_bg.hue.is_nan() {
                left_bg.hue = 0.;
                right_bg.hue = 0.;
            } else if left_bg.hue.is_nan() {
                left_bg.hue = right_bg.hue;
            } else if right_bg.hue.is_nan() {
                right_bg.hue = left_bg.hue;
            }
        }

        if hue_adjuster != HueAdjuster::Specified {
            // Normalize hue into [0, 2 * PI)
            while left_bg.hue < 0. {
                left_bg.hue += TAU;
            }
            while left_bg.hue > TAU {
                left_bg.hue -= TAU;
            }

            while right_bg.hue < 0. {
                right_bg.hue += TAU;
            }
            while right_bg.hue >= TAU {
                right_bg.hue -= TAU;
            }
        }

        match hue_adjuster {
            HueAdjuster::Shorter => {
                let delta = right_bg.hue - left_bg.hue;

                if delta > PI {
                    left_bg.hue += PI;
                } else if delta < -1. * PI {
                    right_bg.hue += PI;
                }
            },

            HueAdjuster::Longer => {
                let delta = right_bg.hue - left_bg.hue;
                if 0. < delta && delta < PI {
                    left_bg.hue += TAU;
                } else if -1. * PI < delta && delta < 0. {
                    right_bg.hue += TAU;
                }
            },

            HueAdjuster::Increasing => {
                if right_bg.hue < left_bg.hue {
                    right_bg.hue += TAU;
                }
            },

            HueAdjuster::Decreasing => {
                if left_bg.hue < right_bg.hue {
                    left_bg.hue += TAU;
                }
            },

            //Angles are not adjusted. They are interpolated like any other
            //component.
            HueAdjuster::Specified => {},
        }

        (left_bg, right_bg)
    }
}

impl ModelledColor for LCHA {
    fn lerp(
        left_bg: Self,
        left_weight: f32,
        right_bg: Self,
        right_weight: f32,
        hue_adjuster: HueAdjuster,
    ) -> Self {
        // Interpolation with alpha, as per
        // https://drafts.csswg.org/css-color/#interpolation-alpha.
        let (left_bg, right_bg) = Self::adjust(left_bg, right_bg, hue_adjuster);

        let mut lightness = 0.;
        let mut chroma = 0.;
        let mut hue = 0.;

        // CIE LCH is a cylindical polar color space, so all component values
        // are multiplied by the alpha value.
        for &(bg, weight) in &[(left_bg, left_weight), (right_bg, right_weight)] {
            lightness += bg.lightness * bg.alpha * weight;
            chroma += bg.chroma * bg.alpha * weight;
            // LCHA is a cylindrical color space so the hue coordinate is not
            // pre-multipled by the alpha component when interpolating.
            hue += bg.hue * weight;
        }

        let alpha = (left_bg.alpha * left_weight + right_bg.alpha * right_weight).min(1.);
        if alpha <= 0. {
            Self::transparent()
        } else {
            let inv = 1. / alpha;
            Self {
                lightness: lightness * inv,
                chroma: chroma * inv,
                hue,
                alpha,
            }
        }
    }
}

impl From<RGBA> for XYZA {
    /// Convert an RGBA colour to XYZ as specified in [1].
    ///
    /// [1]: https://drafts.csswg.org/css-color/#rgb-to-lab
    fn from(rgba: RGBA) -> Self {
        fn linearize(value: f32) -> f32 {
            let sign = if value < 0. { -1. } else { 1. };
            let abs = value.abs();
            if abs < 0.04045 {
                return value / 12.92;
            }

            sign * ((abs + 0.055) / 1.055).powf(2.4)
        }

        #[cfg_attr(rustfmt, rustfmt_skip)]
        const SRGB_TO_XYZ: Transform3D<f32> = Transform3D::new(
            0.41239079926595934,  0.21263900587151027,  0.01933081871559182,  0.,
            0.357584339383878,    0.715168678767756,    0.11919477979462598,  0.,
            0.1804807884018343,   0.07219231536073371,  0.9505321522496607,   0.,
            0.,                   0.,                   0.,                   1.,
        );

        #[cfg_attr(rustfmt, rustfmt_skip)]
        const BRADFORD: Transform3D<f32> = Transform3D::new(
             1.0479298208405488,    0.029627815688159344, -0.009243058152591178,  0.,
             0.022946793341019088,  0.990434484573249,     0.015055144896577895,  0.,
            -0.05019222954313557,  -0.01707382502938514,   0.7518742899580008,    0.,
             0.,                    0.,                    0.,                    1.,
        );

        // 1. Convert from sRGB to linear-light sRGB (undo gamma encoding).
        let rgb = Vector3D::new(
            linearize(rgba.red),
            linearize(rgba.green),
            linearize(rgba.blue),
        );

        // 2. Convert from linear sRGB to CIE XYZ.
        // 3. Convert from a D65 whitepoint (used by sRGB) to the D50 whitepoint used in XYZ
        //    with the Bradford transform.
        let xyz = SRGB_TO_XYZ.then(&BRADFORD).transform_vector3d(rgb);

        XYZA {
            x: xyz.x,
            y: xyz.y,
            z: xyz.z,
            alpha: rgba.alpha,
        }
    }
}

impl From<XYZA> for LABA {
    /// Convert an XYZ colour to LAB as specified in [1] and [2].
    ///
    /// [1]: https://drafts.csswg.org/css-color/#rgb-to-lab
    /// [2]: https://drafts.csswg.org/css-color/#color-conversion-code
    fn from(xyza: XYZA) -> Self {
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
        let hue = laba.b.atan2(laba.a);
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
        let a = lcha.chroma * lcha.hue.cos();
        let b = lcha.chroma * lcha.hue.sin();
        LABA {
            lightness: lcha.lightness,
            a,
            b,
            alpha: lcha.alpha,
        }
    }
}

impl From<LABA> for XYZA {
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

        XYZA {
            x: x * WHITE[0],
            y: y * WHITE[1],
            z: z * WHITE[2],
            alpha: laba.alpha,
        }
    }
}

impl From<XYZA> for RGBA {
    /// Convert an XYZ color to sRGB as specified in [1] and [2].
    ///
    /// [1]: https://www.w3.org/TR/css-color-4/#lab-to-predefined
    /// [2]: https://www.w3.org/TR/css-color-4/#color-conversion-code
    fn from(xyza: XYZA) -> Self {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        const BRADFORD_INVERSE: Transform3D<f32> = Transform3D::new(
            0.9554734527042182,   -0.028369706963208136,  0.012314001688319899,  0.,
           -0.023098536874261423,  1.0099954580058226,   -0.020507696433477912,  0.,
            0.0632593086610217,    0.021041398966943008,  1.3303659366080753,    0.,
            0.,                    0.,                    0.,                    1.,
        );

        #[cfg_attr(rustfmt, rustfmt_skip)]
        const XYZ_TO_SRGB: Transform3D<f32> = Transform3D::new(
             3.2409699419045226,  -0.9692436362808796,   0.05563007969699366,  0.,
            -1.537383177570094,    1.8759675015077202,  -0.20397695888897652,  0.,
            -0.4986107602930034,   0.04155505740717559,  1.0569715142428786,   0.,
             0.,                   0.,                   0.,                   1.,
        );

        // 2. Convert from a D50 whitepoint (used by Lab) to the D65 whitepoint
        //    used in sRGB, with the Bradford transform.
        // 3. Convert from (D65-adapted) CIE XYZ to linear-light srgb
        let xyz = Vector3D::new(xyza.x, xyza.y, xyza.z);
        let linear_rgb = BRADFORD_INVERSE.then(&XYZ_TO_SRGB).transform_vector3d(xyz);

        // 4. Convert from linear-light srgb to srgb (do gamma encoding).
        fn delinearize(value: f32) -> f32 {
            let sign = if value < 0. { -1. } else { 1. };
            let abs = value.abs();

            if abs > 0.0031308 {
                sign * (1.055 * abs.powf(1. / 2.4) - 0.055)
            } else {
                12.92 * value
            }
        }

        let red = delinearize(linear_rgb.x);
        let green = delinearize(linear_rgb.y);
        let blue = delinearize(linear_rgb.z);

        RGBA {
            red,
            green,
            blue,
            alpha: xyza.alpha,
        }
    }
}

impl From<RGBA> for LABA {
    fn from(rgba: RGBA) -> Self {
        let xyza: XYZA = rgba.into();
        xyza.into()
    }
}

impl From<LABA> for RGBA {
    fn from(laba: LABA) -> Self {
        let xyza: XYZA = laba.into();
        xyza.into()
    }
}

impl From<RGBA> for LCHA {
    fn from(rgba: RGBA) -> Self {
        let xyza: XYZA = rgba.into();
        let laba: LABA = xyza.into();
        laba.into()
    }
}

impl From<LCHA> for RGBA {
    fn from(lcha: LCHA) -> Self {
        let laba: LABA = lcha.into();
        let xyza: XYZA = laba.into();
        xyza.into()
    }
}
