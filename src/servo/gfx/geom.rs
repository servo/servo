type point<A> = { x: A, y: A };
type size<A> = { width: A, height: A };
type rect<A> = { origin: point<A>, size: size<A> };

enum au = int;

impl size for size<int> {
    fn area() -> int {
        self.width * self.height
    }
}

fn point<A:copy>(x: A, y: A) -> point<A> {
    {x: x, y: y}
}

fn size<A:copy>(w: A, h: A) -> size<A> {
    {width: w, height: h}
}

fn box<A:copy>(x: A, y: A, w: A, h: A) -> rect<A> {
    {origin: point(x, y),
     size: size(w, h)}
}

fn zero_rect_au() -> rect<au> {
    let z = au(0);
    {origin: point(z, z), size: zero_size_au()}
}

fn zero_size_au() -> size<au> {
    {width: au(0), height: au(0)}
}

pure fn px_to_au(i: int) -> au {
    au(i * 60)
}

pure fn au_to_px(au: au) -> int {
    *au / 60
}
