// FIXME--mut should be inherited
type point<A> = { mut x: A, mut y: A };
type size<A> = { mut width: A, mut height: A };
type rect<A> = { mut origin: point<A>, mut size: size<A> };

enum au = int;

fn point<A:copy>(x: A, y: A) -> point<A> {
    {mut x: x, mut y: y}
}

fn size<A:copy>(w: A, h: A) -> size<A> {
    {mut width: w, mut height: h}
}

fn box<A:copy>(x: A, y: A, w: A, h: A) -> rect<A> {
    {mut origin: point(x, y),
     mut size: size(w, h)}
}

fn zero_rect_au() -> rect<au> {
    let z = au(0);
    {mut origin: point(z, z), mut size: size(z, z)}
}

