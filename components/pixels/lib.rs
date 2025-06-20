/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod snapshot;

use std::borrow::Cow;
use std::io::Cursor;
use std::ops::Range;
use std::time::Duration;
use std::{cmp, fmt, vec};

use euclid::default::{Point2D, Rect, Size2D};
use image::codecs::gif::GifDecoder;
use image::imageops::{self, FilterType};
use image::{AnimationDecoder as _, ImageBuffer, ImageFormat, Rgba};
use ipc_channel::ipc::IpcSharedMemory;
use log::debug;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
pub use snapshot::*;
use webrender_api::ImageKey;

#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum FilterQuality {
    /// No image interpolation (Nearest-neighbor)
    None,
    /// Low-quality image interpolation (Bilinear)
    Low,
    /// Medium-quality image interpolation (CatmullRom, Mitchell)
    Medium,
    /// High-quality image interpolation (Lanczos)
    High,
}

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

/// Computes image byte length, returning None if overflow occurred or the total length exceeds
/// the maximum image allocation size.
pub fn compute_rgba8_byte_length_if_within_limit(width: usize, height: usize) -> Option<usize> {
    // Maximum allowed image allocation size (2^31-1 ~ 2GB).
    const MAX_IMAGE_BYTE_LENGTH: usize = 2147483647;

    // The color components of each pixel must be stored in four sequential
    // elements in the order of red, green, blue, and then alpha.
    4usize
        .checked_mul(width)
        .and_then(|v| v.checked_mul(height))
        .filter(|v| *v <= MAX_IMAGE_BYTE_LENGTH)
}

/// Copies the rectangle of the source image to the destination image.
pub fn copy_rgba8_image(
    src_size: Size2D<u32>,
    src_rect: Rect<u32>,
    src_pixels: &[u8],
    dest_size: Size2D<u32>,
    dest_rect: Rect<u32>,
    dest_pixels: &mut [u8],
) {
    assert!(!src_rect.is_empty());
    assert!(!dest_rect.is_empty());
    assert!(Rect::from_size(src_size).contains_rect(&src_rect));
    assert!(Rect::from_size(dest_size).contains_rect(&dest_rect));
    assert!(src_rect.size == dest_rect.size);
    assert_eq!(src_pixels.len() % 4, 0);
    assert_eq!(dest_pixels.len() % 4, 0);

    if src_size == dest_size && src_rect == dest_rect {
        dest_pixels.copy_from_slice(src_pixels);
        return;
    }

    let src_first_column_start = src_rect.origin.x as usize * 4;
    let src_row_length = src_size.width as usize * 4;
    let src_first_row_start = src_rect.origin.y as usize * src_row_length;

    let dest_first_column_start = dest_rect.origin.x as usize * 4;
    let dest_row_length = dest_size.width as usize * 4;
    let dest_first_row_start = dest_rect.origin.y as usize * dest_row_length;

    let (chunk_length, chunk_count) = (
        src_rect.size.width as usize * 4,
        src_rect.size.height as usize,
    );

    for i in 0..chunk_count {
        let src = &src_pixels[src_first_row_start + i * src_row_length..][src_first_column_start..]
            [..chunk_length];
        let dest = &mut dest_pixels[dest_first_row_start + i * dest_row_length..]
            [dest_first_column_start..][..chunk_length];
        dest.copy_from_slice(src);
    }
}

/// Scales the source image to the required size, performing sampling filter algorithm.
pub fn scale_rgba8_image(
    size: Size2D<u32>,
    pixels: &[u8],
    required_size: Size2D<u32>,
    quality: FilterQuality,
) -> Option<Vec<u8>> {
    let filter = match quality {
        FilterQuality::None => FilterType::Nearest,
        FilterQuality::Low => FilterType::Triangle,
        FilterQuality::Medium => FilterType::CatmullRom,
        FilterQuality::High => FilterType::Lanczos3,
    };

    let buffer: ImageBuffer<Rgba<u8>, &[u8]> =
        ImageBuffer::from_raw(size.width, size.height, pixels)?;

    let scaled_buffer =
        imageops::resize(&buffer, required_size.width, required_size.height, filter);

    Some(scaled_buffer.into_vec())
}

/// Flips the source image vertically in place.
pub fn flip_y_rgba8_image_inplace(size: Size2D<u32>, pixels: &mut [u8]) {
    assert_eq!(pixels.len() % 4, 0);

    let row_length = size.width as usize * 4;
    let half_height = (size.height / 2) as usize;

    let (left, right) = pixels.split_at_mut(pixels.len() - row_length * half_height);

    for i in 0..half_height {
        let top = &mut left[i * row_length..][..row_length];
        let bottom = &mut right[(half_height - i - 1) * row_length..][..row_length];
        top.swap_with_slice(bottom);
    }
}

