/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use std::num::{NumCast, One, Zero};

#[deriving(Clone,Eq)]
pub struct Au(i32);

impl Add<Au,Au> for Au {
    fn add(&self, other: &Au) -> Au { Au(**self + **other) }
}

impl Sub<Au,Au> for Au {
    fn sub(&self, other: &Au) -> Au { Au(**self - **other) }
}

impl Mul<Au,Au> for Au {
    fn mul(&self, other: &Au) -> Au { Au(**self * **other) }
}

impl Div<Au,Au> for Au {
    fn div(&self, other: &Au) -> Au { Au(**self / **other) }
}

impl Rem<Au,Au> for Au {
    fn rem(&self, other: &Au) -> Au { Au(**self % **other) }
}

impl Neg<Au> for Au {
    fn neg(&self) -> Au { Au(-**self) }
}

impl Ord for Au {
    fn lt(&self, other: &Au) -> bool { **self <  **other }
    fn le(&self, other: &Au) -> bool { **self <= **other }
    fn ge(&self, other: &Au) -> bool { **self >= **other }
    fn gt(&self, other: &Au) -> bool { **self >  **other }
}

impl One for Au {
    fn one() -> Au { Au(1) }
}

impl Zero for Au {
    fn zero() -> Au { Au(0) }
    fn is_zero(&self) -> bool { **self == 0 }
}

impl Num for Au {}

pub fn min(x: Au, y: Au) -> Au { if x < y { x } else { y } }
pub fn max(x: Au, y: Au) -> Au { if x > y { x } else { y } }

impl NumCast for Au {
    fn from<T:NumCast>(n: T) -> Au { Au(n.to_i32()) }

    fn to_u8(&self) -> u8       { (**self).to_u8() }
    fn to_u16(&self) -> u16     { (**self).to_u16() }
    fn to_u32(&self) -> u32     { (**self).to_u32() }
    fn to_u64(&self) -> u64     { (**self).to_u64() }
    fn to_uint(&self) -> uint   { (**self).to_uint() }

    fn to_i8(&self) -> i8       { (**self).to_i8() }
    fn to_i16(&self) -> i16     { (**self).to_i16() }
    fn to_i32(&self) -> i32     { (**self).to_i32() }
    fn to_i64(&self) -> i64     { (**self).to_i64() }
    fn to_int(&self) -> int     { (**self).to_int() }

    fn to_f32(&self) -> f32     { (**self).to_f32() }
    fn to_f64(&self) -> f64     { (**self).to_f64() }
    fn to_float(&self) -> float { (**self).to_float() }
}

pub fn box<T:Clone + Ord + Add<T,T> + Sub<T,T>>(x: T, y: T, w: T, h: T) -> Rect<T> {
    Rect(Point2D(x, y), Size2D(w, h))
}

impl Au {
    pub fn scale_by(self, factor: float) -> Au {
        Au(((*self as float) * factor).round() as i32)
    }

    pub fn from_px(px: int) -> Au {
        NumCast::from(px * 60)
    }

    pub fn to_nearest_px(&self) -> int {
        ((**self as float) / 60f).round() as int
    }

    pub fn to_snapped(&self) -> Au {
        let res = **self % 60i32;
        return if res >= 30i32 { return Au(**self - res + 60i32) }
                       else { return Au(**self - res) };
    }

    pub fn zero_point() -> Point2D<Au> {
        Point2D(Au(0), Au(0))
    }

    pub fn zero_rect() -> Rect<Au> {
        let z = Au(0);
        Rect(Point2D(z, z), Size2D(z, z))
    }

    pub fn from_pt(pt: float) -> Au {
        from_px(pt_to_px(pt) as int)
    }

    pub fn from_frac_px(px: float) -> Au {
        Au((px * 60f) as i32)
    }

    pub fn min(x: Au, y: Au) -> Au { if *x < *y { x } else { y } }
    pub fn max(x: Au, y: Au) -> Au { if *x > *y { x } else { y } }
}

// assumes 72 points per inch, and 96 px per inch
pub fn pt_to_px(pt: float) -> float {
    pt / 72f * 96f
}

// assumes 72 points per inch, and 96 px per inch
pub fn px_to_pt(px: float) -> float {
    px / 96f * 72f
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

pub fn from_frac_px(px: float) -> Au {
    Au((px * 60f) as i32)
}

pub fn from_px(px: int) -> Au {
    NumCast::from(px * 60)
}

pub fn to_px(au: Au) -> int {
    (*au / 60) as int
}

pub fn to_frac_px(au: Au) -> float {
    (*au as float) / 60f
}

// assumes 72 points per inch, and 96 px per inch
pub fn from_pt(pt: float) -> Au {
    from_px((pt / 72f * 96f) as int)
}
