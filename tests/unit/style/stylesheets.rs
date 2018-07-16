/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{self, SourceLocation};
use html5ever::{Namespace as NsAtom};
use parking_lot::RwLock;
use selectors::attr::*;
use selectors::parser::*;
use servo_arc::Arc;
use servo_atoms::Atom;
use servo_config::prefs::{PREFS, PrefValue};
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::cell::RefCell;
use std::sync::atomic::AtomicBool;
use style::context::QuirksMode;
use style::error_reporting::{ParseErrorReporter, ContextualParseError};
use style::media_queries::MediaList;
use style::properties::{CSSWideKeyword, CustomDeclaration, DeclarationPushMode};
use style::properties::{DeclaredValueOwned, Importance};
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock};
use style::properties::longhands::{self, animation_timing_function};
use style::shared_lock::SharedRwLock;
use style::stylesheets::{Origin, Namespaces};
use style::stylesheets::{Stylesheet, StylesheetContents, NamespaceRule, CssRule, CssRules, StyleRule, KeyframesRule};
use style::stylesheets::keyframes_rule::{Keyframe, KeyframeSelector, KeyframePercentage};
use style::values::{KeyframesName, CustomIdent};
use style::values::computed::Percentage;
use style::values::specified::{LengthOrPercentageOrAuto, PositionComponent};
use style::values::specified::transform::TimingFunction;

