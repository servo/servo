/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Color conversion algorithms.
//!
//! Algorithms, matrices and constants are from the [color-4] specification,
//! unless otherwise specified:
//!
//! https://drafts.csswg.org/csswg-drafts/css-color-4/#color-conversion-code

use crate::color::ColorComponents;
use std::f32::consts::PI;

type Transform = euclid::default::Transform3D<f32>;
type Vector = euclid::default::Vector3D<f32>;

const RAD_PER_DEG: f32 = PI / 180.0;
const DEG_PER_RAD: f32 = 180.0 / PI;

#[inline]
fn transform(from: &ColorComponents, mat: &Transform) -> ColorComponents {
    let result = mat.transform_vector3d(Vector::new(from.0, from.1, from.2));
    ColorComponents(result.x, result.y, result.z)
}

fn xyz_d65_to_xyz_d50(from: &ColorComponents) -> ColorComponents {
    #[rustfmt::skip]
    const MAT: Transform = Transform::new(
         1.0479298208405488,    0.029627815688159344, -0.009243058152591178, 0.0,
         0.022946793341019088,  0.990434484573249,     0.015055144896577895, 0.0,
        -0.05019222954313557,  -0.01707382502938514,   0.7518742899580008,   0.0,
         0.0,                   0.0,                   0.0,                  1.0,
    );

    transform(from, &MAT)
}

fn xyz_d50_to_xyz_d65(from: &ColorComponents) -> ColorComponents {
    #[rustfmt::skip]
    const MAT: Transform = Transform::new(
         0.9554734527042182,   -0.028369706963208136,  0.012314001688319899, 0.0,
        -0.023098536874261423,  1.0099954580058226,   -0.020507696433477912, 0.0,
         0.0632593086610217,    0.021041398966943008,  1.3303659366080753,   0.0,
         0.0,                   0.0,                   0.0,                  1.0,
    );

    transform(from, &MAT)
}

/// A reference white that is used during color conversion.
pub enum WhitePoint {
    /// D50 white reference.
    D50,
    /// D65 white reference.
    D65,
}

fn convert_white_point(from: WhitePoint, to: WhitePoint, components: &mut ColorComponents) {
    match (from, to) {
        (WhitePoint::D50, WhitePoint::D65) => *components = xyz_d50_to_xyz_d65(components),
        (WhitePoint::D65, WhitePoint::D50) => *components = xyz_d65_to_xyz_d50(components),

        _ => {},
    }
}

/// A trait that allows conversion of color spaces to and from XYZ coordinate
/// space with a specified white point.
///
/// Allows following the specified method of converting between color spaces:
/// - Convert to values to sRGB linear light.
/// - Convert to XYZ coordinate space.
/// - Adjust white point to target white point.
/// - Convert to sRGB linear light in target color space.
/// - Convert to sRGB gamma encoded in target color space.
///
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#color-conversion
pub trait ColorSpaceConversion {
    /// The white point that the implementer is represented in.
    const WHITE_POINT: WhitePoint;

    /// Convert the components from sRGB gamma encoded values to sRGB linear
    /// light values.
    fn to_linear_light(from: &ColorComponents) -> ColorComponents;

    /// Convert the components from sRGB linear light values to XYZ coordinate
    /// space.
    fn to_xyz(from: &ColorComponents) -> ColorComponents;

    /// Convert the components from XYZ coordinate space to sRGB linear light
    /// values.
    fn from_xyz(from: &ColorComponents) -> ColorComponents;

    /// Convert the components from sRGB linear light values to sRGB gamma
    /// encoded values.
    fn to_gamma_encoded(from: &ColorComponents) -> ColorComponents;
}

/// Convert the color components from the specified color space to XYZ and
/// return the components and the white point they are in.
pub fn to_xyz<From: ColorSpaceConversion>(from: &ColorComponents) -> (ColorComponents, WhitePoint) {
    // Convert the color components where in-gamut values are in the range
    // [0 - 1] to linear light (un-companded) form.
    let result = From::to_linear_light(from);

    // Convert the color components from the source color space to XYZ.
    (From::to_xyz(&result), From::WHITE_POINT)
}

