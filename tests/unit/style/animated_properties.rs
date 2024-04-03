/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::color::{AbsoluteColor, ColorSpace};
use style::values::animated::{Animate, Procedure, ToAnimatedValue};

fn interpolate_color(from: AbsoluteColor, to: AbsoluteColor, progress: f64) -> AbsoluteColor {
    let from = from.to_animated_value();
    let to = to.to_animated_value();
    AbsoluteColor::from_animated_value(
        from.animate(&to, Procedure::Interpolate { progress })
            .unwrap(),
    )
}

fn srgb_legacy_from_floats(red: f32, green: f32, blue: f32, alpha: f32) -> AbsoluteColor {
    AbsoluteColor::new(ColorSpace::Srgb, red, green, blue, alpha).into_srgb_legacy()
}

// Color
#[test]
fn test_rgba_color_interepolation_preserves_transparent() {
    let transparent = AbsoluteColor::TRANSPARENT_BLACK;
    assert_eq!(
        interpolate_color(transparent, transparent, 0.5),
        transparent
    );
}

#[test]
fn test_rgba_color_interepolation_alpha_1() {
    assert_eq!(
        interpolate_color(
            AbsoluteColor::srgb_legacy(150, 0, 0, 0.4),
            AbsoluteColor::srgb_legacy(0, 150, 0, 0.8),
            0.5
        ),
        AbsoluteColor::srgb_legacy(50, 100, 0, 0.6)
    );
}

#[test]
fn test_rgba_color_interepolation_alpha_2() {
    assert_eq!(
        interpolate_color(
            srgb_legacy_from_floats(0.6, 0.0, 0.0, 0.4),
            srgb_legacy_from_floats(0.0, 0.6, 0.0, 0.8),
            0.5
        ),
        srgb_legacy_from_floats(0.2, 0.4, 0.0, 0.6)
    );
}

#[test]
fn test_rgba_color_interepolation_out_of_range_1() {
    // Some cubic-bezier functions produce values that are out of range [0, 1].
    // Unclamped cases.
    // Note `AbsoluteColor::srgb_legacy` doesn't accept out of range values,
    // so we only test with `srgb_legacy_from_floats`.
    assert_eq!(
        interpolate_color(
            srgb_legacy_from_floats(0.3, 0.0, 0.0, 0.4),
            srgb_legacy_from_floats(0.0, 1.0, 0.0, 0.6),
            -0.5
        ),
        srgb_legacy_from_floats(0.6, -1.0, 0.0, 0.3)
    );
}

#[test]
fn test_rgba_color_interepolation_out_of_range_2() {
    assert_eq!(
        interpolate_color(
            srgb_legacy_from_floats(1.0, 0.0, 0.0, 0.6),
            srgb_legacy_from_floats(0.0, 0.3, 0.0, 0.4),
            1.5
        ),
        srgb_legacy_from_floats(-1.0, 0.6, 0.0, 0.3)
    );
}

#[test]
fn test_rgba_color_interepolation_out_of_range_clamped_1() {
    assert_eq!(
        interpolate_color(
            srgb_legacy_from_floats(1.0, 0.0, 0.0, 0.8),
            srgb_legacy_from_floats(0.0, 1.0, 0.0, 0.2),
            -0.5
        ),
        srgb_legacy_from_floats(1.2, -0.1, 0.0, 1.0)
    );
}

#[test]
fn test_rgba_color_interepolation_out_of_range_clamped_2() {
    assert_eq!(
        interpolate_color(
            srgb_legacy_from_floats(1.0, 0.0, 0.0, 0.8),
            srgb_legacy_from_floats(0.0, 1.0, 0.0, 0.2),
            1.5
        ),
        srgb_legacy_from_floats(-0.4, 0.3, 0.0, 0.0)
    );
}
