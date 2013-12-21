/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use std::num::{NumCast, One, Zero};
use std::fmt;

// An Au is an "App Unit" and represents 1/60th of a CSS pixel.  It was
// originally proposed in 2002 as a standard unit of measure in Gecko.
// See https://bugzilla.mozilla.org/show_bug.cgi?id=177805 for more info.
pub struct Au(i32);

// We don't use #[deriving] here for inlining.
impl Clone for Au {
    #[inline]
    fn clone(&self) -> Au {
        Au(**self)
    }
}

impl fmt::Default for Au {
    fn fmt(obj: &Au, f: &mut fmt::Formatter) {
        let Au(n) = *obj;
        write!(f.buf, "Au({})", n);
    }
}

impl Eq for Au {
    #[inline]
    fn eq(&self, other: &Au) -> bool {
        **self == **other
    }
    #[inline]
    fn ne(&self, other: &Au) -> bool {
        **self != **other
    }
}

impl Add<Au,Au> for Au {
    #[inline]
    fn add(&self, other: &Au) -> Au { Au(**self + **other) }
}

impl Sub<Au,Au> for Au {
    #[inline]
    fn sub(&self, other: &Au) -> Au { Au(**self - **other) }
}

impl Mul<Au,Au> for Au {
    #[inline]
    fn mul(&self, other: &Au) -> Au { Au(**self * **other) }
}

impl Div<Au,Au> for Au {
    #[inline]
    fn div(&self, other: &Au) -> Au { Au(**self / **other) }
}

impl Rem<Au,Au> for Au {
    #[inline]
    fn rem(&self, other: &Au) -> Au { Au(**self % **other) }
}

impl Neg<Au> for Au {
    #[inline]
    fn neg(&self) -> Au { Au(-**self) }
}

impl Ord for Au {
    #[inline]
    fn lt(&self, other: &Au) -> bool { **self <  **other }
    #[inline]
    fn le(&self, other: &Au) -> bool { **self <= **other }
    #[inline]
    fn ge(&self, other: &Au) -> bool { **self >= **other }
    #[inline]
    fn gt(&self, other: &Au) -> bool { **self >  **other }
}

impl One for Au {
    #[inline]
    fn one() -> Au { Au(1) }
}

impl Zero for Au {
    #[inline]
    fn zero() -> Au { Au(0) }
    #[inline]
    fn is_zero(&self) -> bool { **self == 0 }
}

impl Num for Au {}

#[inline]
pub fn min(x: Au, y: Au) -> Au { if x < y { x } else { y } }
#[inline]
pub fn max(x: Au, y: Au) -> Au { if x > y { x } else { y } }

impl NumCast for Au {
    fn from<T:ToPrimitive>(n: T) -> Option<Au> {
        Some(Au(n.to_i32().unwrap()))
    }
}

impl ToPrimitive for Au {
    fn to_i64(&self) -> Option<i64> {
        Some(**self as i64)
    }

    fn to_u64(&self) -> Option<u64> {
        Some(**self as u64)
    }

    fn to_f32(&self) -> Option<f32> {
        (**self).to_f32()
    }

    fn to_f64(&self) -> Option<f64> {
        (**self).to_f64()
    }
}

impl Au {
    /// FIXME(pcwalton): Workaround for lack of cross crate inlining of newtype structs!
    #[inline]
    pub fn new(value: i32) -> Au {
        Au(value)
    }

    #[inline]
    pub fn scale_by(self, factor: f64) -> Au {
        Au(((*self as f64) * factor) as i32)
    }

    #[inline]
    pub fn from_px(px: int) -> Au {
        NumCast::from(px * 60).unwrap()
    }

    #[inline]
    pub fn to_nearest_px(&self) -> int {
        ((**self as f64) / 60f64).round() as int
    }

    #[inline]
    pub fn to_snapped(&self) -> Au {
        let res = **self % 60i32;
        return if res >= 30i32 { return Au(**self - res + 60i32) }
                       else { return Au(**self - res) };
    }

    #[inline]
    pub fn zero_point() -> Point2D<Au> {
        Point2D(Au(0), Au(0))
    }

    #[inline]
    pub fn zero_rect() -> Rect<Au> {
        let z = Au(0);
        Rect(Point2D(z, z), Size2D(z, z))
    }

    #[inline]
    pub fn from_pt(pt: f64) -> Au {
        from_px(pt_to_px(pt) as int)
    }

    #[inline]
    pub fn from_frac_px(px: f64) -> Au {
        Au((px * 60f64) as i32)
    }

    #[inline]
    pub fn min(x: Au, y: Au) -> Au { if *x < *y { x } else { y } }
    #[inline]
    pub fn max(x: Au, y: Au) -> Au { if *x > *y { x } else { y } }
}

// assumes 72 points per inch, and 96 px per inch
pub fn pt_to_px(pt: f64) -> f64 {
    pt / 72f64 * 96f64
}

// assumes 72 points per inch, and 96 px per inch
pub fn px_to_pt(px: f64) -> f64 {
    px / 96f64 * 72f64
}

pub fn zero_rect() -> Rect<Au> {
    let z = Au(0);
    Rect(Point2D(z, z), Size2D(z, z))
}

pub fn zero_point() -> Point2D<Au> {
    Point2D(Au(0), Au(0))
}

pub fn zero_size() -> Size2D<Au> {
    Size2D(Au(0), Au(0))
}

pub fn from_frac_px(px: f64) -> Au {
    Au((px * 60f64) as i32)
}

pub fn from_px(px: int) -> Au {
    NumCast::from(px * 60).unwrap()
}

pub fn to_px(au: Au) -> int {
    (*au / 60) as int
}

pub fn to_frac_px(au: Au) -> f64 {
    (*au as f64) / 60f64
}

// assumes 72 points per inch, and 96 px per inch
pub fn from_pt(pt: f64) -> Au {
    from_px((pt / 72f64 * 96f64) as int)
}

// assumes 72 points per inch, and 96 px per inch
pub fn to_pt(au: Au) -> f64 {
    (*au as f64) / 60f64 * 72f64 / 96f64
}

