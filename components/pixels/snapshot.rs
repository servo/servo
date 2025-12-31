/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{Bound, Deref, DerefMut, Range, RangeBounds};
use std::sync::Arc;

use base::generic_channel::GenericSharedMemory;
use euclid::default::{Rect, Size2D};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{ExtendedColorType, GenericImageView, ImageEncoder, ImageError, Rgb};
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

use crate::{EncodedImageType, Multiply, rgba8_get_rect, transform_inplace};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum SnapshotPixelFormat {
    #[default]
    RGBA,
    BGRA,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum Alpha {
    Premultiplied,
    NotPremultiplied,
    /// This is used for opaque textures for which the presence of alpha in the
    /// output data format does not matter.
    DontCare,
}

impl Alpha {
    pub const fn from_premultiplied(is_premultiplied: bool) -> Self {
        if is_premultiplied {
            Self::Premultiplied
        } else {
            Self::NotPremultiplied
        }
    }

    pub const fn needs_alpha_multiplication(&self) -> bool {
        match self {
            Alpha::Premultiplied => false,
            Alpha::NotPremultiplied => true,
            Alpha::DontCare => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum SnapshotAlphaMode {
    /// Internal data is opaque (alpha is cleared to 1)
    Opaque,
    /// Internal data should be treated as opaque (does not mean it actually is)
    AsOpaque { premultiplied: bool },
    /// Data is not opaque
    Transparent { premultiplied: bool },
}

impl Default for SnapshotAlphaMode {
    fn default() -> Self {
        Self::Transparent {
            premultiplied: true,
        }
    }
}

impl SnapshotAlphaMode {
    pub const fn alpha(&self) -> Alpha {
        match self {
            SnapshotAlphaMode::Opaque => Alpha::DontCare,
            SnapshotAlphaMode::AsOpaque { premultiplied } => {
                Alpha::from_premultiplied(*premultiplied)
            },
            SnapshotAlphaMode::Transparent { premultiplied } => {
                Alpha::from_premultiplied(*premultiplied)
            },
        }
    }
}

/// The data in a [`Snapshot`]. If created via shared memory, this will be
/// the `SharedMemory` variant, but otherwise it is the `Owned` variant.
/// any attempt to mutate the [`Snapshot`] will convert it to the `Owned`
/// variant.
#[derive(Clone, Debug, MallocSizeOf)]
pub enum SnapshotData {
    SharedMemory(
        #[conditional_malloc_size_of] Arc<GenericSharedMemory>,
        Range<usize>,
    ),
    SharedVec(#[conditional_malloc_size_of] Arc<Vec<u8>>, Range<usize>),
    Owned(Vec<u8>),
}

impl SnapshotData {
    fn to_vec(&self) -> Vec<u8> {
        match &self {
            SnapshotData::SharedMemory(data, byte_range) => Vec::from(&data[byte_range.clone()]),
            SnapshotData::SharedVec(data, byte_range) => Vec::from(&data[byte_range.clone()]),
            SnapshotData::Owned(data) => data.clone(),
        }
    }
}

impl DerefMut for SnapshotData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            SnapshotData::SharedMemory(..) | SnapshotData::SharedVec(..) => {
                *self = SnapshotData::Owned(self.to_vec());
                &mut *self
            },
            SnapshotData::Owned(items) => items,
        }
    }
}

impl Deref for SnapshotData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match &self {
            SnapshotData::SharedMemory(data, byte_range) => &data[byte_range.clone()],
            SnapshotData::SharedVec(data, byte_range) => &data[byte_range.clone()],
            SnapshotData::Owned(items) => items,
        }
    }
}

/// Represents image bitmap with metadata, usually as snapshot of canvas
///
/// This allows us to hold off conversions (BGRA <-> RGBA, (un)premultiply)
/// to when/if they are actually needed (WebGL/WebGPU can load both BGRA and RGBA).
///
/// Inspired by snapshot for concept in WebGPU spec:
/// <https://gpuweb.github.io/gpuweb/#abstract-opdef-get-a-copy-of-the-image-contents-of-a-context>
#[derive(Clone, Debug, MallocSizeOf)]
pub struct Snapshot {
    size: Size2D<u32>,
    /// internal data (can be any format it will be converted on use if needed)
    data: SnapshotData,
    /// RGBA/BGRA (reflect internal data)
    format: SnapshotPixelFormat,
    /// How to treat alpha channel
    alpha_mode: SnapshotAlphaMode,
}

