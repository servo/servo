import geom::point::Point2D;
import geom::rect::Rect;
import geom::size::Size2D;
import num::num;

enum au = int;

impl au of num for au {
    fn add(&&other: au) -> au       { au(*self + *other) }
    fn sub(&&other: au) -> au       { au(*self - *other) }
    fn mul(&&other: au) -> au       { au(*self * *other) }
    fn div(&&other: au) -> au       { au(*self / *other) }
    fn modulo(&&other: au) -> au    { au(*self % *other) }
    fn neg() -> au                  { au(-*self)         }

    fn to_int() -> int              { *self              }
    fn from_int(n: int) -> au       { au(n)              }
}

fn box<A:copy num>(x: A, y: A, w: A, h: A) -> Rect<A> {
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
    au(i * 60)
}

pure fn au_to_px(au: au) -> int {
    *au / 60
}
