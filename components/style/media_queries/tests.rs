/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg(test)]

use super::*;
use super::values::range::Width;
use super::condition::{MediaConditionTerm, MediaConnectiveTerm, MediaInParensTerm};
use super::query::Qualifier;

use ::geom::size::TypedSize2D;
use ::util::geometry::Au;
use ::stylesheets::{iter_stylesheet_media_rules, iter_stylesheet_style_rules, Stylesheet};
use ::stylesheets::Origin;
use url::Url;
use ::values::specified;
use std::borrow::ToOwned;

macro_rules! qualifier {
    ($qualifier:ident) => {
        Some(Qualifier::$qualifier)
    };
}

macro_rules! media_type {
    (All) => {
        super::query::MediaType::All
    };
    (Unknown($media_type:expr)) => {
        super::query::MediaType::Unknown($media_type.to_owned())
    };
    ($media_type:ident) => {
        super::query::MediaType::Defined(MediaType::$media_type)
    };
}

fn test_media_rule<F>(css: &str, callback: F) where F: Fn(&MediaQueryList, &str) {
    let url = Url::parse("http://localhost").unwrap();
    let stylesheet = Stylesheet::from_str(css, url, Origin::Author);
    let mut rule_count: int = 0;
    iter_stylesheet_media_rules(&stylesheet, |rule| {
        rule_count += 1;
        callback(&rule.media_queries, css);
    });
    assert!(rule_count > 0);
}

fn media_query_test(device: &Device, css: &str, expected_rule_count: int) {
    let url = Url::parse("http://localhost").unwrap();
    let ss = Stylesheet::from_str(css, url, Origin::Author);
    let mut rule_count: int = 0;
    iter_stylesheet_style_rules(&ss, device, |_| rule_count += 1);
    assert!(rule_count == expected_rule_count, css.to_owned());
}

