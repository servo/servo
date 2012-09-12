use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use num::{Num, from_int};

enum au = i32;

impl au : Num {
    pure fn add(&&other: au) -> au        { au(*self + *other) }
    pure fn sub(&&other: au) -> au        { au(*self - *other) }
    pure fn mul(&&other: au) -> au        { au(*self * *other) }
    pure fn div(&&other: au) -> au        { au(*self / *other) }
    pure fn modulo(&&other: au) -> au     { au(*self % *other) }
    pure fn neg() -> au                   { au(-*self)         }

    pure fn to_int() -> int               { *self as int       }

    static pure fn from_int(n: int) -> au {
        au((n & (i32::max_value as int)) as i32)
    }
}

impl au : cmp::Ord {
    pure fn lt(&&other: au) -> bool { *self <  *other }
    pure fn le(&&other: au) -> bool { *self <= *other }
    pure fn ge(&&other: au) -> bool { *self >= *other }
    pure fn gt(&&other: au) -> bool { *self >  *other }
}

impl au : cmp::Eq {
    pure fn eq(&&other: au) -> bool { *self == *other }
    pure fn ne(&&other: au) -> bool { *self != *other }
}

pure fn min(x: au, y: au) -> au { if x < y { x } else { y } }
pure fn max(x: au, y: au) -> au { if x > y { x } else { y } }

fn box<A:Copy Num>(x: A, y: A, w: A, h: A) -> Rect<A> {
    Rect(Point2D(x, y), Size2D(w, h))
}

fn zero_rect() -> Rect<au> {
    let z = au(0);
    Rect(Point2D(z, z), Size2D(z, z))
}


fn zero_point() -> Point2D<au> {
    Point2D(au(0), au(0))
}

fn zero_size() -> Size2D<au> {
    Size2D(au(0), au(0))
}

pure fn from_px(i: int) -> au {
    from_int(i * 60)
}

pure fn to_px(au: au) -> int {
    (*au / 60) as int
}
