/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate net_traits;

use net_traits::image::base::detect_image_format;

#[test]
fn test_supported_images() {
    let gif1 = [b'G', b'I', b'F', b'8', b'7', b'a'];
    let gif2 = [b'G', b'I', b'F', b'8', b'9', b'a'];
    let jpeg = [0xff, 0xd8, 0xff];
    let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let bmp = [0x42, 0x4D];
    let ico = [0x00, 0x00, 0x01, 0x00];
    let junk_format = [0x01, 0x02, 0x03, 0x04, 0x05];

    assert!(detect_image_format(&gif1).is_ok());
    assert!(detect_image_format(&gif2).is_ok());
    assert!(detect_image_format(&jpeg).is_ok());
    assert!(detect_image_format(&png).is_ok());
    assert!(detect_image_format(&bmp).is_ok());
    assert!(detect_image_format(&ico).is_ok());
    assert!(detect_image_format(&junk_format).is_err());
}
