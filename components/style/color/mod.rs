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
#[repr(C)]
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
    /// A color specified in the sRGB color space with either the rgb/rgba(..)
    /// functions or the newer color(srgb ..) function. If the color(..)
    /// function is used, the AS_COLOR_FUNCTION flag will be set. Examples:
    /// "color(srgb 0.691 0.139 0.259)", "rgb(176, 35, 66)"
    Srgb = 0,
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
    /// NOTE: https://drafts.csswg.org/css-color-4/#resolving-color-function-values
    ///       specifies that `xyz` is an alias for the `xyz-d65` color space.
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

    /// Returns an index of the hue component in the color space, otherwise
    /// `None`.
    #[inline]
    pub fn hue_index(&self) -> Option<usize> {
        match self {
            Self::Hsl | Self::Hwb => Some(0),
            Self::Lch | Self::Oklch => Some(2),

            _ => {
                debug_assert!(!self.is_polar());
                None
            },
        }
    }
}

bitflags! {
    /// Flags used when serializing colors.
    #[derive(Default, MallocSizeOf, ToShmem)]
    #[repr(C)]
    pub struct ColorFlags : u8 {
        /// If set, serializes sRGB colors into `color(srgb ...)` instead of
        /// `rgba(...)`.
        const AS_COLOR_FUNCTION = 1 << 0;
        /// Whether the 1st color component is `none`.
        const C1_IS_NONE = 1 << 1;
        /// Whether the 2nd color component is `none`.
        const C2_IS_NONE = 1 << 2;
        /// Whether the 3rd color component is `none`.
        const C3_IS_NONE = 1 << 3;
        /// Whether the alpha component is `none`.
        const ALPHA_IS_NONE = 1 << 4;
    }
}

/// An absolutely specified color, using either rgb(), rgba(), lab(), lch(),
/// oklab(), oklch() or color().
#[derive(Copy, Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
#[repr(C)]
pub struct AbsoluteColor {
    /// The 3 components that make up colors in any color space.
    pub components: ColorComponents,
    /// The alpha component of the color.
    pub alpha: f32,
    /// The current color space that the components represent.
    pub color_space: ColorSpace,
    /// Extra flags used durring serialization of this color.
    pub flags: ColorFlags,
}

