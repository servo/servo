/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::Parser;
use style::values::HasViewportPercentage;
use style::values::specified::{AbsoluteLength, ViewportPercentageLength, NoCalcLength};
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
    fn parse(text: &str, unit: CalcUnit) -> Result<CalcLengthOrPercentage, ()> {
        let mut parser = Parser::new(text);
        CalcLengthOrPercentage::parse(&mut parser, unit)
    }
    assert_eq!(parse("1", CalcUnit::Length), Err(()));
    assert_eq!(parse("1", CalcUnit::LengthOrPercentage), Err(()));
    assert_eq!(parse("1", CalcUnit::Angle), Err(()));
    assert_eq!(parse("1", CalcUnit::Time), Err(()));
    assert_eq!(parse("1px  + 1", CalcUnit::Length), Err(()));
    assert_eq!(parse("1em  + 1", CalcUnit::Length), Err(()));
    assert_eq!(parse("1px  + 1", CalcUnit::LengthOrPercentage), Err(()));
    assert_eq!(parse("1%   + 1", CalcUnit::LengthOrPercentage), Err(()));
    assert_eq!(parse("1rad + 1", CalcUnit::Angle), Err(()));
    assert_eq!(parse("1deg + 1", CalcUnit::Angle), Err(()));
    assert_eq!(parse("1s   + 1", CalcUnit::Time), Err(()));
}
