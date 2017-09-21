/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{self, SourceLocation};
use html5ever::{Namespace as NsAtom};
use media_queries::CSSErrorReporterTest;
use parking_lot::RwLock;
use selectors::attr::*;
use selectors::parser::*;
use servo_arc::Arc;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use style::context::QuirksMode;
use style::error_reporting::{ParseErrorReporter, ContextualParseError};
use style::media_queries::MediaList;
use style::properties::Importance;
use style::properties::{CSSWideKeyword, DeclaredValueOwned, PropertyDeclaration, PropertyDeclarationBlock};
use style::properties::longhands;
use style::properties::longhands::animation_timing_function;
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
                animation-timing-function: ease; /* … except animation-timing-function */
            }
        }";
    let url = ServoUrl::parse("about::test").unwrap();
    let lock = SharedRwLock::new();
    let media = Arc::new(lock.wrap(MediaList::empty()));
    let stylesheet = Stylesheet::from_str(css, url.clone(), Origin::UserAgent, media, lock,
                                          None, &CSSErrorReporterTest, QuirksMode::NoQuirks, 0);
    let mut namespaces = Namespaces::default();
    namespaces.default = Some((ns!(html), ()));
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
                        (PropertyDeclaration::Display(longhands::display::SpecifiedValue::none),
                         Importance::Important),
                        (PropertyDeclaration::Custom(Atom::from("a"),
                         DeclaredValueOwned::CSSWideKeyword(CSSWideKeyword::Inherit)),
                         Importance::Important),
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
                        (PropertyDeclaration::Display(longhands::display::SpecifiedValue::block),
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

struct CSSError {
    pub url : ServoUrl,
    pub line: u32,
    pub column: u32,
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
    fn report_error(&self,
                    url: &ServoUrl,
                    location: SourceLocation,
                    error: ContextualParseError) {
        let mut errors = self.errors.lock().unwrap();
        errors.push(
            CSSError{
                url: url.clone(),
                line: location.line,
                column: location.column,
                message: error.to_string(),
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
    let error_reporter = CSSInvalidErrorReporterTest::new();

    let errors = error_reporter.errors.clone();

    let lock = SharedRwLock::new();
    let media = Arc::new(lock.wrap(MediaList::empty()));
    Stylesheet::from_str(css, url.clone(), Origin::UserAgent, media, lock,
                         None, &error_reporter, QuirksMode::NoQuirks, 5);

    let mut errors = errors.lock().unwrap();

    let error = errors.pop().unwrap();
    assert_eq!("Unsupported property declaration: 'invalid: true;', \
                Custom(PropertyDeclaration(UnknownProperty(\"invalid\")))", error.message);
    assert_eq!(9, error.line);
    assert_eq!(9, error.column);

    let error = errors.pop().unwrap();
    assert_eq!("Unsupported property declaration: 'display: invalid;', \
                Custom(PropertyDeclaration(InvalidValue(\"display\", None)))", error.message);
    assert_eq!(8, error.line);
    assert_eq!(9, error.column);

    // testing for the url
    assert_eq!(url, error.url);
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
    let error_reporter = CSSInvalidErrorReporterTest::new();

    let errors = error_reporter.errors.clone();

    let lock = SharedRwLock::new();
    let media = Arc::new(lock.wrap(MediaList::empty()));
    Stylesheet::from_str(css, url, Origin::UserAgent, media, lock,
                         None, &error_reporter, QuirksMode::NoQuirks, 0);

    let mut errors = errors.lock().unwrap();
    let error = errors.pop().unwrap();
    assert_eq!("Unsupported property declaration: '-moz-background-color: red;', \
                Custom(PropertyDeclaration(UnknownProperty(\"-moz-background-color\")))",
               error.message);
    assert!(errors.is_empty());
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
                                              None, &CSSErrorReporterTest, QuirksMode::NoQuirks,
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
                                              None, &CSSErrorReporterTest, QuirksMode::NoQuirks,
                                              0);
        let url_opt = stylesheet.contents.source_url.read();
        assert_eq!(*url_opt, test.1);
    }
}
