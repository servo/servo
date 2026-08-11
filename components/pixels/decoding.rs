/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use std::{cmp, fmt, vec};

use image::codecs::{bmp, gif, ico, jpeg, png, webp};
use image::error::ImageFormatHint;
use image::metadata::LoopCount;
use image::{
    AnimationDecoder, DynamicImage, ImageDecoder, ImageError, ImageFormat, ImageResult, Limits,
};
use log::debug;

use crate::{
    CorsStatus, ImageFrame, ImageMetadata, PixelFormat, RasterImage, Repeat,
    rgba8_premultiply_inplace,
};

enum GenericImageDecoder<'a> {
    Apng(Box<png::ApngDecoder<Cursor<&'a [u8]>>>),
    Png(Box<png::PngDecoder<Cursor<&'a [u8]>>>),
    Gif(Box<gif::GifDecoder<Cursor<&'a [u8]>>>),
    Webp(Box<webp::WebPDecoder<Cursor<&'a [u8]>>>),
    Jpeg(Box<jpeg::JpegDecoder<Cursor<&'a [u8]>>>),
    Bmp(Box<bmp::BmpDecoder<Cursor<&'a [u8]>>>),
    Ico(Box<ico::IcoDecoder<Cursor<&'a [u8]>>>),
}

impl<'a> std::fmt::Debug for GenericImageDecoder<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Apng(_) => f.debug_tuple("Apng").finish(),
            Self::Png(_) => f.debug_tuple("Png").finish(),
            Self::Gif(_) => f.debug_tuple("Gif").finish(),
            Self::Webp(_) => f.debug_tuple("Webp").finish(),
            Self::Jpeg(_) => f.debug_tuple("Jpeg").finish(),
            Self::Bmp(_) => f.debug_tuple("Bmp").finish(),
            Self::Ico(_) => f.debug_tuple("Ico").finish(),
        }
    }
}

/// Notice that we implement methods that are implemented by default. However, these methods are necessary as the default implementation
/// will return not correct defaults.
impl<'a> image::ImageDecoder for GenericImageDecoder<'a> {
    fn dimensions(&self) -> (u32, u32) {
        match self {
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            GenericImageDecoder::Png(d) => d.dimensions(),
            GenericImageDecoder::Gif(d) => d.dimensions(),
            GenericImageDecoder::Webp(d) => d.dimensions(),
            GenericImageDecoder::Jpeg(d) => d.dimensions(),
            GenericImageDecoder::Bmp(d) => d.dimensions(),
            GenericImageDecoder::Ico(d) => d.dimensions(),
        }
    }

    fn color_type(&self) -> image::ColorType {
        match self {
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            GenericImageDecoder::Png(d) => d.color_type(),
            GenericImageDecoder::Gif(d) => d.color_type(),
            GenericImageDecoder::Webp(d) => d.color_type(),
            GenericImageDecoder::Jpeg(d) => d.color_type(),
            GenericImageDecoder::Bmp(d) => d.color_type(),
            GenericImageDecoder::Ico(d) => d.color_type(),
        }
    }

    fn read_image(self, buf: &mut [u8]) -> ImageResult<()>
    where
        Self: Sized,
    {
        match self {
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            GenericImageDecoder::Png(d) => d.read_image(buf),
            GenericImageDecoder::Gif(d) => d.read_image(buf),
            GenericImageDecoder::Webp(d) => d.read_image(buf),
            GenericImageDecoder::Jpeg(d) => d.read_image(buf),
            GenericImageDecoder::Bmp(d) => d.read_image(buf),
            GenericImageDecoder::Ico(d) => d.read_image(buf),
        }
    }

    fn read_image_boxed(self: Box<Self>, buf: &mut [u8]) -> ImageResult<()> {
        match *self {
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            GenericImageDecoder::Png(d) => d.read_image_boxed(buf),
            GenericImageDecoder::Gif(d) => d.read_image_boxed(buf),
            GenericImageDecoder::Webp(d) => d.read_image_boxed(buf),
            GenericImageDecoder::Jpeg(d) => d.read_image_boxed(buf),
            GenericImageDecoder::Bmp(d) => d.read_image_boxed(buf),
            GenericImageDecoder::Ico(d) => d.read_image_boxed(buf),
        }
    }

    fn icc_profile(&mut self) -> ImageResult<Option<Vec<u8>>> {
        match self {
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            GenericImageDecoder::Png(d) => d.icc_profile(),
            GenericImageDecoder::Gif(d) => d.icc_profile(),
            GenericImageDecoder::Webp(d) => d.icc_profile(),
            GenericImageDecoder::Jpeg(d) => d.icc_profile(),
            GenericImageDecoder::Bmp(d) => d.icc_profile(),
            GenericImageDecoder::Ico(d) => d.icc_profile(),
        }
    }

    fn exif_metadata(&mut self) -> ImageResult<Option<Vec<u8>>> {
        match self {
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            GenericImageDecoder::Png(d) => d.exif_metadata(),
            GenericImageDecoder::Gif(d) => d.exif_metadata(),
            GenericImageDecoder::Webp(d) => d.exif_metadata(),
            GenericImageDecoder::Jpeg(d) => d.exif_metadata(),
            GenericImageDecoder::Bmp(d) => d.exif_metadata(),
            GenericImageDecoder::Ico(d) => d.exif_metadata(),
        }
    }

    fn xmp_metadata(&mut self) -> ImageResult<Option<Vec<u8>>> {
        match self {
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            GenericImageDecoder::Png(d) => d.xmp_metadata(),
            GenericImageDecoder::Gif(d) => d.xmp_metadata(),
            GenericImageDecoder::Webp(d) => d.xmp_metadata(),
            GenericImageDecoder::Jpeg(d) => d.xmp_metadata(),
            GenericImageDecoder::Bmp(d) => d.xmp_metadata(),
            GenericImageDecoder::Ico(d) => d.xmp_metadata(),
        }
    }

    fn iptc_metadata(&mut self) -> ImageResult<Option<Vec<u8>>> {
        match self {
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            GenericImageDecoder::Png(d) => d.iptc_metadata(),
            GenericImageDecoder::Gif(d) => d.iptc_metadata(),
            GenericImageDecoder::Webp(d) => d.iptc_metadata(),
            GenericImageDecoder::Jpeg(d) => d.iptc_metadata(),
            GenericImageDecoder::Bmp(d) => d.iptc_metadata(),
            GenericImageDecoder::Ico(d) => d.iptc_metadata(),
        }
    }
}

/// See documentation on impl `image::ImageDecoder`
impl<'a> AnimationDecoder<'a> for GenericImageDecoder<'a> {
    fn into_frames(self) -> image::Frames<'a> {
        match self {
            GenericImageDecoder::Apng(decoder) => decoder.into_frames(),
            GenericImageDecoder::Gif(decoder) => decoder.into_frames(),
            GenericImageDecoder::Webp(decoder) => decoder.into_frames(),
            _ => unreachable!("Should never decode these images with animated decoder"),
        }
    }

    fn loop_count(&self) -> LoopCount {
        match self {
            GenericImageDecoder::Apng(decoder) => decoder.loop_count(),
            GenericImageDecoder::Gif(decoder) => decoder.loop_count(),
            GenericImageDecoder::Webp(decoder) => decoder.loop_count(),
            _ => unreachable!("Should never decode these images with animated decoder"),
        }
    }
}

#[derive(Debug)]
/// Servo Default Image decoder using image-rs for decoding.
pub(crate) struct DefaultImageDecoder<'a> {
    decoder: GenericImageDecoder<'a>,
}

