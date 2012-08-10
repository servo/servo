export Image;

export load;
export load_from_memory;
export test_image_bin;

import stb_image::image::{image, load, load_from_memory};

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.

type Image = image;

fn test_image_bin() -> ~[u8] {
    #include_bin("test.jpeg")
}