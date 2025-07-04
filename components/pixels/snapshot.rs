/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{Deref, DerefMut};

use euclid::default::{Rect, Size2D};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{ColorType, ImageEncoder, ImageError};
use ipc_channel::ipc::IpcSharedMemory;
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
    /// Internal data should be threated as opaque (does not mean it actually is)
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

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum SnapshotData {
    // TODO: https://github.com/servo/servo/issues/36594
    //IPC(IpcSharedMemory),
    Owned(Vec<u8>),
}

impl Deref for SnapshotData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match &self {
            //Data::IPC(ipc_shared_memory) => ipc_shared_memory,
            SnapshotData::Owned(items) => items,
        }
    }
}

impl DerefMut for SnapshotData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            //Data::IPC(ipc_shared_memory) => unsafe { ipc_shared_memory.deref_mut() },
            SnapshotData::Owned(items) => items,
        }
    }
}

pub type IpcSnapshot = Snapshot<IpcSharedMemory>;

/// Represents image bitmap with metadata, usually as snapshot of canvas
///
/// This allows us to hold off conversions (BGRA <-> RGBA, (un)premultiply)
/// to when/if they are actually needed (WebGL/WebGPU can load both BGRA and RGBA).
///
/// Inspired by snapshot for concept in WebGPU spec:
/// <https://gpuweb.github.io/gpuweb/#abstract-opdef-get-a-copy-of-the-image-contents-of-a-context>
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct Snapshot<T = SnapshotData> {
    size: Size2D<u32>,
    /// internal data (can be any format it will be converted on use if needed)
    data: T,
    /// RGBA/BGRA (reflect internal data)
    format: SnapshotPixelFormat,
    /// How to treat alpha channel
    alpha_mode: SnapshotAlphaMode,
}

impl<T> Snapshot<T> {
    pub const fn size(&self) -> Size2D<u32> {
        self.size
    }

    pub const fn format(&self) -> SnapshotPixelFormat {
        self.format
    }

    pub const fn alpha_mode(&self) -> SnapshotAlphaMode {
        self.alpha_mode
    }
}

impl Snapshot<SnapshotData> {
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

    pub fn get_rect(&self, rect: Rect<u32>) -> Self {
        let data = rgba8_get_rect(self.as_raw_bytes(), self.size(), rect).to_vec();
        Self::from_vec(rect.size, self.format, self.alpha_mode, data)
    }

    // TODO: https://github.com/servo/servo/issues/36594
    /*
    /// # Safety
    ///
    /// This is safe if data is owned by this process only
    /// (ownership is transferred on send)
    pub unsafe fn from_shared_memory(
        size: Size2D<u32>,
        format: PixelFormat,
        alpha_mode: AlphaMode,
        ism: IpcSharedMemory,
    ) -> Self {
        Self {
            size,
            data: Data::IPC(ism),
            format,
            alpha_mode,
        }
    }
    */

    /// Convert inner data of snapshot to target format and alpha mode.
    /// If data is already in target format and alpha mode no work will be done.
    pub fn transform(
        &mut self,
        target_alpha_mode: SnapshotAlphaMode,
        target_format: SnapshotPixelFormat,
    ) {
        let swap_rb = target_format != self.format;
        let multiply = match (self.alpha_mode, target_alpha_mode) {
            (SnapshotAlphaMode::Opaque, _) => Multiply::None,
            (alpha_mode, SnapshotAlphaMode::Opaque) => {
                if alpha_mode.alpha() == Alpha::Premultiplied {
                    Multiply::UnMultiply
                } else {
                    Multiply::None
                }
            },
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

    pub fn as_bytes(
        &mut self,
        target_alpha_mode: Option<SnapshotAlphaMode>,
        target_format: Option<SnapshotPixelFormat>,
    ) -> (&mut [u8], SnapshotAlphaMode, SnapshotPixelFormat) {
        let target_alpha_mode = target_alpha_mode.unwrap_or(self.alpha_mode);
        let target_format = target_format.unwrap_or(self.format);
        self.transform(target_alpha_mode, target_format);
        (&mut self.data, target_alpha_mode, target_format)
    }

    pub fn to_vec(
        mut self,
        target_alpha_mode: Option<SnapshotAlphaMode>,
        target_format: Option<SnapshotPixelFormat>,
    ) -> (Vec<u8>, SnapshotAlphaMode, SnapshotPixelFormat) {
        let target_alpha_mode = target_alpha_mode.unwrap_or(self.alpha_mode);
        let target_format = target_format.unwrap_or(self.format);
        self.transform(target_alpha_mode, target_format);
        let SnapshotData::Owned(data) = self.data;
        (data, target_alpha_mode, target_format)
    }

    pub fn as_ipc(self) -> Snapshot<IpcSharedMemory> {
        let Snapshot {
            size,
            data,
            format,
            alpha_mode,
        } = self;
        let data = match data {
            //Data::IPC(ipc_shared_memory) => ipc_shared_memory,
            SnapshotData::Owned(items) => IpcSharedMemory::from_bytes(&items),
        };
        Snapshot {
            size,
            data,
            format,
            alpha_mode,
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

        let (data, _, _) = self.as_bytes(
            if *image_type == EncodedImageType::Jpeg {
                Some(SnapshotAlphaMode::AsOpaque {
                    premultiplied: true,
                })
            } else {
                Some(SnapshotAlphaMode::Transparent {
                    premultiplied: false,
                })
            },
            Some(SnapshotPixelFormat::RGBA),
        );

        match image_type {
            EncodedImageType::Png => {
                // FIXME(nox): https://github.com/image-rs/image-png/issues/86
                // FIXME(nox): https://github.com/image-rs/image-png/issues/87
                PngEncoder::new(encoder).write_image(data, width, height, ColorType::Rgba8)
            },
            EncodedImageType::Jpeg => {
                let jpeg_encoder = if let Some(quality) = quality {
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

                jpeg_encoder.write_image(data, width, height, ColorType::Rgba8)
            },
            EncodedImageType::Webp => {
                // No quality support because of https://github.com/image-rs/image/issues/1984
                WebPEncoder::new_lossless(encoder).write_image(
                    data,
                    width,
                    height,
                    ColorType::Rgba8,
                )
            },
        }
    }
}

impl Snapshot<IpcSharedMemory> {
    // TODO: https://github.com/servo/servo/issues/36594
    /*
    /// # Safety
    ///
    /// This is safe if data is owned by this process only
    /// (ownership is transferred on send)
    pub unsafe fn to_data(self) -> Snapshot<Data> {
        let Snapshot {
            size,
            data,
            format,
            alpha_mode,
        } = self;
        Snapshot {
            size,
            data: Data::IPC(data),
            format,
            alpha_mode,
        }
    }
    */
    pub fn to_owned(self) -> Snapshot<SnapshotData> {
        let Snapshot {
            size,
            data,
            format,
            alpha_mode,
        } = self;
        Snapshot {
            size,
            data: SnapshotData::Owned(data.to_vec()),
            format,
            alpha_mode,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn to_ipc_shared_memory(self) -> IpcSharedMemory {
        self.data
    }
}
