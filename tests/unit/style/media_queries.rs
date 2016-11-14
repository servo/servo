/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::{Parser, SourcePosition};
use euclid::size::TypedSize2D;
use std::borrow::ToOwned;
use style::Atom;
use style::error_reporting::ParseErrorReporter;
use style::media_queries::*;
use style::parser::ParserContextExtraData;
use style::stylesheets::{Stylesheet, Origin, CssRule};
use style::values::specified;
use url::Url;

pub struct CSSErrorReporterTest;

impl ParseErrorReporter for CSSErrorReporterTest {
     fn report_error(&self, _input: &mut Parser, _position: SourcePosition, _message: &str) {
     }
     fn clone(&self) -> Box<ParseErrorReporter + Send + Sync> {
        Box::new(CSSErrorReporterTest)
     }
}

fn test_media_rule<F>(css: &str, callback: F) where F: Fn(&MediaList, &str) {
    let url = Url::parse("http://localhost").unwrap();
    let stylesheet = Stylesheet::from_str(css, url, Origin::Author, Box::new(CSSErrorReporterTest),
                                          ParserContextExtraData::default());
    let mut rule_count = 0;
    media_queries(&stylesheet.rules, &mut |mq| {
        rule_count += 1;
        callback(mq, css);
    });
    assert!(rule_count > 0);
}

fn media_queries<F>(rules: &[CssRule], f: &mut F) where F: FnMut(&MediaList) {
    for rule in rules {
        rule.with_nested_rules_and_mq(|rules, mq| {
            if let Some(mq) = mq {
                f(mq)
            }
            media_queries(rules, f)
        })
    }
}

fn media_query_test(device: &Device, css: &str, expected_rule_count: usize) {
    let url = Url::parse("http://localhost").unwrap();
    let ss = Stylesheet::from_str(css, url, Origin::Author, Box::new(CSSErrorReporterTest),
                                  ParserContextExtraData::default());
    let mut rule_count = 0;
    ss.effective_style_rules(device, |_| rule_count += 1);
    assert!(rule_count == expected_rule_count, css.to_owned());
}

#[test]
fn test_mq_empty() {
    test_media_rule("@media { }", |list, css| {
        assert!(list.media_queries.len() == 0, css.to_owned());
    });
}