/// Convert the color components from XYZ at the given white point to the
/// specified color space.
pub fn from_xyz<To: ColorSpaceConversion>(
    from: &ColorComponents,
    white_point: WhitePoint,
) -> ColorComponents {
    let mut xyz = from.clone();

    // Convert the white point if needed.
    convert_white_point(white_point, To::WHITE_POINT, &mut xyz);

    // Convert the color from XYZ to the target color space.
    let result = To::from_xyz(&xyz);

    // Convert the color components of linear-light values in the range
    // [0 - 1] to a gamma corrected form.
    To::to_gamma_encoded(&result)
}

/// The sRGB color space.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#predefined-sRGB
pub struct Srgb;

impl Srgb {
    #[rustfmt::skip]
    const TO_XYZ: Transform = Transform::new(
        0.4123907992659595,  0.21263900587151036, 0.01933081871559185, 0.0,
        0.35758433938387796, 0.7151686787677559,  0.11919477979462599, 0.0,
        0.1804807884018343,  0.07219231536073371, 0.9505321522496606,  0.0,
        0.0,                 0.0,                 0.0,                 1.0,
    );

    #[rustfmt::skip]
    const FROM_XYZ: Transform = Transform::new(
         3.2409699419045213, -0.9692436362808798,  0.05563007969699361, 0.0,
        -1.5373831775700935,  1.8759675015077206, -0.20397695888897657, 0.0,
        -0.4986107602930033,  0.04155505740717561, 1.0569715142428786,  0.0,
         0.0,                 0.0,                 0.0,                 1.0,
    );
}

impl ColorSpaceConversion for Srgb {
    const WHITE_POINT: WhitePoint = WhitePoint::D65;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        from.clone().map(|value| {
            let abs = value.abs();

            if abs < 0.04045 {
                value / 12.92
            } else {
                value.signum() * ((abs + 0.055) / 1.055).powf(2.4)
            }
        })
    }

    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        transform(from, &Self::TO_XYZ)
    }

    fn from_xyz(from: &ColorComponents) -> ColorComponents {
        transform(from, &Self::FROM_XYZ)
    }

    fn to_gamma_encoded(from: &ColorComponents) -> ColorComponents {
        from.clone().map(|value| {
            let abs = value.abs();

            if abs > 0.0031308 {
                value.signum() * (1.055 * abs.powf(1.0 / 2.4) - 0.055)
            } else {
                12.92 * value
            }
        })
    }
}

/// The same as sRGB color space, except the transfer function is linear light.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#predefined-sRGB-linear
pub struct SrgbLinear;

impl ColorSpaceConversion for SrgbLinear {
    const WHITE_POINT: WhitePoint = Srgb::WHITE_POINT;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        // Already in linear light form.
        from.clone()
    }

    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        Srgb::to_xyz(from)
    }

    fn from_xyz(from: &ColorComponents) -> ColorComponents {
        Srgb::from_xyz(from)
    }

    fn to_gamma_encoded(from: &ColorComponents) -> ColorComponents {
        // Stay in linear light form.
        from.clone()
    }
}

/// The Display-P3 color space.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#predefined-display-p3
pub struct DisplayP3;

impl DisplayP3 {
    #[rustfmt::skip]
    const TO_XYZ: Transform = Transform::new(
        0.48657094864821626, 0.22897456406974884, 0.0,                  0.0,
        0.26566769316909294, 0.6917385218365062,  0.045113381858902575, 0.0,
        0.1982172852343625,  0.079286914093745,   1.0439443689009757,   0.0,
        0.0,                 0.0,                 0.0,                  1.0,
    );
}

impl ColorSpaceConversion for DisplayP3 {
    const WHITE_POINT: WhitePoint = WhitePoint::D65;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        Srgb::to_linear_light(from)
    }

    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        transform(from, &Self::TO_XYZ)
    }

    fn from_xyz(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }

    fn to_gamma_encoded(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }
}

