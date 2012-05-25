export image;
export load;
import stb_image::image::{image, load};

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.

