/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use style::values::HasViewportPercentage;
use style::values::specified::{ViewportPercentageLength, Length};

#[test]
fn length_has_viewport_percentage() {
    let l = Length::ViewportPercentage(ViewportPercentageLength::Vw(100.));
    assert!(l.has_viewport_percentage());
    let l = Length::Absolute(Au(100));
    assert!(!l.has_viewport_percentage());
}
