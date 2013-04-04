use geom::size::Size2D;

#[deriving(Eq)]
pub enum format {
    fo_rgba_8888
    // TODO: RGB 565, others?
}

impl format {
    fn bpp(self) -> uint {
        match self {
            fo_rgba_8888 => 32u 
        }
    }
}

pub struct ImageSurface {
    size: Size2D<int>,
    format: format,
    buffer: ~[u8]
}

impl ImageSurface {
    pub fn new(size: Size2D<int>, format: format) -> ImageSurface {
        ImageSurface {
            size: copy size,
            format: format,
            buffer: vec::from_elem((size.area() as uint) * format.bpp(), 0u8)
        }
    }
}
