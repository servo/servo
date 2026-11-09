/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use std::{cmp, fmt, vec};

#[cfg(feature = "png")]
use image::Limits;
#[cfg(any(
    feature = "png",
    feature = "gif",
    feature = "webp",
    feature = "jpeg",
    feature = "bmp",
    feature = "ico"
))]
use image::codecs;
use image::error::ImageFormatHint;
use image::metadata::LoopCount;
use image::{AnimationDecoder, DynamicImage, ImageDecoder, ImageError, ImageFormat, ImageResult};
use log::debug;

use crate::{
    CorsStatus, ImageFrame, ImageMetadata, PixelFormat, RasterImage, Repeat,
    rgba8_premultiply_inplace,
};

enum GenericImageDecoder<'a> {
    #[cfg(feature = "png")]
    Apng(Box<codecs::png::ApngDecoder<Cursor<&'a [u8]>>>),
    #[cfg(feature = "png")]
    Png(Box<codecs::png::PngDecoder<Cursor<&'a [u8]>>>),
    #[cfg(feature = "gif")]
    Gif(Box<codecs::gif::GifDecoder<Cursor<&'a [u8]>>>),
    #[cfg(feature = "webp")]
    Webp(Box<codecs::webp::WebPDecoder<Cursor<&'a [u8]>>>),
    #[cfg(feature = "jpeg")]
    Jpeg(Box<codecs::jpeg::JpegDecoder<Cursor<&'a [u8]>>>),
    #[cfg(feature = "bmp")]
    Bmp(Box<codecs::bmp::BmpDecoder<Cursor<&'a [u8]>>>),
    #[cfg(feature = "ico")]
    Ico(Box<codecs::ico::IcoDecoder<Cursor<&'a [u8]>>>),
    #[doc(hidden)]
    _Lifetime(std::marker::PhantomData<&'a [u8]>),
}

impl<'a> std::fmt::Debug for GenericImageDecoder<'a> {
    fn fmt(&self, #[allow(unused)] f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "png")]
            Self::Apng(_) => f.debug_tuple("Apng").finish(),
            #[cfg(feature = "png")]
            Self::Png(_) => f.debug_tuple("Png").finish(),
            #[cfg(feature = "gif")]
            Self::Gif(_) => f.debug_tuple("Gif").finish(),
            #[cfg(feature = "webp")]
            Self::Webp(_) => f.debug_tuple("Webp").finish(),
            #[cfg(feature = "jpeg")]
            Self::Jpeg(_) => f.debug_tuple("Jpeg").finish(),
            #[cfg(feature = "bmp")]
            Self::Bmp(_) => f.debug_tuple("Bmp").finish(),
            #[cfg(feature = "ico")]
            Self::Ico(_) => f.debug_tuple("Ico").finish(),
            Self::_Lifetime(_) => unreachable!(),
        }
    }
}

/// Notice that we implement methods that are implemented by default. However, these methods are necessary as the default implementation
/// will return not correct defaults.
impl<'a> image::ImageDecoder for GenericImageDecoder<'a> {
    fn dimensions(&self) -> (u32, u32) {
        match self {
            #[cfg(feature = "png")]
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            #[cfg(feature = "png")]
            GenericImageDecoder::Png(d) => d.dimensions(),
            #[cfg(feature = "gif")]
            GenericImageDecoder::Gif(d) => d.dimensions(),
            #[cfg(feature = "webp")]
            GenericImageDecoder::Webp(d) => d.dimensions(),
            #[cfg(feature = "jpeg")]
            GenericImageDecoder::Jpeg(d) => d.dimensions(),
            #[cfg(feature = "bmp")]
            GenericImageDecoder::Bmp(d) => d.dimensions(),
            #[cfg(feature = "ico")]
            GenericImageDecoder::Ico(d) => d.dimensions(),
            Self::_Lifetime(_) => unreachable!(),
        }
    }

    fn color_type(&self) -> image::ColorType {
        match self {
            #[cfg(feature = "png")]
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            #[cfg(feature = "png")]
            GenericImageDecoder::Png(d) => d.color_type(),
            #[cfg(feature = "gif")]
            GenericImageDecoder::Gif(d) => d.color_type(),
            #[cfg(feature = "webp")]
            GenericImageDecoder::Webp(d) => d.color_type(),
            #[cfg(feature = "jpeg")]
            GenericImageDecoder::Jpeg(d) => d.color_type(),
            #[cfg(feature = "bmp")]
            GenericImageDecoder::Bmp(d) => d.color_type(),
            #[cfg(feature = "ico")]
            GenericImageDecoder::Ico(d) => d.color_type(),
            Self::_Lifetime(_) => unreachable!(),
        }
    }

    fn read_image(self, #[allow(unused)] buf: &mut [u8]) -> ImageResult<()>
    where
        Self: Sized,
    {
        match self {
            #[cfg(feature = "png")]
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            #[cfg(feature = "png")]
            GenericImageDecoder::Png(d) => d.read_image(buf),
            #[cfg(feature = "gif")]
            GenericImageDecoder::Gif(d) => d.read_image(buf),
            #[cfg(feature = "webp")]
            GenericImageDecoder::Webp(d) => d.read_image(buf),
            #[cfg(feature = "jpeg")]
            GenericImageDecoder::Jpeg(d) => d.read_image(buf),
            #[cfg(feature = "bmp")]
            GenericImageDecoder::Bmp(d) => d.read_image(buf),
            #[cfg(feature = "ico")]
            GenericImageDecoder::Ico(d) => d.read_image(buf),
            Self::_Lifetime(_) => unreachable!(),
        }
    }

    fn read_image_boxed(self: Box<Self>, #[allow(unused)] buf: &mut [u8]) -> ImageResult<()> {
        match *self {
            #[cfg(feature = "png")]
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            #[cfg(feature = "png")]
            GenericImageDecoder::Png(d) => d.read_image_boxed(buf),
            #[cfg(feature = "gif")]
            GenericImageDecoder::Gif(d) => d.read_image_boxed(buf),
            #[cfg(feature = "webp")]
            GenericImageDecoder::Webp(d) => d.read_image_boxed(buf),
            #[cfg(feature = "jpeg")]
            GenericImageDecoder::Jpeg(d) => d.read_image_boxed(buf),
            #[cfg(feature = "bmp")]
            GenericImageDecoder::Bmp(d) => d.read_image_boxed(buf),
            #[cfg(feature = "ico")]
            GenericImageDecoder::Ico(d) => d.read_image_boxed(buf),
            Self::_Lifetime(_) => unreachable!(),
        }
    }

    fn icc_profile(&mut self) -> ImageResult<Option<Vec<u8>>> {
        match self {
            #[cfg(feature = "png")]
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            #[cfg(feature = "png")]
            GenericImageDecoder::Png(d) => d.icc_profile(),
            #[cfg(feature = "gif")]
            GenericImageDecoder::Gif(d) => d.icc_profile(),
            #[cfg(feature = "webp")]
            GenericImageDecoder::Webp(d) => d.icc_profile(),
            #[cfg(feature = "jpeg")]
            GenericImageDecoder::Jpeg(d) => d.icc_profile(),
            #[cfg(feature = "bmp")]
            GenericImageDecoder::Bmp(d) => d.icc_profile(),
            #[cfg(feature = "ico")]
            GenericImageDecoder::Ico(d) => d.icc_profile(),
            Self::_Lifetime(_) => unreachable!(),
        }
    }

    fn exif_metadata(&mut self) -> ImageResult<Option<Vec<u8>>> {
        match self {
            #[cfg(feature = "png")]
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            #[cfg(feature = "png")]
            GenericImageDecoder::Png(d) => d.exif_metadata(),
            #[cfg(feature = "gif")]
            GenericImageDecoder::Gif(d) => d.exif_metadata(),
            #[cfg(feature = "webp")]
            GenericImageDecoder::Webp(d) => d.exif_metadata(),
            #[cfg(feature = "jpeg")]
            GenericImageDecoder::Jpeg(d) => d.exif_metadata(),
            #[cfg(feature = "bmp")]
            GenericImageDecoder::Bmp(d) => d.exif_metadata(),
            #[cfg(feature = "ico")]
            GenericImageDecoder::Ico(d) => d.exif_metadata(),
            Self::_Lifetime(_) => unreachable!(),
        }
    }

    fn xmp_metadata(&mut self) -> ImageResult<Option<Vec<u8>>> {
        match self {
            #[cfg(feature = "png")]
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            #[cfg(feature = "png")]
            GenericImageDecoder::Png(d) => d.xmp_metadata(),
            #[cfg(feature = "gif")]
            GenericImageDecoder::Gif(d) => d.xmp_metadata(),
            #[cfg(feature = "webp")]
            GenericImageDecoder::Webp(d) => d.xmp_metadata(),
            #[cfg(feature = "jpeg")]
            GenericImageDecoder::Jpeg(d) => d.xmp_metadata(),
            #[cfg(feature = "bmp")]
            GenericImageDecoder::Bmp(d) => d.xmp_metadata(),
            #[cfg(feature = "ico")]
            GenericImageDecoder::Ico(d) => d.xmp_metadata(),
            Self::_Lifetime(_) => unreachable!(),
        }
    }

    fn iptc_metadata(&mut self) -> ImageResult<Option<Vec<u8>>> {
        match self {
            #[cfg(feature = "png")]
            GenericImageDecoder::Apng(_) => {
                unreachable!("Animated image should never go into non-animated values")
            },
            #[cfg(feature = "png")]
            GenericImageDecoder::Png(d) => d.iptc_metadata(),
            #[cfg(feature = "gif")]
            GenericImageDecoder::Gif(d) => d.iptc_metadata(),
            #[cfg(feature = "webp")]
            GenericImageDecoder::Webp(d) => d.iptc_metadata(),
            #[cfg(feature = "jpeg")]
            GenericImageDecoder::Jpeg(d) => d.iptc_metadata(),
            #[cfg(feature = "bmp")]
            GenericImageDecoder::Bmp(d) => d.iptc_metadata(),
            #[cfg(feature = "ico")]
            GenericImageDecoder::Ico(d) => d.iptc_metadata(),
            Self::_Lifetime(_) => unreachable!(),
        }
    }
}

/// See documentation on impl `image::ImageDecoder`
impl<'a> AnimationDecoder<'a> for GenericImageDecoder<'a> {
    fn into_frames(self) -> image::Frames<'a> {
        match self {
            #[cfg(feature = "png")]
            GenericImageDecoder::Apng(decoder) => decoder.into_frames(),
            #[cfg(feature = "gif")]
            GenericImageDecoder::Gif(decoder) => decoder.into_frames(),
            #[cfg(feature = "webp")]
            GenericImageDecoder::Webp(decoder) => decoder.into_frames(),
            _ => unreachable!("Should never decode these images with animated decoder"),
        }
    }

    fn loop_count(&self) -> LoopCount {
        match self {
            #[cfg(feature = "png")]
            GenericImageDecoder::Apng(decoder) => decoder.loop_count(),
            #[cfg(feature = "gif")]
            GenericImageDecoder::Gif(decoder) => decoder.loop_count(),
            #[cfg(feature = "webp")]
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
        #[allow(unused)]
        let reader = Cursor::new(buffer);
        let decoder = match format {
            #[cfg(feature = "png")]
            ImageFormat::Png => {
                let limits = Limits::default();
                let png_decoder = codecs::png::PngDecoder::with_limits(reader, limits)?;
                if png_decoder.is_apng().unwrap_or_default() {
                    let decoder = png_decoder.apng()?;
                    GenericImageDecoder::Apng(Box::new(decoder))
                } else {
                    GenericImageDecoder::Png(Box::new(png_decoder))
                }
            },
            #[cfg(feature = "gif")]
            ImageFormat::Gif => {
                GenericImageDecoder::Gif(Box::new(codecs::gif::GifDecoder::new(reader)?))
            },
            #[cfg(feature = "webp")]
            ImageFormat::WebP => {
                GenericImageDecoder::Webp(Box::new(codecs::webp::WebPDecoder::new(reader)?))
            },
            #[cfg(feature = "jpeg")]
            ImageFormat::Jpeg => {
                GenericImageDecoder::Jpeg(Box::new(codecs::jpeg::JpegDecoder::new(reader)?))
            },
            #[cfg(feature = "bmp")]
            ImageFormat::Bmp => {
                GenericImageDecoder::Bmp(Box::new(codecs::bmp::BmpDecoder::new(reader)?))
            },
            #[cfg(feature = "ico")]
            ImageFormat::Ico => {
                GenericImageDecoder::Ico(Box::new(codecs::ico::IcoDecoder::new(reader)?))
            },
            _ => {
                return Err(ImageError::Unsupported(
                    ImageFormatHint::Exact(format).into(),
                ));
            },
        };
        #[allow(unreachable_code)]
        Ok(DefaultImageDecoder { decoder })
    }

    fn is_animated(&self) -> bool {
        match &self.decoder {
            #[cfg(feature = "png")]
            GenericImageDecoder::Apng(_) => true,
            #[cfg(feature = "gif")]
            GenericImageDecoder::Gif(_) => true,
            #[cfg(feature = "webp")]
            GenericImageDecoder::Webp(decoder) => decoder.has_animation(),
            #[cfg(feature = "png")]
            GenericImageDecoder::Png(_) => false,
            #[cfg(feature = "jpeg")]
            GenericImageDecoder::Jpeg(_) => false,
            #[cfg(feature = "bmp")]
            GenericImageDecoder::Bmp(_) => false,
            #[cfg(feature = "ico")]
            GenericImageDecoder::Ico(_) => false,
            GenericImageDecoder::_Lifetime(_) => unreachable!(),
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
        load_time: None,
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
        load_time: None,
        frames,
        id: None,
        format: PixelFormat::RGBA8,
        bytes: Arc::new(bytes),
        is_opaque,
        loop_count: Some(loop_count),
    })
}
