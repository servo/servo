/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use euclid::scale_factor::ScaleFactor;
use euclid::size::TypedSize2D;
use media_queries::CSSErrorReporterTest;
use servo_config::prefs::{PREFS, PrefValue};
use servo_url::ServoUrl;
use style::error_reporting::ParseErrorReporter;
use style::media_queries::{Device, MediaType};
use style::parser::{ParserContext, ParserContextExtraData};
use style::stylesheets::{Stylesheet, Origin};
use style::values::specified::Length::{self, ViewportPercentage};
use style::values::specified::LengthOrPercentageOrAuto::{self, Auto};
use style::values::specified::ViewportPercentageLength::Vw;
use style::viewport::*;
use style_traits::viewport::*;

macro_rules! stylesheet {
    ($css:expr, $origin:ident, $error_reporter:expr) => {
        Box::new(Stylesheet::from_str(
            $css,
            ServoUrl::parse("http://localhost").unwrap(),
            Origin::$origin,
            Default::default(),
            None,
            $error_reporter,
            ParserContextExtraData::default()
        ))
    }
}

fn test_viewport_rule<F>(css: &str,
                         device: &Device,
                         callback: F)
    where F: Fn(&Vec<ViewportDescriptorDeclaration>, &str)
{
    PREFS.set("layout.viewport.enabled", PrefValue::Boolean(true));
    let stylesheet = stylesheet!(css, Author, Box::new(CSSErrorReporterTest));
    let mut rule_count = 0;
    stylesheet.effective_viewport_rules(&device, |rule| {
        rule_count += 1;
        callback(&rule.declarations, css);
    });
    assert!(rule_count > 0);
}

fn test_meta_viewport<F>(meta: &str, callback: F)
    where F: Fn(&Vec<ViewportDescriptorDeclaration>, &str)
{
    if let Some(mut rule) = ViewportRule::from_meta(meta) {
        // from_meta uses a hash-map to collect the declarations, so we need to
        // sort them in a stable order for the tests
        rule.declarations.sort_by(|a, b| {
            let a = a.descriptor.discriminant_value();
            let b = b.descriptor.discriminant_value();
            a.cmp(&b)
        });

        callback(&rule.declarations, meta);
    } else {
        panic!("no @viewport rule for {}", meta);
    }
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

macro_rules! viewport_length {
    ($value:expr, px) => {
        ViewportLength::Specified(LengthOrPercentageOrAuto::Length(Length::from_px($value)))
    };
    ($value:expr, vw) => {
        ViewportLength::Specified(LengthOrPercentageOrAuto::Length(ViewportPercentage(Vw($value))))
    }
}

#[test]
fn empty_viewport_rule() {
    let device = Device::new(MediaType::Screen, TypedSize2D::new(800., 600.));

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
    let device = Device::new(MediaType::Screen, TypedSize2D::new(800., 600.));

    test_viewport_rule("@viewport { width: auto; height: auto;\
                                    zoom: auto; min-zoom: 0; max-zoom: 200%;\
                                    user-zoom: zoom; orientation: auto; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 9);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::Specified(Auto));
        assert_decl_eq!(&declarations[1], Author, MaxWidth: ViewportLength::Specified(Auto));
        assert_decl_eq!(&declarations[2], Author, MinHeight: ViewportLength::Specified(Auto));
        assert_decl_eq!(&declarations[3], Author, MaxHeight: ViewportLength::Specified(Auto));
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
        assert_decl_eq!(&declarations[0], Author, MinWidth: viewport_length!(200., px));
        assert_decl_eq!(&declarations[1], Author, MaxWidth: ViewportLength::Specified(Auto));
        assert_decl_eq!(&declarations[2], Author, MinHeight: viewport_length!(200., px));
        assert_decl_eq!(&declarations[3], Author, MaxHeight: ViewportLength::Specified(Auto));
    });
}

#[test]
fn simple_meta_viewport_contents() {
    test_meta_viewport("width=500, height=600", |declarations, meta| {
        println!("{}", meta);
        assert_decl_len!(declarations == 4);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::ExtendToZoom);
        assert_decl_eq!(&declarations[1], Author, MaxWidth: viewport_length!(500., px));
        assert_decl_eq!(&declarations[2], Author, MinHeight: ViewportLength::ExtendToZoom);
        assert_decl_eq!(&declarations[3], Author, MaxHeight: viewport_length!(600., px));
    });

    test_meta_viewport("initial-scale=1.0", |declarations, meta| {
        println!("{}", meta);
        assert_decl_len!(declarations == 3);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::ExtendToZoom);
        assert_decl_eq!(&declarations[1], Author, MaxWidth: ViewportLength::ExtendToZoom);
        assert_decl_eq!(&declarations[2], Author, Zoom: Zoom::Number(1.));
    });

    test_meta_viewport("initial-scale=2.0, height=device-width", |declarations, meta| {
        println!("{}", meta);
        assert_decl_len!(declarations == 5);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::Specified(Auto));
        assert_decl_eq!(&declarations[1], Author, MaxWidth: ViewportLength::Specified(Auto));
        assert_decl_eq!(&declarations[2], Author, MinHeight: ViewportLength::ExtendToZoom);
        assert_decl_eq!(&declarations[3], Author, MaxHeight: viewport_length!(100., vw));
        assert_decl_eq!(&declarations[4], Author, Zoom: Zoom::Number(2.));
    });

    test_meta_viewport("width=480, initial-scale=2.0, user-scalable=1", |declarations, meta| {
        println!("{}", meta);
        assert_decl_len!(declarations == 4);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::ExtendToZoom);
        assert_decl_eq!(&declarations[1], Author, MaxWidth: viewport_length!(480., px));
        assert_decl_eq!(&declarations[2], Author, Zoom: Zoom::Number(2.));
        assert_decl_eq!(&declarations[3], Author, UserZoom: UserZoom::Zoom);
    });
}