pub fn block_from<I>(iterable: I) -> PropertyDeclarationBlock
where I: IntoIterator<Item=(PropertyDeclaration, Importance)> {
    let mut block = PropertyDeclarationBlock::new();
    for (d, i) in iterable {
        block.push(d, i, DeclarationPushMode::Append);
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
                animation-timing-function: ease; /* … except animation-timing-function */
            }
        }";
    let url = ServoUrl::parse("about::test").unwrap();
    let lock = SharedRwLock::new();
    let media = Arc::new(lock.wrap(MediaList::empty()));
    let stylesheet = Stylesheet::from_str(css, url.clone(), Origin::UserAgent, media, lock,
                                          None, None, QuirksMode::NoQuirks, 0);
    let mut namespaces = Namespaces::default();
    namespaces.default = Some(ns!(html));
    let expected = Stylesheet {
        contents: StylesheetContents {
            origin: Origin::UserAgent,
            namespaces: RwLock::new(namespaces),
            url_data: RwLock::new(url),
            quirks_mode: QuirksMode::NoQuirks,
            rules: CssRules::new(vec![
                CssRule::Namespace(Arc::new(stylesheet.shared_lock.wrap(NamespaceRule {
                    prefix: None,
                    url: NsAtom::from("http://www.w3.org/1999/xhtml"),
                    source_location: SourceLocation {
                        line: 1,
                        column: 19,
                    },
                }))),
                CssRule::Style(Arc::new(stylesheet.shared_lock.wrap(StyleRule {
                    selectors: SelectorList::from_vec(vec!(
                        Selector::from_vec(vec!(
                            Component::DefaultNamespace(NsAtom::from("http://www.w3.org/1999/xhtml")),
                            Component::LocalName(LocalName {
                                name: local_name!("input"),
                                lower_name: local_name!("input"),
                            }),
                            Component::AttributeInNoNamespace {
                                local_name: local_name!("type"),
                                local_name_lower: local_name!("type"),
                                operator: AttrSelectorOperator::Equal,
                                value: "hidden".to_owned(),
                                case_sensitivity: ParsedCaseSensitivity::AsciiCaseInsensitive,
                                never_matches: false,
                            }
                        ), (0 << 20) + (1 << 10) + (1 << 0))
                    )),
                    block: Arc::new(stylesheet.shared_lock.wrap(block_from(vec![
                        (
                            PropertyDeclaration::Display(longhands::display::SpecifiedValue::None),
                            Importance::Important,
                        ),
                        (
                            PropertyDeclaration::Custom(CustomDeclaration {
                                name: Atom::from("a"),
                                value: DeclaredValueOwned::CSSWideKeyword(CSSWideKeyword::Inherit),
                            }),
                            Importance::Important,
                        ),
                    ]))),
                    source_location: SourceLocation {
                        line: 3,
                        column: 9,
                    },
                }))),
                CssRule::Style(Arc::new(stylesheet.shared_lock.wrap(StyleRule {
                    selectors: SelectorList::from_vec(vec!(
                        Selector::from_vec(vec!(
                                Component::DefaultNamespace(NsAtom::from("http://www.w3.org/1999/xhtml")),
                                Component::LocalName(LocalName {
                                    name: local_name!("html"),
                                    lower_name: local_name!("html"),
                                }),
                            ), (0 << 20) + (0 << 10) + (1 << 0)),
                        Selector::from_vec(vec!(
                            Component::DefaultNamespace(NsAtom::from("http://www.w3.org/1999/xhtml")),
                            Component::LocalName(LocalName {
                                name: local_name!("body"),
                                lower_name: local_name!("body"),
                            })
                            ), (0 << 20) + (0 << 10) + (1 << 0)
                        ),
                    )),
                    block: Arc::new(stylesheet.shared_lock.wrap(block_from(vec![
                        (PropertyDeclaration::Display(longhands::display::SpecifiedValue::Block),
                         Importance::Normal),
                    ]))),
                    source_location: SourceLocation {
                        line: 11,
                        column: 9,
                    },
                }))),
                CssRule::Style(Arc::new(stylesheet.shared_lock.wrap(StyleRule {
                    selectors: SelectorList::from_vec(vec!(
                        Selector::from_vec(vec!(
                            Component::DefaultNamespace(NsAtom::from("http://www.w3.org/1999/xhtml")),
                            Component::ID(Atom::from("d1")),
                            Component::Combinator(Combinator::Child),
                            Component::DefaultNamespace(NsAtom::from("http://www.w3.org/1999/xhtml")),
                            Component::Class(Atom::from("ok"))
                        ), (1 << 20) + (1 << 10) + (0 << 0))
                    )),
                    block: Arc::new(stylesheet.shared_lock.wrap(block_from(vec![
                        (PropertyDeclaration::BackgroundColor(
                            longhands::background_color::SpecifiedValue::Numeric {
                                authored: Some("blue".to_owned().into_boxed_str()),
                                parsed: cssparser::RGBA::new(0, 0, 255, 255),
                            }
                         ),
                         Importance::Normal),
                        (PropertyDeclaration::BackgroundPositionX(
                            longhands::background_position_x::SpecifiedValue(
                            vec![PositionComponent::zero()])),
                         Importance::Normal),
                        (PropertyDeclaration::BackgroundPositionY(
                            longhands::background_position_y::SpecifiedValue(
                            vec![PositionComponent::zero()])),
                         Importance::Normal),
                        (PropertyDeclaration::BackgroundRepeat(
                            longhands::background_repeat::SpecifiedValue(
                            vec![longhands::background_repeat::single_value
                                                       ::get_initial_specified_value()])),
                         Importance::Normal),
                        (PropertyDeclaration::BackgroundAttachment(
                            longhands::background_attachment::SpecifiedValue(
                            vec![longhands::background_attachment::single_value
                                                       ::get_initial_specified_value()])),
                         Importance::Normal),
                        (PropertyDeclaration::BackgroundImage(
                            longhands::background_image::SpecifiedValue(
                            vec![longhands::background_image::single_value
                                                       ::get_initial_specified_value()])),
                         Importance::Normal),
                        (PropertyDeclaration::BackgroundSize(
                            longhands::background_size::SpecifiedValue(
                            vec![longhands::background_size::single_value
                                                       ::get_initial_specified_value()])),
                         Importance::Normal),
                        (PropertyDeclaration::BackgroundOrigin(
                            longhands::background_origin::SpecifiedValue(
                            vec![longhands::background_origin::single_value
                                                       ::get_initial_specified_value()])),
                         Importance::Normal),
                        (PropertyDeclaration::BackgroundClip(
                            longhands::background_clip::SpecifiedValue(
                            vec![longhands::background_clip::single_value
                                                       ::get_initial_specified_value()])),
                         Importance::Normal),
                    ]))),
                    source_location: SourceLocation {
                        line: 15,
                        column: 9,
                    },
                }))),
                CssRule::Keyframes(Arc::new(stylesheet.shared_lock.wrap(KeyframesRule {
                    name: KeyframesName::Ident(CustomIdent("foo".into())),
                    keyframes: vec![
                        Arc::new(stylesheet.shared_lock.wrap(Keyframe {
                            selector: KeyframeSelector::new_for_unit_testing(
                                          vec![KeyframePercentage::new(0.)]),
                            block: Arc::new(stylesheet.shared_lock.wrap(block_from(vec![
                                (PropertyDeclaration::Width(
                                    LengthOrPercentageOrAuto::Percentage(Percentage(0.))),
                                 Importance::Normal),
                            ]))),
                            source_location: SourceLocation {
                                line: 17,
                                column: 13,
                            },
                        })),
                        Arc::new(stylesheet.shared_lock.wrap(Keyframe {
                            selector: KeyframeSelector::new_for_unit_testing(
                                          vec![KeyframePercentage::new(1.)]),
                            block: Arc::new(stylesheet.shared_lock.wrap(block_from(vec![
                                (PropertyDeclaration::Width(
                                    LengthOrPercentageOrAuto::Percentage(Percentage(1.))),
                                 Importance::Normal),
                                (PropertyDeclaration::AnimationTimingFunction(
                                    animation_timing_function::SpecifiedValue(
                                        vec![TimingFunction::ease()])),
                                 Importance::Normal),
                            ]))),
                            source_location: SourceLocation {
                                line: 18,
                                column: 13,
                            },
                        })),
                    ],
                    vendor_prefix: None,
                    source_location: SourceLocation {
                        line: 16,
                        column: 19,
                    },
                })))
            ], &stylesheet.shared_lock),
            source_map_url: RwLock::new(None),
            source_url: RwLock::new(None),
        },
        media: Arc::new(stylesheet.shared_lock.wrap(MediaList::empty())),
        shared_lock: stylesheet.shared_lock.clone(),
        disabled: AtomicBool::new(false),
    };

    assert_eq!(format!("{:#?}", stylesheet), format!("{:#?}", expected));
}

