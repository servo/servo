/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Color support functions.

/// cbindgen:ignore
pub mod convert;
pub mod mix;

use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// The 3 components that make up a color.  (Does not include the alpha component)
#[derive(Copy, Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub struct ColorComponents(pub f32, pub f32, pub f32);

impl ColorComponents {
    /// Apply a function to each of the 3 components of the color.
    pub fn map(self, f: impl Fn(f32) -> f32) -> Self {
        Self(f(self.0), f(self.1), f(self.2))
    }
}

/// A color space representation in the CSS specification.
///
/// https://drafts.csswg.org/css-color-4/#typedef-color-space
///
/// NOTE: Right now HSL and HWB colors can not be constructed by the user. They
///       are converted to RGB in the parser. The parser should return the
///       HSL/HWB values as is to avoid unnescessary conversions to/from RGB.
///       See: https://bugzilla.mozilla.org/show_bug.cgi?id=1817035
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
pub enum ColorSpace {
    /// A color specified in the Hsl notation in the sRGB color space, e.g.
    /// "hsl(289.18 93.136% 65.531%)"
    /// https://drafts.csswg.org/css-color-4/#the-hsl-notation
    Hsl,
    /// A color specified in the Hwb notation in the sRGB color space, e.g.
    /// "hwb(740deg 20% 30%)"
    /// https://drafts.csswg.org/css-color-4/#the-hwb-notation
    Hwb,
    /// A color specified in the Lab color format, e.g.
    /// "lab(29.2345% 39.3825 20.0664)".
    /// https://w3c.github.io/csswg-drafts/css-color-4/#lab-colors
    Lab,
    /// A color specified in the Lch color format, e.g.
    /// "lch(29.2345% 44.2 27)".
    /// https://w3c.github.io/csswg-drafts/css-color-4/#lch-colors
    Lch,
    /// A color specified in the Oklab color format, e.g.
    /// "oklab(40.101% 0.1147 0.0453)".
    /// https://w3c.github.io/csswg-drafts/css-color-4/#lab-colors
    Oklab,
    /// A color specified in the Oklch color format, e.g.
    /// "oklch(40.101% 0.12332 21.555)".
    /// https://w3c.github.io/csswg-drafts/css-color-4/#lch-colors
    Oklch,
    /// A color specified with the color(..) function and the "srgb" color
    /// space, e.g. "color(srgb 0.691 0.139 0.259)".
    Srgb,
    /// A color specified with the color(..) function and the "srgb-linear"
    /// color space, e.g. "color(srgb-linear 0.435 0.017 0.055)".
    SrgbLinear,
    /// A color specified with the color(..) function and the "display-p3"
    /// color space, e.g. "color(display-p3 0.84 0.19 0.72)".
    DisplayP3,
    /// A color specified with the color(..) function and the "a98-rgb" color
    /// space, e.g. "color(a98-rgb 0.44091 0.49971 0.37408)".
    A98Rgb,
    /// A color specified with the color(..) function and the "prophoto-rgb"
    /// color space, e.g. "color(prophoto-rgb 0.36589 0.41717 0.31333)".
    ProphotoRgb,
    /// A color specified with the color(..) function and the "rec2020" color
    /// space, e.g. "color(rec2020 0.42210 0.47580 0.35605)".
    Rec2020,
    /// A color specified with the color(..) function and the "xyz-d50" color
    /// space, e.g. "color(xyz-d50 0.2005 0.14089 0.4472)".
    XyzD50,
    /// A color specified with the color(..) function and the "xyz-d65" or "xyz"
    /// color space, e.g. "color(xyz-d65 0.21661 0.14602 0.59452)".
    #[parse(aliases = "xyz")]
    XyzD65,
}

impl ColorSpace {
    /// Returns whether this is a `<rectangular-color-space>`.
    #[inline]
    pub fn is_rectangular(&self) -> bool {
        !self.is_polar()
    }

    /// Returns whether this is a `<polar-color-space>`.
    #[inline]
    pub fn is_polar(&self) -> bool {
        matches!(self, Self::Hsl | Self::Hwb | Self::Lch | Self::Oklch)
    }
}

bitflags! {
    /// Flags used when serializing colors.
    #[derive(Default, MallocSizeOf, ToShmem)]
    #[repr(C)]
    pub struct SerializationFlags : u8 {
        /// If set, serializes sRGB colors into `color(srgb ...)` instead of
        /// `rgba(...)`.
        const AS_COLOR_FUNCTION = 0x01;
    }
}

/// An absolutely specified color, using either rgb(), rgba(), lab(), lch(),
/// oklab(), oklch() or color().
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
#[repr(C)]
pub struct AbsoluteColor {
    /// The 3 components that make up colors in any color space.
    pub components: ColorComponents,
    /// The alpha component of the color.
    pub alpha: f32,
    /// The current color space that the components represent.
    pub color_space: ColorSpace,
    /// Extra flags used durring serialization of this color.
    pub flags: SerializationFlags,
}