#[test]
fn cascading_within_viewport_rule() {
    let device = Device::new(MediaType::Screen, TypedSize2D::new(800., 600.));

    // normal order of appearance
    test_viewport_rule("@viewport { min-width: 200px; min-width: auto; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 1);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::Specified(Auto));
    });

    // !important order of appearance
    test_viewport_rule("@viewport { min-width: 200px !important; min-width: auto !important; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 1);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::Specified(Auto), !important);
    });

    // !important vs normal
    test_viewport_rule("@viewport { min-width: auto !important; min-width: 200px; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 1);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::Specified(Auto), !important);
    });

    // normal longhands vs normal shorthand
    test_viewport_rule("@viewport { min-width: 200px; max-width: 200px; width: auto; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 2);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::Specified(Auto));
        assert_decl_eq!(&declarations[1], Author, MaxWidth: ViewportLength::Specified(Auto));
    });

    // normal shorthand vs normal longhands
    test_viewport_rule("@viewport { width: 200px; min-width: auto; max-width: auto; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 2);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::Specified(Auto));
        assert_decl_eq!(&declarations[1], Author, MaxWidth: ViewportLength::Specified(Auto));
    });

    // one !important longhand vs normal shorthand
    test_viewport_rule("@viewport { min-width: auto !important; width: 200px; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 2);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::Specified(Auto), !important);
        assert_decl_eq!(&declarations[1], Author, MaxWidth: viewport_length!(200., px));
    });

    // both !important longhands vs normal shorthand
    test_viewport_rule("@viewport { min-width: auto !important; max-width: auto !important; width: 200px; }",
                       &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 2);
        assert_decl_eq!(&declarations[0], Author, MinWidth: ViewportLength::Specified(Auto), !important);
        assert_decl_eq!(&declarations[1], Author, MaxWidth: ViewportLength::Specified(Auto), !important);
    });
}

#[test]
fn multiple_stylesheets_cascading() {
    PREFS.set("layout.viewport.enabled", PrefValue::Boolean(true));
    let device = Device::new(MediaType::Screen, TypedSize2D::new(800., 600.));
    let error_reporter = CSSErrorReporterTest;
    let stylesheets = vec![
        stylesheet!("@viewport { min-width: 100px; min-height: 100px; zoom: 1; }", UserAgent, error_reporter.clone()),
        stylesheet!("@viewport { min-width: 200px; min-height: 200px; }", User, error_reporter.clone()),
        stylesheet!("@viewport { min-width: 300px; }", Author, error_reporter.clone())];

    let declarations = Cascade::from_stylesheets(&stylesheets, &device).finish();
    assert_decl_len!(declarations == 3);
    assert_decl_eq!(&declarations[0], UserAgent, Zoom: Zoom::Number(1.));
    assert_decl_eq!(&declarations[1], User, MinHeight: viewport_length!(200., px));
    assert_decl_eq!(&declarations[2], Author, MinWidth: viewport_length!(300., px));

    let stylesheets = vec![
        stylesheet!("@viewport { min-width: 100px !important; }", UserAgent, error_reporter.clone()),
        stylesheet!("@viewport { min-width: 200px !important; min-height: 200px !important; }",
        User, error_reporter.clone()),
        stylesheet!("@viewport { min-width: 300px !important; min-height: 300px !important; zoom: 3 !important; }",
        Author, error_reporter.clone())];
    let declarations = Cascade::from_stylesheets(&stylesheets, &device).finish();
    assert_decl_len!(declarations == 3);
    assert_decl_eq!(&declarations[0], UserAgent, MinWidth: viewport_length!(100., px), !important);
    assert_decl_eq!(&declarations[1], User, MinHeight: viewport_length!(200., px), !important);
    assert_decl_eq!(&declarations[2], Author, Zoom: Zoom::Number(3.), !important);
}

#[test]
fn constrain_viewport() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));

    macro_rules! from_css {
        ($css:expr) => {
            &ViewportRule::parse(&mut Parser::new($css), &context).unwrap()
        }
    }

    let initial_viewport = TypedSize2D::new(800., 600.);
    assert_eq!(ViewportConstraints::maybe_new(initial_viewport, from_css!("")),
               None);

    let initial_viewport = TypedSize2D::new(800., 600.);
    assert_eq!(ViewportConstraints::maybe_new(initial_viewport, from_css!("width: 320px auto")),
               Some(ViewportConstraints {
                   size: initial_viewport,

                   initial_zoom: ScaleFactor::new(1.),
                   min_zoom: None,
                   max_zoom: None,

                   user_zoom: UserZoom::Zoom,
                   orientation: Orientation::Auto
               }));

    let initial_viewport = TypedSize2D::new(200., 150.);
    assert_eq!(ViewportConstraints::maybe_new(initial_viewport, from_css!("width: 320px auto")),
               Some(ViewportConstraints {
                   size: TypedSize2D::new(320., 240.),

                   initial_zoom: ScaleFactor::new(1.),
                   min_zoom: None,
                   max_zoom: None,

                   user_zoom: UserZoom::Zoom,
                   orientation: Orientation::Auto
               }));

    let initial_viewport = TypedSize2D::new(800., 600.);
    assert_eq!(ViewportConstraints::maybe_new(initial_viewport, from_css!("width: 320px auto")),
               Some(ViewportConstraints {
                   size: initial_viewport,

                   initial_zoom: ScaleFactor::new(1.),
                   min_zoom: None,
                   max_zoom: None,

                   user_zoom: UserZoom::Zoom,
                   orientation: Orientation::Auto
               }));

    let initial_viewport = TypedSize2D::new(800., 600.);
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
