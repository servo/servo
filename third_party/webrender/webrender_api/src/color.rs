/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use peek_poke::PeekPoke;
use std::cmp;
use std::hash::{Hash, Hasher};

/// Represents pre-multiplied RGBA colors with floating point numbers.
///
/// All components must be between 0.0 and 1.0.
/// An alpha value of 1.0 is opaque while 0.0 is fully transparent.
///
/// In premultiplied colors transitions to transparent always look "nice"
/// therefore they are used in CSS gradients.
#[repr(C)]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct PremultipliedColorF {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[allow(missing_docs)]
impl PremultipliedColorF {
    pub const BLACK: PremultipliedColorF = PremultipliedColorF { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: PremultipliedColorF = PremultipliedColorF { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const WHITE: PremultipliedColorF = PremultipliedColorF { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// Represents RGBA screen colors with floating point numbers.
///
/// All components must be between 0.0 and 1.0.
/// An alpha value of 1.0 is opaque while 0.0 is fully transparent.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize, PeekPoke)]
pub struct ColorF {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[allow(missing_docs)]
impl ColorF {
    pub const BLACK: ColorF = ColorF { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: ColorF = ColorF { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const WHITE: ColorF = ColorF { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    /// Constructs a new `ColorF` from its components.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        ColorF { r, g, b, a }
    }

    /// Multiply the RGB channels (but not alpha) with a given factor.
    pub fn scale_rgb(&self, scale: f32) -> Self {
        ColorF {
            r: self.r * scale,
            g: self.g * scale,
            b: self.b * scale,
            a: self.a,
        }
    }

    // Scale the alpha by a given factor.
    pub fn scale_alpha(&self, scale: f32) -> Self {
        ColorF {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a * scale,
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Multiply the RGB components with the alpha channel.
    pub fn premultiplied(&self) -> PremultipliedColorF {
        let c = self.scale_rgb(self.a);
        PremultipliedColorF { r: c.r, g: c.g, b: c.b, a: c.a }
    }
}

// Floats don't impl Hash/Eq/Ord...
impl Eq for PremultipliedColorF {}
impl Ord for PremultipliedColorF {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).unwrap_or(cmp::Ordering::Equal)
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::derive_hash_xor_eq))]
impl Hash for PremultipliedColorF {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Note: this is inconsistent with the Eq impl for -0.0 (don't care).
        self.r.to_bits().hash(state);
        self.g.to_bits().hash(state);
        self.b.to_bits().hash(state);
        self.a.to_bits().hash(state);
    }
}

/// Represents RGBA screen colors with one byte per channel.
///
/// If the alpha value `a` is 255 the color is opaque.
#[repr(C)]
#[derive(Clone, Copy, Hash, Eq, Debug, Deserialize, MallocSizeOf, PartialEq, PartialOrd, Ord, Serialize)]
pub struct ColorU {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorU {
    /// Constructs a new additive `ColorU` from its components.
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        ColorU { r, g, b, a }
    }
}

fn round_to_int(x: f32) -> u8 {
    debug_assert!((0.0 <= x) && (x <= 1.0), "{} should be between 0 and 1", x);
    let f = (255.0 * x) + 0.5;
    let val = f.floor();
    debug_assert!(val <= 255.0);
    val as u8
}

// TODO: We shouldn't really convert back to `ColorU` ever,
// since it's lossy. One of the blockers is that all of our debug colors
// are specified in `ColorF`. Changing it to `ColorU` would be nice.
impl From<ColorF> for ColorU {
    fn from(color: ColorF) -> Self {
        ColorU {
            r: round_to_int(color.r),
            g: round_to_int(color.g),
            b: round_to_int(color.b),
            a: round_to_int(color.a),
        }
    }
}

impl From<ColorU> for ColorF {
    fn from(color: ColorU) -> Self {
        ColorF {
            r: color.r as f32 / 255.0,
            g: color.g as f32 / 255.0,
            b: color.b as f32 / 255.0,
            a: color.a as f32 / 255.0,
        }
    }
}