#[test]
fn test_mq_screen() {
    test_media_rule("@media screen { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media only screen { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Only), css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media not screen { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });
}

#[test]
fn test_mq_print() {
    test_media_rule("@media print { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media only print { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Only), css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media not print { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });
}

#[test]
fn test_mq_unknown() {
    test_media_rule("@media fridge { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Unknown(Atom::from("fridge"))), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media only glass { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Only), css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Unknown(Atom::from("glass"))), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media not wood { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Unknown(Atom::from("wood"))), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });
}

#[test]
fn test_mq_all() {
    test_media_rule("@media all { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media only all { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Only), css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media not all { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });
}

#[test]
fn test_mq_or() {
    test_media_rule("@media screen, print { }", |list, css| {
        assert!(list.media_queries.len() == 2, css.to_owned());
        let q0 = &list.media_queries[0];
        assert!(q0.qualifier == None, css.to_owned());
        assert!(q0.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
        assert!(q0.expressions.len() == 0, css.to_owned());

        let q1 = &list.media_queries[1];
        assert!(q1.qualifier == None, css.to_owned());
        assert!(q1.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
        assert!(q1.expressions.len() == 0, css.to_owned());
    });
}

#[test]
fn test_mq_default_expressions() {
    test_media_rule("@media (min-width: 100px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 1, css.to_owned());
        match q.expressions[0] {
            Expression::Width(Range::Min(w)) => assert!(w == specified::Length::Absolute(Au::from_px(100))),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media (max-width: 43px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 1, css.to_owned());
        match q.expressions[0] {
            Expression::Width(Range::Max(w)) => assert!(w == specified::Length::Absolute(Au::from_px(43))),
            _ => panic!("wrong expression type"),
        }
    });
}

#[test]
fn test_mq_expressions() {
    test_media_rule("@media screen and (min-width: 100px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
        assert!(q.expressions.len() == 1, css.to_owned());
        match q.expressions[0] {
            Expression::Width(Range::Min(w)) => assert!(w == specified::Length::Absolute(Au::from_px(100))),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media print and (max-width: 43px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
        assert!(q.expressions.len() == 1, css.to_owned());
        match q.expressions[0] {
            Expression::Width(Range::Max(w)) => assert!(w == specified::Length::Absolute(Au::from_px(43))),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media fridge and (max-width: 52px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Unknown(Atom::from("fridge"))), css.to_owned());
        assert!(q.expressions.len() == 1, css.to_owned());
        match q.expressions[0] {
            Expression::Width(Range::Max(w)) => assert!(w == specified::Length::Absolute(Au::from_px(52))),
            _ => panic!("wrong expression type"),
        }
    });
}

#[test]
fn test_mq_multiple_expressions() {
    test_media_rule("@media (min-width: 100px) and (max-width: 200px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 2, css.to_owned());
        match q.expressions[0] {
            Expression::Width(Range::Min(w)) => assert!(w == specified::Length::Absolute(Au::from_px(100))),
            _ => panic!("wrong expression type"),
        }
        match q.expressions[1] {
            Expression::Width(Range::Max(w)) => assert!(w == specified::Length::Absolute(Au::from_px(200))),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media not screen and (min-width: 100px) and (max-width: 200px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
        assert!(q.expressions.len() == 2, css.to_owned());
        match q.expressions[0] {
            Expression::Width(Range::Min(w)) => assert!(w == specified::Length::Absolute(Au::from_px(100))),
            _ => panic!("wrong expression type"),
        }
        match q.expressions[1] {
            Expression::Width(Range::Max(w)) => assert!(w == specified::Length::Absolute(Au::from_px(200))),
            _ => panic!("wrong expression type"),
        }
    });
}

#[test]
fn test_mq_malformed_expressions() {
    test_media_rule("@media (min-width: 100blah) and (max-width: 200px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media screen and (height: 200px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media (min-width: 30em foo bar) {}", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media not {}", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media not (min-width: 300px) {}", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media , {}", |list, css| {
        assert!(list.media_queries.len() == 2, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
        let q = &list.media_queries[1];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media screen 4px, print {}", |list, css| {
        assert!(list.media_queries.len() == 2, css.to_owned());
        let q0 = &list.media_queries[0];
        assert!(q0.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q0.media_type == MediaQueryType::All, css.to_owned());
        assert!(q0.expressions.len() == 0, css.to_owned());
        let q1 = &list.media_queries[1];
        assert!(q1.qualifier == None, css.to_owned());
        assert!(q1.media_type == MediaQueryType::MediaType(MediaType::Print), css.to_owned());
        assert!(q1.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media screen, {}", |list, css| {
        assert!(list.media_queries.len() == 2, css.to_owned());
        let q0 = &list.media_queries[0];
        assert!(q0.qualifier == None, css.to_owned());
        assert!(q0.media_type == MediaQueryType::MediaType(MediaType::Screen), css.to_owned());
        assert!(q0.expressions.len() == 0, css.to_owned());
        let q1 = &list.media_queries[1];
        assert!(q1.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q1.media_type == MediaQueryType::All, css.to_owned());
        assert!(q1.expressions.len() == 0, css.to_owned());
    });
}

#[test]
fn test_matching_simple() {
    let device = Device {
        media_type: MediaType::Screen,
        viewport_size: TypedSize2D::new(200.0, 100.0),
    };

    media_query_test(&device, "@media not all { a { color: red; } }", 0);
    media_query_test(&device, "@media not screen { a { color: red; } }", 0);
    media_query_test(&device, "@media not print { a { color: red; } }", 1);

    media_query_test(&device, "@media unknown { a { color: red; } }", 0);
    media_query_test(&device, "@media not unknown { a { color: red; } }", 1);

    media_query_test(&device, "@media { a { color: red; } }", 1);
    media_query_test(&device, "@media screen { a { color: red; } }", 1);
    media_query_test(&device, "@media print { a { color: red; } }", 0);
}

#[test]
fn test_matching_width() {
    let device = Device {
        media_type: MediaType::Screen,
        viewport_size: TypedSize2D::new(200.0, 100.0),
    };

    media_query_test(&device, "@media { a { color: red; } }", 1);

    media_query_test(&device, "@media (min-width: 50px) { a { color: red; } }", 1);
    media_query_test(&device, "@media (min-width: 150px) { a { color: red; } }", 1);
    media_query_test(&device, "@media (min-width: 300px) { a { color: red; } }", 0);

    media_query_test(&device, "@media screen and (min-width: 50px) { a { color: red; } }", 1);
    media_query_test(&device, "@media screen and (min-width: 150px) { a { color: red; } }", 1);
    media_query_test(&device, "@media screen and (min-width: 300px) { a { color: red; } }", 0);

    media_query_test(&device, "@media not screen and (min-width: 50px) { a { color: red; } }", 0);
    media_query_test(&device, "@media not screen and (min-width: 150px) { a { color: red; } }", 0);
    media_query_test(&device, "@media not screen and (min-width: 300px) { a { color: red; } }", 1);

    media_query_test(&device, "@media (max-width: 50px) { a { color: red; } }", 0);
    media_query_test(&device, "@media (max-width: 150px) { a { color: red; } }", 0);
    media_query_test(&device, "@media (max-width: 300px) { a { color: red; } }", 1);

    media_query_test(&device, "@media screen and (min-width: 50px) and (max-width: 100px) { a { color: red; } }", 0);
    media_query_test(&device, "@media screen and (min-width: 250px) and (max-width: 300px) { a { color: red; } }", 0);
    media_query_test(&device, "@media screen and (min-width: 50px) and (max-width: 250px) { a { color: red; } }", 1);

    media_query_test(
        &device, "@media not screen and (min-width: 50px) and (max-width: 100px) { a { color: red; } }", 1);
    media_query_test(
        &device, "@media not screen and (min-width: 250px) and (max-width: 300px) { a { color: red; } }", 1);
    media_query_test(
        &device, "@media not screen and (min-width: 50px) and (max-width: 250px) { a { color: red; } }", 0);

    media_query_test(
        &device, "@media not screen and (min-width: 3.1em) and (max-width: 6em) { a { color: red; } }", 1);
    media_query_test(
        &device, "@media not screen and (min-width: 16em) and (max-width: 19.75em) { a { color: red; } }", 1);
    media_query_test(
        &device, "@media not screen and (min-width: 3em) and (max-width: 250px) { a { color: red; } }", 0);
}

#[test]
fn test_matching_invalid() {
    let device = Device {
        media_type: MediaType::Screen,
        viewport_size: TypedSize2D::new(200.0, 100.0),
    };

    media_query_test(&device, "@media fridge { a { color: red; } }", 0);
    media_query_test(&device, "@media screen and (height: 100px) { a { color: red; } }", 0);
    media_query_test(&device, "@media not print and (width: 100) { a { color: red; } }", 0);
}