impl Snapshot {
    pub const fn size(&self) -> Size2D<u32> {
        self.size
    }

    pub const fn format(&self) -> SnapshotPixelFormat {
        self.format
    }

    pub const fn alpha_mode(&self) -> SnapshotAlphaMode {
        self.alpha_mode
    }

    pub fn empty() -> Self {
        Self {
            size: Size2D::zero(),
            data: SnapshotData::Owned(vec![]),
            format: SnapshotPixelFormat::RGBA,
            alpha_mode: SnapshotAlphaMode::Transparent {
                premultiplied: true,
            },
        }
    }

    /// Returns snapshot with provided size that is black transparent alpha
    pub fn cleared(size: Size2D<u32>) -> Self {
        Self {
            size,
            data: SnapshotData::Owned(vec![0; size.area() as usize * 4]),
            format: SnapshotPixelFormat::RGBA,
            alpha_mode: SnapshotAlphaMode::Transparent {
                premultiplied: true,
            },
        }
    }

    pub fn from_vec(
        size: Size2D<u32>,
        format: SnapshotPixelFormat,
        alpha_mode: SnapshotAlphaMode,
        data: Vec<u8>,
    ) -> Self {
        Self {
            size,
            data: SnapshotData::Owned(data),
            format,
            alpha_mode,
        }
    }

    pub fn from_arc_vec(
        size: Size2D<u32>,
        format: SnapshotPixelFormat,
        alpha_mode: SnapshotAlphaMode,
        data: Arc<Vec<u8>>,
        byte_range_bounds: impl RangeBounds<usize>,
    ) -> Self {
        let range_start = match byte_range_bounds.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => *bound + 1,
            Bound::Unbounded => 0,
        };
        let range_end = match byte_range_bounds.end_bound() {
            Bound::Included(bound) => *bound + 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => data.len(),
        };
        Self {
            size,
            data: SnapshotData::SharedVec(data, range_start..range_end),
            format,
            alpha_mode,
        }
    }

    pub fn get_rect(&self, rect: Rect<u32>) -> Self {
        let data = rgba8_get_rect(self.as_raw_bytes(), self.size(), rect).to_vec();
        Self::from_vec(rect.size, self.format, self.alpha_mode, data)
    }

    /// Convert inner data of snapshot to target format and alpha mode.
    /// If data is already in target format and alpha mode no work will be done.
    pub fn transform(
        &mut self,
        target_alpha_mode: SnapshotAlphaMode,
        target_format: SnapshotPixelFormat,
    ) {
        if self.alpha_mode == target_alpha_mode && target_format == self.format {
            return;
        }

        let swap_rb = target_format != self.format;
        let multiply = match (self.alpha_mode, target_alpha_mode) {
            (SnapshotAlphaMode::Opaque, _) => Multiply::None,
            (alpha_mode, SnapshotAlphaMode::Opaque)
                if alpha_mode.alpha() == Alpha::Premultiplied =>
            {
                Multiply::UnMultiply
            },
            (_, SnapshotAlphaMode::Opaque) => Multiply::None,
            (
                SnapshotAlphaMode::Transparent { premultiplied } |
                SnapshotAlphaMode::AsOpaque { premultiplied },
                SnapshotAlphaMode::Transparent {
                    premultiplied: target_premultiplied,
                } |
                SnapshotAlphaMode::AsOpaque {
                    premultiplied: target_premultiplied,
                },
            ) => {
                if premultiplied == target_premultiplied {
                    Multiply::None
                } else if target_premultiplied {
                    Multiply::PreMultiply
                } else {
                    Multiply::UnMultiply
                }
            },
        };

        let clear_alpha = !matches!(self.alpha_mode, SnapshotAlphaMode::Opaque) &&
            matches!(target_alpha_mode, SnapshotAlphaMode::Opaque);

        if matches!(multiply, Multiply::None) && !swap_rb && !clear_alpha {
            return;
        }

        transform_inplace(self.data.deref_mut(), multiply, swap_rb, clear_alpha);
        self.alpha_mode = target_alpha_mode;
        self.format = target_format;
    }

    pub fn as_raw_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn as_raw_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn to_shared(&self) -> SharedSnapshot {
        let (data, byte_range) = match &self.data {
            SnapshotData::SharedMemory(data, byte_range) => (data.clone(), byte_range.clone()),
            SnapshotData::SharedVec(data, byte_range) => (
                Arc::new(GenericSharedMemory::from_bytes(data)),
                byte_range.clone(),
            ),
            SnapshotData::Owned(data) => (
                Arc::new(GenericSharedMemory::from_bytes(data)),
                0..data.len(),
            ),
        };
        SharedSnapshot {
            size: self.size,
            data,
            byte_range,
            format: self.format,
            alpha_mode: self.alpha_mode,
        }
    }

    pub fn encode_for_mime_type<W: std::io::Write>(
        &mut self,
        image_type: &EncodedImageType,
        quality: Option<f64>,
        encoder: &mut W,
    ) -> Result<(), ImageError> {
        let width = self.size.width;
        let height = self.size.height;
        let alpha_mode = match image_type {
            EncodedImageType::Jpeg => SnapshotAlphaMode::AsOpaque {
                premultiplied: true,
            },
            _ => SnapshotAlphaMode::Transparent {
                premultiplied: false,
            },
        };

        self.transform(alpha_mode, SnapshotPixelFormat::RGBA);
        let data = &self.data;

        match image_type {
            EncodedImageType::Png => {
                // FIXME(nox): https://github.com/image-rs/image-png/issues/86
                // FIXME(nox): https://github.com/image-rs/image-png/issues/87
                PngEncoder::new(encoder).write_image(data, width, height, ExtendedColorType::Rgba8)
            },
            EncodedImageType::Jpeg => {
                let mut jpeg_encoder = if let Some(quality) = quality {
                    // The specification allows quality to be in [0.0..1.0] but the JPEG encoder
                    // expects it to be in [1..100]
                    if (0.0..=1.0).contains(&quality) {
                        JpegEncoder::new_with_quality(
                            encoder,
                            (quality * 100.0).round().clamp(1.0, 100.0) as u8,
                        )
                    } else {
                        JpegEncoder::new(encoder)
                    }
                } else {
                    JpegEncoder::new(encoder)
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
                WebPEncoder::new_lossless(encoder).write_image(
                    data,
                    width,
                    height,
                    ExtendedColorType::Rgba8,
                )
            },
        }
    }
}

