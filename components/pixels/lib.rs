/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::fmt;

use euclid::default::{Point2D, Rect, Size2D};
use image::ImageFormat;
use ipc_channel::ipc::IpcSharedMemory;
use log::debug;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use webrender_api::ImageKey;

#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum PixelFormat {
    /// Luminance channel only
    K8,
    /// Luminance + alpha
    KA8,
    /// RGB, 8 bits per channel
    RGB8,
    /// RGB + alpha, 8 bits per channel
    RGBA8,
    /// BGR + alpha, 8 bits per channel
    BGRA8,
}

pub fn rgba8_get_rect(pixels: &[u8], size: Size2D<u64>, rect: Rect<u64>) -> Cow<[u8]> {
    assert!(!rect.is_empty());
    assert!(Rect::from_size(size).contains_rect(&rect));
    assert_eq!(pixels.len() % 4, 0);
    assert_eq!(size.area() as usize, pixels.len() / 4);
    let area = rect.size.area() as usize;
    let first_column_start = rect.origin.x as usize * 4;
    let row_length = size.width as usize * 4;
    let first_row_start = rect.origin.y as usize * row_length;
    if rect.origin.x == 0 && rect.size.width == size.width || rect.size.height == 1 {
        let start = first_column_start + first_row_start;
        return Cow::Borrowed(&pixels[start..start + area * 4]);
    }
    let mut data = Vec::with_capacity(area * 4);
    for row in pixels[first_row_start..]
        .chunks(row_length)
        .take(rect.size.height as usize)
    {
        data.extend_from_slice(&row[first_column_start..][..rect.size.width as usize * 4]);
    }
    data.into()
}

// TODO(pcwalton): Speed up with SIMD, or better yet, find some way to not do this.
pub fn rgba8_byte_swap_colors_inplace(pixels: &mut [u8]) {
    assert!(pixels.len() % 4 == 0);
    for rgba in pixels.chunks_mut(4) {
        rgba.swap(0, 2);
    }
}

pub fn rgba8_byte_swap_and_premultiply_inplace(pixels: &mut [u8]) {
    assert!(pixels.len() % 4 == 0);
    for rgba in pixels.chunks_mut(4) {
        let b = rgba[0];
        rgba[0] = multiply_u8_color(rgba[2], rgba[3]);
        rgba[1] = multiply_u8_color(rgba[1], rgba[3]);
        rgba[2] = multiply_u8_color(b, rgba[3]);
    }
}

/// Returns true if the pixels were found to be completely opaque.
pub fn rgba8_premultiply_inplace(pixels: &mut [u8]) -> bool {
    assert!(pixels.len() % 4 == 0);
    let mut is_opaque = true;
    for rgba in pixels.chunks_mut(4) {
        rgba[0] = multiply_u8_color(rgba[0], rgba[3]);
        rgba[1] = multiply_u8_color(rgba[1], rgba[3]);
        rgba[2] = multiply_u8_color(rgba[2], rgba[3]);
        is_opaque = is_opaque && rgba[3] == 255;
    }
    is_opaque
}

pub fn multiply_u8_color(a: u8, b: u8) -> u8 {
    (a as u32 * b as u32 / 255) as u8
}

pub fn clip(
    mut origin: Point2D<i32>,
    mut size: Size2D<u64>,
    surface: Size2D<u64>,
) -> Option<Rect<u64>> {
    if origin.x < 0 {
        size.width = size.width.saturating_sub(-origin.x as u64);
        origin.x = 0;
    }
    if origin.y < 0 {
        size.height = size.height.saturating_sub(-origin.y as u64);
        origin.y = 0;
    }
    let origin = Point2D::new(origin.x as u64, origin.y as u64);
    Rect::new(origin, size)
        .intersection(&Rect::from_size(surface))
        .filter(|rect| !rect.is_empty())
}

/// Whether this response passed any CORS checks, and is thus safe to read from
/// in cross-origin environments.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum CorsStatus {
    /// The response is either same-origin or cross-origin but passed CORS checks.
    Safe,
    /// The response is cross-origin and did not pass CORS checks. It is unsafe
    /// to expose pixel data to the requesting environment.
    Unsafe,
}

#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    #[ignore_malloc_size_of = "Defined in ipc-channel"]
    pub bytes: IpcSharedMemory,
    #[ignore_malloc_size_of = "Defined in webrender_api"]
    pub id: Option<ImageKey>,
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
        Ok(_) => match image::load_from_memory(buffer) {
            Ok(image) => {
                let mut rgba = image.into_rgba8();
                rgba8_byte_swap_colors_inplace(&mut rgba);
                Some(Image {
                    width: rgba.width(),
                    height: rgba.height(),
                    format: PixelFormat::BGRA8,
                    bytes: IpcSharedMemory::from_bytes(&rgba),
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
    } else if is_webp(buffer) {
        Ok(ImageFormat::WebP)
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

fn is_webp(buffer: &[u8]) -> bool {
    buffer.starts_with(b"RIFF") && buffer.len() >= 14 && &buffer[8..14] == b"WEBPVP"
}

#[cfg(test)]
mod test {
    use super::detect_image_format;

    #[test]
    fn test_supported_images() {
        let gif1 = [b'G', b'I', b'F', b'8', b'7', b'a'];
        let gif2 = [b'G', b'I', b'F', b'8', b'9', b'a'];
        let jpeg = [0xff, 0xd8, 0xff];
        let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let webp = [
            b'R', b'I', b'F', b'F', 0x01, 0x02, 0x03, 0x04, b'W', b'E', b'B', b'P', b'V', b'P',
        ];
        let bmp = [0x42, 0x4D];
        let ico = [0x00, 0x00, 0x01, 0x00];
        let junk_format = [0x01, 0x02, 0x03, 0x04, 0x05];

        assert!(detect_image_format(&gif1).is_ok());
        assert!(detect_image_format(&gif2).is_ok());
        assert!(detect_image_format(&jpeg).is_ok());
        assert!(detect_image_format(&png).is_ok());
        assert!(detect_image_format(&webp).is_ok());
        assert!(detect_image_format(&bmp).is_ok());
        assert!(detect_image_format(&ico).is_ok());
        assert!(detect_image_format(&junk_format).is_err());
    }
}
