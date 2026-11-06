/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{ImageFrame, ImageMetadata, PixelFormat, Repeat};

/// The decoded Image bytes and metadata.
pub struct DecodedImage {
    /// Image Metadata
    pub metadata: ImageMetadata,
    /// PixelFormat RGBA8 prefered.
    pub format: PixelFormat,
    /// The bytes of the decoded image
    pub bytes: Vec<u8>,
    /// THe frames of the image.
    pub frames: Vec<ImageFrame>,
    /// Whether or not all of the frames of this image are opaque.
    pub is_opaque: bool,
    /// The loop count for this image's animation. For animated images, this
    /// has a default value of `Repeat::Infinite` (if no loop count is specified in
    /// the image).  For images that do not animate, this will be `None`.
    pub loop_count: Option<Repeat>,
}

/// Main Image Decoder Trait.
/// Implement this to create your custom decoder and set it in [`ServoBuilder`].
pub trait ServoImageDecoder: Sync + Send {
    /// Takes an encoded image as a buffer of bytes
    /// and returns a decoded [`DecodedImage`].
    fn decode_image(&self, buffer: &[u8]) -> Option<DecodedImage>;
}
