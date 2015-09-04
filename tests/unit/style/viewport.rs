/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use euclid::scale_factor::ScaleFactor;
use euclid::size::Size2D;
use style::media_queries::{Device, MediaType};
use style::parser::ParserContext;
use style::stylesheets::{Origin, Stylesheet, CSSRuleIteratorExt};
use style::values::specified::{Length, LengthOrPercentageOrAuto};
use style::viewport::*;
use style_traits::viewport::*;
use url::Url;

macro_rules! stylesheet {
    ($css:expr, $origin:ident) => {
        Stylesheet::from_str($css,
                             Url::parse("http://localhost").unwrap(),
                             Origin::$origin);
    }
}

fn test_viewport_rule<F>(css: &str,
                         device: &Device,
                         callback: F)
    where F: Fn(&Vec<ViewportDescriptorDeclaration>, &str)
{
    ::util::prefs::set_pref("layout.viewport.enabled", true);

    let stylesheet = stylesheet!(css, Author);
    let mut rule_count = 0;
    for rule in stylesheet.effective_rules(&device).viewport() {
        rule_count += 1;
        callback(&rule.declarations, css);
    }
    assert!(rule_count > 0);
}

macro_rules! assert_decl_len {
    ($declarations:ident == 1) => {
        assert!($declarations.len() == 1,
                "expected 1 declaration; have {}: {:?})",
                $declarations.len(), $declarations)
    };
    ($declarations:ident == $len:expr) => {
        assert!($declarations.len() == $len,
                "expected {} declarations; have {}: {:?})",
                $len, $declarations.len(), $declarations)
    }
}

#[test]
fn empty_viewport_rule() {
    let device = Device::new(MediaType::Screen, Size2D::typed(800., 600.));

    test_viewport_rule("@viewport {}", &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 0);
    });
}

macro_rules! assert_decl_eq {
    ($d:expr, $origin:ident, $expected:ident: $value:expr) => {{
        assert_eq!($d.origin, Origin::$origin);
        assert_eq!($d.descriptor, ViewportDescriptor::$expected($value));
        assert!($d.important == false, "descriptor should not be !important");
    }};
    ($d:expr, $origin:ident, $expected:ident: $value:expr, !important) => {{
        assert_eq!($d.origin, Origin::$origin);
        assert_eq!($d.descriptor, ViewportDescriptor::$expected($value));
        assert!($d.important == true, "descriptor should be !important");
    }};
}

#[test]
fn simple_viewport_rules() {
    let device = Device::new(MediaType::Screen, Size2D::typed(800., 600.));

    test_viewport_rule("@viewport { width: auto; height: auto;\
                                    zoom: auto; min-zoom: 0; max-zoom: 200%;\
                                    user-zoom: zoom; orientation: auto; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 9);
        assert_decl_eq!(&declarations[0], Author, MinWidth: LengthOrPercentageOrAuto::Auto);
        assert_decl_eq!(&declarations[1], Author, MaxWidth: LengthOrPercentageOrAuto::Auto);
        assert_decl_eq!(&declarations[2], Author, MinHeight: LengthOrPercentageOrAuto::Auto);
        assert_decl_eq!(&declarations[3], Author, MaxHeight: LengthOrPercentageOrAuto::Auto);
        assert_decl_eq!(&declarations[4], Author, Zoom: Zoom::Auto);
        assert_decl_eq!(&declarations[5], Author, MinZoom: Zoom::Number(0.));
        assert_decl_eq!(&declarations[6], Author, MaxZoom: Zoom::Percentage(2.));
        assert_decl_eq!(&declarations[7], Author, UserZoom: UserZoom::Zoom);
        assert_decl_eq!(&declarations[8], Author, Orientation: Orientation::Auto);
    });

    test_viewport_rule("@viewport { min-width: 200px; max-width: auto;\
                                    min-height: 200px; max-height: auto; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 4);
        assert_decl_eq!(&declarations[0], Author, MinWidth: LengthOrPercentageOrAuto::Length(Length::from_px(200.)));
        assert_decl_eq!(&declarations[1], Author, MaxWidth: LengthOrPercentageOrAuto::Auto);
        assert_decl_eq!(&declarations[2], Author, MinHeight: LengthOrPercentageOrAuto::Length(Length::from_px(200.)));
        assert_decl_eq!(&declarations[3], Author, MaxHeight: LengthOrPercentageOrAuto::Auto);
    });
}

