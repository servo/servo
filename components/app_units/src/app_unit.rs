/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::num::Zero;
use rustc_serialize::{Encodable, Encoder};
use std::default::Default;
use std::fmt;
use std::i32;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

/// The number of app units in a pixel.
pub const AU_PER_PX: i32 = 60;

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
pub struct Au(pub i32);

impl Default for Au {
    #[inline]
    fn default() -> Au {
        Au(0)
    }
}

impl Zero for Au {
    #[inline]
    fn zero() -> Au {
        Au(0)
    }
}

pub const MIN_AU: Au = Au(i32::MIN);
pub const MAX_AU: Au = Au(i32::MAX);

impl Encodable for Au {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_f64(self.to_f64_px())
    }
}

impl fmt::Debug for Au {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}px", self.to_f64_px())
    }
}

impl Add for Au {
    type Output = Au;

    #[inline]
    fn add(self, other: Au) -> Au {
        Au(self.0.wrapping_add(other.0))
    }
}

impl Sub for Au {
    type Output = Au;

    #[inline]
    fn sub(self, other: Au) -> Au {
        Au(self.0.wrapping_sub(other.0))
    }

}

impl Mul<i32> for Au {
    type Output = Au;

    #[inline]
    fn mul(self, other: i32) -> Au {
        Au(self.0.wrapping_mul(other))
    }
}

impl Div<i32> for Au {
    type Output = Au;

    #[inline]
    fn div(self, other: i32) -> Au {
        Au(self.0 / other)
    }
}

impl Rem<i32> for Au {
    type Output = Au;

    #[inline]
    fn rem(self, other: i32) -> Au {
        Au(self.0 % other)
    }
}

impl Neg for Au {
    type Output = Au;

    #[inline]
    fn neg(self) -> Au {
        Au(-self.0)
    }
}

impl Au {
    /// FIXME(pcwalton): Workaround for lack of cross crate inlining of newtype structs!
    #[inline]
    pub fn new(value: i32) -> Au {
        Au(value)
    }

    #[inline]
    pub fn scale_by(self, factor: f32) -> Au {
        Au(((self.0 as f32) * factor) as i32)
    }

    #[inline]
    pub fn from_px(px: i32) -> Au {
        Au((px * AU_PER_PX) as i32)
    }

    /// Rounds this app unit down to the pixel towards zero and returns it.
    #[inline]
    pub fn to_px(self) -> i32 {
        self.0 / AU_PER_PX
    }

    #[inline]
    pub fn to_nearest_px(self) -> i32 {
        ((self.0 as f64) / (AU_PER_PX as f64)).round() as i32
    }

    #[inline]
    pub fn to_nearest_pixel(self, pixels_per_px: f32) -> f32 {
        ((self.0 as f32) / (AU_PER_PX as f32) * pixels_per_px).round() / pixels_per_px
    }

    #[inline]
    pub fn to_f32_px(self) -> f32 {
        (self.0 as f32) / (AU_PER_PX as f32)
    }

    #[inline]
    pub fn to_f64_px(self) -> f64 {
        (self.0 as f64) / (AU_PER_PX as f64)
    }

    #[inline]
    pub fn from_f32_px(px: f32) -> Au {
        Au((px * (AU_PER_PX as f32)) as i32)
    }

    #[inline]
    pub fn from_f64_px(px: f64) -> Au {
        Au((px * (AU_PER_PX as f64)) as i32)
    }
}
