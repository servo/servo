/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::iter::range_step;
use stb_image::image as stb_image;
use png;

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.
pub type Image = png::Image;


static TEST_IMAGE: &'static [u8] = include_bin!("test.jpeg");

pub fn test_image_bin() -> Vec<u8> {
    TEST_IMAGE.iter().map(|&x| x).collect()
}

// TODO(pcwalton): Speed up with SIMD, or better yet, find some way to not do this.
fn byte_swap(data: &mut [u8]) {
    let length = data.len();
    for i in range_step(0, length, 4) {
        let r = data[i + 2];
        data[i + 2] = data[i + 0];
        data[i + 0] = r;
    }
}

// TODO(pcwalton): Speed up with SIMD, or better yet, find some way to not do this.
fn byte_swap_and_premultiply(data: &mut [u8]) {
    let length = data.len();
    for i in range_step(0, length, 4) {
        let r = data[i + 2];
        let g = data[i + 1];
        let b = data[i + 0];
        let a = data[i + 3];
        data[i + 0] = ((r as u32) * (a as u32) / 255) as u8;
        data[i + 1] = ((g as u32) * (a as u32) / 255) as u8;
        data[i + 2] = ((b as u32) * (a as u32) / 255) as u8;
    }
}

pub fn load_from_memory(buffer: &[u8]) -> Option<Image> {
    if buffer.len() == 0 {
        return None;
    }

    if png::is_png(buffer) {
        match png::load_png_from_memory(buffer) {
            Ok(mut png_image) => {
                match png_image.pixels {
                    png::RGB8(ref mut data) => byte_swap(data.as_mut_slice()),
                    png::RGBA8(ref mut data) => {
                        byte_swap_and_premultiply(data.as_mut_slice())
                    }
                    _ => {}
                }
                Some(png_image)
            }
            Err(_err) => None,
        }
    } else {
        // For non-png images, we use stb_image
        // Can't remember why we do this. Maybe it's what cairo wants
        static FORCE_DEPTH: uint = 4;

        match stb_image::load_from_memory_with_depth(buffer, FORCE_DEPTH, true) {
            stb_image::ImageU8(mut image) => {
                assert!(image.depth == 4);
                byte_swap(image.data.as_mut_slice());
                Some(png::Image {
                    width: image.width as u32,
                    height: image.height as u32,
                    pixels: png::RGBA8(image.data)
                })
            }
            stb_image::ImageF32(_image) => fail!("HDR images not implemented"),
            stb_image::Error(_) => None
        }
    }
}
