use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use num::{Num, from_int};

pub enum Au = i32;

impl Au : Num {
    pure fn add(other: &Au) -> Au        { Au(*self + **other) }
    pure fn sub(other: &Au) -> Au        { Au(*self - **other) }
    pure fn mul(other: &Au) -> Au        { Au(*self * **other) }
    pure fn div(other: &Au) -> Au        { Au(*self / **other) }
    pure fn modulo(other: &Au) -> Au     { Au(*self % **other) }
    pure fn neg() -> Au                   { Au(-*self)         }

    pure fn to_int() -> int               { *self as int       }

    static pure fn from_int(n: int) -> Au {
        Au((n & (i32::max_value as int)) as i32)
    }
}

impl Au : cmp::Ord {
    pure fn lt(&self, other: &Au) -> bool { **self <  **other }
    pure fn le(&self, other: &Au) -> bool { **self <= **other }
    pure fn ge(&self, other: &Au) -> bool { **self >= **other }
    pure fn gt(&self, other: &Au) -> bool { **self >  **other }
}

impl Au : cmp::Eq {
    pure fn eq(&self, other: &Au) -> bool { **self == **other }
    pure fn ne(&self, other: &Au) -> bool { **self != **other }
}

pub pure fn min(x: Au, y: Au) -> Au { if x < y { x } else { y } }
pub pure fn max(x: Au, y: Au) -> Au { if x > y { x } else { y } }

pub fn box<A:Copy Num>(x: A, y: A, w: A, h: A) -> Rect<A> {
    Rect(Point2D(x, y), Size2D(w, h))
}

impl Au {
    pub pure fn scale_by(factor: float) -> Au {
        Au(((*self as float) * factor) as i32)
    }

    static pub pure fn from_px(i: int) -> Au {
        from_int(i * 60)
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
    from_int(i * 60)
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