#[derive(Debug)]
struct CSSError {
    pub url : ServoUrl,
    pub line: u32,
    pub column: u32,
    pub message: String
}

struct TestingErrorReporter {
    errors: RefCell<Vec<CSSError>>,
}

impl TestingErrorReporter {
    pub fn new() -> Self {
        TestingErrorReporter {
            errors: RefCell::new(Vec::new()),
        }
    }

    fn assert_messages_contain(&self, expected_errors: &[(u32, u32, &str)]) {
        let errors = self.errors.borrow();
        for (i, (error, &(line, column, message))) in errors.iter().zip(expected_errors).enumerate() {
            assert_eq!((error.line, error.column), (line, column),
                       "line/column numbers of the {}th error: {:?}", i + 1, error.message);
            assert!(error.message.contains(message),
                    "{:?} does not contain {:?}", error.message, message);
        }
        if errors.len() < expected_errors.len() {
            panic!("Missing errors: {:#?}", &expected_errors[errors.len()..]);
        }
        if errors.len() > expected_errors.len() {
            panic!("Extra errors: {:#?}", &errors[expected_errors.len()..]);
        }
    }
}

impl ParseErrorReporter for TestingErrorReporter {
    fn report_error(&self,
                    url: &ServoUrl,
                    location: SourceLocation,
                    error: ContextualParseError) {
        self.errors.borrow_mut().push(
            CSSError{
                url: url.clone(),
                line: location.line,
                column: location.column,
                message: error.to_string(),
            }
        )
    }
}