/// Main Image decoder trait.
pub(crate) trait ServoImageDecoder<'a>: Sized + std::fmt::Debug {
    /// Create a decoder for a `format` from a `buffer`.
    fn make_decoder(format: ImageFormat, buffer: &'a [u8]) -> ImageResult<Self>;
    fn is_animated(&self) -> bool;
    /// Return the created decoder in `impl ImageDecoder`
    fn decoder(self) -> impl ImageDecoder;
    /// Return the created decoder in `impl AnimationDecoder` if animated.
    fn animated_decoder(self) -> impl AnimationDecoder<'a>;
}

impl<'a> ServoImageDecoder<'a> for DefaultImageDecoder<'a> {
    fn make_decoder(format: ImageFormat, buffer: &'a [u8]) -> ImageResult<Self> {
        let reader = Cursor::new(buffer);
        let decoder = match format {
            ImageFormat::Png => {
                let limits = Limits::default();
                let png_decoder = png::PngDecoder::with_limits(reader, limits)?;
                if png_decoder.is_apng().unwrap_or_default() {
                    let decoder = png_decoder.apng()?;
                    GenericImageDecoder::Apng(Box::new(decoder))
                } else {
                    GenericImageDecoder::Png(Box::new(png_decoder))
                }
            },
            ImageFormat::Gif => GenericImageDecoder::Gif(Box::new(gif::GifDecoder::new(reader)?)),
            ImageFormat::WebP => {
                GenericImageDecoder::Webp(Box::new(webp::WebPDecoder::new(reader)?))
            },
            ImageFormat::Jpeg => {
                GenericImageDecoder::Jpeg(Box::new(jpeg::JpegDecoder::new(reader)?))
            },
            ImageFormat::Bmp => GenericImageDecoder::Bmp(Box::new(bmp::BmpDecoder::new(reader)?)),
            ImageFormat::Ico => GenericImageDecoder::Ico(Box::new(ico::IcoDecoder::new(reader)?)),
            _ => {
                return Err(ImageError::Unsupported(
                    ImageFormatHint::Exact(format).into(),
                ));
            },
        };
        Ok(DefaultImageDecoder { decoder })
    }

    fn is_animated(&self) -> bool {
        match &self.decoder {
            GenericImageDecoder::Apng(_) | GenericImageDecoder::Gif(_) => true,
            GenericImageDecoder::Webp(decoder) => decoder.has_animation(),
            GenericImageDecoder::Png(_) |
            GenericImageDecoder::Jpeg(_) |
            GenericImageDecoder::Bmp(_) |
            GenericImageDecoder::Ico(_) => false,
        }
    }

    fn decoder(self) -> impl ImageDecoder {
        self.decoder
    }

    fn animated_decoder(self) -> impl AnimationDecoder<'a> {
        self.decoder
    }
}