pub fn rgba8_get_rect(pixels: &[u8], size: Size2D<u32>, rect: Rect<u32>) -> Cow<[u8]> {
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

/// Returns a*b/255, rounding any fractional bits to nearest integer
/// to reduce the loss of precision after multiple consequence alpha
/// (un)premultiply operations.
#[inline(always)]
pub fn multiply_u8_color(a: u8, b: u8) -> u8 {
    let c = a as u32 * b as u32 + 128;
    ((c + (c >> 8)) >> 8) as u8
}

pub fn clip(
    mut origin: Point2D<i32>,
    mut size: Size2D<u32>,
    surface: Size2D<u32>,
) -> Option<Rect<u32>> {
    if origin.x < 0 {
        size.width = size.width.saturating_sub(-origin.x as u32);
        origin.x = 0;
    }
    if origin.y < 0 {
        size.height = size.height.saturating_sub(-origin.y as u32);
        origin.y = 0;
    }
    let origin = Point2D::new(origin.x as u32, origin.y as u32);
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
pub struct RasterImage {
    pub metadata: ImageMetadata,
    pub format: PixelFormat,
    pub id: Option<ImageKey>,
    pub cors_status: CorsStatus,
    pub bytes: IpcSharedMemory,
    pub frames: Vec<ImageFrame>,
}

#[derive(Clone, Deserialize, MallocSizeOf, Serialize)]
pub struct ImageFrame {
    pub delay: Option<Duration>,
    /// References a range of the `bytes` field from the image that this
    /// frame belongs to.
    pub byte_range: Range<usize>,
    pub width: u32,
    pub height: u32,
}

/// A non-owning reference to the data of an [ImageFrame]
pub struct ImageFrameView<'a> {
    pub delay: Option<Duration>,
    pub bytes: &'a [u8],
    pub width: u32,
    pub height: u32,
}

impl RasterImage {
    pub fn should_animate(&self) -> bool {
        self.frames.len() > 1
    }

    pub fn frames(&self) -> impl Iterator<Item = ImageFrameView> {
        self.frames.iter().map(|frame| ImageFrameView {
            delay: frame.delay,
            bytes: self.bytes.get(frame.byte_range.clone()).unwrap(),
            width: frame.width,
            height: frame.height,
        })
    }

    pub fn first_frame(&self) -> ImageFrameView {
        self.frames()
            .next()
            .expect("All images should have at least one frame")
    }
}

impl fmt::Debug for RasterImage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Image {{ width: {}, height: {}, format: {:?}, ..., id: {:?} }}",
            self.metadata.width, self.metadata.height, self.format, self.id
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
}

// FIXME: Images must not be copied every frame. Instead we should atomically
// reference count them.