/// Given an [`AbsoluteColor`], return the 4 float components as the type given,
/// e.g.:
///
/// ```rust
/// let srgb = AbsoluteColor::new(ColorSpace::Srgb, 1.0, 0.0, 0.0, 0.0);
/// let floats = color_components_as!(&srgb, [f32; 4]); // [1.0, 0.0, 0.0, 0.0]
/// ```
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
    /// Create a new [`AbsoluteColor`] with the given [`ColorSpace`] and
    /// components.
    pub fn new(color_space: ColorSpace, components: ColorComponents, alpha: f32) -> Self {
        let mut components = components;

        // Lightness must not be less than 0.
        if matches!(
            color_space,
            ColorSpace::Lab | ColorSpace::Lch | ColorSpace::Oklab | ColorSpace::Oklch
        ) {
            components.0 = components.0.max(0.0);
        }

        // Chroma must not be less than 0.
        if matches!(color_space, ColorSpace::Lch | ColorSpace::Oklch) {
            components.1 = components.1.max(0.0);
        }

        Self {
            components,
            alpha: alpha.clamp(0.0, 1.0),
            color_space,
            flags: ColorFlags::empty(),
        }
    }

    /// Create a new [`AbsoluteColor`] from rgba values in the sRGB color space.
    pub fn srgb(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self::new(ColorSpace::Srgb, ColorComponents(red, green, blue), alpha)
    }

    /// Create a new transparent color.
    pub fn transparent() -> Self {
        Self::srgb(0.0, 0.0, 0.0, 0.0)
    }

    /// Create a new opaque black color.
    pub fn black() -> Self {
        Self::srgb(0.0, 0.0, 0.0, 1.0)
    }

    /// Create a new opaque white color.
    pub fn white() -> Self {
        Self::srgb(1.0, 1.0, 1.0, 1.0)
    }

    /// Return all the components of the color in an array.  (Includes alpha)
    #[inline]
    pub fn raw_components(&self) -> &[f32; 4] {
        unsafe { color_components_as!(self, [f32; 4]) }
    }

    /// Returns true if this color is in one of the legacy color formats.
    #[inline]
    pub fn is_legacy_color(&self) -> bool {
        // rgb(), rgba(), hsl(), hsla(), hwb(), hwba()
        match self.color_space {
            ColorSpace::Srgb => !self.flags.contains(ColorFlags::AS_COLOR_FUNCTION),
            ColorSpace::Hsl | ColorSpace::Hwb => true,
            _ => false,
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

        // We have simplified conversions that do not need to convert to XYZ
        // first.  This improves performance, because it skips 2 matrix
        // multiplications and reduces float rounding errors.
        match (self.color_space, color_space) {
            (Srgb, Hsl) => {
                return Self::new(
                    color_space,
                    convert::rgb_to_hsl(&self.components),
                    self.alpha,
                );
            },

            (Srgb, Hwb) => {
                return Self::new(
                    color_space,
                    convert::rgb_to_hwb(&self.components),
                    self.alpha,
                );
            },

            (Hsl, Srgb) => {
                return Self::new(
                    color_space,
                    convert::hsl_to_rgb(&self.components),
                    self.alpha,
                );
            },

            (Hwb, Srgb) => {
                return Self::new(
                    color_space,
                    convert::hwb_to_rgb(&self.components),
                    self.alpha,
                );
            },

            (Lab, Lch) | (Oklab, Oklch) => {
                return Self::new(
                    color_space,
                    convert::lab_to_lch(&self.components),
                    self.alpha,
                );
            },

            (Lch, Lab) | (Oklch, Oklab) => {
                return Self::new(
                    color_space,
                    convert::lch_to_lab(&self.components),
                    self.alpha,
                );
            },

            _ => {},
        }

        let (xyz, white_point) = match self.color_space {
            Lab => convert::to_xyz::<convert::Lab>(&self.components),
            Lch => convert::to_xyz::<convert::Lch>(&self.components),
            Oklab => convert::to_xyz::<convert::Oklab>(&self.components),
            Oklch => convert::to_xyz::<convert::Oklch>(&self.components),
            Srgb => convert::to_xyz::<convert::Srgb>(&self.components),
            Hsl => convert::to_xyz::<convert::Hsl>(&self.components),
            Hwb => convert::to_xyz::<convert::Hwb>(&self.components),
            SrgbLinear => convert::to_xyz::<convert::SrgbLinear>(&self.components),
            DisplayP3 => convert::to_xyz::<convert::DisplayP3>(&self.components),
            A98Rgb => convert::to_xyz::<convert::A98Rgb>(&self.components),
            ProphotoRgb => convert::to_xyz::<convert::ProphotoRgb>(&self.components),
            Rec2020 => convert::to_xyz::<convert::Rec2020>(&self.components),
            XyzD50 => convert::to_xyz::<convert::XyzD50>(&self.components),
            XyzD65 => convert::to_xyz::<convert::XyzD65>(&self.components),
        };

        let result = match color_space {
            Lab => convert::from_xyz::<convert::Lab>(&xyz, white_point),
            Lch => convert::from_xyz::<convert::Lch>(&xyz, white_point),
            Oklab => convert::from_xyz::<convert::Oklab>(&xyz, white_point),
            Oklch => convert::from_xyz::<convert::Oklch>(&xyz, white_point),
            Srgb => convert::from_xyz::<convert::Srgb>(&xyz, white_point),
            Hsl => convert::from_xyz::<convert::Hsl>(&xyz, white_point),
            Hwb => convert::from_xyz::<convert::Hwb>(&xyz, white_point),
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

impl ToCss for AbsoluteColor {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        macro_rules! value_or_none {
            ($v:expr,$flag:tt) => {{
                if self.flags.contains(ColorFlags::$flag) {
                    None
                } else {
                    Some($v)
                }
            }};
        }

        let maybe_c1 = value_or_none!(self.components.0, C1_IS_NONE);
        let maybe_c2 = value_or_none!(self.components.1, C2_IS_NONE);
        let maybe_c3 = value_or_none!(self.components.2, C3_IS_NONE);
        let maybe_alpha = value_or_none!(self.alpha, ALPHA_IS_NONE);

        match self.color_space {
            ColorSpace::Hsl => {
                let rgb = convert::hsl_to_rgb(&self.components);
                Self::new(ColorSpace::Srgb, rgb, self.alpha).to_css(dest)
            },

            ColorSpace::Hwb => {
                let rgb = convert::hwb_to_rgb(&self.components);

                Self::new(ColorSpace::Srgb, rgb, self.alpha).to_css(dest)
            },

            ColorSpace::Srgb if !self.flags.contains(ColorFlags::AS_COLOR_FUNCTION) => {
                // Althought we are passing Option<_> in here, the to_css fn
                // knows that the "none" keyword is not supported in the
                // rgb/rgba legacy syntax.
                cssparser::ToCss::to_css(
                    &cssparser::RGBA::from_floats(maybe_c1, maybe_c2, maybe_c3, maybe_alpha),
                    dest,
                )
            },
            ColorSpace::Lab => cssparser::ToCss::to_css(
                &cssparser::Lab::new(maybe_c1, maybe_c2, maybe_c3, maybe_alpha),
                dest,
            ),
            ColorSpace::Lch => cssparser::ToCss::to_css(
                &cssparser::Lch::new(maybe_c1, maybe_c2, maybe_c3, maybe_alpha),
                dest,
            ),
            ColorSpace::Oklab => cssparser::ToCss::to_css(
                &cssparser::Oklab::new(maybe_c1, maybe_c2, maybe_c3, maybe_alpha),
                dest,
            ),
            ColorSpace::Oklch => cssparser::ToCss::to_css(
                &cssparser::Oklch::new(maybe_c1, maybe_c2, maybe_c3, maybe_alpha),
                dest,
            ),
            _ => {
                let color_space = match self.color_space {
                    ColorSpace::Srgb => {
                        debug_assert!(
                            self.flags.contains(ColorFlags::AS_COLOR_FUNCTION),
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
                    c1: maybe_c1,
                    c2: maybe_c2,
                    c3: maybe_c3,
                    alpha: maybe_alpha,
                };
                let color = cssparser::Color::ColorFunction(color_function);
                cssparser::ToCss::to_css(&color, dest)
            },
        }
    }
}
