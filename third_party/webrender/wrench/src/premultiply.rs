/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// These are slow. Gecko's gfx/2d/Swizzle.cpp has better versions
pub fn premultiply(data: &mut [u8]) {
    for pixel in data.chunks_mut(4) {
        let a = pixel[3] as u32;
        let b = pixel[2] as u32;
        let g = pixel[1] as u32;
        let r = pixel[0] as u32;

        pixel[3] = a as u8;
        pixel[2] = ((r * a + 128) / 255) as u8;
        pixel[1] = ((g * a + 128) / 255) as u8;
        pixel[0] = ((b * a + 128) / 255) as u8;
    }
}

#[allow(unused)]
pub fn unpremultiply(data: &mut [u8]) {
    for pixel in data.chunks_mut(4) {
        let a = pixel[3] as u32;
        let mut b = pixel[2] as u32;
        let mut g = pixel[1] as u32;
        let mut r = pixel[0] as u32;

        if a > 0 {
            r = r * 255 / a;
            g = g * 255 / a;
            b = b * 255 / a;
        }

        pixel[3] = a as u8;
        pixel[2] = r as u8;
        pixel[1] = g as u8;
        pixel[0] = b as u8;
    }
}

#[test]
fn it_works() {
    let mut f = [0xff, 0xff, 0xff, 0x80, 0x00, 0xff, 0x00, 0x80];
    premultiply(&mut f);
    println!("{:?}", f);
    assert!(
        f[0] == 0x80 && f[1] == 0x80 && f[2] == 0x80 && f[3] == 0x80 && f[4] == 0x00 &&
            f[5] == 0x80 && f[6] == 0x00 && f[7] == 0x80
    );
    unpremultiply(&mut f);
    println!("{:?}", f);
    assert!(
        f[0] == 0xff && f[1] == 0xff && f[2] == 0xff && f[3] == 0x80 && f[4] == 0x00 &&
            f[5] == 0xff && f[6] == 0x00 && f[7] == 0x80
    );
}
