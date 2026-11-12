/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::DecodedRasterImage;

/// A trait that implements image encoding and decoding. This allows embedders to
/// use custom image encoding and ecoding when setting it in `ServoBuilder`.
pub trait ImageCodec: Sync + Send {
    /// Takes an encoded image as a buffer of bytes and returns a decoded [`DecodedRasterImage`].
    ///
    /// IMPORTANT: When this produces an image it is assumed to be in pre-multiplied alpha in
    /// either RGBA8 or BGRA8 and should produce at least a single frame. Not doing these this
    /// will cause a panic in Servo.
    fn decode_image(&self, buffer: &[u8]) -> Option<DecodedRasterImage>;
}
