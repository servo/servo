/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use style::properties::longhands::border_top_width;
use style::properties::{DeclaredValue, PropertyDeclaration};
use style::values::HasViewportPercentage;
use style::values::specified::{Length, ViewportPercentageLength};

#[test]
fn has_viewport_percentage_for_specified_value() {
    //TODO: test all specified value with a HasViewportPercentage impl
    let pvw = PropertyDeclaration::BorderTopWidth(
        DeclaredValue::Value(border_top_width::SpecifiedValue(
            Length::ViewportPercentage(ViewportPercentageLength::Vw(100.))
        ))
    );
    assert!(pvw.has_viewport_percentage());

    let pabs = PropertyDeclaration::BorderTopWidth(
        DeclaredValue::Value(border_top_width::SpecifiedValue(
            Length::Absolute(Au(100))
        ))
    );
    assert!(!pabs.has_viewport_percentage());
}