#[test]
fn test_mq_empty() {
    test_media_rule("@media { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });
}

#[test]
fn test_mq_screen() {
    test_media_rule("@media screen { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(Screen), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });

    test_media_rule("@media only screen { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Only), css.to_owned());
        assert!(q.media_type == media_type!(Screen), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });

    test_media_rule("@media not screen { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(Screen), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });
}

#[test]
fn test_mq_print() {
    test_media_rule("@media print { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(Print), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });

    test_media_rule("@media only print { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Only), css.to_owned());
        assert!(q.media_type == media_type!(Print), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });

    test_media_rule("@media not print { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(Print), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });
}

#[test]
fn test_mq_unknown() {
    test_media_rule("@media fridge { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(Unknown("fridge")), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });

    test_media_rule("@media only glass { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Only), css.to_owned());
        assert!(q.media_type == media_type!(Unknown("glass")), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });

    test_media_rule("@media not wood { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(Unknown("wood")), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });
}

#[test]
fn test_mq_all() {
    test_media_rule("@media all { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });

    test_media_rule("@media only all { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Only), css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });

    test_media_rule("@media not all { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });
}

#[test]
fn test_mq_or() {
    test_media_rule("@media screen, print { }", |list, css| {
        assert!(list.queries.len() == 2, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(Screen), css.to_owned());
        assert!(q.condition == None, css.to_owned());;

        let q = &list.queries[1];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(Print), css.to_owned());
        assert!(q.condition == None, css.to_owned());
    });
}

#[test]
fn test_mq_default_expressions() {
    test_media_rule("@media (min-width: 100px) { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition.is_some(), css.to_owned());
        match q.condition.as_ref().unwrap().0 {
            MediaConditionTerm::InParens(
                MediaInParensTerm::Feature(
                    MediaFeature::Width(
                        Some(Width(Range::Ge(w)))))) => assert!(w == specified::Length::Absolute(Au::from_px(100))),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media (max-width: 43px) { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition.is_some(), css.to_owned());
        match q.condition.as_ref().unwrap().0 {
            MediaConditionTerm::InParens(
                MediaInParensTerm::Feature(
                    MediaFeature::Width(
                        Some(Width(Range::Le(w)))))) => assert!(w == specified::Length::Absolute(Au::from_px(43))),
            _ => panic!("wrong expression type"),
        }
    });
}

#[test]
fn test_mq_expressions() {
    test_media_rule("@media screen and (min-width: 100px) { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(Screen), css.to_owned());
        assert!(q.condition.is_some(), css.to_owned());
        match q.condition.as_ref().unwrap().0 {
            MediaConditionTerm::InParens(
                MediaInParensTerm::Feature(
                    MediaFeature::Width(
                        Some(Width(Range::Ge(w)))))) => assert!(w == specified::Length::Absolute(Au::from_px(100))),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media print and (max-width: 43px) { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(Print), css.to_owned());
        assert!(q.condition.is_some(), css.to_owned());
        match q.condition.as_ref().unwrap().0 {
            MediaConditionTerm::InParens(
                MediaInParensTerm::Feature(
                    MediaFeature::Width(
                        Some(Width(Range::Le(w)))))) => assert!(w == specified::Length::Absolute(Au::from_px(43))),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media fridge and (max-width: 52px) { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(Unknown("fridge")), css.to_owned());
        assert!(q.condition.is_some(), css.to_owned());
        match q.condition.as_ref().unwrap().0 {
            MediaConditionTerm::InParens(
                MediaInParensTerm::Feature(
                    MediaFeature::Width(
                        Some(Width(Range::Le(w)))))) => assert!(w == specified::Length::Absolute(Au::from_px(52))),
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media not (min-width: 300px) {}", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition.is_some(), css.to_owned());
        match q.condition.as_ref().unwrap().0 {
            MediaConditionTerm::Connective(
                MediaConnectiveTerm::Not(
                    MediaInParensTerm::Feature(
                        MediaFeature::Width(
                            Some(Width(Range::Ge(w))))))) => assert!(w == specified::Length::Absolute(Au::from_px(300))),
            _ => panic!("wrong expression type"),
        }
    });
}

#[test]
fn test_mq_multiple_expressions() {
    test_media_rule("@media (min-width: 100px) and (max-width: 200px) { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition.is_some(), css.to_owned());
        match q.condition.as_ref().unwrap().0 {
            MediaConditionTerm::Connective(
                MediaConnectiveTerm::And(ref e)) => match &e[..] {
                    [MediaInParensTerm::Feature(
                        MediaFeature::Width(
                            Some(Width(Range::Ge(e0))))),
                     MediaInParensTerm::Feature(
                         MediaFeature::Width(
                             Some(Width(Range::Le(e1)))))] => {
                        assert!(e0 == specified::Length::Absolute(Au::from_px(100)));
                        assert!(e1 == specified::Length::Absolute(Au::from_px(200)));
                    }
                    _ => panic!("wrong expression type"),
                },
            _ => panic!("wrong expression type"),
        }
    });

    test_media_rule("@media not screen and (min-width: 100px) and (max-width: 200px) { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(Screen), css.to_owned());
        assert!(q.condition.is_some(), css.to_owned());
        match q.condition.as_ref().unwrap().0 {
            MediaConditionTerm::Connective(
                MediaConnectiveTerm::And(ref e)) => match &e[..] {
                    [MediaInParensTerm::Feature(
                        MediaFeature::Width(
                            Some(Width(Range::Ge(e0))))),
                     MediaInParensTerm::Feature(
                         MediaFeature::Width(
                             Some(Width(Range::Le(e1)))))] => {
                        assert!(e0 == specified::Length::Absolute(Au::from_px(100)));
                        assert!(e1 == specified::Length::Absolute(Au::from_px(200)));
                    }
                    _ => panic!("wrong expression type"),
                },
            _ => panic!("wrong expression type"),
        }
    });
}

#[test]
fn test_mq_malformed_expressions() {
    test_media_rule("@media (min-width: 100blah) and (max-width: 200px) { }", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition == None);
    });

    test_media_rule("@media (min-width: 30em foo bar) {}", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition == None);
    });

    test_media_rule("@media not {}", |list, css| {
        assert!(list.queries.len() == 1, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition == None);
    });

    test_media_rule("@media , {}", |list, css| {
        assert!(list.queries.len() == 2, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition == None);
        let q = &list.queries[1];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition == None);
    });

    test_media_rule("@media screen 4px, print {}", |list, css| {
        assert!(list.queries.len() == 2, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition == None);
        let q = &list.queries[1];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(Print), css.to_owned());
        assert!(q.condition == None);
    });

    test_media_rule("@media screen, {}", |list, css| {
        assert!(list.queries.len() == 2, css.to_owned());
        let q = &list.queries[0];
        assert!(q.qualifier == None, css.to_owned());
        assert!(q.media_type == media_type!(Screen), css.to_owned());
        assert!(q.condition == None);
        let q = &list.queries[1];
        assert!(q.qualifier == qualifier!(Not), css.to_owned());
        assert!(q.media_type == media_type!(All), css.to_owned());
        assert!(q.condition == None);
    });
}

#[test]
fn test_matching_simple() {
    let device = Device::new(MediaType::Screen,
                             TypedSize2D(200.0, 100.0));

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
    let device = Device::new(MediaType::Screen,
                             TypedSize2D(200.0, 100.0));

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

    media_query_test(&device, "@media not screen and (min-width: 50px) and (max-width: 100px) { a { color: red; } }", 1);
    media_query_test(&device, "@media not screen and (min-width: 250px) and (max-width: 300px) { a { color: red; } }", 1);
    media_query_test(&device, "@media not screen and (min-width: 50px) and (max-width: 250px) { a { color: red; } }", 0);

    media_query_test(&device, "@media not screen and (min-width: 3.1em) and (max-width: 6em) { a { color: red; } }", 1);
    media_query_test(&device, "@media not screen and (min-width: 16em) and (max-width: 19.75em) { a { color: red; } }", 1);
    media_query_test(&device, "@media not screen and (min-width: 3em) and (max-width: 250px) { a { color: red; } }", 0);
}

#[test]
fn test_matching_invalid() {
    let device = Device::new(MediaType::Screen,
                             TypedSize2D(200.0, 100.0));

    media_query_test(&device, "@media fridge { a { color: red; } }", 0);
    media_query_test(&device, "@media not print and (width: 100) { a { color: red; } }", 0);
}
