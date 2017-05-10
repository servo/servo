/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Color, RGBA};
use style::properties::animated_properties::Animatable;

#[test]
fn test_rgba_color_interepolation_preserves_transparent() {
    assert_eq!(Color::RGBA(RGBA::transparent())
                .interpolate(&Color::RGBA(RGBA::transparent()), 0.5).unwrap(),
               Color::RGBA(RGBA::transparent()));
}

#[test]
fn test_rgba_color_interepolation_alpha() {
    assert_eq!(Color::RGBA(RGBA::new(200, 0, 0, 100))
                .interpolate(&Color::RGBA(RGBA::new(0, 200, 0, 200)), 0.5).unwrap(),
               Color::RGBA(RGBA::new(66, 133, 0, 150)));
}

#[test]
fn test_rgba_color_interepolation_out_of_range_1() {
    // Some cubic-bezier functions produce values that are out of range [0, 1].
    // Unclamped cases.
    assert_eq!(Color::RGBA(RGBA::from_floats(0.3, 0.0, 0.0, 0.4)).interpolate(
              &Color::RGBA(RGBA::from_floats(0.0, 1.0, 0.0, 0.6)), -0.5).unwrap(),
               Color::RGBA(RGBA::new(152, 0, 0, 76)));
}

#[test]
fn test_rgba_color_interepolation_out_of_range_2() {
    assert_eq!(Color::RGBA(RGBA::from_floats(1.0, 0.0, 0.0, 0.6)).interpolate(
              &Color::RGBA(RGBA::from_floats(0.0, 0.3, 0.0, 0.4)), 1.5).unwrap(),
               Color::RGBA(RGBA::new(0, 152, 0, 76)));
}

#[test]
fn test_rgba_color_interepolation_out_of_range_clamped_1() {
    assert_eq!(Color::RGBA(RGBA::from_floats(1.0, 0.0, 0.0, 0.8)).interpolate(
              &Color::RGBA(RGBA::from_floats(0.0, 1.0, 0.0, 0.2)), -0.5).unwrap(),
               Color::RGBA(RGBA::from_floats(1.0, 0.0, 0.0, 1.0)));
}

#[test]
fn test_rgba_color_interepolation_out_of_range_clamped_2() {
    assert_eq!(Color::RGBA(RGBA::from_floats(1.0, 0.0, 0.0, 0.8)).interpolate(
              &Color::RGBA(RGBA::from_floats(0.0, 1.0, 0.0, 0.2)), 1.5).unwrap(),
               Color::RGBA(RGBA::from_floats(0.0, 0.0, 0.0, 0.0)));
}