#[test]
fn cascading_within_viewport_rule() {
    let device = Device::new(MediaType::Screen, Size2D::typed(800., 600.));

    // normal order of appearance
    test_viewport_rule("@viewport { min-width: 200px; min-width: auto; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 1);
        assert_decl_eq!(&declarations[0], Author, MinWidth: LengthOrPercentageOrAuto::Auto);
    });

    // !important order of appearance
    test_viewport_rule("@viewport { min-width: 200px !important; min-width: auto !important; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 1);
        assert_decl_eq!(&declarations[0], Author, MinWidth: LengthOrPercentageOrAuto::Auto, !important);
    });

    // !important vs normal
    test_viewport_rule("@viewport { min-width: auto !important; min-width: 200px; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 1);
        assert_decl_eq!(&declarations[0], Author, MinWidth: LengthOrPercentageOrAuto::Auto, !important);
    });

    // normal longhands vs normal shorthand
    test_viewport_rule("@viewport { min-width: 200px; max-width: 200px; width: auto; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 2);
        assert_decl_eq!(&declarations[0], Author, MinWidth: LengthOrPercentageOrAuto::Auto);
        assert_decl_eq!(&declarations[1], Author, MaxWidth: LengthOrPercentageOrAuto::Auto);
    });

    // normal shorthand vs normal longhands
    test_viewport_rule("@viewport { width: 200px; min-width: auto; max-width: auto; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 2);
        assert_decl_eq!(&declarations[0], Author, MinWidth: LengthOrPercentageOrAuto::Auto);
        assert_decl_eq!(&declarations[1], Author, MaxWidth: LengthOrPercentageOrAuto::Auto);
    });

    // one !important longhand vs normal shorthand
    test_viewport_rule("@viewport { min-width: auto !important; width: 200px; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 2);
        assert_decl_eq!(&declarations[0], Author, MinWidth: LengthOrPercentageOrAuto::Auto, !important);
        assert_decl_eq!(&declarations[1], Author, MaxWidth: LengthOrPercentageOrAuto::Length(Length::from_px(200.)));
    });

    // both !important longhands vs normal shorthand
    test_viewport_rule("@viewport { min-width: auto !important; max-width: auto !important; width: 200px; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 2);
        assert_decl_eq!(&declarations[0], Author, MinWidth: LengthOrPercentageOrAuto::Auto, !important);
        assert_decl_eq!(&declarations[1], Author, MaxWidth: LengthOrPercentageOrAuto::Auto, !important);
    });
}

#[test]
fn multiple_stylesheets_cascading() {
    ::util::prefs::set_pref("layout.viewport.enabled", true);
    let device = Device::new(MediaType::Screen, Size2D::typed(800., 600.));

    let stylesheets = vec![
        stylesheet!("@viewport { min-width: 100px; min-height: 100px; zoom: 1; }", UserAgent),
        stylesheet!("@viewport { min-width: 200px; min-height: 200px; }", User),
        stylesheet!("@viewport { min-width: 300px; }", Author)];

    let declarations = stylesheets.iter()
        .flat_map(|s| s.effective_rules(&device).viewport())
        .cascade()
        .declarations;
    assert_decl_len!(declarations == 3);
    assert_decl_eq!(&declarations[0], UserAgent, Zoom: Zoom::Number(1.));
    assert_decl_eq!(&declarations[1], User, MinHeight: LengthOrPercentageOrAuto::Length(Length::from_px(200.)));
    assert_decl_eq!(&declarations[2], Author, MinWidth: LengthOrPercentageOrAuto::Length(Length::from_px(300.)));

    let stylesheets = vec![
        stylesheet!("@viewport { min-width: 100px !important; }", UserAgent),
        stylesheet!("@viewport { min-width: 200px !important; min-height: 200px !important; }", User),
        stylesheet!(
            "@viewport { min-width: 300px !important; min-height: 300px !important; zoom: 3 !important; }", Author)];

    let declarations = stylesheets.iter()
        .flat_map(|s| s.effective_rules(&device).viewport())
        .cascade()
        .declarations;
    assert_decl_len!(declarations == 3);
    assert_decl_eq!(
        &declarations[0], UserAgent, MinWidth: LengthOrPercentageOrAuto::Length(Length::from_px(100.)), !important);
    assert_decl_eq!(
        &declarations[1], User, MinHeight: LengthOrPercentageOrAuto::Length(Length::from_px(200.)), !important);
    assert_decl_eq!(&declarations[2], Author, Zoom: Zoom::Number(3.), !important);
}

#[test]
fn constrain_viewport() {
    let url = Url::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url);

    macro_rules! from_css {
        ($css:expr) => {
            &ViewportRule::parse(&mut Parser::new($css), &context).unwrap()
        }
    }

    let initial_viewport = Size2D::typed(800., 600.);
    assert_eq!(ViewportConstraints::maybe_new(initial_viewport, from_css!("")),
               None);

    let initial_viewport = Size2D::typed(800., 600.);
    assert_eq!(ViewportConstraints::maybe_new(initial_viewport, from_css!("width: 320px auto")),
               Some(ViewportConstraints {
                   size: initial_viewport,

                   initial_zoom: ScaleFactor::new(1.),
                   min_zoom: None,
                   max_zoom: None,

                   user_zoom: UserZoom::Zoom,
                   orientation: Orientation::Auto
               }));

    let initial_viewport = Size2D::typed(200., 150.);
    assert_eq!(ViewportConstraints::maybe_new(initial_viewport, from_css!("width: 320px auto")),
               Some(ViewportConstraints {
                   size: Size2D::typed(320., 240.),

                   initial_zoom: ScaleFactor::new(1.),
                   min_zoom: None,
                   max_zoom: None,

                   user_zoom: UserZoom::Zoom,
                   orientation: Orientation::Auto
               }));

    let initial_viewport = Size2D::typed(800., 600.);
    assert_eq!(ViewportConstraints::maybe_new(initial_viewport, from_css!("width: 320px auto")),
               Some(ViewportConstraints {
                   size: initial_viewport,

                   initial_zoom: ScaleFactor::new(1.),
                   min_zoom: None,
                   max_zoom: None,

                   user_zoom: UserZoom::Zoom,
                   orientation: Orientation::Auto
               }));

    let initial_viewport = Size2D::typed(800., 600.);
    assert_eq!(ViewportConstraints::maybe_new(initial_viewport, from_css!("width: 800px; height: 600px;\
                                                                     zoom: 1;\
                                                                     user-zoom: zoom;\
                                                                     orientation: auto;")),
               Some(ViewportConstraints {
                   size: initial_viewport,

                   initial_zoom: ScaleFactor::new(1.),
                   min_zoom: None,
                   max_zoom: None,

                   user_zoom: UserZoom::Zoom,
                   orientation: Orientation::Auto
               }));
}
