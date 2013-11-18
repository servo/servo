/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::vec;
use stb_image = stb_image::image;
use png;

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.
pub type Image = png::Image;

pub fn Image(width: u32, height: u32, color_type: png::ColorType, data: ~[u8]) -> Image {
    png::Image {
        width: width,
        height: height,
        color_type: color_type,
        pixels: data,
    }
}

static TEST_IMAGE: &'static [u8] = include_bin!("test.jpeg");

pub fn test_image_bin() -> ~[u8] {
    TEST_IMAGE.into_owned()
}

pub fn load_from_memory(buffer: &[u8]) -> Option<Image> {
    if png::is_png(buffer) {
        match png::load_png_from_memory(buffer) {
            Ok(png_image) => Some(png_image),
            Err(_err) => None,
        }
    } else {
        // For non-png images, we use stb_image
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

                Some(Image(image.width as u32, image.height as u32, png::RGBA8, data))
            }
            stb_image::ImageF32(_image) => fail!(~"HDR images not implemented"),
            stb_image::Error => None
        }
    }
}
