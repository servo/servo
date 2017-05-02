/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use parsing::parse;
use style::values::HasViewportPercentage;
use style::values::specified::{AbsoluteLength, NoCalcLength, ViewportPercentageLength};
use style::values::specified::length::{CalcLengthOrPercentage, CalcUnit};

#[test]
fn length_has_viewport_percentage() {
    let l = NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vw(100.));
    assert!(l.has_viewport_percentage());
    let l = NoCalcLength::Absolute(AbsoluteLength::Px(Au(100).to_f32_px()));
    assert!(!l.has_viewport_percentage());
}

#[test]
fn calc_top_level_number_with_unit() {
    fn parse_value(text: &str, unit: CalcUnit) -> Result<CalcLengthOrPercentage, ()> {
        parse(|context, input| CalcLengthOrPercentage::parse(context, input, unit), text)
    }
    assert_eq!(parse_value("1", CalcUnit::Length), Err(()));
    assert_eq!(parse_value("1", CalcUnit::LengthOrPercentage), Err(()));
    assert_eq!(parse_value("1", CalcUnit::Angle), Err(()));
    assert_eq!(parse_value("1", CalcUnit::Time), Err(()));
    assert_eq!(parse_value("1px  + 1", CalcUnit::Length), Err(()));
    assert_eq!(parse_value("1em  + 1", CalcUnit::Length), Err(()));
    assert_eq!(parse_value("1px  + 1", CalcUnit::LengthOrPercentage), Err(()));
    assert_eq!(parse_value("1%   + 1", CalcUnit::LengthOrPercentage), Err(()));
    assert_eq!(parse_value("1rad + 1", CalcUnit::Angle), Err(()));
    assert_eq!(parse_value("1deg + 1", CalcUnit::Angle), Err(()));
    assert_eq!(parse_value("1s   + 1", CalcUnit::Time), Err(()));
}
