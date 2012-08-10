export ImageBuffer, SharedImageBuffer;
export image;
export load;

import stb_image::image::{image, load};
import core::arc::arc;

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.

type SharedImageBuffer = arc<ImageBuffer>;

struct ImageBuffer {
    data: ~[u8];
}
