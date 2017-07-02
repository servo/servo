/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, SourcePosition};
use euclid::TypedSize2D;
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use style::Atom;
use style::context::QuirksMode;
use style::error_reporting::{ParseErrorReporter, ContextualParseError};
use style::media_queries::*;
use style::servo::media_queries::*;
use style::shared_lock::SharedRwLock;
use style::stylearc::Arc;
use style::stylesheets::{AllRules, Stylesheet, StylesheetInDocument, Origin, CssRule};
use style::values::specified;
use style_traits::ToCss;

pub struct CSSErrorReporterTest;

impl ParseErrorReporter for CSSErrorReporterTest {
    fn report_error<'a>(&self,
                        _input: &mut Parser,
                        _position: SourcePosition,
                        _error: ContextualParseError<'a>,
                        _url: &ServoUrl,
                        _line_number_offset: u64) {
    }
}

fn test_media_rule<F>(css: &str, callback: F)
    where F: Fn(&MediaList, &str),
{
    let url = ServoUrl::parse("http://localhost").unwrap();
    let css_str = css.to_owned();
    let lock = SharedRwLock::new();
    let media_list = Arc::new(lock.wrap(MediaList::empty()));
    let stylesheet = Stylesheet::from_str(
        css, url, Origin::Author, media_list, lock,
        None, &CSSErrorReporterTest, QuirksMode::NoQuirks, 0u64);
    let dummy = Device::new(MediaType::Screen, TypedSize2D::new(200.0, 100.0));
    let mut rule_count = 0;
    let guard = stylesheet.shared_lock.read();
    for rule in stylesheet.iter_rules::<AllRules>(&dummy, &guard) {
        if let CssRule::Media(ref lock) = *rule {
            rule_count += 1;
            callback(&lock.read_with(&guard).media_queries.read_with(&guard), css);
        }
    }
    assert!(rule_count > 0, css_str);
}

fn media_query_test(device: &Device, css: &str, expected_rule_count: usize) {
    let url = ServoUrl::parse("http://localhost").unwrap();
    let lock = SharedRwLock::new();
    let media_list = Arc::new(lock.wrap(MediaList::empty()));
    let ss = Stylesheet::from_str(
        css, url, Origin::Author, media_list, lock,
        None, &CSSErrorReporterTest, QuirksMode::NoQuirks, 0u64);
    let mut rule_count = 0;
    ss.effective_style_rules(device, &ss.shared_lock.read(), |_| rule_count += 1);
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
        assert!(q.media_type == MediaQueryType::Known(MediaType::Screen), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media only screen { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Only), css.to_owned());
        assert!(q.media_type == MediaQueryType::Known(MediaType::Screen), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media not screen { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::Known(MediaType::Screen), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });
}