#[test]
fn test_report_error_stylesheet() {
    PREFS.set("layout.viewport.enabled", PrefValue::Boolean(true));
    let css = r"
    div {
        background-color: red;
        display: invalid;
        background-image: linear-gradient(0deg, black, invalid, transparent);
        invalid: true;
    }
    @media (min-width: 10px invalid 1000px) {}
    @font-face { src: url(), invalid, url(); }
    @counter-style foo { symbols: a 0invalid b }
    @font-feature-values Sans Sans { @foo {} @swash { foo: 1 invalid 2 } }
    @invalid;
    @media screen { @invalid; }
    @supports (color: green) and invalid and (margin: 0) {}
    @keyframes foo { from invalid {} to { margin: 0 invalid 0; } }
    @viewport { width: 320px invalid auto; }
    ";
    let url = ServoUrl::parse("about::test").unwrap();
    let error_reporter = TestingErrorReporter::new();

    let lock = SharedRwLock::new();
    let media = Arc::new(lock.wrap(MediaList::empty()));
    Stylesheet::from_str(css, url.clone(), Origin::UserAgent, media, lock,
                         None, Some(&error_reporter), QuirksMode::NoQuirks, 5);

    error_reporter.assert_messages_contain(&[
        (8, 18, "Unsupported property declaration: 'display: invalid;'"),
        (9, 27, "Unsupported property declaration: 'background-image:"),  // FIXME: column should be around 56
        (10, 17, "Unsupported property declaration: 'invalid: true;'"),
        (12, 28, "Invalid media rule"),
        (13, 30, "Unsupported @font-face descriptor declaration"),

        // When @counter-style is supported, this should be replaced with two errors
        (14, 19, "Invalid rule: '@counter-style "),

        // When @font-feature-values is supported, this should be replaced with two errors
        (15, 25, "Invalid rule: '@font-feature-values "),

        (16, 13, "Invalid rule: '@invalid'"),
        (17, 29, "Invalid rule: '@invalid'"),

        (18, 34, "Invalid rule: '@supports "),
        (19, 26, "Invalid keyframe rule: 'from invalid '"),
        (19, 52, "Unsupported keyframe property declaration: 'margin: 0 invalid 0;'"),
        (20, 29, "Unsupported @viewport descriptor declaration: 'width: 320px invalid auto;'"),
    ]);

    assert_eq!(error_reporter.errors.borrow()[0].url, url);
}

#[test]
fn test_no_report_unrecognized_vendor_properties() {
    let css = r"
    div {
        -o-background-color: red;
        _background-color: red;
        -moz-background-color: red;
    }
    ";
    let url = ServoUrl::parse("about::test").unwrap();
    let error_reporter = TestingErrorReporter::new();

    let lock = SharedRwLock::new();
    let media = Arc::new(lock.wrap(MediaList::empty()));
    Stylesheet::from_str(css, url, Origin::UserAgent, media, lock,
                         None, Some(&error_reporter), QuirksMode::NoQuirks, 0);

    error_reporter.assert_messages_contain(&[
        (4, 31, "Unsupported property declaration: '-moz-background-color: red;'"),
    ]);
}

#[test]
fn test_source_map_url() {
    let tests = vec![
        ("", None),
        ("/*# sourceMappingURL=something */", Some("something".to_string())),
    ];

    for test in tests {
        let url = ServoUrl::parse("about::test").unwrap();
        let lock = SharedRwLock::new();
        let media = Arc::new(lock.wrap(MediaList::empty()));
        let stylesheet = Stylesheet::from_str(test.0, url.clone(), Origin::UserAgent, media, lock,
                                              None, None, QuirksMode::NoQuirks,
                                              0);
        let url_opt = stylesheet.contents.source_map_url.read();
        assert_eq!(*url_opt, test.1);
    }
}

#[test]
fn test_source_url() {
    let tests = vec![
        ("", None),
        ("/*# sourceURL=something */", Some("something".to_string())),
    ];

    for test in tests {
        let url = ServoUrl::parse("about::test").unwrap();
        let lock = SharedRwLock::new();
        let media = Arc::new(lock.wrap(MediaList::empty()));
        let stylesheet = Stylesheet::from_str(test.0, url.clone(), Origin::UserAgent, media, lock,
                                              None, None, QuirksMode::NoQuirks,
                                              0);
        let url_opt = stylesheet.contents.source_url.read();
        assert_eq!(*url_opt, test.1);
    }
}