/// The a98-rgb color space.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#predefined-a98-rgb
pub struct A98Rgb;

impl A98Rgb {
    #[rustfmt::skip]
    const TO_XYZ: Transform = Transform::new(
        0.5766690429101308,  0.29734497525053616, 0.027031361386412378, 0.0,
        0.18555823790654627, 0.627363566255466,   0.07068885253582714,  0.0,
        0.18822864623499472, 0.07529145849399789, 0.9913375368376389,   0.0,
        0.0,                 0.0,                 0.0,                  1.0,
    );
}

impl ColorSpaceConversion for A98Rgb {
    const WHITE_POINT: WhitePoint = WhitePoint::D65;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        #[inline]
        fn map(value: f32) -> f32 {
            value.signum() * value.abs().powf(2.19921875)
        }

        ColorComponents(map(from.0), map(from.1), map(from.2))
    }

    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        transform(from, &Self::TO_XYZ)
    }

    fn from_xyz(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }

    fn to_gamma_encoded(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }
}

/// The ProPhoto RGB color space.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#predefined-prophoto-rgb
pub struct ProphotoRgb;

impl ProphotoRgb {
    #[rustfmt::skip]
    const TO_XYZ: Transform = Transform::new(
        0.7977604896723027,  0.2880711282292934,     0.0,                0.0,
        0.13518583717574031, 0.7118432178101014,     0.0,                0.0,
        0.0313493495815248,  0.00008565396060525902, 0.8251046025104601, 0.0,
        0.0,                 0.0,                    0.0,                1.0,
    );
}

impl ColorSpaceConversion for ProphotoRgb {
    const WHITE_POINT: WhitePoint = WhitePoint::D50;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        from.clone().map(|value| {
            const ET2: f32 = 16.0 / 512.0;

            let abs = value.abs();

            if abs <= ET2 {
                value / 16.0
            } else {
                value.signum() * abs.powf(1.8)
            }
        })
    }

    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        transform(from, &Self::TO_XYZ)
    }

    fn from_xyz(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }

    fn to_gamma_encoded(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }
}

/// The Rec.2020 color space.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#predefined-rec2020
pub struct Rec2020;

impl Rec2020 {
    #[rustfmt::skip]
    const TO_XYZ: Transform = Transform::new(
        0.6369580483012913,  0.26270021201126703,  0.0,                  0.0,
        0.14461690358620838, 0.677998071518871,    0.028072693049087508, 0.0,
        0.16888097516417205, 0.059301716469861945, 1.0609850577107909,   0.0,
        0.0,                 0.0,                  0.0,                  1.0,
    );
}

impl ColorSpaceConversion for Rec2020 {
    const WHITE_POINT: WhitePoint = WhitePoint::D65;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        from.clone().map(|value| {
            const ALPHA: f32 = 1.09929682680944;
            const BETA: f32 = 0.018053968510807;

            let abs = value.abs();

            if abs < BETA * 4.5 {
                value / 4.5
            } else {
                value.signum() * ((abs + ALPHA - 1.0) / ALPHA).powf(1.0 / 0.45)
            }
        })
    }

    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        transform(from, &Self::TO_XYZ)
    }

    fn from_xyz(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }

    fn to_gamma_encoded(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }
}

/// A color in the XYZ coordinate space with a D50 white reference.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#predefined-xyz
pub struct XyzD50;

impl ColorSpaceConversion for XyzD50 {
    const WHITE_POINT: WhitePoint = WhitePoint::D50;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        from.clone()
    }

    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        from.clone()
    }

    fn from_xyz(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }

    fn to_gamma_encoded(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }
}

/// A color in the XYZ coordinate space with a D65 white reference.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#predefined-xyz
pub struct XyzD65;

impl ColorSpaceConversion for XyzD65 {
    const WHITE_POINT: WhitePoint = WhitePoint::D65;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        from.clone()
    }

    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        from.clone()
    }

    fn from_xyz(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }

    fn to_gamma_encoded(_from: &ColorComponents) -> ColorComponents {
        todo!()
    }
}

