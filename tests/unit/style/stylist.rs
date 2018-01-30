/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::SourceLocation;
use euclid::TypedScale;
use euclid::TypedSize2D;
use selectors::parser::{AncestorHashes, Selector};
use servo_arc::Arc;
use servo_atoms::Atom;
use style::context::QuirksMode;
use style::media_queries::{Device, MediaType};
use style::properties::{PropertyDeclarationBlock, PropertyDeclaration};
use style::properties::{longhands, Importance};
use style::selector_map::SelectorMap;
use style::selector_parser::{SelectorImpl, SelectorParser};
use style::shared_lock::SharedRwLock;
use style::stylesheets::StyleRule;
use style::stylist::{Stylist, Rule};
use style::stylist::needs_revalidation_for_testing;
use style::thread_state::{self, ThreadState};

/// Helper method to get some Rules from selector strings.
/// Each sublist of the result contains the Rules for one StyleRule.
fn get_mock_rules(css_selectors: &[&str]) -> (Vec<Vec<Rule>>, SharedRwLock) {
    let shared_lock = SharedRwLock::new();
    (css_selectors.iter().enumerate().map(|(i, selectors)| {
        let selectors = SelectorParser::parse_author_origin_no_namespace(selectors).unwrap();

        let locked = Arc::new(shared_lock.wrap(StyleRule {
            selectors: selectors,
            block: Arc::new(shared_lock.wrap(PropertyDeclarationBlock::with_one(
                PropertyDeclaration::Display(
                    longhands::display::SpecifiedValue::Block),
                Importance::Normal
            ))),
            source_location: SourceLocation {
                line: 0,
                column: 0,
            },
        }));

        let guard = shared_lock.read();
        let rule = locked.read_with(&guard);
        rule.selectors.0.iter().map(|s| {
            Rule::new(s.clone(), AncestorHashes::new(s, QuirksMode::NoQuirks), locked.clone(), i as u32)
        }).collect()
    }).collect(), shared_lock)
}

fn parse_selectors(selectors: &[&str]) -> Vec<Selector<SelectorImpl>> {
    selectors.iter()
             .map(|x| SelectorParser::parse_author_origin_no_namespace(x).unwrap().0
                                                                         .into_iter()
                                                                         .nth(0)
                                                                         .unwrap())
             .collect()
}

#[test]
fn test_revalidation_selectors() {
    let test = parse_selectors(&[
        // Not revalidation selectors.
        "div",
        "div:not(.foo)",
        "div span",
        "div > span",

        // ID selectors.
        "#foo1",
        "#foo2::before",
        "#foo3 > span",
        "#foo1 > span", // FIXME(bz): This one should not be a
                        // revalidation selector, since #foo1 should be in the
                        // rule hash.

        // Attribute selectors.
        "div[foo]",
        "div:not([foo])",
        "div[foo = \"bar\"]",
        "div[foo ~= \"bar\"]",
        "div[foo |= \"bar\"]",
        "div[foo ^= \"bar\"]",
        "div[foo $= \"bar\"]",
        "div[foo *= \"bar\"]",
        "*|div[foo][bar = \"baz\"]",

        // Non-state-based pseudo-classes.
        "div:empty",
        "div:first-child",
        "div:last-child",
        "div:only-child",
        "div:nth-child(2)",
        "div:nth-last-child(2)",
        "div:nth-of-type(2)",
        "div:nth-last-of-type(2)",
        "div:first-of-type",
        "div:last-of-type",
        "div:only-of-type",

        // Note: it would be nice to test :moz-any and the various other non-TS
        // pseudo classes supported by gecko, but we don't have access to those
        // in these unit tests. :-(

        // Sibling combinators.
        "span + div",
        "span ~ div",

        // Selectors in the ancestor chain (needed for cousin sharing).
        "p:first-child span",
    ]).into_iter()
      .filter(|s| needs_revalidation_for_testing(&s))
      .collect::<Vec<_>>();

    let reference = parse_selectors(&[
        // ID selectors.
        "#foo3 > span",
        "#foo1 > span",

        // Attribute selectors.
        "div[foo]",
        "div:not([foo])",
        "div[foo = \"bar\"]",
        "div[foo ~= \"bar\"]",
        "div[foo |= \"bar\"]",
        "div[foo ^= \"bar\"]",
        "div[foo $= \"bar\"]",
        "div[foo *= \"bar\"]",
        "*|div[foo][bar = \"baz\"]",

        // Non-state-based pseudo-classes.
        "div:empty",
        "div:first-child",
        "div:last-child",
        "div:only-child",
        "div:nth-child(2)",
        "div:nth-last-child(2)",
        "div:nth-of-type(2)",
        "div:nth-last-of-type(2)",
        "div:first-of-type",
        "div:last-of-type",
        "div:only-of-type",

        // Sibling combinators.
        "span + div",
        "span ~ div",

        // Selectors in the ancestor chain (needed for cousin sharing).
        "p:first-child span",
    ]).into_iter()
      .collect::<Vec<_>>();

    assert_eq!(test.len(), reference.len());
    for (t, r) in test.into_iter().zip(reference.into_iter()) {
        assert_eq!(t, r)
    }
}

#[test]
fn test_rule_ordering_same_specificity() {
    let (rules_list, _) = get_mock_rules(&["a.intro", "img.sidebar"]);
    let a = &rules_list[0][0];
    let b = &rules_list[1][0];
    assert!((a.specificity(), a.source_order) < ((b.specificity(), b.source_order)),
            "The rule that comes later should win.");
}

#[test]
fn test_insert() {
    let (rules_list, _) = get_mock_rules(&[".intro.foo", "#top"]);
    let mut selector_map = SelectorMap::new();
    selector_map.insert(rules_list[1][0].clone(), QuirksMode::NoQuirks)
                .expect("OOM");
    assert_eq!(1, selector_map.id_hash.get(&Atom::from("top"), QuirksMode::NoQuirks).unwrap()[0].source_order);
    selector_map.insert(rules_list[0][0].clone(), QuirksMode::NoQuirks)
                .expect("OOM");
    assert_eq!(0, selector_map.class_hash.get(&Atom::from("foo"), QuirksMode::NoQuirks).unwrap()[0].source_order);
    assert!(selector_map.class_hash.get(&Atom::from("intro"), QuirksMode::NoQuirks).is_none());
}

fn mock_stylist() -> Stylist {
    let device = Device::new(MediaType::screen(), TypedSize2D::new(0f32, 0f32), TypedScale::new(1.0));
    Stylist::new(device, QuirksMode::NoQuirks)
}

#[test]
fn test_stylist_device_accessors() {
    thread_state::initialize(ThreadState::LAYOUT);
    let stylist = mock_stylist();
    assert_eq!(stylist.device().media_type(), MediaType::screen());
    let mut stylist_mut = mock_stylist();
    assert_eq!(stylist_mut.device_mut().media_type(), MediaType::screen());
}

#[test]
fn test_stylist_rule_tree_accessors() {
    thread_state::initialize(ThreadState::LAYOUT);
    let stylist = mock_stylist();
    stylist.rule_tree();
    stylist.rule_tree().root();
}