#[test]
fn test_mq_print() {
    test_media_rule("@media print { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::Known(MediaType::Print), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media only print { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Only), css.to_owned());
        assert!(q.media_type == MediaQueryType::Known(MediaType::Print), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media not print { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::Known(MediaType::Print), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });
}

#[test]
fn test_mq_unknown() {
    test_media_rule("@media fridge { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::Unknown(Atom::from("fridge")), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media only glass { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Only), css.to_owned());
        assert!(q.media_type == MediaQueryType::Unknown(Atom::from("glass")), css.to_owned());
        assert!(q.expressions.len() == 0, css.to_owned());
    });

    test_media_rule("@media not wood { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::Unknown(Atom::from("wood")), css.to_owned());
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
        assert!(q0.media_type == MediaQueryType::Known(MediaType::Screen), css.to_owned());
        assert!(q0.expressions.len() == 0, css.to_owned());

        let q1 = &list.media_queries[1];
        assert!(q1.qualifier == None, css.to_owned());
        assert!(q1.media_type == MediaQueryType::Known(MediaType::Print), css.to_owned());
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
        match *q.expressions[0].kind_for_testing() {
            ExpressionKind::Width(Range::Min(ref w)) => assert!(*w == specified::Length::from_px(100.)),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media (max-width: 43px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::All, css.to_owned());
        assert!(q.expressions.len() == 1, css.to_owned());
        match *q.expressions[0].kind_for_testing() {
            ExpressionKind::Width(Range::Max(ref w)) => assert!(*w == specified::Length::from_px(43.)),
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
        assert!(q.media_type == MediaQueryType::Known(MediaType::Screen), css.to_owned());
        assert!(q.expressions.len() == 1, css.to_owned());
        match *q.expressions[0].kind_for_testing() {
            ExpressionKind::Width(Range::Min(ref w)) => assert!(*w == specified::Length::from_px(100.)),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media print and (max-width: 43px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::Known(MediaType::Print), css.to_owned());
        assert!(q.expressions.len() == 1, css.to_owned());
        match *q.expressions[0].kind_for_testing() {
            ExpressionKind::Width(Range::Max(ref w)) => assert!(*w == specified::Length::from_px(43.)),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media print and (width: 43px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::Known(MediaType::Print), css.to_owned());
        assert!(q.expressions.len() == 1, css.to_owned());
        match *q.expressions[0].kind_for_testing() {
            ExpressionKind::Width(Range::Eq(ref w)) => assert!(*w == specified::Length::from_px(43.)),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media fridge and (max-width: 52px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == MediaQueryType::Unknown(Atom::from("fridge")), css.to_owned());
        assert!(q.expressions.len() == 1, css.to_owned());
        match *q.expressions[0].kind_for_testing() {
            ExpressionKind::Width(Range::Max(ref w)) => assert!(*w == specified::Length::from_px(52.)),
            _ => panic!("wrong expression type"),
        }
    });
}

#[test]
fn test_to_css() {
    test_media_rule("@media print and (width: 43px) { }", |list, _| {
        let q = &list.media_queries[0];
        let mut dest = String::new();
        assert_eq!(Ok(()), q.to_css(&mut dest));
        assert_eq!(dest, "print and (width: 43px)");
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
        match *q.expressions[0].kind_for_testing() {
            ExpressionKind::Width(Range::Min(ref w)) => assert!(*w == specified::Length::from_px(100.)),
            _ => panic!("wrong expression type"),
        }
        match *q.expressions[1].kind_for_testing() {
            ExpressionKind::Width(Range::Max(ref w)) => assert!(*w == specified::Length::from_px(200.)),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media not screen and (min-width: 100px) and (max-width: 200px) { }", |list, css| {
        assert!(list.media_queries.len() == 1, css.to_owned());
        let q = &list.media_queries[0];
        assert!(q.qualifier == Some(Qualifier::Not), css.to_owned());
        assert!(q.media_type == MediaQueryType::Known(MediaType::Screen), css.to_owned());
        assert!(q.expressions.len() == 2, css.to_owned());
        match *q.expressions[0].kind_for_testing() {
            ExpressionKind::Width(Range::Min(ref w)) => assert!(*w == specified::Length::from_px(100.)),
            _ => panic!("wrong expression type"),
        }
        match *q.expressions[1].kind_for_testing() {
            ExpressionKind::Width(Range::Max(ref w)) => assert!(*w == specified::Length::from_px(200.)),
            _ => panic!("wrong expression type"),
        }
    });
}

#[test]
fn test_mq_malformed_expressions() {
    fn check_malformed_expr(list: &MediaList, css: &str) {
        assert!(!list.media_queries.is_empty(), css.to_owned());
        for mq in &list.media_queries {
            assert!(mq.qualifier == Some(Qualifier::Not), css.to_owned());
            assert!(mq.media_type == MediaQueryType::All, css.to_owned());
            assert!(mq.expressions.is_empty(), css.to_owned());
        }
    }

    for rule in &[
        "@media (min-width: 100blah) and (max-width: 200px) { }",
        "@media screen and (height: 200px) { }",
        "@media (min-width: 30em foo bar) {}",
        "@media not {}",
        "@media not (min-width: 300px) {}",
        "@media , {}",
    ] {
        test_media_rule(rule, check_malformed_expr);
    }
}

#[test]
fn test_matching_simple() {
    let device = Device::new(MediaType::Screen, TypedSize2D::new(200.0, 100.0));

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
    let device = Device::new(MediaType::Screen, TypedSize2D::new(200.0, 100.0));

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
    let device = Device::new(MediaType::Screen, TypedSize2D::new(200.0, 100.0));

    media_query_test(&device, "@media fridge { a { color: red; } }", 0);
    media_query_test(&device, "@media screen and (height: 100px) { a { color: red; } }", 0);
    media_query_test(&device, "@media not print and (width: 100) { a { color: red; } }", 0);
}
