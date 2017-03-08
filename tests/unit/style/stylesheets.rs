/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{self, Parser as CssParser, SourcePosition};
use html5ever_atoms::{Namespace as NsAtom};
use media_queries::CSSErrorReporterTest;
use parking_lot::RwLock;
use selectors::parser::*;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use style::error_reporting::ParseErrorReporter;
use style::keyframes::{Keyframe, KeyframeSelector, KeyframePercentage};
use style::parser::ParserContextExtraData;
use style::properties::Importance;
use style::properties::{CSSWideKeyword, PropertyDeclaration, PropertyDeclarationBlock};
use style::properties::{DeclaredValue, longhands};
use style::properties::longhands::animation_play_state;
use style::stylesheets::{Origin, Namespaces};
use style::stylesheets::{Stylesheet, NamespaceRule, CssRule, CssRules, StyleRule, KeyframesRule};
use style::values::specified::{LengthOrPercentageOrAuto, Percentage};

pub fn block_from<I>(iterable: I) -> PropertyDeclarationBlock
where I: IntoIterator<Item=(PropertyDeclaration, Importance)> {
    let mut block = PropertyDeclarationBlock::new();
    for (d, i) in iterable {
        block.push(d, i)
    }
    block
}

#[test]
fn test_parse_stylesheet() {
    let css = r"
        @namespace url(http://www.w3.org/1999/xhtml);
        /* FIXME: only if scripting is enabled */
        input[type=hidden i] {
            display: block !important;
            display: none !important;
            display: inline;
            --a: b !important;
            --a: inherit !important;
            --a: c;
        }
        html , body /**/ {
            display: none;
            display: block;
        }
        #d1 > .ok { background: blue; }
        @keyframes foo {
            from { width: 0% }
            to {
                width: 100%;
                width: 50% !important; /* !important not allowed here */
                animation-name: 'foo'; /* animation properties not allowed here */
                animation-play-state: running; /* … except animation-play-state */
            }
        }";
    let url = ServoUrl::parse("about::test").unwrap();
    let stylesheet = Stylesheet::from_str(css, url.clone(), Origin::UserAgent, Default::default(),
                                          None,
                                          Box::new(CSSErrorReporterTest),
                                          ParserContextExtraData::default());
    let mut namespaces = Namespaces::default();
    namespaces.default = Some(ns!(html));
    let expected = Stylesheet {
        origin: Origin::UserAgent,
        media: Default::default(),
        namespaces: RwLock::new(namespaces),
        base_url: url,
        dirty_on_viewport_size_change: AtomicBool::new(false),
        disabled: AtomicBool::new(false),
        rules: CssRules::new(vec![
            CssRule::Namespace(Arc::new(RwLock::new(NamespaceRule {
                prefix: None,
                url: NsAtom::from("http://www.w3.org/1999/xhtml")
            }))),
            CssRule::Style(Arc::new(RwLock::new(StyleRule {
                selectors: SelectorList(vec![
                    Selector {
                        complex_selector: Arc::new(ComplexSelector {
                            compound_selector: vec![
                                SimpleSelector::Namespace(Namespace {
                                    prefix: None,
                                    url: NsAtom::from("http://www.w3.org/1999/xhtml")
                                }),
                                SimpleSelector::LocalName(LocalName {
                                    name: local_name!("input"),
                                    lower_name: local_name!("input"),
                                }),
                                SimpleSelector::AttrEqual(AttrSelector {
                                    name: local_name!("type"),
                                    lower_name: local_name!("type"),
                                    namespace: NamespaceConstraint::Specific(Namespace {
                                        prefix: None,
                                        url: ns!()
                                    }),
                                }, "hidden".to_owned(), CaseSensitivity::CaseInsensitive)
                            ],
                            next: None,
                        }),
                        pseudo_element: None,
                        specificity: (0 << 20) + (1 << 10) + (1 << 0),
                    },
                ]),
                block: Arc::new(RwLock::new(block_from(vec![
                    (PropertyDeclaration::Display(DeclaredValue::Value(
                        longhands::display::SpecifiedValue::none)),
                     Importance::Important),
                    (PropertyDeclaration::Custom(Atom::from("a"),
                     DeclaredValue::CSSWideKeyword(CSSWideKeyword::Inherit)),
                     Importance::Important),
                ]))),
            }))),
            CssRule::Style(Arc::new(RwLock::new(StyleRule {
                selectors: SelectorList(vec![
                    Selector {
                        complex_selector: Arc::new(ComplexSelector {
                            compound_selector: vec![
                                SimpleSelector::Namespace(Namespace {
                                    prefix: None,
                                    url: NsAtom::from("http://www.w3.org/1999/xhtml")
                                }),
                                SimpleSelector::LocalName(LocalName {
                                    name: local_name!("html"),
                                    lower_name: local_name!("html"),
                                }),
                            ],
                            next: None,
                        }),
                        pseudo_element: None,
                        specificity: (0 << 20) + (0 << 10) + (1 << 0),
                    },
                    Selector {
                        complex_selector: Arc::new(ComplexSelector {
                            compound_selector: vec![
                                SimpleSelector::Namespace(Namespace {
                                    prefix: None,
                                    url: NsAtom::from("http://www.w3.org/1999/xhtml")
                                }),
                                SimpleSelector::LocalName(LocalName {
                                    name: local_name!("body"),
                                    lower_name: local_name!("body"),
                                }),
                            ],
                            next: None,
                        }),
                        pseudo_element: None,
                        specificity: (0 << 20) + (0 << 10) + (1 << 0),
                    },
                ]),
                block: Arc::new(RwLock::new(block_from(vec![
                    (PropertyDeclaration::Display(DeclaredValue::Value(
                        longhands::display::SpecifiedValue::block)),
                     Importance::Normal),
                ]))),
            }))),
            CssRule::Style(Arc::new(RwLock::new(StyleRule {
                selectors: SelectorList(vec![
                    Selector {
                        complex_selector: Arc::new(ComplexSelector {
                            compound_selector: vec![
                                SimpleSelector::Namespace(Namespace {
                                    prefix: None,
                                    url: NsAtom::from("http://www.w3.org/1999/xhtml")
                                }),
                                SimpleSelector::Class(Atom::from("ok")),
                            ],
                            next: Some((Arc::new(ComplexSelector {
                                compound_selector: vec![
                                    SimpleSelector::Namespace(Namespace {
                                        prefix: None,
                                        url: NsAtom::from("http://www.w3.org/1999/xhtml")
                                    }),
                                    SimpleSelector::ID(Atom::from("d1")),
                                ],
                                next: None,
                            }), Combinator::Child)),
                        }),
                        pseudo_element: None,
                        specificity: (1 << 20) + (1 << 10) + (0 << 0),
                    },
                ]),
                block: Arc::new(RwLock::new(block_from(vec![
                    (PropertyDeclaration::BackgroundColor(DeclaredValue::Value(
                        longhands::background_color::SpecifiedValue {
                            authored: Some("blue".to_owned().into_boxed_str()),
                            parsed: cssparser::Color::RGBA(cssparser::RGBA::new(0, 0, 255, 255)),
                        }
                     )),
                     Importance::Normal),
                    (PropertyDeclaration::BackgroundPositionX(DeclaredValue::Value(
                        longhands::background_position_x::SpecifiedValue(
                        vec![longhands::background_position_x::single_value
                                                   ::get_initial_position_value()]))),
                    Importance::Normal),
                    (PropertyDeclaration::BackgroundPositionY(DeclaredValue::Value(
                        longhands::background_position_y::SpecifiedValue(
                        vec![longhands::background_position_y::single_value
                                                   ::get_initial_position_value()]))),
                     Importance::Normal),
                    (PropertyDeclaration::BackgroundRepeat(DeclaredValue::Value(
                        longhands::background_repeat::SpecifiedValue(
                        vec![longhands::background_repeat::single_value
                                                   ::get_initial_specified_value()]))),
                     Importance::Normal),
                    (PropertyDeclaration::BackgroundAttachment(DeclaredValue::Value(
                        longhands::background_attachment::SpecifiedValue(
                        vec![longhands::background_attachment::single_value
                                                   ::get_initial_specified_value()]))),
                     Importance::Normal),
                    (PropertyDeclaration::BackgroundImage(DeclaredValue::Value(
                        longhands::background_image::SpecifiedValue(
                        vec![longhands::background_image::single_value
                                                   ::get_initial_specified_value()]))),
                     Importance::Normal),
                    (PropertyDeclaration::BackgroundSize(DeclaredValue::Value(
                        longhands::background_size::SpecifiedValue(
                        vec![longhands::background_size::single_value
                                                   ::get_initial_specified_value()]))),
                     Importance::Normal),
                    (PropertyDeclaration::BackgroundOrigin(DeclaredValue::Value(
                        longhands::background_origin::SpecifiedValue(
                        vec![longhands::background_origin::single_value
                                                   ::get_initial_specified_value()]))),
                     Importance::Normal),
                    (PropertyDeclaration::BackgroundClip(DeclaredValue::Value(
                        longhands::background_clip::SpecifiedValue(
                        vec![longhands::background_clip::single_value
                                                   ::get_initial_specified_value()]))),
                     Importance::Normal),
                ]))),
            }))),
            CssRule::Keyframes(Arc::new(RwLock::new(KeyframesRule {
                name: "foo".into(),
                keyframes: vec![
                    Arc::new(RwLock::new(Keyframe {
                        selector: KeyframeSelector::new_for_unit_testing(
                                      vec![KeyframePercentage::new(0.)]),
                        block: Arc::new(RwLock::new(block_from(vec![
                            (PropertyDeclaration::Width(DeclaredValue::Value(
                                LengthOrPercentageOrAuto::Percentage(Percentage(0.)))),
                             Importance::Normal),
                        ])))
                    })),
                    Arc::new(RwLock::new(Keyframe {
                        selector: KeyframeSelector::new_for_unit_testing(
                                      vec![KeyframePercentage::new(1.)]),
                        block: Arc::new(RwLock::new(block_from(vec![
                            (PropertyDeclaration::Width(DeclaredValue::Value(
                                LengthOrPercentageOrAuto::Percentage(Percentage(1.)))),
                             Importance::Normal),
                            (PropertyDeclaration::AnimationPlayState(DeclaredValue::Value(
                                animation_play_state::SpecifiedValue(
                                    vec![animation_play_state::SingleSpecifiedValue::running]))),
                             Importance::Normal),
                        ]))),
                    })),
                ]
            })))

        ]),
    };

    assert_eq!(format!("{:#?}", stylesheet), format!("{:#?}", expected));
}

struct CSSError {
    pub url : ServoUrl,
    pub line: usize,
    pub column: usize,
    pub message: String
}

struct CSSInvalidErrorReporterTest {
    pub errors: Arc<Mutex<Vec<CSSError>>>
}

impl CSSInvalidErrorReporterTest {
    pub fn new() -> CSSInvalidErrorReporterTest {
        return CSSInvalidErrorReporterTest{
            errors: Arc::new(Mutex::new(Vec::new()))
        }
    }
}

impl ParseErrorReporter for CSSInvalidErrorReporterTest {
    fn report_error(&self, input: &mut CssParser, position: SourcePosition, message: &str,
        url: &ServoUrl) {

        let location = input.source_location(position);

        let errors = self.errors.clone();
        let mut errors = errors.lock().unwrap();

        errors.push(
            CSSError{
                url: url.clone(),
                line: location.line,
                column: location.column,
                message: message.to_owned()
            }
        );
    }

    fn clone(&self) -> Box<ParseErrorReporter + Send + Sync> {
        return Box::new(
            CSSInvalidErrorReporterTest{
                errors: self.errors.clone()
            }
        );
    }
}


#[test]
fn test_report_error_stylesheet() {
    let css = r"
    div {
        background-color: red;
        display: invalid;
        invalid: true;
    }
    ";
    let url = ServoUrl::parse("about::test").unwrap();
    let error_reporter = Box::new(CSSInvalidErrorReporterTest::new());

    let errors = error_reporter.errors.clone();

    Stylesheet::from_str(css, url.clone(), Origin::UserAgent, Default::default(),
                         None,
                         error_reporter,
                         ParserContextExtraData::default());

    let mut errors = errors.lock().unwrap();

    let error = errors.pop().unwrap();
    assert_eq!("Unsupported property declaration: 'invalid: true;'", error.message);
    assert_eq!(5, error.line);
    assert_eq!(9, error.column);

    let error = errors.pop().unwrap();
    assert_eq!("Unsupported property declaration: 'display: invalid;'", error.message);
    assert_eq!(4, error.line);
    assert_eq!(9, error.column);

    // testing for the url
    assert_eq!(url, error.url);
}