impl From<Snapshot> for Vec<u8> {
    fn from(value: Snapshot) -> Self {
        match value.data {
            SnapshotData::SharedMemory(..) => Vec::from(value.as_raw_bytes()),
            SnapshotData::SharedVec(..) => Vec::from(value.as_raw_bytes()),
            SnapshotData::Owned(data) => data,
        }
    }
}

/// A version of [`Snapshot`] that can be sent across IPC channels.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct SharedSnapshot {
    /// The physical size of this [`SharedSnapshot`].
    size: Size2D<u32>,
    /// The shared data of this [`SharedSnapshot`].
    #[conditional_malloc_size_of]
    data: Arc<GenericSharedMemory>,
    /// The byte range of the data within the shared memory segment. This is used to
    /// send individual image frames of animated images.
    byte_range: Range<usize>,
    /// The [`SnapshotPixelFormat`] of this [`SharedSnapshot`]
    format: SnapshotPixelFormat,
    /// The [`SnapshotAlphaMode`] of this [`SharedSnapshot`].
    alpha_mode: SnapshotAlphaMode,
}

impl SharedSnapshot {
    pub fn to_owned(&self) -> Snapshot {
        Snapshot {
            size: self.size,
            data: SnapshotData::SharedMemory(self.data.clone(), self.byte_range.clone()),
            format: self.format,
            alpha_mode: self.alpha_mode,
        }
    }

    /// Returns snapshot with provided size that is black transparent alpha
    pub fn cleared(size: Size2D<u32>) -> Self {
        let length_in_bytes = size.area() as usize * 4;
        Self {
            size,
            data: Arc::new(GenericSharedMemory::from_byte(0, length_in_bytes)),
            byte_range: 0..length_in_bytes,
            format: SnapshotPixelFormat::RGBA,
            alpha_mode: SnapshotAlphaMode::Transparent {
                premultiplied: true,
            },
        }
    }

    pub const fn size(&self) -> Size2D<u32> {
        self.size
    }

    pub const fn format(&self) -> SnapshotPixelFormat {
        self.format
    }

    pub const fn alpha_mode(&self) -> SnapshotAlphaMode {
        self.alpha_mode
    }

    pub fn data(&self) -> &[u8] {
        &(&self.data)[self.byte_range.clone()]
    }

    pub fn shared_memory(&self) -> GenericSharedMemory {
        (*self.data).clone()
    }
}