pub fn load_from_memory(buffer: &[u8], cors_status: CorsStatus) -> Option<RasterImage> {
    if buffer.is_empty() {
        return None;
    }

    let image_fmt_result = detect_image_format(buffer);
    match image_fmt_result {
        Err(msg) => {
            debug!("{}", msg);
            None
        },
        Ok(format) => match format {
            ImageFormat::Gif => decode_gif(buffer, cors_status),
            _ => match image::load_from_memory(buffer) {
                Ok(image) => {
                    let mut rgba = image.into_rgba8();
                    rgba8_byte_swap_colors_inplace(&mut rgba);
                    let frame = ImageFrame {
                        delay: None,
                        byte_range: 0..rgba.len(),
                        width: rgba.width(),
                        height: rgba.height(),
                    };
                    Some(RasterImage {
                        metadata: ImageMetadata {
                            width: rgba.width(),
                            height: rgba.height(),
                        },
                        format: PixelFormat::BGRA8,
                        frames: vec![frame],
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

pub fn unmultiply_inplace<const SWAP_RB: bool>(pixels: &mut [u8]) {
    for rgba in pixels.chunks_mut(4) {
        let a = rgba[3] as u32;
        let mut b = rgba[2] as u32;
        let mut g = rgba[1] as u32;
        let mut r = rgba[0] as u32;

        if a > 0 {
            r = r * 255 / a;
            g = g * 255 / a;
            b = b * 255 / a;

            if SWAP_RB {
                rgba[2] = r as u8;
                rgba[1] = g as u8;
                rgba[0] = b as u8;
            } else {
                rgba[2] = b as u8;
                rgba[1] = g as u8;
                rgba[0] = r as u8;
            }
        }
    }
}

#[repr(u8)]
pub enum Multiply {
    None = 0,
    PreMultiply = 1,
    UnMultiply = 2,
}

pub fn transform_inplace(pixels: &mut [u8], multiply: Multiply, swap_rb: bool, clear_alpha: bool) {
    match (multiply, swap_rb, clear_alpha) {
        (Multiply::None, true, true) => generic_transform_inplace::<0, true, true>(pixels),
        (Multiply::None, true, false) => generic_transform_inplace::<0, true, false>(pixels),
        (Multiply::None, false, true) => generic_transform_inplace::<0, false, true>(pixels),
        (Multiply::None, false, false) => generic_transform_inplace::<0, false, false>(pixels),
        (Multiply::PreMultiply, true, true) => generic_transform_inplace::<1, true, true>(pixels),
        (Multiply::PreMultiply, true, false) => generic_transform_inplace::<1, true, false>(pixels),
        (Multiply::PreMultiply, false, true) => generic_transform_inplace::<1, false, true>(pixels),
        (Multiply::PreMultiply, false, false) => {
            generic_transform_inplace::<1, false, false>(pixels)
        },
        (Multiply::UnMultiply, true, true) => generic_transform_inplace::<2, true, true>(pixels),
        (Multiply::UnMultiply, true, false) => generic_transform_inplace::<2, true, false>(pixels),
        (Multiply::UnMultiply, false, true) => generic_transform_inplace::<2, false, true>(pixels),
        (Multiply::UnMultiply, false, false) => {
            generic_transform_inplace::<2, false, false>(pixels)
        },
    }
}

pub fn generic_transform_inplace<
    const MULTIPLY: u8, // 1 premultiply, 2 unmultiply
    const SWAP_RB: bool,
    const CLEAR_ALPHA: bool,
>(
    pixels: &mut [u8],
) {
    for rgba in pixels.chunks_mut(4) {
        match MULTIPLY {
            1 => {
                let a = rgba[3];

                rgba[0] = multiply_u8_color(rgba[0], a);
                rgba[1] = multiply_u8_color(rgba[1], a);
                rgba[2] = multiply_u8_color(rgba[2], a);
            },
            2 => {
                let a = rgba[3] as u32;

                if a > 0 {
                    rgba[0] = (rgba[0] as u32 * 255 / a) as u8;
                    rgba[1] = (rgba[1] as u32 * 255 / a) as u8;
                    rgba[2] = (rgba[2] as u32 * 255 / a) as u8;
                }
            },
            _ => {},
        }
        if SWAP_RB {
            rgba.swap(0, 2);
        }
        if CLEAR_ALPHA {
            rgba[3] = u8::MAX;
        }
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
    // https://developers.google.com/speed/webp/docs/riff_container
    // First four bytes: `RIFF`, header size 12 bytes
    if !buffer.starts_with(b"RIFF") || buffer.len() < 12 {
        return false;
    }
    let size: [u8; 4] = [buffer[4], buffer[5], buffer[6], buffer[7]];
    // Bytes 4..8 are a little endian u32 indicating
    // > The size of the file in bytes, starting at offset 8.
    // > The maximum value of this field is 2^32 minus 10 bytes and thus the size
    // > of the whole file is at most 4 GiB minus 2 bytes.
    let len: usize = u32::from_le_bytes(size) as usize;
    buffer[8..].len() >= len && &buffer[8..12] == b"WEBP"
}

fn decode_gif(buffer: &[u8], cors_status: CorsStatus) -> Option<RasterImage> {
    let Ok(decoded_gif) = GifDecoder::new(Cursor::new(buffer)) else {
        return None;
    };
    let mut width = 0;
    let mut height = 0;

    // This uses `map_while`, because the first non-decodable frame seems to
    // send the frame iterator into an infinite loop. See
    // <https://github.com/image-rs/image/issues/2442>.
    let mut frame_data = vec![];
    let mut total_number_of_bytes = 0;
    let frames: Vec<ImageFrame> = decoded_gif
        .into_frames()
        .map_while(|decoded_frame| {
            let mut gif_frame = match decoded_frame {
                Ok(decoded_frame) => decoded_frame,
                Err(error) => {
                    debug!("decode GIF frame error: {error}");
                    return None;
                },
            };
            rgba8_byte_swap_colors_inplace(gif_frame.buffer_mut());
            let frame_start = total_number_of_bytes;
            total_number_of_bytes += gif_frame.buffer().len();

            // The image size should be at least as large as the largest frame.
            let frame_width = gif_frame.buffer().width();
            let frame_height = gif_frame.buffer().height();
            width = cmp::max(width, frame_width);
            height = cmp::max(height, frame_height);

            let frame = ImageFrame {
                byte_range: frame_start..total_number_of_bytes,
                delay: Some(Duration::from(gif_frame.delay())),
                width: frame_width,
                height: frame_height,
            };

            frame_data.push(gif_frame);

            Some(frame)
        })
        .collect();

    if frames.is_empty() {
        debug!("Animated Image decoding error");
        return None;
    }

    // Coalesce the frame data into one single shared memory region.
    let mut bytes = Vec::with_capacity(total_number_of_bytes);
    for frame in frame_data {
        bytes.extend_from_slice(frame.buffer());
    }

    Some(RasterImage {
        metadata: ImageMetadata { width, height },
        cors_status,
        frames,
        id: None,
        format: PixelFormat::BGRA8,
        bytes: IpcSharedMemory::from_bytes(&bytes),
    })
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
            b'R', b'I', b'F', b'F', 0x04, 0x00, 0x00, 0x00, b'W', b'E', b'B', b'P',
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
