export Image;

export load;
export load_from_memory;

import stb_image::image::{image, load, load_from_memory};

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.

type Image = image;
