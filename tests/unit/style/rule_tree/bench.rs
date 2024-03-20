/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::SourceLocation;
use rayon;
use servo_arc::Arc;
use style::applicable_declarations::CascadePriority;
use style::context::QuirksMode;
use style::error_reporting::{ContextualParseError, ParseErrorReporter};
use style::media_queries::MediaList;
use style::properties::{longhands, Importance, PropertyDeclaration, PropertyDeclarationBlock};
use style::rule_tree::{CascadeLevel, RuleTree, StrongRuleNode, StyleSource};
use style::shared_lock::{SharedRwLock, StylesheetGuards};
use style::stylesheets::layer_rule::LayerOrder;
use style::stylesheets::{AllowImportRules, CssRule, Origin, Stylesheet, UrlExtraData};
use style::thread_state::{self, ThreadState};
use test::{self, Bencher};
use url::Url;

struct ErrorringErrorReporter;
impl ParseErrorReporter for ErrorringErrorReporter {
    fn report_error(
        &self,
        url: &UrlExtraData,
        location: SourceLocation,
        error: ContextualParseError,
    ) {
        panic!(
            "CSS error: {}\t\n{}:{} {}",
            url.as_str(),
            location.line,
            location.column,
            error
        );
    }
}

struct AutoGCRuleTree<'a>(&'a RuleTree, &'a SharedRwLock);

impl<'a> AutoGCRuleTree<'a> {
    fn new(r: &'a RuleTree, lock: &'a SharedRwLock) -> Self {
        AutoGCRuleTree(r, lock)
    }
}

impl<'a> Drop for AutoGCRuleTree<'a> {
    fn drop(&mut self) {
        const DEBUG: bool = false;
        unsafe {
            self.0.gc();
            if DEBUG {
                let guard = self.1.read();
                self.0.dump_stdout(&StylesheetGuards::same(&guard));
            }
            assert!(
                ::std::thread::panicking() || !self.0.root().has_children_for_testing(),
                "No rule nodes other than the root shall remain!"
            );
        }
    }
}

fn parse_rules(lock: &SharedRwLock, css: &str) -> Vec<(StyleSource, CascadeLevel)> {
    let media = Arc::new(lock.wrap(MediaList::empty()));

    let url_data = Url::parse("http://localhost").unwrap().into();
    let s = Stylesheet::from_str(
        css,
        url_data,
        Origin::Author,
        media,
        lock.clone(),
        None,
        Some(&ErrorringErrorReporter),
        QuirksMode::NoQuirks,
        AllowImportRules::Yes,
    );
    let guard = s.shared_lock.read();
    let rules = s.contents.rules.read_with(&guard);
    rules
        .0
        .iter()
        .filter_map(|rule| match *rule {
            CssRule::Style(ref style_rule) => Some((
                StyleSource::from_rule(style_rule.clone()),
                CascadeLevel::UserNormal,
            )),
            _ => None,
        })
        .collect()
}

fn test_insertion(rule_tree: &RuleTree, rules: Vec<(StyleSource, CascadeLevel)>) -> StrongRuleNode {
    rule_tree.insert_ordered_rules(rules.into_iter().map(|(style_source, cascade_level)| {
        (
            style_source,
            CascadePriority::new(cascade_level, LayerOrder::root()),
        )
    }))
}

fn test_insertion_style_attribute(
    rule_tree: &RuleTree,
    rules: &[(StyleSource, CascadeLevel)],
    shared_lock: &SharedRwLock,
) -> StrongRuleNode {
    let mut rules = rules.to_vec();
    rules.push((
        StyleSource::from_declarations(Arc::new(shared_lock.wrap(
            PropertyDeclarationBlock::with_one(
                PropertyDeclaration::Display(longhands::display::SpecifiedValue::Block),
                Importance::Normal,
            ),
        ))),
        CascadeLevel::UserNormal,
    ));
    test_insertion(rule_tree, rules)
}

#[bench]
fn bench_insertion_basic(b: &mut Bencher) {
    let r = RuleTree::new();
    thread_state::initialize(ThreadState::SCRIPT);
    let lock = SharedRwLock::new();
    let rules_matched = parse_rules(
        &lock,
        ".foo { width: 200px; } \
         .bar { height: 500px; } \
         .baz { display: block; }",
    );

    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r, &lock);

        for _ in 0..(4000 + 400) {
            test::black_box(test_insertion(&r, rules_matched.clone()));
        }
    })
}

#[bench]
fn bench_insertion_basic_per_element(b: &mut Bencher) {
    let r = RuleTree::new();
    thread_state::initialize(ThreadState::SCRIPT);

    let lock = SharedRwLock::new();
    let rules_matched = parse_rules(
        &lock,
        ".foo { width: 200px; } \
         .bar { height: 500px; } \
         .baz { display: block; }",
    );

    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r, &lock);

        test::black_box(test_insertion(&r, rules_matched.clone()));
    });
}

#[bench]
fn bench_expensive_insertion(b: &mut Bencher) {
    let r = RuleTree::new();
    thread_state::initialize(ThreadState::SCRIPT);

    // This test case tests a case where you style a bunch of siblings
    // matching the same rules, with a different style attribute each
    // one.
    let lock = SharedRwLock::new();
    let rules_matched = parse_rules(
        &lock,
        ".foo { width: 200px; } \
         .bar { height: 500px; } \
         .baz { display: block; }",
    );

    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r, &lock);

        for _ in 0..(4000 + 400) {
            test::black_box(test_insertion_style_attribute(&r, &rules_matched, &lock));
        }
    });
}

#[bench]
fn bench_insertion_basic_parallel(b: &mut Bencher) {
    let r = RuleTree::new();
    thread_state::initialize(ThreadState::SCRIPT);

    let lock = SharedRwLock::new();
    let rules_matched = parse_rules(
        &lock,
        ".foo { width: 200px; } \
         .bar { height: 500px; } \
         .baz { display: block; }",
    );

    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r, &lock);

        rayon::scope_fifo(|s| {
            for _ in 0..4 {
                s.spawn_fifo(|s| {
                    for _ in 0..1000 {
                        test::black_box(test_insertion(&r, rules_matched.clone()));
                    }
                    s.spawn_fifo(|_| {
                        for _ in 0..100 {
                            test::black_box(test_insertion(&r, rules_matched.clone()));
                        }
                    })
                })
            }
        });
    });
}

#[bench]
fn bench_expensive_insertion_parallel(b: &mut Bencher) {
    let r = RuleTree::new();
    thread_state::initialize(ThreadState::SCRIPT);

    let lock = SharedRwLock::new();
    let rules_matched = parse_rules(
        &lock,
        ".foo { width: 200px; } \
         .bar { height: 500px; } \
         .baz { display: block; }",
    );

    b.iter(|| {
        let _gc = AutoGCRuleTree::new(&r, &lock);

        rayon::scope_fifo(|s| {
            for _ in 0..4 {
                s.spawn_fifo(|s| {
                    for _ in 0..1000 {
                        test::black_box(test_insertion_style_attribute(&r, &rules_matched, &lock));
                    }
                    s.spawn_fifo(|_| {
                        for _ in 0..100 {
                            test::black_box(test_insertion_style_attribute(
                                &r,
                                &rules_matched,
                                &lock,
                            ));
                        }
                    })
                })
            }
        });
    });
}
