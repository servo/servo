/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{self, Parser, SourcePosition};
use media_queries::CSSErrorReporterTest;
use selectors::parser::*;
use std::borrow::ToOwned;
use std::sync::Arc;
use std::sync::Mutex;
use string_cache::{Atom, Namespace as NsAtom};
use style::error_reporting::ParseErrorReporter;
use style::keyframes::{Keyframe, KeyframeSelector, KeyframePercentage};
use style::parser::ParserContextExtraData;
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock, DeclaredValue, longhands};
use style::stylesheets::{Stylesheet, CSSRule, StyleRule, KeyframesRule, Origin};
use style::values::specified::{LengthOrPercentageOrAuto, Percentage};
use url::Url;

#[test]
fn test_parse_stylesheet() {
    let css = r"
        @namespace url(http://www.w3.org/1999/xhtml);
        /* FIXME: only if scripting is enabled */
        input[type=hidden i] { display: none !important; }
        html , body /**/ { display: block; }
        #d1 > .ok { background: blue; }
        @keyframes foo {
            from { width: 0% }
            to { width: 100%}
        }";
    let url = Url::parse("about::test").unwrap();
    let stylesheet = Stylesheet::from_str(css, url, Origin::UserAgent,
                                          Box::new(CSSErrorReporterTest),
                                          ParserContextExtraData::default());
    assert_eq!(stylesheet, Stylesheet {
        origin: Origin::UserAgent,
        media: None,
        dirty_on_viewport_size_change: false,
        rules: vec![
            CSSRule::Namespace {
                prefix: None,
                url: NsAtom(Atom::from("http://www.w3.org/1999/xhtml"))
            },
            CSSRule::Style(StyleRule {
                selectors: vec![
                    Selector {
                        complex_selector: Arc::new(ComplexSelector {
                            compound_selector: vec![
                                SimpleSelector::Namespace(Namespace {
                                    prefix: None,
                                    url: NsAtom(Atom::from("http://www.w3.org/1999/xhtml"))
                                }),
                                SimpleSelector::LocalName(LocalName {
                                    name: atom!("input"),
                                    lower_name: atom!("input"),
                                }),
                                SimpleSelector::AttrEqual(AttrSelector {
                                    name: atom!("type"),
                                    lower_name: atom!("type"),
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
                ],
                declarations: PropertyDeclarationBlock {
                    normal: Arc::new(vec![]),
                    important: Arc::new(vec![
                        PropertyDeclaration::Display(DeclaredValue::Value(
                            longhands::display::SpecifiedValue::none)),
                    ]),
                },
            }),
            CSSRule::Style(StyleRule {
                selectors: vec![
                    Selector {
                        complex_selector: Arc::new(ComplexSelector {
                            compound_selector: vec![
                                SimpleSelector::Namespace(Namespace {
                                    prefix: None,
                                    url: NsAtom(Atom::from("http://www.w3.org/1999/xhtml"))
                                }),
                                SimpleSelector::LocalName(LocalName {
                                    name: atom!("html"),
                                    lower_name: atom!("html"),
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
                                    url: NsAtom(Atom::from("http://www.w3.org/1999/xhtml"))
                                }),
                                SimpleSelector::LocalName(LocalName {
                                    name: atom!("body"),
                                    lower_name: atom!("body"),
                                }),
                            ],
                            next: None,
                        }),
                        pseudo_element: None,
                        specificity: (0 << 20) + (0 << 10) + (1 << 0),
                    },
                ],
                declarations: PropertyDeclarationBlock {
                    normal: Arc::new(vec![
                        PropertyDeclaration::Display(DeclaredValue::Value(
                            longhands::display::SpecifiedValue::block)),
                    ]),
                    important: Arc::new(vec![]),
                },
            }),
            CSSRule::Style(StyleRule {
                selectors: vec![
                    Selector {
                        complex_selector: Arc::new(ComplexSelector {
                            compound_selector: vec![
                                SimpleSelector::Namespace(Namespace {
                                    prefix: None,
                                    url: NsAtom(Atom::from("http://www.w3.org/1999/xhtml"))
                                }),
                                SimpleSelector::Class(Atom::from("ok")),
                            ],
                            next: Some((Arc::new(ComplexSelector {
                                compound_selector: vec![
                                    SimpleSelector::Namespace(Namespace {
                                        prefix: None,
                                        url: NsAtom(Atom::from("http://www.w3.org/1999/xhtml"))
                                    }),
                                    SimpleSelector::ID(Atom::from("d1")),
                                ],
                                next: None,
                            }), Combinator::Child)),
                        }),
                        pseudo_element: None,
                        specificity: (1 << 20) + (1 << 10) + (0 << 0),
                    },
                ],
                declarations: PropertyDeclarationBlock {
                    normal: Arc::new(vec![
                        PropertyDeclaration::BackgroundClip(DeclaredValue::Initial),
                        PropertyDeclaration::BackgroundOrigin(DeclaredValue::Initial),
                        PropertyDeclaration::BackgroundSize(DeclaredValue::Initial),
                        PropertyDeclaration::BackgroundImage(DeclaredValue::Initial),
                        PropertyDeclaration::BackgroundAttachment(DeclaredValue::Initial),
                        PropertyDeclaration::BackgroundRepeat(DeclaredValue::Initial),
                        PropertyDeclaration::BackgroundPosition(DeclaredValue::Initial),
                        PropertyDeclaration::BackgroundColor(DeclaredValue::Value(
                            longhands::background_color::SpecifiedValue {
                                authored: Some("blue".to_owned()),
                                parsed: cssparser::Color::RGBA(cssparser::RGBA {
                                    red: 0., green: 0., blue: 1., alpha: 1.
                                }),
                            }
                        )),
                    ]),
                    important: Arc::new(vec![]),
                },
            }),
            CSSRule::Keyframes(KeyframesRule {
                name: "foo".into(),
                keyframes: vec![
                    Keyframe {
                        selector: KeyframeSelector::new_for_unit_testing(
                                      vec![KeyframePercentage::new(0.)]),
                        declarations: Arc::new(vec![
                            PropertyDeclaration::Width(DeclaredValue::Value(
                                LengthOrPercentageOrAuto::Percentage(Percentage(0.)))),
                        ]),
                    },
                    Keyframe {
                        selector: KeyframeSelector::new_for_unit_testing(
                                      vec![KeyframePercentage::new(1.)]),
                        declarations: Arc::new(vec![
                            PropertyDeclaration::Width(DeclaredValue::Value(
                                LengthOrPercentageOrAuto::Percentage(Percentage(1.)))),
                        ]),
                    },
                ]
            })

        ],
    });
}

struct CSSError {
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
    fn report_error(&self, input: &mut Parser, position: SourcePosition, message: &str) {
        let location = input.source_location(position);

        let errors = self.errors.clone();
        let mut errors = errors.lock().unwrap();

        errors.push(
            CSSError{
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
    let url = Url::parse("about::test").unwrap();
    let error_reporter = Box::new(CSSInvalidErrorReporterTest::new());

    let errors = error_reporter.errors.clone();

    Stylesheet::from_str(css, url, Origin::UserAgent, error_reporter,
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
}
