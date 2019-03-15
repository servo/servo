/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput};
use euclid::TypedScale;
use euclid::TypedSize2D;
use servo_arc::Arc;
use servo_config::prefs::{PrefValue, PREFS};
use servo_url::ServoUrl;
use style::context::QuirksMode;
use style::media_queries::{Device, MediaList, MediaType};
use style::parser::ParserContext;
use style::shared_lock::{SharedRwLock, StylesheetGuards};
use style::stylesheets::viewport_rule::*;
use style::stylesheets::{CssRuleType, Origin, Stylesheet, StylesheetInDocument};
use style::values::specified::LengthPercentage;
use style::values::specified::LengthPercentageOrAuto::{self, Auto};
use style::values::specified::NoCalcLength::{self, ViewportPercentage};
use style::values::specified::ViewportPercentageLength::Vw;
use style_traits::viewport::*;
use style_traits::{ParsingMode, PinchZoomFactor};

macro_rules! stylesheet {
    ($css:expr, $origin:ident) => {
        stylesheet!($css, $origin, SharedRwLock::new())
    };
    ($css:expr, $origin:ident, $shared_lock:expr) => {
        Arc::new(Stylesheet::from_str(
            $css,
            ServoUrl::parse("http://localhost").unwrap(),
            Origin::$origin,
            Arc::new($shared_lock.wrap(MediaList::empty())),
            $shared_lock,
            None,
            None,
            QuirksMode::NoQuirks,
            0,
        ))
    };
}

fn test_viewport_rule<F>(css: &str, device: &Device, callback: F)
where
    F: Fn(&Vec<ViewportDescriptorDeclaration>, &str),
{
    PREFS.set("layout.viewport.enabled", PrefValue::Boolean(true));
    let stylesheet = stylesheet!(css, Author);
    let mut rule_count = 0;
    stylesheet.effective_viewport_rules(&device, &stylesheet.shared_lock.read(), |rule| {
        rule_count += 1;
        callback(&rule.declarations, css);
    });
    assert!(rule_count > 0);
}

fn test_meta_viewport<F>(meta: &str, callback: F)
where
    F: Fn(&Vec<ViewportDescriptorDeclaration>, &str),
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
        assert_eq!(
            $declarations.len(),
            1,
            "expected 1 declaration; have {}: {:?})",
            $declarations.len(),
            $declarations
        )
    };
    ($declarations:ident == $len:expr) => {
        assert_eq!(
            $declarations.len(),
            $len,
            "expected {} declarations; have {}: {:?})",
            $len,
            $declarations.len(),
            $declarations
        )
    };
}

macro_rules! viewport_length {
    ($value:expr, px) => {
        ViewportLength::Specified(LengthPercentageOrAuto::LengthPercentage(
            LengthPercentage::Length(NoCalcLength::from_px($value)),
        ))
    };
    ($value:expr, vw) => {
        ViewportLength::Specified(LengthPercentageOrAuto::LengthPercentage(
            LengthPercentage::Length(ViewportPercentage(Vw($value))),
        ))
    };
}

#[test]
fn empty_viewport_rule() {
    let device = Device::new(
        MediaType::screen(),
        TypedSize2D::new(800., 600.),
        TypedScale::new(1.0),
    );

    test_viewport_rule("@viewport {}", &device, |declarations, css| {
        println!("{}", css);
        assert_decl_len!(declarations == 0);
    });
}

macro_rules! assert_decl_eq {
    ($d:expr, $origin:ident, $expected:ident: $value:expr) => {{
        assert_eq!($d.origin, Origin::$origin);
        assert_eq!($d.descriptor, ViewportDescriptor::$expected($value));
        assert_eq!($d.important, false, "descriptor should not be !important");
    }};
    ($d:expr, $origin:ident, $expected:ident: $value:expr, !important) => {{
        assert_eq!($d.origin, Origin::$origin);
        assert_eq!($d.descriptor, ViewportDescriptor::$expected($value));
        assert_eq!($d.important, true, "descriptor should be !important");
    }};
}

