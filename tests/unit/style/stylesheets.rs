/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser;
use selectors::parser::*;
use std::borrow::ToOwned;
use std::sync::Arc;
use string_cache::Atom;
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock, DeclaredValue, longhands};
use style::stylesheets::{CSSRule, StyleRule, Origin, Stylesheet};
use url::Url;


#[test]
fn test_parse_stylesheet() {
    let css = r"
        @namespace url(http://www.w3.org/1999/xhtml);
        /* FIXME: only if scripting is enabled */
        input[type=hidden i] { display: none !important; }
        html , body /**/ { display: block; }
        #d1 > .ok { background: blue; }
    ";
    let url = Url::parse("about::test").unwrap();
    let stylesheet = Stylesheet::from_str(css, url, Origin::UserAgent);
    assert_eq!(stylesheet, Stylesheet {
        origin: Origin::UserAgent,
        rules: vec![
            CSSRule::Namespace(None, ns!(HTML)),
            CSSRule::Style(StyleRule {
                selectors: vec![
                    Selector {
                        compound_selectors: Arc::new(CompoundSelector {
                            simple_selectors: vec![
                                SimpleSelector::Namespace(ns!(HTML)),
                                SimpleSelector::LocalName(LocalName {
                                    name: atom!(input),
                                    lower_name: atom!(input),
                                }),
                                SimpleSelector::AttrEqual(AttrSelector {
                                    name: atom!(type),
                                    lower_name: atom!(type),
                                    namespace: NamespaceConstraint::Specific(ns!("")),
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
                        PropertyDeclaration::Display(DeclaredValue::SpecifiedValue(
                            longhands::display::SpecifiedValue::none)),
                    ]),
                },
            }),
            CSSRule::Style(StyleRule {
                selectors: vec![
                    Selector {
                        compound_selectors: Arc::new(CompoundSelector {
                            simple_selectors: vec![
                                SimpleSelector::Namespace(ns!(HTML)),
                                SimpleSelector::LocalName(LocalName {
                                    name: atom!(html),
                                    lower_name: atom!(html),
                                }),
                            ],
                            next: None,
                        }),
                        pseudo_element: None,
                        specificity: (0 << 20) + (0 << 10) + (1 << 0),
                    },
                    Selector {
                        compound_selectors: Arc::new(CompoundSelector {
                            simple_selectors: vec![
                                SimpleSelector::Namespace(ns!(HTML)),
                                SimpleSelector::LocalName(LocalName {
                                    name: atom!(body),
                                    lower_name: atom!(body),
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
                        PropertyDeclaration::Display(DeclaredValue::SpecifiedValue(
                            longhands::display::SpecifiedValue::block)),
                    ]),
                    important: Arc::new(vec![]),
                },
            }),
            CSSRule::Style(StyleRule {
                selectors: vec![
                    Selector {
                        compound_selectors: Arc::new(CompoundSelector {
                            simple_selectors: vec![
                                SimpleSelector::Class(Atom::from_slice("ok")),
                            ],
                            next: Some((Box::new(CompoundSelector {
                                simple_selectors: vec![
                                    SimpleSelector::ID(Atom::from_slice("d1")),
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
                        PropertyDeclaration::BackgroundColor(DeclaredValue::SpecifiedValue(
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
        ],
    });
}
