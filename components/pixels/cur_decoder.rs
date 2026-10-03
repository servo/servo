/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::{BufRead, Error, ErrorKind, Seek};

use ico::{IconDir, IconDirEntry};
use image::error::{ParameterError, ParameterErrorKind};
use image::metadata::LoopCount;
use image::{AnimationDecoder, ImageError, ImageResult};
use log::debug;

// rust-ico decoder for cur files
#[derive(Debug)]
pub struct RustIcoDecoder {
    entry: IconDirEntry,
}

impl RustIcoDecoder {
    pub fn new<R: BufRead + Seek>(r: R) -> ImageResult<Self> {
        let icon_dir = IconDir::read(r).map_err(ImageError::IoError)?;
        // Following convention established by image::ico decoder, we pick the "best" icon entry to decode.
        // Best is defined as largest size by pixels
        // TODO: Handle .cur hotspot coordinates, multiple image entries for cur and ico files
        let mut best_entry = None;
        let mut best_score = 0;
        for entry in icon_dir.entries() {
            let score = entry.width() * entry.height();
            if score > best_score {
                best_score = score;
                best_entry = Some(entry);
            }
        }

        let Some(best_entry) = best_entry else {
            debug!(".cur file had no image entries");
            return Err(ImageError::Parameter(ParameterError::from_kind(
                ParameterErrorKind::NoMoreData,
            )));
        };
        Ok(RustIcoDecoder {
            entry: best_entry.clone(),
        })
    }
}

impl image::ImageDecoder for RustIcoDecoder {
    fn dimensions(&self) -> (u32, u32) {
        (self.entry.width(), self.entry.height())
    }

    fn color_type(&self) -> image::ColorType {
        image::ColorType::Rgba8
    }

    fn read_image(self, buffer: &mut [u8]) -> ImageResult<()>
    where
        Self: Sized,
    {
        let decoded_image = match self.entry.decode() {
            Ok(image) => image,
            Err(error) => {
                debug!("Error decoding .cur file image: {error}");
                return Err(ImageError::IoError(error));
            },
        };
        let rgba = decoded_image.into_rgba_data();
        if buffer.len() != rgba.len() {
            return Err(ImageError::IoError(Error::new(
                ErrorKind::InvalidData,
                "Decoded image length does not equal image buffer length.",
            )));
        }
        buffer.copy_from_slice(&rgba);
        Ok(())
    }

    fn read_image_boxed(self: Box<Self>, buf: &mut [u8]) -> ImageResult<()> {
        (*self).read_image(buf)
    }

    fn icc_profile(&mut self) -> ImageResult<Option<Vec<u8>>> {
        Ok(None)
    }

    fn exif_metadata(&mut self) -> ImageResult<Option<Vec<u8>>> {
        Ok(None)
    }

    fn xmp_metadata(&mut self) -> ImageResult<Option<Vec<u8>>> {
        Ok(None)
    }

    fn iptc_metadata(&mut self) -> ImageResult<Option<Vec<u8>>> {
        Ok(None)
    }
}

// TODO: Animated cursors are part of the spec, and a few wpt cursor tests use animated cursors
impl<'a> AnimationDecoder<'a> for RustIcoDecoder {
    fn into_frames(self) -> image::Frames<'a> {
        unreachable!("Should never decode these images with animated decoder");
    }

    fn loop_count(&self) -> LoopCount {
        unreachable!("Should never decode these images with animated decoder");
    }
}
