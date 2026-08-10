/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{ExtendedColorType, GenericImageView, ImageEncoder, ImageResult, Rgb};

#[derive(PartialEq)]
pub enum EncodedImageType {
    Png,
    Jpeg,
    Webp,
}

impl From<&str> for EncodedImageType {
    // From: https://html.spec.whatwg.org/multipage/#serialising-bitmaps-to-a-file
    // User agents must support PNG ("image/png"). User agents may support other
    // types. If the user agent does not support the requested type, then it
    // must create the file using the PNG format.
    // Anything different than image/jpeg or image/webp is thus treated as PNG.
    fn from(mime_string: &str) -> Self {
        if mime_string.eq_ignore_ascii_case("image/jpeg") {
            Self::Jpeg
        } else if mime_string.eq_ignore_ascii_case("image/webp") {
            Self::Webp
        } else {
            Self::Png
        }
    }
}

impl EncodedImageType {
    pub fn as_mime_type(&self) -> String {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
        .to_owned()
    }
}

pub(crate) trait ServoImageEncoder<W: std::io::Write> {
    fn encode_to_writer(
        data: &[u8],
        image_type: &EncodedImageType,
        width: u32,
        height: u32,
        writer: W,
        quality: Option<f64>,
    ) -> ImageResult<()>;
}

pub(crate) struct DefaultImageEncoder {}

impl<W: std::io::Write> ServoImageEncoder<W> for DefaultImageEncoder {
    fn encode_to_writer(
        data: &[u8],
        image_type: &EncodedImageType,
        width: u32,
        height: u32,
        writer: W,
        quality: Option<f64>,
    ) -> ImageResult<()> {
        match image_type {
            EncodedImageType::Png => {
                // FIXME(nox): https://github.com/image-rs/image-png/issues/86
                // FIXME(nox): https://github.com/image-rs/image-png/issues/87
                PngEncoder::new(writer).write_image(data, width, height, ExtendedColorType::Rgba8)
            },
            EncodedImageType::Jpeg => {
                let mut jpeg_encoder = if let Some(quality) = quality {
                    // The specification allows quality to be in [0.0..1.0] but the JPEG encoder
                    // expects it to be in [1..100]
                    if (0.0..=1.0).contains(&quality) {
                        JpegEncoder::new_with_quality(
                            writer,
                            (quality * 100.0).round().clamp(1.0, 100.0) as u8,
                        )
                    } else {
                        JpegEncoder::new(writer)
                    }
                } else {
                    JpegEncoder::new(writer)
                };

                // JPEG doesn't support transparency, so simply calling jpeg_encoder.write_image fails here.
                // Instead we have to create a struct to translate from rgba to rgb.
                struct RgbaDataForJpegEncoder<'a> {
                    width: u32,
                    height: u32,
                    data: &'a [u8],
                }

                impl<'a> GenericImageView for RgbaDataForJpegEncoder<'a> {
                    type Pixel = Rgb<u8>;

                    fn dimensions(&self) -> (u32, u32) {
                        (self.width, self.height)
                    }

                    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
                        let offset = (self.width * y + x) as usize * 4;
                        Rgb([
                            self.data[offset],
                            self.data[offset + 1],
                            self.data[offset + 2],
                        ])
                    }
                }

                let image = RgbaDataForJpegEncoder {
                    width,
                    height,
                    data,
                };

                jpeg_encoder.encode_image(&image)
            },
            EncodedImageType::Webp => {
                // No quality support because of https://github.com/image-rs/image/issues/1984
                WebPEncoder::new_lossless(writer).write_image(
                    data,
                    width,
                    height,
                    ExtendedColorType::Rgba8,
                )
            },
        }
    }
}
