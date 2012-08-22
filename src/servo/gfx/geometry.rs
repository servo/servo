import geom::point::Point2D;
import geom::rect::Rect;
import geom::size::Size2D;
import num::{Num, from_int};

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

fn box<A:copy Num>(x: A, y: A, w: A, h: A) -> Rect<A> {
    Rect(Point2D(x, y), Size2D(w, h))
}

fn zero_rect_au() -> Rect<au> {
    let z = au(0);
    Rect(Point2D(z, z), Size2D(z, z))
}

fn zero_size_au() -> Size2D<au> {
    Size2D(au(0), au(0))
}

pure fn px_to_au(i: int) -> au {
    from_int(i * 60)
}

pure fn au_to_px(au: au) -> int {
    (*au / 60) as int
}
