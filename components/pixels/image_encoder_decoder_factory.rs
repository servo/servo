/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This is the traits to implement a custom encoder or decoder.

use image::metadata::LoopCount;
use image::{Frames, ImageDecoder, ImageResult};

/// Main image decoder encoder factory trait. Implement this if you want to have custom Encoding and Decoding facilities.
/// Compare [`DefaultImageEncoderDecoderFactory']
#[typetag::serde]
pub trait ImageEncoderDecoderFactory: Send + Sync {
    /// Create a decoder from bytes. Return an error if the image format is not supported.
    fn make_from_bytes<'a>(
        &self,
        buffer: &'a [u8],
    ) -> ImageResult<Box<dyn ServoImageDecoder<'a> + 'a>>;
}

/// Main Image decoder trait.
pub trait ServoImageDecoder<'a> {
    /// Get the decoder.
    fn get(self: Box<Self>) -> Box<dyn ImageDecoder + 'a>;
    fn is_animated(&self) -> bool;
    /// Return an animation decoder.
    fn get_animated_decoder(self: Box<Self>) -> Box<dyn ServoAnimation<'a> + 'a>;
}

/// This is a simple trait to get AnimationDecoder into something we can use.
/// Otherwise we get problems with the into_frames method as it takes self by value.
pub trait ServoAnimation<'a> {
    fn boxed_into_frames(self: Box<Self>) -> Frames<'a>;
    fn loop_count(&self) -> LoopCount;
}
