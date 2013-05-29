/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use stb_image = stb_image::image;

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.

pub type Image = stb_image::Image<u8>;

pub fn Image(width: uint, height: uint, depth: uint, data: ~[u8]) -> Image {
    stb_image::new_image(width, height, depth, data)
}

static TEST_IMAGE: [u8, ..4962] = include_bin!("test.jpeg");

pub fn test_image_bin() -> ~[u8] {
    return vec::from_fn(4962, |i| TEST_IMAGE[i]);
}

pub fn load_from_memory(buffer: &[u8]) -> Option<Image> {
    // Can't remember why we do this. Maybe it's what cairo wants
    static FORCE_DEPTH: uint = 4;

    match stb_image::load_from_memory_with_depth(buffer, FORCE_DEPTH, true) {
        stb_image::ImageU8(image) => {
            assert!(image.depth == 4);
            // Do color space conversion :(
            let data = do vec::from_fn(image.width * image.height * 4) |i| {
                let color = i % 4;
                let pixel = i / 4;
                match color {
                    0 => image.data[pixel * 4 + 2],
                    1 => image.data[pixel * 4 + 1],
                    2 => image.data[pixel * 4 + 0],
                    3 => 0xffu8,
                    _ => fail!()
                }
            };

            assert!(image.data.len() == data.len());

            Some(Image(image.width, image.height, image.depth, data))
        }
        stb_image::ImageF32(_image) => fail!(~"HDR images not implemented"),
        stb_image::Error => None
    }
}