pub(crate) fn decode_static_image(
    cors_status: CorsStatus,
    mut image_decoder: impl ImageDecoder,
) -> Option<RasterImage> {
    let orientation = image_decoder.orientation();

    let Ok(mut dynamic_image) = DynamicImage::from_decoder(image_decoder) else {
        debug!("Image decoding error");
        return None;
    };

    if let Ok(orientation) = orientation {
        dynamic_image.apply_orientation(orientation);
    }

    let mut rgba = dynamic_image.into_rgba8();

    // Store pre-multiplied data as that prevents having to do conversions of the data at later
    // times. This does cause an issue with some canvas APIs. See:
    // https://github.com/servo/servo/issues/40257
    let is_opaque = rgba8_premultiply_inplace(&mut rgba);

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
        format: PixelFormat::RGBA8,
        frames: vec![frame],
        bytes: Arc::new(rgba.into_vec()),
        id: None,
        cors_status,
        is_opaque,
        loop_count: None,
    })
}

pub(crate) fn decode_animated_image<'a, T>(
    cors_status: CorsStatus,
    animated_image_decoder: T,
) -> Option<RasterImage>
where
    T: AnimationDecoder<'a>,
{
    let mut width = 0;
    let mut height = 0;

    // This uses `map_while`, because the first non-decodable frame seems to
    // send the frame iterator into an infinite loop. See
    // <https://github.com/image-rs/image/issues/2442>.
    let mut frame_data = vec![];
    let mut total_number_of_bytes = 0;
    let mut is_opaque = true;
    let loop_count = match animated_image_decoder.loop_count() {
        LoopCount::Finite(repeat_time) => Repeat::Finite(repeat_time),
        LoopCount::Infinite => Repeat::Infinite,
    };
    let frames: Vec<ImageFrame> = animated_image_decoder
        .into_frames()
        .map_while(|decoded_frame| {
            let mut animated_frame = match decoded_frame {
                Ok(decoded_frame) => decoded_frame,
                Err(error) => {
                    debug!("decode Animated frame error: {error}");
                    return None;
                },
            };

            // Store pre-multiplied data as that prevents having to do conversions of the data at later
            // times. This does cause an issue with some canvas APIs. See:
            // https://github.com/servo/servo/issues/40257
            is_opaque = rgba8_premultiply_inplace(animated_frame.buffer_mut()) && is_opaque;

            let frame_start = total_number_of_bytes;
            total_number_of_bytes += animated_frame.buffer().len();

            // The image size should be at least as large as the largest frame.
            let frame_width = animated_frame.buffer().width();
            let frame_height = animated_frame.buffer().height();
            width = cmp::max(width, frame_width);
            height = cmp::max(height, frame_height);

            let frame = ImageFrame {
                byte_range: frame_start..total_number_of_bytes,
                delay: Some(Duration::from(animated_frame.delay())),
                width: frame_width,
                height: frame_height,
            };

            frame_data.push(animated_frame);

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
        format: PixelFormat::RGBA8,
        bytes: Arc::new(bytes),
        is_opaque,
        loop_count: Some(loop_count),
    })
}
