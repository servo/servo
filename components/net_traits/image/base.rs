/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::IpcSharedMemory;
use piston_image::{self, DynamicImage, ImageFormat};
use webrender_traits;

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize, HeapSizeOf)]
pub enum PixelFormat {
    /// Luminance channel only
    K8,
    /// Luminance + alpha
    KA8,
    /// RGB, 8 bits per channel
    RGB8,
    /// RGB + alpha, 8 bits per channel
    RGBA8,
}

#[derive(Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    pub bytes: IpcSharedMemory,
    #[ignore_heap_size_of = "Defined in webrender_traits"]
    pub id: Option<webrender_traits::ImageKey>,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize, HeapSizeOf)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
}

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.

// TODO(pcwalton): Speed up with SIMD, or better yet, find some way to not do this.
fn byte_swap_and_premultiply(data: &mut [u8]) {
    let length = data.len();

    for i in Iterator::step_by(0..length, 4) {
        let r = data[i + 2];
        let g = data[i + 1];
        let b = data[i + 0];

        data[i + 0] = r;
        data[i + 1] = g;
        data[i + 2] = b;
    }
}

pub fn load_from_memory(buffer: &[u8]) -> Option<Image> {
    if buffer.is_empty() {
        return None;
    }

    let image_fmt_result = detect_image_format(buffer);
    match image_fmt_result {
        Err(msg) => {
            debug!("{}", msg);
            None
        },
        Ok(_) => {
            match piston_image::load_from_memory(buffer) {
                Ok(image) => {
                    let mut rgba = match image {
                        DynamicImage::ImageRgba8(rgba) => rgba,
                        image => image.to_rgba(),
                    };
                    byte_swap_and_premultiply(&mut *rgba);
                    Some(Image {
                        width: rgba.width(),
                        height: rgba.height(),
                        format: PixelFormat::RGBA8,
                        bytes: IpcSharedMemory::from_bytes(&*rgba),
                        id: None,
                    })
                },
                Err(e) => {
                    debug!("Image decoding error: {:?}", e);
                    None
                },
            }
        },
    }
}


// https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img
pub fn detect_image_format(buffer: &[u8]) -> Result<ImageFormat, &str> {
    if is_gif(buffer) {
        Ok(ImageFormat::GIF)
    } else if is_jpeg(buffer) {
        Ok(ImageFormat::JPEG)
    } else if is_png(buffer) {
        Ok(ImageFormat::PNG)
    } else if is_bmp(buffer) {
        Ok(ImageFormat::BMP)
    } else if is_ico(buffer) {
        Ok(ImageFormat::ICO)
    } else {
        Err("Image Format Not Supported")
    }
}

fn is_gif(buffer: &[u8]) -> bool {
    buffer.starts_with(b"GIF87a") || buffer.starts_with(b"GIF89a")
}

fn is_jpeg(buffer: &[u8]) -> bool {
    buffer.starts_with(&[0xff, 0xd8, 0xff])
}

fn is_png(buffer: &[u8]) -> bool {
    buffer.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
}

fn is_bmp(buffer: &[u8]) -> bool {
    buffer.starts_with(&[0x42, 0x4D])
}

fn is_ico(buffer: &[u8]) -> bool {
    buffer.starts_with(&[0x00, 0x00, 0x01, 0x00])
}