/// The Lab color space.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#specifying-lab-lch
pub struct Lab;

impl Lab {
    const KAPPA: f32 = 24389.0 / 27.0;
    const EPSILON: f32 = 216.0 / 24389.0;
    const WHITE: ColorComponents = ColorComponents(0.96422, 1.0, 0.82521);
}

impl ColorSpaceConversion for Lab {
    const WHITE_POINT: WhitePoint = WhitePoint::D50;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        // No need for conversion.
        from.clone()
    }

    /// Convert a CIELAB color to XYZ as specified in [1] and [2].
    ///
    /// [1]: https://drafts.csswg.org/css-color/#lab-to-predefined
    /// [2]: https://drafts.csswg.org/css-color/#color-conversion-code
    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        let f1 = (from.0 + 16.0) / 116.0;
        let f0 = (from.1 / 500.0) + f1;
        let f2 = f1 - from.2 / 200.0;

        let x = if f0.powf(3.0) > Self::EPSILON {
            f0.powf(3.)
        } else {
            (116.0 * f0 - 16.0) / Self::KAPPA
        };
        let y = if from.0 > Self::KAPPA * Self::EPSILON {
            ((from.0 + 16.0) / 116.0).powf(3.0)
        } else {
            from.0 / Self::KAPPA
        };
        let z = if f2.powf(3.0) > Self::EPSILON {
            f2.powf(3.0)
        } else {
            (116.0 * f2 - 16.0) / Self::KAPPA
        };

        ColorComponents(x * Self::WHITE.0, y * Self::WHITE.1, z * Self::WHITE.2)
    }

    /// Convert an XYZ colour to LAB as specified in [1] and [2].
    ///
    /// [1]: https://drafts.csswg.org/css-color/#rgb-to-lab
    /// [2]: https://drafts.csswg.org/css-color/#color-conversion-code
    fn from_xyz(from: &ColorComponents) -> ColorComponents {
        macro_rules! compute_f {
            ($value:expr) => {{
                if $value > Self::EPSILON {
                    $value.cbrt()
                } else {
                    (Self::KAPPA * $value + 16.0) / 116.0
                }
            }};
        }

        // 4. Convert D50-adapted XYZ to Lab.
        let f = [
            compute_f!(from.0 / Self::WHITE.0),
            compute_f!(from.1 / Self::WHITE.1),
            compute_f!(from.2 / Self::WHITE.2),
        ];

        let lightness = 116.0 * f[1] - 16.0;
        let a = 500.0 * (f[0] - f[1]);
        let b = 200.0 * (f[1] - f[2]);

        ColorComponents(lightness, a, b)
    }

    fn to_gamma_encoded(from: &ColorComponents) -> ColorComponents {
        // No need for conversion.
        from.clone()
    }
}

/// The Lch color space.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#specifying-lab-lch
pub struct Lch;

impl ColorSpaceConversion for Lch {
    const WHITE_POINT: WhitePoint = Lab::WHITE_POINT;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        // No need for conversion.
        from.clone()
    }

    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        // Convert LCH to Lab first.
        let hue = from.2 * RAD_PER_DEG;
        let a = from.1 * hue.cos();
        let b = from.1 * hue.sin();

        let lab = ColorComponents(from.0, a, b);

        // Then convert the Lab to XYZ.
        Lab::to_xyz(&lab)
    }

    fn from_xyz(from: &ColorComponents) -> ColorComponents {
        // First convert the XYZ to LAB.
        let ColorComponents(lightness, a, b) = Lab::from_xyz(&from);

        // Then conver the Lab to LCH.
        let hue = b.atan2(a) * DEG_PER_RAD;
        let chroma = (a * a + b * b).sqrt();

        ColorComponents(lightness, chroma, hue)
    }

    fn to_gamma_encoded(from: &ColorComponents) -> ColorComponents {
        // No need for conversion.
        from.clone()
    }
}

