/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, SourcePosition};
use rayon;
use servo_url::ServoUrl;
use style::context::QuirksMode;
use style::error_reporting::{ParseErrorReporter, ContextualParseError};
use style::media_queries::MediaList;
use style::properties::{longhands, Importance, PropertyDeclaration, PropertyDeclarationBlock};
use style::rule_tree::{CascadeLevel, RuleTree, StrongRuleNode, StyleSource};
use style::shared_lock::SharedRwLock;
use style::stylearc::Arc;
use style::stylesheets::{Origin, Stylesheet, CssRule};
use test::{self, Bencher};

struct ErrorringErrorReporter;
impl ParseErrorReporter for ErrorringErrorReporter {
    fn report_error<'a>(&self,
                        input: &mut Parser,
                        position: SourcePosition,
                        error: ContextualParseError<'a>,
                        url: &ServoUrl,
                        line_number_offset: u64) {
        let location = input.source_location(position);
        let line_offset = location.line + line_number_offset as u32;
        panic!("CSS error: {}\t\n{}:{} {}", url.as_str(), line_offset, location.column, error.to_string());
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
        unsafe {
            self.0.gc();
            assert!(::std::thread::panicking() ||
                    !self.0.root().has_children_for_testing(),
                    "No rule nodes other than the root shall remain!");
        }
    }
}

fn parse_rules(css: &str) -> Vec<(StyleSource, CascadeLevel)> {
    let lock = SharedRwLock::new();
    let media = Arc::new(lock.wrap(MediaList::empty()));

    let s = Stylesheet::from_str(css,
                                 ServoUrl::parse("http://localhost").unwrap(),
                                 Origin::Author,
                                 media,
                                 lock,
                                 None,
                                 &ErrorringErrorReporter,
                                 QuirksMode::NoQuirks,
                                 0u64);
    let guard = s.shared_lock.read();
    let rules = s.contents.rules.read_with(&guard);
    rules.0.iter().filter_map(|rule| {
        match *rule {
            CssRule::Style(ref style_rule) => Some(style_rule),
            _ => None,
        }
    }).cloned().map(StyleSource::Style).map(|s| {
        (s, CascadeLevel::UserNormal)
    }).collect()
}

fn test_insertion(rule_tree: &RuleTree, rules: Vec<(StyleSource, CascadeLevel)>) -> StrongRuleNode {
    rule_tree.insert_ordered_rules(rules.into_iter())
}

fn test_insertion_style_attribute(rule_tree: &RuleTree, rules: &[(StyleSource, CascadeLevel)],
                                  shared_lock: &SharedRwLock)
                                  -> StrongRuleNode {
    let mut rules = rules.to_vec();
    rules.push((StyleSource::Declarations(Arc::new(shared_lock.wrap(PropertyDeclarationBlock::with_one(
        PropertyDeclaration::Display(
            longhands::display::SpecifiedValue::block),
        Importance::Normal
    )))), CascadeLevel::UserNormal));
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

    let shared_lock = SharedRwLock::new();
    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r);

        for _ in 0..(4000 + 400) {
            test::black_box(test_insertion_style_attribute(&r, &rules_matched, &shared_lock));
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

    let shared_lock = SharedRwLock::new();
    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r);

        rayon::scope(|s| {
            for _ in 0..4 {
                s.spawn(|s| {
                    for _ in 0..1000 {
                        test::black_box(test_insertion_style_attribute(&r,
                                                                       &rules_matched,
                                                                       &shared_lock));
                    }
                    s.spawn(|_| {
                        for _ in 0..100 {
                            test::black_box(test_insertion_style_attribute(&r,
                                                                           &rules_matched,
                                                                           &shared_lock));
                        }
                    })
                })
            }
        });
    });
}
