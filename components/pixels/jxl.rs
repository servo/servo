/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! JPEG XL decoder integration backed by the `jxl-rs` crate.
//!
//! The actual decoder is gated behind the `jxl` cargo feature so embedders
//! that don't need JPEG XL avoid pulling in the crate. Signature sniffing
//! ([`is_jxl`]) is always available and just compares magic bytes, so
//! `MimeClassifier` and friends can keep working uniformly.
//!
//! When the `jxl` feature is enabled, [`decode_jxl`] produces a
//! [`RasterImage`] in the same RGBA8 (pre-multiplied) format used by the
//! other decoders in [`crate`]. SIMD acceleration is enabled via the
//! `all-simd` feature on the `jxl` crate (see workspace `Cargo.toml`).

use crate::{CorsStatus, RasterImage};

/// Magic bytes for a bare JPEG XL codestream.
const JXL_CODESTREAM_SIGNATURE: [u8; 2] = [0xff, 0x0a];

/// Magic bytes for a JPEG XL ISO BMFF container (a `JXL ` box).
const JXL_CONTAINER_SIGNATURE: [u8; 12] = [
    0x00, 0x00, 0x00, 0x0c, b'J', b'X', b'L', b' ', 0x0d, 0x0a, 0x87, 0x0a,
];

/// Returns true if the buffer starts with a valid JPEG XL signature, either a
/// bare codestream (`FF 0A`) or a JXL container (`JXL ` ISO BMFF box).
///
/// This check intentionally does **not** depend on the `jxl` crate so that
/// MIME classification keeps working even when the `jxl` cargo feature is
/// disabled.
pub(crate) fn is_jxl(buffer: &[u8]) -> bool {
    buffer.starts_with(&JXL_CODESTREAM_SIGNATURE) || buffer.starts_with(&JXL_CONTAINER_SIGNATURE)
}

/// Decode a complete JPEG XL bitstream into a [`RasterImage`].
///
/// Returns `None` when the `jxl` cargo feature is disabled (so callers can
/// always invoke this function and just observe an unsupported-format
/// failure), as well as for malformed input or oversized images.
#[cfg(feature = "jxl")]
pub(crate) fn decode_jxl(buffer: &[u8], cors_status: CorsStatus) -> Option<RasterImage> {
    decode_jxl_impl(buffer, cors_status)
}

/// Stub used when the `jxl` cargo feature is disabled.
#[cfg(not(feature = "jxl"))]
pub(crate) fn decode_jxl(_buffer: &[u8], _cors_status: CorsStatus) -> Option<RasterImage> {
    None
}

