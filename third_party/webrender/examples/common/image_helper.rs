/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use webrender::api::{ImageData, ImageDescriptor, ImageFormat, ImageDescriptorFlags};

pub fn make_checkerboard(width: u32, height: u32) -> (ImageDescriptor, ImageData) {
    let mut image_data = Vec::new();
    for y in 0 .. height {
        for x in 0 .. width {
            let lum = 255 * (((x & 8) == 0) ^ ((y & 8) == 0)) as u8;
            image_data.extend_from_slice(&[lum, lum, lum, 0xff]);
        }
    }
    (
        ImageDescriptor::new(width as i32, height as i32, ImageFormat::BGRA8, ImageDescriptorFlags::IS_OPAQUE),
        ImageData::new(image_data)
    )
}
