use geom::size::Size2D;

enum format {
    fo_rgba_8888
    // TODO: RGB 565, others?
}

impl format: cmp::Eq {
    pure fn eq(&&other: format) -> bool {
        match (self, other) {
          (fo_rgba_8888, fo_rgba_8888) => true,
       }
    }
    pure fn ne(&&other: format) -> bool {
        return !self.eq(other);
    }
}

type image_surface = {
    size: Size2D<int>,
    format: format,
    buffer: ~[u8]
};

impl format {
    fn bpp() -> uint {
        match self {
            fo_rgba_8888 => 32u 
        }
    }
}

fn image_surface(size: Size2D<int>, format: format) -> image_surface {
    {
        size: copy size,
        format: format,
        buffer: vec::from_elem((size.area() as uint) * format.bpp(), 0u8)
    }
}

