/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use style::properties::PropertyDeclaration;
use style::values::specified::{AbsoluteLength, Length, NoCalcLength, ViewportPercentageLength};
use style::values::specified::border::BorderSideWidth;
use style_traits::HasViewportPercentage;

#[test]
fn has_viewport_percentage_for_specified_value() {
    //TODO: test all specified value with a HasViewportPercentage impl
    let pvw = PropertyDeclaration::BorderTopWidth(
        BorderSideWidth::Length(
            Length::NoCalc(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vw(100.)))
        )
    );
    assert!(pvw.has_viewport_percentage());

    let pabs = PropertyDeclaration::BorderTopWidth(
        BorderSideWidth::Length(
            Length::NoCalc(NoCalcLength::Absolute(AbsoluteLength::Px(Au(100).to_f32_px())))
        )
    );
    assert!(!pabs.has_viewport_percentage());
}