/// The Oklab color space.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#specifying-oklab-oklch
pub struct Oklab;

impl Oklab {
    #[rustfmt::skip]
    const XYZ_TO_LMS: Transform = Transform::new(
         0.8190224432164319,  0.0329836671980271,  0.048177199566046255, 0.0,
         0.3619062562801221,  0.9292868468965546,  0.26423952494422764,  0.0,
        -0.12887378261216414, 0.03614466816999844, 0.6335478258136937,   0.0,
         0.0,                 0.0,                 0.0,                  1.0,
    );

    #[rustfmt::skip]
    const LMS_TO_OKLAB: Transform = Transform::new(
         0.2104542553,  1.9779984951,  0.0259040371, 0.0,
         0.7936177850, -2.4285922050,  0.7827717662, 0.0,
        -0.0040720468,  0.4505937099, -0.8086757660, 0.0,
         0.0,           0.0,           0.0,          1.0,
    );

    #[rustfmt::skip]
    const LMS_TO_XYZ: Transform = Transform::new(
         1.2268798733741557,  -0.04057576262431372, -0.07637294974672142, 0.0,
        -0.5578149965554813,   1.1122868293970594,  -0.4214933239627914,  0.0,
         0.28139105017721583, -0.07171106666151701,  1.5869240244272418,  0.0,
         0.0,                  0.0,                  0.0,                 1.0,
    );

    #[rustfmt::skip]
    const OKLAB_TO_LMS: Transform = Transform::new(
        0.99999999845051981432,  1.0000000088817607767,    1.0000000546724109177,   0.0,
        0.39633779217376785678, -0.1055613423236563494,   -0.089484182094965759684, 0.0,
        0.21580375806075880339, -0.063854174771705903402, -1.2914855378640917399,   0.0,
        0.0,                     0.0,                      0.0,                     1.0,
    );
}

impl ColorSpaceConversion for Oklab {
    const WHITE_POINT: WhitePoint = WhitePoint::D65;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        // No need for conversion.
        from.clone()
    }

    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        let lms = transform(&from, &Self::OKLAB_TO_LMS);
        let lms = lms.map(|v| v.powf(3.0));
        transform(&lms, &Self::LMS_TO_XYZ)
    }

    fn from_xyz(from: &ColorComponents) -> ColorComponents {
        let lms = transform(&from, &Self::XYZ_TO_LMS);
        let lms = lms.map(|v| v.cbrt());
        transform(&lms, &Self::LMS_TO_OKLAB)
    }

    fn to_gamma_encoded(from: &ColorComponents) -> ColorComponents {
        // No need for conversion.
        from.clone()
    }
}

/// The Oklch color space.
/// https://drafts.csswg.org/csswg-drafts/css-color-4/#specifying-oklab-oklch

pub struct Oklch;

impl ColorSpaceConversion for Oklch {
    const WHITE_POINT: WhitePoint = Oklab::WHITE_POINT;

    fn to_linear_light(from: &ColorComponents) -> ColorComponents {
        // No need for conversion.
        from.clone()
    }

    fn to_xyz(from: &ColorComponents) -> ColorComponents {
        // First convert OkLCH to Oklab.
        let hue = from.2 * RAD_PER_DEG;
        let a = from.1 * hue.cos();
        let b = from.1 * hue.sin();
        let oklab = ColorComponents(from.0, a, b);

        // Then convert Oklab to XYZ.
        Oklab::to_xyz(&oklab)
    }

    fn from_xyz(from: &ColorComponents) -> ColorComponents {
        // First convert XYZ to Oklab.
        let ColorComponents(lightness, a, b) = Oklab::from_xyz(&from);

        // Then convert Oklab to OkLCH.
        let hue = b.atan2(a) * DEG_PER_RAD;
        let chroma = (a * a + b * b).sqrt();

        ColorComponents(lightness, chroma, hue)
    }

    fn to_gamma_encoded(from: &ColorComponents) -> ColorComponents {
        // No need for conversion.
        from.clone()
    }
}
