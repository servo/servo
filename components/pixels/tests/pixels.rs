/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::default::Size2D;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use pixels::{
    CorsStatus, Multiply, flip_y_rgba8_image_inplace, load_from_memory, multiply_u8_color,
    rgba8_premultiply_inplace, transform_inplace, unmultiply_inplace,
};

const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const YELLOW: [u8; 4] = [255, 255, 0, 255];

const COLORS: [[u8; 4]; 4] = [RED, GREEN, BLUE, YELLOW];

fn create_rgba8_image(number_of_pixels: usize) -> Vec<u8> {
    (0..number_of_pixels)
        .map(|i| COLORS[i % 4])
        .flatten()
        .collect()
}

#[test]
fn test_flip_y_rgba8_image_inplace() {
    // | R G | B Y | -> | B Y | R G |
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

    // | R G B | Y R G | B Y R | -> | B Y R | Y R G | R G B |
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

fn encode_png(pixels: &[u8], width: u32, color_type: ExtendedColorType) -> Vec<u8> {
    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(pixels, width, 1, color_type)
        .expect("a single-row image of matching length encodes");
    png
}

#[test]
fn test_decoded_image_premultiplication() {
    let rgb = encode_png(&[10, 20, 30, 40, 50, 60], 2, ExtendedColorType::Rgb8);
    let rgb = load_from_memory(&rgb, CorsStatus::Safe).expect("encoded PNG decodes");
    assert!(rgb.is_opaque, "an image without alpha is opaque");
    assert_eq!(*rgb.bytes, [10, 20, 30, 255, 40, 50, 60, 255]);

    let [r, g, b, a] = [200, 100, 50, 128];
    let rgba = encode_png(&[r, g, b, a, 1, 2, 3, 255], 2, ExtendedColorType::Rgba8);
    let rgba = load_from_memory(&rgba, CorsStatus::Safe).expect("encoded PNG decodes");
    assert!(
        !rgba.is_opaque,
        "an image with a translucent pixel is not opaque"
    );
    assert_eq!(
        *rgba.bytes,
        [
            multiply_u8_color(r, a),
            multiply_u8_color(g, a),
            multiply_u8_color(b, a),
            a,
            1,
            2,
            3,
            255
        ]
    );
}

#[test]
fn test_rgba8_premultiply_inplace() {
    // Every color under every alpha, with distinct channels.
    let pixels: Vec<u8> = (0..=u8::MAX)
        .flat_map(|alpha| (0..=u8::MAX).map(move |color| [color, !color, color / 3, alpha]))
        .flatten()
        .collect();
    let expected: Vec<u8> = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[r, g, b, a]| {
            [
                multiply_u8_color(r, a),
                multiply_u8_color(g, a),
                multiply_u8_color(b, a),
                a,
            ]
        })
        .collect();
    let mut premultiplied = pixels;
    assert!(!rgba8_premultiply_inplace(&mut premultiplied));
    assert_eq!(premultiplied, expected);

    let opaque: Vec<u8> = (0..=u8::MAX)
        .flat_map(|color| [color, !color, color / 3, u8::MAX])
        .collect();
    let mut premultiplied = opaque.clone();
    assert!(rgba8_premultiply_inplace(&mut premultiplied));
    assert_eq!(premultiplied, opaque);
}

#[test]
fn test_unmultiply_then_premultiply_restores_pixels() {
    // Every valid premultiplied color under every non-zero alpha.
    let premultiplied: Vec<u8> = (1..=u8::MAX)
        .flat_map(|alpha| (0..=alpha).map(move |color| [color, color, color, alpha]))
        .flatten()
        .collect();

    let mut pixels = premultiplied.clone();
    unmultiply_inplace::<false>(&mut pixels);
    rgba8_premultiply_inplace(&mut pixels);
    assert_eq!(pixels, premultiplied, "unmultiply_inplace");

    let mut pixels = premultiplied.clone();
    transform_inplace(&mut pixels, Multiply::UnMultiply, false, false);
    transform_inplace(&mut pixels, Multiply::PreMultiply, false, false);
    assert_eq!(pixels, premultiplied, "transform_inplace");
}