#[test]
fn simple_viewport_rules() {
    let device = Device::new(
        MediaType::screen(),
        TypedSize2D::new(800., 600.),
        TypedScale::new(1.0),
    );

    test_viewport_rule(
        "@viewport { width: auto; height: auto;\
         zoom: auto; min-zoom: 0; max-zoom: 200%;\
         user-zoom: zoom; orientation: auto; }",
        &device,
        |declarations, css| {
            println!("{}", css);
            assert_decl_len!(declarations == 9);
            assert_decl_eq!(
                &declarations[0],
                Author,
                MinWidth: ViewportLength::Specified(Auto)
            );
            assert_decl_eq!(
                &declarations[1],
                Author,
                MaxWidth: ViewportLength::Specified(Auto)
            );
            assert_decl_eq!(
                &declarations[2],
                Author,
                MinHeight: ViewportLength::Specified(Auto)
            );
            assert_decl_eq!(
                &declarations[3],
                Author,
                MaxHeight: ViewportLength::Specified(Auto)
            );
            assert_decl_eq!(&declarations[4], Author, Zoom: Zoom::Auto);
            assert_decl_eq!(&declarations[5], Author, MinZoom: Zoom::Number(0.));
            assert_decl_eq!(&declarations[6], Author, MaxZoom: Zoom::Percentage(2.));
            assert_decl_eq!(&declarations[7], Author, UserZoom: UserZoom::Zoom);
            assert_decl_eq!(&declarations[8], Author, Orientation: Orientation::Auto);
        },
    );

    test_viewport_rule(
        "@viewport { min-width: 200px; max-width: auto;\
         min-height: 200px; max-height: auto; }",
        &device,
        |declarations, css| {
            println!("{}", css);
            assert_decl_len!(declarations == 4);
            assert_decl_eq!(
                &declarations[0],
                Author,
                MinWidth: viewport_length!(200., px)
            );
            assert_decl_eq!(
                &declarations[1],
                Author,
                MaxWidth: ViewportLength::Specified(Auto)
            );
            assert_decl_eq!(
                &declarations[2],
                Author,
                MinHeight: viewport_length!(200., px)
            );
            assert_decl_eq!(
                &declarations[3],
                Author,
                MaxHeight: ViewportLength::Specified(Auto)
            );
        },
    );
}

#[test]
fn simple_meta_viewport_contents() {
    test_meta_viewport("width=500, height=600", |declarations, meta| {
        println!("{}", meta);
        assert_decl_len!(declarations == 4);
        assert_decl_eq!(
            &declarations[0],
            Author,
            MinWidth: ViewportLength::ExtendToZoom
        );
        assert_decl_eq!(
            &declarations[1],
            Author,
            MaxWidth: viewport_length!(500., px)
        );
        assert_decl_eq!(
            &declarations[2],
            Author,
            MinHeight: ViewportLength::ExtendToZoom
        );
        assert_decl_eq!(
            &declarations[3],
            Author,
            MaxHeight: viewport_length!(600., px)
        );
    });

    test_meta_viewport("initial-scale=1.0", |declarations, meta| {
        println!("{}", meta);
        assert_decl_len!(declarations == 3);
        assert_decl_eq!(
            &declarations[0],
            Author,
            MinWidth: ViewportLength::ExtendToZoom
        );
        assert_decl_eq!(
            &declarations[1],
            Author,
            MaxWidth: ViewportLength::ExtendToZoom
        );
        assert_decl_eq!(&declarations[2], Author, Zoom: Zoom::Number(1.));
    });

    test_meta_viewport(
        "initial-scale=2.0, height=device-width",
        |declarations, meta| {
            println!("{}", meta);
            assert_decl_len!(declarations == 5);
            assert_decl_eq!(
                &declarations[0],
                Author,
                MinWidth: ViewportLength::Specified(Auto)
            );
            assert_decl_eq!(
                &declarations[1],
                Author,
                MaxWidth: ViewportLength::Specified(Auto)
            );
            assert_decl_eq!(
                &declarations[2],
                Author,
                MinHeight: ViewportLength::ExtendToZoom
            );
            assert_decl_eq!(
                &declarations[3],
                Author,
                MaxHeight: viewport_length!(100., vw)
            );
            assert_decl_eq!(&declarations[4], Author, Zoom: Zoom::Number(2.));
        },
    );

    test_meta_viewport(
        "width=480, initial-scale=2.0, user-scalable=1",
        |declarations, meta| {
            println!("{}", meta);
            assert_decl_len!(declarations == 4);
            assert_decl_eq!(
                &declarations[0],
                Author,
                MinWidth: ViewportLength::ExtendToZoom
            );
            assert_decl_eq!(
                &declarations[1],
                Author,
                MaxWidth: viewport_length!(480., px)
            );
            assert_decl_eq!(&declarations[2], Author, Zoom: Zoom::Number(2.));
            assert_decl_eq!(&declarations[3], Author, UserZoom: UserZoom::Zoom);
        },
    );
}

