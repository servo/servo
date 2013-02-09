use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use core::num::NumCast;

pub struct Au(i32);

pub impl Add<Au,Au> for Au {
    pure fn add(&self, other: &Au) -> Au { Au(**self + **other) }
}

pub impl Sub<Au,Au> for Au {
    pure fn sub(&self, other: &Au) -> Au { Au(**self - **other) }
}

pub impl Mul<Au,Au> for Au {
    pure fn mul(&self, other: &Au) -> Au { Au(**self * **other) }
}

pub impl Div<Au,Au> for Au {
    pure fn div(&self, other: &Au) -> Au { Au(**self / **other) }
}

pub impl Modulo<Au,Au> for Au {
    pure fn modulo(&self, other: &Au) -> Au { Au(**self % **other) }
}

pub impl Neg<Au> for Au {
    pure fn neg(&self) -> Au { Au(-**self) }
}

pub impl cmp::Ord for Au {
    pure fn lt(&self, other: &Au) -> bool { **self <  **other }
    pure fn le(&self, other: &Au) -> bool { **self <= **other }
    pure fn ge(&self, other: &Au) -> bool { **self >= **other }
    pure fn gt(&self, other: &Au) -> bool { **self >  **other }
}

pub impl cmp::Eq for Au {
    pure fn eq(&self, other: &Au) -> bool { **self == **other }
    pure fn ne(&self, other: &Au) -> bool { **self != **other }
}

pub pure fn min(x: Au, y: Au) -> Au { if x < y { x } else { y } }
pub pure fn max(x: Au, y: Au) -> Au { if x > y { x } else { y } }

impl NumCast for Au {
    static pure fn from<T:NumCast>(n: T) -> Au { Au(n.to_i32()) }

    pure fn to_u8(&self) -> u8       { (**self).to_u8() }
    pure fn to_u16(&self) -> u16     { (**self).to_u16() }
    pure fn to_u32(&self) -> u32     { (**self).to_u32() }
    pure fn to_u64(&self) -> u64     { (**self).to_u64() }
    pure fn to_uint(&self) -> uint   { (**self).to_uint() }

    pure fn to_i8(&self) -> i8       { (**self).to_i8() }
    pure fn to_i16(&self) -> i16     { (**self).to_i16() }
    pure fn to_i32(&self) -> i32     { (**self).to_i32() }
    pure fn to_i64(&self) -> i64     { (**self).to_i64() }
    pure fn to_int(&self) -> int     { (**self).to_int() }

    pure fn to_f32(&self) -> f32     { (**self).to_f32() }
    pure fn to_f64(&self) -> f64     { (**self).to_f64() }
    pure fn to_float(&self) -> float { (**self).to_float() }
}

pub fn box<T:Copy + Ord + Add<T,T> + Sub<T,T>>(x: T, y: T, w: T, h: T) -> Rect<T> {
    Rect(Point2D(x, y), Size2D(w, h))
}

pub impl Au {
    pub pure fn scale_by(factor: float) -> Au {
        Au(((*self as float) * factor) as i32)
    }

    static pub pure fn from_px(i: int) -> Au {
        NumCast::from(i * 60)
    }

    pub pure fn to_px(&const self) -> int {
        (**self / 60) as int
    }

    pub pure fn to_snapped(&const self) -> Au {
        let res = **self % 60i32;
        return if res >= 30i32 { return Au(**self - res + 60i32) }
                       else { return Au(**self - res) };
    }

    static pub pure fn zero_point() -> Point2D<Au> {
        Point2D(Au(0), Au(0))
    }

    static pub pure fn zero_rect() -> Rect<Au> {
        let z = Au(0);
        Rect(Point2D(z, z), Size2D(z, z))
    }

    // assumes 72 points per inch, and 96 px per inch
    static pub pure fn from_pt(f: float) -> Au {
        from_px((f / 72f * 96f) as int)
    }

    static pub pure fn from_frac_px(f: float) -> Au {
        Au((f * 60f) as i32)
    }

    static pub pure fn min(x: Au, y: Au) -> Au { if *x < *y { x } else { y } }
    static pub pure fn max(x: Au, y: Au) -> Au { if *x > *y { x } else { y } }
}

pub pure fn zero_rect() -> Rect<Au> {
    let z = Au(0);
    Rect(Point2D(z, z), Size2D(z, z))
}

pub pure fn zero_point() -> Point2D<Au> {
    Point2D(Au(0), Au(0))
}

pub pure fn zero_size() -> Size2D<Au> {
    Size2D(Au(0), Au(0))
}

pub pure fn from_frac_px(f: float) -> Au {
    Au((f * 60f) as i32)
}

pub pure fn from_px(i: int) -> Au {
    NumCast::from(i * 60)
}

pub pure fn to_px(au: Au) -> int {
    (*au / 60) as int
}

pub pure fn to_frac_px(au: Au) -> float {
    (*au as float) / 60f
}

// assumes 72 points per inch, and 96 px per inch
pub pure fn from_pt(f: float) -> Au {
    from_px((f / 72f * 96f) as int)
}
