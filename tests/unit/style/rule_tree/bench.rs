/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, SourcePosition};
use parking_lot::RwLock;
use rayon;
use servo_url::ServoUrl;
use std::sync::Arc;
use style::error_reporting::ParseErrorReporter;
use style::media_queries::MediaList;
use style::parser::ParserContextExtraData;
use style::properties::{longhands, DeclaredValue, Importance, PropertyDeclaration, PropertyDeclarationBlock};
use style::rule_tree::{RuleTree, StrongRuleNode, StyleSource};
use style::stylesheets::{Origin, Stylesheet, CssRule};
use test::{self, Bencher};

struct ErrorringErrorReporter;
impl ParseErrorReporter for ErrorringErrorReporter {
    fn report_error(&self, _: &mut Parser, position: SourcePosition, message: &str) {
        panic!("CSS error: {:?} {}", position, message);
    }

    fn clone(&self) -> Box<ParseErrorReporter + Send + Sync> {
        Box::new(ErrorringErrorReporter)
    }
}

struct AutoGCRuleTree<'a>(&'a RuleTree);

impl<'a> AutoGCRuleTree<'a> {
    fn new(r: &'a RuleTree) -> Self {
        AutoGCRuleTree(r)
    }
}

impl<'a> Drop for AutoGCRuleTree<'a> {
    fn drop(&mut self) {
        unsafe { self.0.gc() }
    }
}

fn parse_rules(css: &str) -> Vec<(StyleSource, Importance)> {
    let s = Stylesheet::from_str(css,
                                 ServoUrl::parse("http://localhost").unwrap(),
                                 Origin::Author,
                                 MediaList {
                                     media_queries: vec![],
                                 },
                                 None,
                                 Box::new(ErrorringErrorReporter),
                                 ParserContextExtraData {});
    let rules = s.rules.read();
    rules.0.iter().filter_map(|rule| {
        match *rule {
            CssRule::Style(ref style_rule) => Some(style_rule),
            _ => None,
        }
    }).cloned().map(StyleSource::Style).map(|s| {
        (s, Importance::Normal)
    }).collect()
}

fn test_insertion(rule_tree: &RuleTree, rules: Vec<(StyleSource, Importance)>) -> StrongRuleNode {
    rule_tree.insert_ordered_rules(rules.into_iter())
}

fn test_insertion_style_attribute(rule_tree: &RuleTree, rules: &[(StyleSource, Importance)]) -> StrongRuleNode {
    let mut rules = rules.to_vec();
    rules.push((StyleSource::Declarations(Arc::new(RwLock::new(PropertyDeclarationBlock {
        declarations: vec![
            (PropertyDeclaration::Display(DeclaredValue::Value(
                longhands::display::SpecifiedValue::block)),
            Importance::Normal),
        ],
        important_count: 0,
    }))), Importance::Normal));
    test_insertion(rule_tree, rules)
}

#[bench]
fn bench_insertion_basic(b: &mut Bencher) {
    let r = RuleTree::new();

    let rules_matched = parse_rules(
        ".foo { width: 200px; } \
         .bar { height: 500px; } \
         .baz { display: block; }");

    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r);

        for _ in 0..(4000 + 400) {
            test::black_box(test_insertion(&r, rules_matched.clone()));
        }
    })
}

#[bench]
fn bench_insertion_basic_per_element(b: &mut Bencher) {
    let r = RuleTree::new();

    let rules_matched = parse_rules(
        ".foo { width: 200px; } \
         .bar { height: 500px; } \
         .baz { display: block; }");

    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r);

        test::black_box(test_insertion(&r, rules_matched.clone()));
    });
}

#[bench]
fn bench_expensive_insertion(b: &mut Bencher) {
    let r = RuleTree::new();

    // This test case tests a case where you style a bunch of siblings
    // matching the same rules, with a different style attribute each
    // one.
    let rules_matched = parse_rules(
        ".foo { width: 200px; } \
         .bar { height: 500px; } \
         .baz { display: block; }");

    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r);

        for _ in 0..(4000 + 400) {
            test::black_box(test_insertion_style_attribute(&r, &rules_matched));
        }
    });
}

#[bench]
fn bench_insertion_basic_parallel(b: &mut Bencher) {
    let r = RuleTree::new();

    let rules_matched = parse_rules(
        ".foo { width: 200px; } \
         .bar { height: 500px; } \
         .baz { display: block; }");

    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r);

        rayon::scope(|s| {
            for _ in 0..4 {
                s.spawn(|s| {
                    for _ in 0..1000 {
                        test::black_box(test_insertion(&r,
                                                       rules_matched.clone()));
                    }
                    s.spawn(|_| {
                        for _ in 0..100 {
                            test::black_box(test_insertion(&r,
                                                           rules_matched.clone()));
                        }
                    })
                })
            }
        });
    });
}

#[bench]
fn bench_expensive_insersion_parallel(b: &mut Bencher) {
    let r = RuleTree::new();

    let rules_matched = parse_rules(
        ".foo { width: 200px; } \
         .bar { height: 500px; } \
         .baz { display: block; }");

    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r);

        rayon::scope(|s| {
            for _ in 0..4 {
                s.spawn(|s| {
                    for _ in 0..1000 {
                        test::black_box(test_insertion_style_attribute(&r,
                                                                       &rules_matched));
                    }
                    s.spawn(|_| {
                        for _ in 0..100 {
                            test::black_box(test_insertion_style_attribute(&r,
                                                                           &rules_matched));
                        }
                    })
                })
            }
        });
    });
}