#[test]
fn cascading_within_viewport_rule() {
    let device = Device::new(
        MediaType::screen(),
        TypedSize2D::new(800., 600.),
        TypedScale::new(1.0),
    );

    // normal order of appearance
    test_viewport_rule(
        "@viewport { min-width: 200px; min-width: auto; }",
        &device,
        |declarations, css| {
            println!("{}", css);
            assert_decl_len!(declarations == 1);
            assert_decl_eq!(
                &declarations[0],
                Author,
                MinWidth: ViewportLength::Specified(Auto)
            );
        },
    );

    // !important order of appearance
    test_viewport_rule(
        "@viewport { min-width: 200px !important; min-width: auto !important; }",
        &device,
        |declarations, css| {
            println!("{}", css);
            assert_decl_len!(declarations == 1);
            assert_decl_eq!(
                &declarations[0],
                Author,
                MinWidth: ViewportLength::Specified(Auto),
                !important
            );
        },
    );

    // !important vs normal
    test_viewport_rule(
        "@viewport { min-width: auto !important; min-width: 200px; }",
        &device,
        |declarations, css| {
            println!("{}", css);
            assert_decl_len!(declarations == 1);
            assert_decl_eq!(
                &declarations[0],
                Author,
                MinWidth: ViewportLength::Specified(Auto),
                !important
            );
        },
    );

    // normal longhands vs normal shorthand
    test_viewport_rule(
        "@viewport { min-width: 200px; max-width: 200px; width: auto; }",
        &device,
        |declarations, css| {
            println!("{}", css);
            assert_decl_len!(declarations == 2);
            assert_decl_eq!(
                &declarations[0],
                Author,
                MinWidth: ViewportLength::Specified(Auto)
            );
            assert_decl_eq!(
                &declarations[1],
                Author,
                MaxWidth: ViewportLength::Specified(Auto)
            );
        },
    );

    // normal shorthand vs normal longhands
    test_viewport_rule(
        "@viewport { width: 200px; min-width: auto; max-width: auto; }",
        &device,
        |declarations, css| {
            println!("{}", css);
            assert_decl_len!(declarations == 2);
            assert_decl_eq!(
                &declarations[0],
                Author,
                MinWidth: ViewportLength::Specified(Auto)
            );
            assert_decl_eq!(
                &declarations[1],
                Author,
                MaxWidth: ViewportLength::Specified(Auto)
            );
        },
    );

    // one !important longhand vs normal shorthand
    test_viewport_rule(
        "@viewport { min-width: auto !important; width: 200px; }",
        &device,
        |declarations, css| {
            println!("{}", css);
            assert_decl_len!(declarations == 2);
            assert_decl_eq!(
                &declarations[0],
                Author,
                MinWidth: ViewportLength::Specified(Auto),
                !important
            );
            assert_decl_eq!(
                &declarations[1],
                Author,
                MaxWidth: viewport_length!(200., px)
            );
        },
    );

    // both !important longhands vs normal shorthand
    test_viewport_rule(
        "@viewport { min-width: auto !important; max-width: auto !important; width: 200px; }",
        &device,
        |declarations, css| {
            println!("{}", css);
            assert_decl_len!(declarations == 2);
            assert_decl_eq!(
                &declarations[0],
                Author,
                MinWidth: ViewportLength::Specified(Auto),
                !important
            );
            assert_decl_eq!(
                &declarations[1],
                Author,
                MaxWidth: ViewportLength::Specified(Auto),
                !important
            );
        },
    );
}