#[cfg(feature = "jxl")]
fn decode_jxl_impl(buffer: &[u8], cors_status: CorsStatus) -> Option<RasterImage> {
    use std::sync::Arc;
    use std::time::Duration;

    use jxl::api::states::{Initialized, WithImageInfo};
    use jxl::api::{
        JxlColorType, JxlDataFormat, JxlDecoder, JxlDecoderOptions, JxlOutputBuffer,
        JxlPixelFormat, ProcessingResult,
    };
    use log::debug;

    use crate::{ImageFrame, ImageMetadata, PixelFormat, rgba8_premultiply_inplace};

    /// Maximum number of pixels (× channels) we are willing to allocate for
    /// any single JPEG XL image or frame, mirroring the limit used by other
    /// major browsers when decoding untrusted JPEG XL content.
    const JXL_PIXEL_LIMIT: usize = 1024 * 1024 * 1024;

    if buffer.is_empty() || !is_jxl(buffer) {
        return None;
    }

    let mut options = JxlDecoderOptions::default();
    options.pixel_limit = Some(JXL_PIXEL_LIMIT);

    let initialized = JxlDecoder::<Initialized>::new(options);
    let mut input: &[u8] = buffer;

    // Stage 1: parse image-level headers.
    let mut decoder: JxlDecoder<WithImageInfo> = match initialized.process(&mut input) {
        Ok(ProcessingResult::Complete { result }) => result,
        Ok(ProcessingResult::NeedsMoreInput { .. }) => {
            debug!("JXL decoder ran out of input while parsing image header");
            return None;
        },
        Err(error) => {
            debug!("JXL image header parse error: {error:?}");
            return None;
        },
    };

    // After image headers we know the canvas size. We always request RGBA8
    // (interleaved); extra channels are ignored to keep the output simple.
    let basic_info = decoder.basic_info().clone();
    let width = u32::try_from(basic_info.size.0).ok()?;
    let height = u32::try_from(basic_info.size.1).ok()?;
    if width == 0 || height == 0 {
        return None;
    }

    let num_extra_channels = basic_info.extra_channels.len();
    decoder.set_pixel_format(JxlPixelFormat {
        color_type: JxlColorType::Rgba,
        color_data_format: Some(JxlDataFormat::U8 { bit_depth: 8 }),
        extra_channel_format: vec![None; num_extra_channels],
    });

    let row_bytes = (width as usize).checked_mul(4)?;
    let frame_bytes = row_bytes.checked_mul(height as usize)?;

    // We accumulate every frame's pixel data into a single shared buffer that
    // backs the resulting `RasterImage`, mirroring the layout produced by the
    // other decoders in `lib.rs`.
    let mut all_bytes: Vec<u8> = Vec::new();
    let mut frames: Vec<ImageFrame> = Vec::new();
    let mut is_opaque = true;

    loop {
        if !decoder.has_more_frames() {
            break;
        }

        // Stage 2: parse the next frame's header.
        let with_frame_info = match decoder.process(&mut input) {
            Ok(ProcessingResult::Complete { result }) => result,
            Ok(ProcessingResult::NeedsMoreInput { .. }) => {
                debug!("JXL decoder ran out of input while parsing frame header");
                return None;
            },
            Err(error) => {
                debug!("JXL frame header parse error: {error:?}");
                return None;
            },
        };

        let frame_header = with_frame_info.frame_header();
        // `coalescing: true` (the default) ensures that the decoder always
        // composites onto the canvas, so the emitted frames are at canvas
        // resolution.
        let duration_ms = frame_header.duration;

        // Reserve a frame-sized slot in the accumulated buffer.
        let frame_start = all_bytes.len();
        all_bytes.resize(frame_start.checked_add(frame_bytes)?, 0);

        // Stage 3: decode pixels into the freshly allocated slot.
        decoder = {
            let frame_slot = &mut all_bytes[frame_start..frame_start + frame_bytes];
            let output = JxlOutputBuffer::new(frame_slot, height as usize, row_bytes);
            match with_frame_info.process(&mut input, &mut [output]) {
                Ok(ProcessingResult::Complete { result }) => result,
                Ok(ProcessingResult::NeedsMoreInput { .. }) => {
                    debug!("JXL decoder ran out of input while decoding pixels");
                    return None;
                },
                Err(error) => {
                    debug!("JXL pixel decode error: {error:?}");
                    return None;
                },
            }
        };

        // Match the rest of the pipeline: store data pre-multiplied. See
        // <https://github.com/servo/servo/issues/40257> for the trade-off.
        let frame_slice = &mut all_bytes[frame_start..frame_start + frame_bytes];
        is_opaque = rgba8_premultiply_inplace(frame_slice) && is_opaque;

        let delay = duration_ms
            .filter(|duration| duration.is_finite() && *duration > 0.0)
            .map(|duration| Duration::from_secs_f64(duration / 1000.0));

        frames.push(ImageFrame {
            delay,
            byte_range: frame_start..frame_start + frame_bytes,
            width,
            height,
        });
    }

    if frames.is_empty() {
        debug!("JXL decoding produced no frames");
        return None;
    }

    Some(RasterImage {
        metadata: ImageMetadata { width, height },
        format: PixelFormat::RGBA8,
        id: None,
        cors_status,
        bytes: Arc::new(all_bytes),
        frames,
        is_opaque,
    })
}

#[cfg(test)]
mod test {
    use super::is_jxl;

    #[test]
    fn detects_jxl_codestream_signature() {
        // Bare JXL codestream magic.
        assert!(is_jxl(&[0xff, 0x0a, 0x00]));
    }

    #[test]
    fn detects_jxl_container_signature() {
        // ISO BMFF container with the `JXL ` brand.
        let container = [
            0x00, 0x00, 0x00, 0x0c, b'J', b'X', b'L', b' ', 0x0d, 0x0a, 0x87, 0x0a,
        ];
        assert!(is_jxl(&container));
    }

    #[test]
    fn rejects_non_jxl_bytes() {
        // PNG header.
        assert!(!is_jxl(&[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        ]));
        assert!(!is_jxl(&[]));
        assert!(!is_jxl(&[0x00]));
    }
}
