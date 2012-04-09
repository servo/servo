import gfx::geom;
import gfx::geom::size;

enum format {
    fo_rgba_8888
    // TODO: RGB 565, others?
}

type image_surface = {
    size: geom::size<int>,
    format: format,
    buffer: [u8]
};

impl format for format {
    fn bpp() -> uint {
        alt self {
            fo_rgba_8888 { 32u }
        }
    }
}

fn image_surface(size: geom::size<int>, format: format) -> image_surface {
    {
        size: size,
        format: format,
        buffer: vec::from_elem((size.area() as uint) * format.bpp(), 0u8)
    }
}