#[test]
fn multiple_stylesheets_cascading() {
    PREFS.set("layout.viewport.enabled", PrefValue::Boolean(true));
    let device = Device::new(
        MediaType::screen(),
        TypedSize2D::new(800., 600.),
        TypedScale::new(1.0),
    );
    let shared_lock = SharedRwLock::new();
    let stylesheets = vec![
        stylesheet!(
            "@viewport { min-width: 100px; min-height: 100px; zoom: 1; }",
            UserAgent,
            shared_lock.clone()
        ),
        stylesheet!(
            "@viewport { min-width: 200px; min-height: 200px; }",
            User,
            shared_lock.clone()
        ),
        stylesheet!(
            "@viewport { min-width: 300px; }",
            Author,
            shared_lock.clone()
        ),
    ];

    let declarations = Cascade::from_stylesheets(
        stylesheets.iter().map(|s| (&**s, Origin::Author)),
        &StylesheetGuards::same(&shared_lock.read()),
        &device,
    )
    .finish();
    assert_decl_len!(declarations == 3);
    assert_decl_eq!(&declarations[0], UserAgent, Zoom: Zoom::Number(1.));
    assert_decl_eq!(
        &declarations[1],
        User,
        MinHeight: viewport_length!(200., px)
    );
    assert_decl_eq!(
        &declarations[2],
        Author,
        MinWidth: viewport_length!(300., px)
    );

    let stylesheets = vec![
        stylesheet!("@viewport { min-width: 100px !important; }",
                    UserAgent, shared_lock.clone()),
        stylesheet!("@viewport { min-width: 200px !important; min-height: 200px !important; }",
                    User, shared_lock.clone()),
        stylesheet!("@viewport { min-width: 300px !important; min-height: 300px !important; zoom: 3 !important; }",
                    Author, shared_lock.clone())
    ];
    let declarations = Cascade::from_stylesheets(
        stylesheets.iter().map(|s| (&**s, Origin::Author)),
        &StylesheetGuards::same(&shared_lock.read()),
        &device,
    )
    .finish();
    assert_decl_len!(declarations == 3);
    assert_decl_eq!(
        &declarations[0],
        UserAgent,
        MinWidth: viewport_length!(100., px),
        !important
    );
    assert_decl_eq!(
        &declarations[1],
        User,
        MinHeight: viewport_length!(200., px),
        !important
    );
    assert_decl_eq!(&declarations[2], Author, Zoom: Zoom::Number(3.), !important);
}

#[test]
fn constrain_viewport() {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(
        Origin::Author,
        &url,
        Some(CssRuleType::Viewport),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
        None,
        None,
    );

    macro_rules! from_css {
        ($css:expr) => {
            &ViewportRule::parse(&context, &mut Parser::new(&mut $css)).unwrap()
        };
    }

    let initial_viewport = TypedSize2D::new(800., 600.);
    let device = Device::new(MediaType::screen(), initial_viewport, TypedScale::new(1.0));
    let mut input = ParserInput::new("");
    assert_eq!(
        ViewportConstraints::maybe_new(&device, from_css!(input), QuirksMode::NoQuirks),
        None
    );

    let mut input = ParserInput::new("width: 320px auto");
    assert_eq!(
        ViewportConstraints::maybe_new(&device, from_css!(input), QuirksMode::NoQuirks),
        Some(ViewportConstraints {
            size: initial_viewport,

            initial_zoom: PinchZoomFactor::new(1.),
            min_zoom: None,
            max_zoom: None,

            user_zoom: UserZoom::Zoom,
            orientation: Orientation::Auto
        })
    );

    let mut input = ParserInput::new("width: 320px auto");
    assert_eq!(
        ViewportConstraints::maybe_new(&device, from_css!(input), QuirksMode::NoQuirks),
        Some(ViewportConstraints {
            size: initial_viewport,

            initial_zoom: PinchZoomFactor::new(1.),
            min_zoom: None,
            max_zoom: None,

            user_zoom: UserZoom::Zoom,
            orientation: Orientation::Auto
        })
    );

    let mut input = ParserInput::new(
        "width: 800px; height: 600px;\
         zoom: 1;\
         user-zoom: zoom;\
         orientation: auto;",
    );
    assert_eq!(
        ViewportConstraints::maybe_new(&device, from_css!(input), QuirksMode::NoQuirks),
        Some(ViewportConstraints {
            size: initial_viewport,

            initial_zoom: PinchZoomFactor::new(1.),
            min_zoom: None,
            max_zoom: None,

            user_zoom: UserZoom::Zoom,
            orientation: Orientation::Auto
        })
    );

    let initial_viewport = TypedSize2D::new(200., 150.);
    let device = Device::new(MediaType::screen(), initial_viewport, TypedScale::new(1.0));
    let mut input = ParserInput::new("width: 320px auto");
    assert_eq!(
        ViewportConstraints::maybe_new(&device, from_css!(input), QuirksMode::NoQuirks),
        Some(ViewportConstraints {
            size: TypedSize2D::new(320., 240.),

            initial_zoom: PinchZoomFactor::new(1.),
            min_zoom: None,
            max_zoom: None,

            user_zoom: UserZoom::Zoom,
            orientation: Orientation::Auto
        })
    );
}
