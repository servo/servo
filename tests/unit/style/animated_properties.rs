/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::color::AbsoluteColor;
use style::values::animated::{Animate, Procedure, ToAnimatedValue};

fn interpolate_color(from: AbsoluteColor, to: AbsoluteColor, progress: f64) -> AbsoluteColor {
    let from = from.to_animated_value();
    let to = to.to_animated_value();
    AbsoluteColor::from_animated_value(
        from.animate(&to, Procedure::Interpolate { progress })
            .unwrap(),
    )
}

// Color
#[test]
fn test_rgba_color_interepolation_preserves_transparent() {
    assert_eq!(
        interpolate_color(
            AbsoluteColor::transparent(),
            AbsoluteColor::transparent(),
            0.5
        ),
        AbsoluteColor::transparent()
    );
}

#[test]
fn test_rgba_color_interepolation_alpha() {
    assert_eq!(
        interpolate_color(
            AbsoluteColor::srgb(0.6, 0.0, 0.0, 0.4),
            AbsoluteColor::srgb(0.0, 0.6, 0.0, 0.8),
            0.5
        ),
        AbsoluteColor::srgb(0.2, 0.4, 0.0, 0.6)
    );
}

#[test]
fn test_rgba_color_interepolation_out_of_range_1() {
    // Some cubic-bezier functions produce values that are out of range [0, 1].
    // Unclamped cases.
    assert_eq!(
        interpolate_color(
            AbsoluteColor::srgb(0.3, 0.0, 0.0, 0.4),
            AbsoluteColor::srgb(0.0, 1.0, 0.0, 0.6),
            -0.5
        ),
        AbsoluteColor::srgb(0.6, -1.0, 0.0, 0.3)
    );
}

#[test]
fn test_rgba_color_interepolation_out_of_range_2() {
    assert_eq!(
        interpolate_color(
            AbsoluteColor::srgb(1.0, 0.0, 0.0, 0.6),
            AbsoluteColor::srgb(0.0, 0.3, 0.0, 0.4),
            1.5
        ),
        AbsoluteColor::srgb(-1.0, 0.6, 0.0, 0.3)
    );
}

#[test]
fn test_rgba_color_interepolation_out_of_range_clamped_1() {
    assert_eq!(
        interpolate_color(
            AbsoluteColor::srgb(1.0, 0.0, 0.0, 0.8),
            AbsoluteColor::srgb(0.0, 1.0, 0.0, 0.2),
            -0.5
        ),
        AbsoluteColor::srgb(1.2, -0.1, 0.0, 1.0)
    );
}

#[test]
fn test_rgba_color_interepolation_out_of_range_clamped_2() {
    assert_eq!(
        interpolate_color(
            AbsoluteColor::srgb(1.0, 0.0, 0.0, 0.8),
            AbsoluteColor::srgb(0.0, 1.0, 0.0, 0.2),
            1.5
        ),
        AbsoluteColor::srgb(-0.4, 0.3, 0.0, 0.0)
    );
}
