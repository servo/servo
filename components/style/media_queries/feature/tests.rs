/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg(test)]

use super::*;
use ::FromCss;
use ::cssparser::Parser;

#[test]
fn parse_discrete() {
    macro_rules! assert_from_css_eq {
        ($css:expr, $feature:ident(None)) => {
            assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                       Ok(MediaFeature::$feature(None)))
        };
        ($css:expr, $feature:ident(Some($value:ident))) => {
            assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                       Ok(MediaFeature::$feature(Some($feature::$value))))
        };
        ($css:expr, $feature:ident(Some(value = $value:expr))) => {
            assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                       Ok(MediaFeature::$feature(Some($feature($value)))))
        }
    }

    // boolean context
    assert_from_css_eq!("(orientation", Orientation(None));
    assert_from_css_eq!("(orientation)", Orientation(None));

    // normal context
    assert_from_css_eq!("(orientation: portrait)", Orientation(Some(Portrait)));
    assert_from_css_eq!("(orientation: landscape)", Orientation(Some(Landscape)));

    // 'grid' is a special case, due to the numeric booleans
    assert_from_css_eq!("(grid)", Grid(None));
    assert_from_css_eq!("(grid: 0)", Grid(Some(value = false)));
    assert_from_css_eq!("(grid: 1)", Grid(Some(value = true)));
}

#[test]
fn parse_range() {
    use ::values::specified::Length;

    macro_rules! assert_from_css_eq {
        ($css:expr, Err) => {
            assert_eq!(<MediaFeature as FromCss>::from_css(&mut Parser::new($css)),
                       Err(()))
        };
        ($css:expr, $feature:ident(None)) => {
            assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                       Ok(MediaFeature::$feature(None)))
        };
        ($css:expr, $feature:ident($op:ident($value:expr))) => {
            assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                       Ok(MediaFeature::$feature(Some($feature(Range::$op($value))))))
        };
        ($css:expr, $feature:ident(Interval($a:expr,$ac:expr,$b:expr,$bc:expr))) => {
            assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                       Ok(MediaFeature::$feature(Some($feature(Range::Interval($a,$ac,$b,$bc))))))
        };
    }

    // boolean context
    assert_from_css_eq!("(width", Width(None));
    assert_from_css_eq!("(width)", Width(None));

    // normal context
    assert_from_css_eq!("(width: 200px)", Width(Eq(Length::from_px(200.))));
    assert_from_css_eq!("(min-width: 200px)", Width(Ge(Length::from_px(200.))));
    assert_from_css_eq!("(max-width: 200px)", Width(Le(Length::from_px(200.))));

    // range context (name first)
    assert_from_css_eq!("(width  = 200px)", Width(Eq(Length::from_px(200.))));
    assert_from_css_eq!("(width <  200px)", Width(Lt(Length::from_px(200.))));
    assert_from_css_eq!("(width <= 200px)", Width(Le(Length::from_px(200.))));
    assert_from_css_eq!("(width >  200px)", Width(Gt(Length::from_px(200.))));
    assert_from_css_eq!("(width >= 200px)", Width(Ge(Length::from_px(200.))));

    // range context (value first)
    assert_from_css_eq!("(200px  = width)", Width(Eq(Length::from_px(200.))));
    assert_from_css_eq!("(200px <  width)", Width(Gt(Length::from_px(200.))));
    assert_from_css_eq!("(200px <= width)", Width(Ge(Length::from_px(200.))));
    assert_from_css_eq!("(200px >  width)", Width(Lt(Length::from_px(200.))));
    assert_from_css_eq!("(200px >= width)", Width(Le(Length::from_px(200.))));

    // range context (interval)
    assert_from_css_eq!("(0px <  width <  200px)", Width(Interval(Length::from_px(0.), false,
                                                                  Length::from_px(200.), false)));
    assert_from_css_eq!("(0px <= width <  200px)", Width(Interval(Length::from_px(0.), true,
                                                                  Length::from_px(200.), false)));
    assert_from_css_eq!("(0px <  width <= 200px)", Width(Interval(Length::from_px(0.), false,
                                                                  Length::from_px(200.), true)));
    assert_from_css_eq!("(0px <= width <= 200px)", Width(Interval(Length::from_px(0.), true,
                                                                  Length::from_px(200.), true)));

    assert_from_css_eq!("(200px >  width >  0px)", Width(Interval(Length::from_px(0.), false,
                                                                  Length::from_px(200.), false)));
    assert_from_css_eq!("(200px >= width >  0px)", Width(Interval(Length::from_px(0.), false,
                                                                  Length::from_px(200.), true)));
    assert_from_css_eq!("(200px >  width >= 0px)", Width(Interval(Length::from_px(0.), true,
                                                                  Length::from_px(200.), false)));
    assert_from_css_eq!("(200px >= width >= 0px)", Width(Interval(Length::from_px(0.), true,
                                                                  Length::from_px(200.), true)));

    // invalid
    assert_from_css_eq!("width", Err);
    assert_from_css_eq!("width)", Err);
}
