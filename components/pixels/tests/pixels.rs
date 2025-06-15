/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::default::Size2D;
use pixels::flip_y_rgba8_image_inplace;

const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];

const COLORS: [[u8; 4]; 4] = [RED, GREEN, BLUE, BLACK];

fn create_rgba8_image(number_of_pixels: usize) -> Vec<u8> {
    (0..number_of_pixels)
        .map(|i| COLORS[i % 4])
        .flatten()
        .collect()
}

#[test]
fn test_flip_y_rgba8_image_inplace() {
    // | R G | B Bl | -> | B Bl | R G |
    let mut image2x2 = create_rgba8_image(4);

    flip_y_rgba8_image_inplace(Size2D::new(2, 2), &mut image2x2);

    assert_eq!(
        &image2x2[0..4],
        &BLUE,
        "Expected blue color at [0, 0] (image2x2)"
    );
    assert_eq!(
        &image2x2[12..16],
        &GREEN,
        "Expected green color at [1, 1] (image2x2)"
    );

    // | R G B | Bl R G | B Bl R | -> | B Bl R | Bl R G | R G B |
    let mut image3x3 = create_rgba8_image(9);

    flip_y_rgba8_image_inplace(Size2D::new(3, 3), &mut image3x3);

    assert_eq!(
        &image3x3[0..4],
        &BLUE,
        "Expected blue color at [0, 0] (image3x3)"
    );
    assert_eq!(
        &image3x3[16..20],
        &RED,
        "Expected red color at [1, 1] (image3x3)"
    );
    assert_eq!(
        &image3x3[32..36],
        &BLUE,
        "Expected blue color at [2, 2] (image3x3)"
    );
}