/// Given an [`AbsoluteColor`], return the 4 float components as the type given,
/// e.g.:
///
/// ```rust
/// let srgb = AbsoluteColor::new(ColorSpace::Srgb, 1.0, 0.0, 0.0, 0.0);
/// let floats = color_components_as!(&srgb, [f32; 4]); // [1.0, 0.0, 0.0, 0.0]
/// ```
#[macro_export]
macro_rules! color_components_as {
    ($c:expr, $t:ty) => {{
        // This macro is not an inline function, because we can't use the
        // generic  type ($t) in a constant expression as per:
        // https://github.com/rust-lang/rust/issues/76560
        const_assert_eq!(std::mem::size_of::<$t>(), std::mem::size_of::<[f32; 4]>());
        const_assert_eq!(std::mem::align_of::<$t>(), std::mem::align_of::<[f32; 4]>());
        const_assert!(std::mem::size_of::<AbsoluteColor>() >= std::mem::size_of::<$t>());
        const_assert_eq!(
            std::mem::align_of::<AbsoluteColor>(),
            std::mem::align_of::<$t>()
        );

        std::mem::transmute::<&ColorComponents, &$t>(&$c.components)
    }};
}

impl AbsoluteColor {
    /// Create a new [AbsoluteColor] with the given [ColorSpace] and components.
    pub fn new(color_space: ColorSpace, components: ColorComponents, alpha: f32) -> Self {
        Self {
            components,
            alpha,
            color_space,
            flags: SerializationFlags::empty(),
        }
    }

    /// Return the alpha component.
    #[inline]
    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    /// Convert this color to the specified color space.
    pub fn to_color_space(&self, color_space: ColorSpace) -> Self {
        use ColorSpace::*;

        if self.color_space == color_space {
            return self.clone();
        }

        let (xyz, white_point) = match self.color_space {
            Hsl => {
                let rgb = convert::hsl_to_rgb(&self.components);
                convert::to_xyz::<convert::Srgb>(&rgb)
            },
            Hwb => {
                let rgb = convert::hwb_to_rgb(&self.components);
                convert::to_xyz::<convert::Srgb>(&rgb)
            },
            Lab => convert::to_xyz::<convert::Lab>(&self.components),
            Lch => convert::to_xyz::<convert::Lch>(&self.components),
            Oklab => convert::to_xyz::<convert::Oklab>(&self.components),
            Oklch => convert::to_xyz::<convert::Oklch>(&self.components),
            Srgb => convert::to_xyz::<convert::Srgb>(&self.components),
            SrgbLinear => convert::to_xyz::<convert::SrgbLinear>(&self.components),
            DisplayP3 => convert::to_xyz::<convert::DisplayP3>(&self.components),
            A98Rgb => convert::to_xyz::<convert::A98Rgb>(&self.components),
            ProphotoRgb => convert::to_xyz::<convert::ProphotoRgb>(&self.components),
            Rec2020 => convert::to_xyz::<convert::Rec2020>(&self.components),
            XyzD50 => convert::to_xyz::<convert::XyzD50>(&self.components),
            XyzD65 => convert::to_xyz::<convert::XyzD65>(&self.components),
        };

        let result = match color_space {
            Hsl => {
                let rgb = convert::from_xyz::<convert::Srgb>(&xyz, white_point);
                convert::rgb_to_hsl(&rgb)
            },
            Hwb => {
                let rgb = convert::from_xyz::<convert::Srgb>(&xyz, white_point);
                convert::rgb_to_hwb(&rgb)
            },
            Lab => convert::from_xyz::<convert::Lab>(&xyz, white_point),
            Lch => convert::from_xyz::<convert::Lch>(&xyz, white_point),
            Oklab => convert::from_xyz::<convert::Oklab>(&xyz, white_point),
            Oklch => convert::from_xyz::<convert::Oklch>(&xyz, white_point),
            Srgb => convert::from_xyz::<convert::Srgb>(&xyz, white_point),
            SrgbLinear => convert::from_xyz::<convert::SrgbLinear>(&xyz, white_point),
            DisplayP3 => convert::from_xyz::<convert::DisplayP3>(&xyz, white_point),
            A98Rgb => convert::from_xyz::<convert::A98Rgb>(&xyz, white_point),
            ProphotoRgb => convert::from_xyz::<convert::ProphotoRgb>(&xyz, white_point),
            Rec2020 => convert::from_xyz::<convert::Rec2020>(&xyz, white_point),
            XyzD50 => convert::from_xyz::<convert::XyzD50>(&xyz, white_point),
            XyzD65 => convert::from_xyz::<convert::XyzD65>(&xyz, white_point),
        };

        Self::new(color_space, result, self.alpha)
    }
}

