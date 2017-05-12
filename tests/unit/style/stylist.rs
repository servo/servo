/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::SourceLocation;
use html5ever::LocalName;
use selectors::parser::LocalName as LocalNameSelector;
use selectors::parser::Selector;
use servo_atoms::Atom;
use style::properties::{PropertyDeclarationBlock, PropertyDeclaration};
use style::properties::{longhands, Importance};
use style::rule_tree::CascadeLevel;
use style::selector_parser::{SelectorImpl, SelectorParser};
use style::shared_lock::SharedRwLock;
use style::stylearc::Arc;
use style::stylesheets::StyleRule;
use style::stylist;
use style::stylist::{Rule, SelectorMap};
use style::stylist::needs_revalidation;
use style::thread_state;

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
                    longhands::display::SpecifiedValue::block),
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
            Rule::new(s.clone(), locked.clone(), i)
        }).collect()
    }).collect(), shared_lock)
}

fn get_mock_map(selectors: &[&str]) -> (SelectorMap<Rule>, SharedRwLock) {
    let mut map = SelectorMap::<Rule>::new();
    let (selector_rules, shared_lock) = get_mock_rules(selectors);

    for rules in selector_rules.into_iter() {
        for rule in rules.into_iter() {
            map.insert(rule)
        }
    }

    (map, shared_lock)
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
        "#bar",
        "div:not(.foo)",
        "div span",
        "div > span",

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
      .filter(|s| needs_revalidation(&s))
      .map(|s| s.inner.complex)
      .collect::<Vec<_>>();

    let reference = parse_selectors(&[
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
      .map(|s| s.inner.complex)
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
fn test_get_id_name() {
    let (rules_list, _) = get_mock_rules(&[".intro", "#top"]);
    assert_eq!(stylist::get_id_name(&rules_list[0][0].selector.inner), None);
    assert_eq!(stylist::get_id_name(&rules_list[1][0].selector.inner), Some(Atom::from("top")));
}

#[test]
fn test_get_class_name() {
    let (rules_list, _) = get_mock_rules(&[".intro.foo", "#top"]);
    assert_eq!(stylist::get_class_name(&rules_list[0][0].selector.inner), Some(Atom::from("foo")));
    assert_eq!(stylist::get_class_name(&rules_list[1][0].selector.inner), None);
}

#[test]
fn test_get_local_name() {
    let (rules_list, _) = get_mock_rules(&["img.foo", "#top", "IMG", "ImG"]);
    let check = |i: usize, names: Option<(&str, &str)>| {
        assert!(stylist::get_local_name(&rules_list[i][0].selector.inner)
                == names.map(|(name, lower_name)| LocalNameSelector {
                        name: LocalName::from(name),
                        lower_name: LocalName::from(lower_name) }))
    };
    check(0, Some(("img", "img")));
    check(1, None);
    check(2, Some(("IMG", "img")));
    check(3, Some(("ImG", "img")));
}

#[test]
fn test_insert() {
    let (rules_list, _) = get_mock_rules(&[".intro.foo", "#top"]);
    let mut selector_map = SelectorMap::new();
    selector_map.insert(rules_list[1][0].clone());
    assert_eq!(1, selector_map.id_hash.get(&Atom::from("top")).unwrap()[0].source_order);
    selector_map.insert(rules_list[0][0].clone());
    assert_eq!(0, selector_map.class_hash.get(&Atom::from("foo")).unwrap()[0].source_order);
    assert!(selector_map.class_hash.get(&Atom::from("intro")).is_none());
}

#[test]
fn test_get_universal_rules() {
    thread_state::initialize(thread_state::LAYOUT);
    let (map, shared_lock) = get_mock_map(&["*|*", "#foo > *|*", ".klass", "#id"]);

    let decls = map.get_universal_rules(CascadeLevel::UserNormal);

    assert_eq!(decls.len(), 1);
}
