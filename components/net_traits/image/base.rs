/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSharedMemory;
use png;
use stb_image::image as stb_image2;
use std::mem;
use util::mem::HeapSizeOf;
use util::vec::byte_swap;

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.

#[derive(Deserialize, Serialize, HeapSizeOf)]
pub enum PixelFormat {
    K8,         // Luminance channel only
    KA8,        // Luminance + alpha
    RGB8,       // RGB, 8 bits per channel
    RGBA8,      // RGB + alpha, 8 bits per channel
}

#[derive(Deserialize, Serialize, HeapSizeOf)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    pub bytes: IpcSharedMemory,
}

// TODO(pcwalton): Speed up with SIMD, or better yet, find some way to not do this.
fn byte_swap_and_premultiply(data: &mut [u8]) {
    let length = data.len();
    for i in (0..length).step_by(4) {
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
    if buffer.is_empty() {
        return None;
    }

    if png::is_png(buffer) {
        match png::load_png_from_memory(buffer) {
            Ok(mut png_image) => {
                let (bytes, format) = match png_image.pixels {
                    png::PixelsByColorType::K8(ref mut data) => {
                        (data, PixelFormat::K8)
                    }
                    png::PixelsByColorType::KA8(ref mut data) => {
                        (data, PixelFormat::KA8)
                    }
                    png::PixelsByColorType::RGB8(ref mut data) => {
                        byte_swap(data);
                        (data, PixelFormat::RGB8)
                    }
                    png::PixelsByColorType::RGBA8(ref mut data) => {
                        byte_swap_and_premultiply(data);
                        (data, PixelFormat::RGBA8)
                    }
                };

                let bytes = mem::replace(bytes, Vec::new());
                let bytes = IpcSharedMemory::from_bytes(&bytes[..]);
                let image = Image {
                    width: png_image.width,
                    height: png_image.height,
                    format: format,
                    bytes: bytes,
                };

                Some(image)
            }
            Err(_err) => None,
        }
    } else {
        // For non-png images, we use stb_image
        // Can't remember why we do this. Maybe it's what cairo wants
        static FORCE_DEPTH: usize = 4;

        match stb_image2::load_from_memory_with_depth(buffer, FORCE_DEPTH, true) {
            stb_image2::LoadResult::ImageU8(mut image) => {
                assert!(image.depth == 4);
                // handle gif separately because the alpha-channel has to be premultiplied
                if is_gif(buffer) {
                    byte_swap_and_premultiply(&mut image.data);
                } else {
                    byte_swap(&mut image.data);
                }
                Some(Image {
                    width: image.width as u32,
                    height: image.height as u32,
                    format: PixelFormat::RGBA8,
                    bytes: IpcSharedMemory::from_bytes(&image.data[..]),
                })
            }
            stb_image2::LoadResult::ImageF32(_image) => {
                debug!("HDR images not implemented");
                None
            }
            stb_image2::LoadResult::Error(e) => {
                debug!("stb_image failed: {}", e);
                None
            }
        }
    }
}

fn is_gif(buffer: &[u8]) -> bool {
    match buffer {
        [b'G', b'I', b'F', b'8', n, b'a', ..] if n == b'7' || n == b'9' => true,
        _ => false
    }
}
