/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::image_cache::CorsStatus;
use ipc_channel::ipc::IpcSharedMemory;
use piston_image::{DynamicImage, ImageFormat};
use pixels::PixelFormat;
use std::fmt;

#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    #[ignore_malloc_size_of = "Defined in ipc-channel"]
    pub bytes: IpcSharedMemory,
    #[ignore_malloc_size_of = "Defined in webrender_api"]
    pub id: Option<webrender_api::ImageKey>,
    pub cors_status: CorsStatus,
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Image {{ width: {}, height: {}, format: {:?}, ..., id: {:?} }}",
            self.width, self.height, self.format, self.id
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
}

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.

pub fn load_from_memory(buffer: &[u8], cors_status: CorsStatus) -> Option<Image> {
    if buffer.is_empty() {
        return None;
    }

    let image_fmt_result = detect_image_format(buffer);
    match image_fmt_result {
        Err(msg) => {
            debug!("{}", msg);
            None
        },
        Ok(_) => match piston_image::load_from_memory(buffer) {
            Ok(image) => {
                let mut rgba = match image {
                    DynamicImage::ImageRgba8(rgba) => rgba,
                    image => image.to_rgba(),
                };
                pixels::rgba8_byte_swap_colors_inplace(&mut *rgba);
                Some(Image {
                    width: rgba.width(),
                    height: rgba.height(),
                    format: PixelFormat::BGRA8,
                    bytes: IpcSharedMemory::from_bytes(&*rgba),
                    id: None,
                    cors_status,
                })
            },
            Err(e) => {
                debug!("Image decoding error: {:?}", e);
                None
            },
        },
    }
}

// https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img
pub fn detect_image_format(buffer: &[u8]) -> Result<ImageFormat, &str> {
    if is_gif(buffer) {
        Ok(ImageFormat::Gif)
    } else if is_jpeg(buffer) {
        Ok(ImageFormat::Jpeg)
    } else if is_png(buffer) {
        Ok(ImageFormat::Png)
    } else if is_bmp(buffer) {
        Ok(ImageFormat::Bmp)
    } else if is_ico(buffer) {
        Ok(ImageFormat::Ico)
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
