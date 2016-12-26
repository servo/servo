/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Color, RGBA};
use style::properties::animated_properties::Interpolate;

#[test]
fn test_rgba_color_interepolation() {
    assert_eq!(Color::RGBA(RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 }).interpolate(
              &Color::RGBA(RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 }), 1.0).unwrap(),
               Color::RGBA(RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 }));

    assert_eq!(Color::RGBA(RGBA { red: 1.0, green: 0.0, blue: 0.0, alpha: 0.6 }).interpolate(
              &Color::RGBA(RGBA { red: 0.0, green: 1.0, blue: 0.0, alpha: 0.4 }), 0.5).unwrap(),
               Color::RGBA(RGBA { red: 0.6, green: 0.4, blue: 0.0, alpha: 0.5 }));

    // Some cubic-bezier functions produce values that are out of range [0, 1].
    // Unclamped cases.
    assert_eq!(Color::RGBA(RGBA { red: 0.3, green: 0.0, blue: 0.0, alpha: 0.4 }).interpolate(
              &Color::RGBA(RGBA { red: 0.0, green: 1.0, blue: 0.0, alpha: 0.6 }), -0.5).unwrap(),
               Color::RGBA(RGBA { red: 0.6, green: 0.0, blue: 0.0, alpha: 0.3 }));

    assert_eq!(Color::RGBA(RGBA { red: 1.0, green: 0.0, blue: 0.0, alpha: 0.6 }).interpolate(
              &Color::RGBA(RGBA { red: 0.0, green: 0.3, blue: 0.0, alpha: 0.4 }), 1.5).unwrap(),
               Color::RGBA(RGBA { red: 0.0, green: 0.6, blue: 0.0, alpha: 0.3 }));

    // Clamped cases.
    assert_eq!(Color::RGBA(RGBA { red: 1.0, green: 0.0, blue: 0.0, alpha: 0.8 }).interpolate(
              &Color::RGBA(RGBA { red: 0.0, green: 1.0, blue: 0.0, alpha: 0.2 }), -0.5).unwrap(),
               Color::RGBA(RGBA { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 }));

    assert_eq!(Color::RGBA(RGBA { red: 1.0, green: 0.0, blue: 0.0, alpha: 0.8 }).interpolate(
              &Color::RGBA(RGBA { red: 0.0, green: 1.0, blue: 0.0, alpha: 0.2 }), 1.5).unwrap(),
               Color::RGBA(RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 }));
}