impl From<cssparser::PredefinedColorSpace> for ColorSpace {
    fn from(value: cssparser::PredefinedColorSpace) -> Self {
        match value {
            cssparser::PredefinedColorSpace::Srgb => ColorSpace::Srgb,
            cssparser::PredefinedColorSpace::SrgbLinear => ColorSpace::SrgbLinear,
            cssparser::PredefinedColorSpace::DisplayP3 => ColorSpace::DisplayP3,
            cssparser::PredefinedColorSpace::A98Rgb => ColorSpace::A98Rgb,
            cssparser::PredefinedColorSpace::ProphotoRgb => ColorSpace::ProphotoRgb,
            cssparser::PredefinedColorSpace::Rec2020 => ColorSpace::Rec2020,
            cssparser::PredefinedColorSpace::XyzD50 => ColorSpace::XyzD50,
            cssparser::PredefinedColorSpace::XyzD65 => ColorSpace::XyzD65,
        }
    }
}

impl From<cssparser::AbsoluteColor> for AbsoluteColor {
    fn from(f: cssparser::AbsoluteColor) -> Self {
        match f {
            cssparser::AbsoluteColor::Rgba(rgba) => Self::from_rgba(rgba),

            cssparser::AbsoluteColor::Lab(lab) => Self::new(
                ColorSpace::Lab,
                ColorComponents(lab.lightness, lab.a, lab.b),
                lab.alpha,
            ),

            cssparser::AbsoluteColor::Lch(lch) => Self::new(
                ColorSpace::Lch,
                ColorComponents(lch.lightness, lch.chroma, lch.hue),
                lch.alpha,
            ),

            cssparser::AbsoluteColor::Oklab(oklab) => Self::new(
                ColorSpace::Oklab,
                ColorComponents(oklab.lightness, oklab.a, oklab.b),
                oklab.alpha,
            ),

            cssparser::AbsoluteColor::Oklch(oklch) => Self::new(
                ColorSpace::Oklch,
                ColorComponents(oklch.lightness, oklch.chroma, oklch.hue),
                oklch.alpha,
            ),

            cssparser::AbsoluteColor::ColorFunction(c) => {
                let mut result = AbsoluteColor::new(
                    c.color_space.into(),
                    ColorComponents(c.c1, c.c2, c.c3),
                    c.alpha,
                );

                if matches!(c.color_space, cssparser::PredefinedColorSpace::Srgb) {
                    result.flags |= SerializationFlags::AS_COLOR_FUNCTION;
                }

                result
            },
        }
    }
}

impl ToCss for AbsoluteColor {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match self.color_space {
            ColorSpace::Hsl => {
                let rgb = convert::hsl_to_rgb(&self.components);
                Self::new(ColorSpace::Srgb, rgb, self.alpha).to_css(dest)
            },

            ColorSpace::Hwb => {
                let rgb = convert::hwb_to_rgb(&self.components);

                Self::new(ColorSpace::Srgb, rgb, self.alpha).to_css(dest)
            },

            ColorSpace::Srgb if !self.flags.contains(SerializationFlags::AS_COLOR_FUNCTION) => {
                cssparser::ToCss::to_css(
                    &cssparser::RGBA::from_floats(
                        self.components.0,
                        self.components.1,
                        self.components.2,
                        self.alpha(),
                    ),
                    dest,
                )
            },
            ColorSpace::Lab => cssparser::ToCss::to_css(
                unsafe { color_components_as!(self, cssparser::Lab) },
                dest,
            ),
            ColorSpace::Lch => cssparser::ToCss::to_css(
                unsafe { color_components_as!(self, cssparser::Lch) },
                dest,
            ),
            ColorSpace::Oklab => cssparser::ToCss::to_css(
                unsafe { color_components_as!(self, cssparser::Oklab) },
                dest,
            ),
            ColorSpace::Oklch => cssparser::ToCss::to_css(
                unsafe { color_components_as!(self, cssparser::Oklch) },
                dest,
            ),
            _ => {
                let color_space = match self.color_space {
                    ColorSpace::Srgb => {
                        debug_assert!(
                            self.flags.contains(SerializationFlags::AS_COLOR_FUNCTION),
                             "The case without this flag should be handled in the wrapping match case!!"
                          );

                        cssparser::PredefinedColorSpace::Srgb
                    },
                    ColorSpace::SrgbLinear => cssparser::PredefinedColorSpace::SrgbLinear,
                    ColorSpace::DisplayP3 => cssparser::PredefinedColorSpace::DisplayP3,
                    ColorSpace::A98Rgb => cssparser::PredefinedColorSpace::A98Rgb,
                    ColorSpace::ProphotoRgb => cssparser::PredefinedColorSpace::ProphotoRgb,
                    ColorSpace::Rec2020 => cssparser::PredefinedColorSpace::Rec2020,
                    ColorSpace::XyzD50 => cssparser::PredefinedColorSpace::XyzD50,
                    ColorSpace::XyzD65 => cssparser::PredefinedColorSpace::XyzD65,

                    _ => {
                        unreachable!("other color spaces do not support color() syntax")
                    },
                };

                let color_function = cssparser::ColorFunction {
                    color_space,
                    c1: self.components.0,
                    c2: self.components.1,
                    c3: self.components.2,
                    alpha: self.alpha,
                };
                let color = cssparser::AbsoluteColor::ColorFunction(color_function);
                cssparser::ToCss::to_css(&color, dest)
            },
        }
    }
}
