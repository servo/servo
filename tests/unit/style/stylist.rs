/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use html5ever_atoms::LocalName;
use selectors::parser::LocalName as LocalNameSelector;
use servo_atoms::Atom;
use std::sync::Arc;
use style::properties::{PropertyDeclarationBlock, PropertyDeclaration};
use style::properties::{longhands, Importance};
use style::rule_tree::CascadeLevel;
use style::selector_parser::SelectorParser;
use style::shared_lock::SharedRwLock;
use style::stylesheets::StyleRule;
use style::stylist::{Rule, SelectorMap};
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
        }));

        let guard = shared_lock.read();
        let rule = locked.read_with(&guard);
        rule.selectors.0.iter().map(|s| {
            Rule {
                selector: s.inner.clone(),
                style_rule: locked.clone(),
                specificity: s.specificity,
                source_order: i,
            }
        }).collect()
    }).collect(), shared_lock)
}

fn get_mock_map(selectors: &[&str]) -> (SelectorMap, SharedRwLock) {
    let mut map = SelectorMap::new();
    let (selector_rules, shared_lock) = get_mock_rules(selectors);

    for rules in selector_rules.into_iter() {
        for rule in rules.into_iter() {
            map.insert(rule)
        }
    }

    (map, shared_lock)
}

#[test]
fn test_rule_ordering_same_specificity() {
    let (rules_list, _) = get_mock_rules(&["a.intro", "img.sidebar"]);
    let a = &rules_list[0][0];
    let b = &rules_list[1][0];
    assert!((a.specificity, a.source_order) < ((b.specificity, b.source_order)),
            "The rule that comes later should win.");
}


#[test]
fn test_get_id_name() {
    let (rules_list, _) = get_mock_rules(&[".intro", "#top"]);
    assert_eq!(SelectorMap::get_id_name(&rules_list[0][0]), None);
    assert_eq!(SelectorMap::get_id_name(&rules_list[1][0]), Some(Atom::from("top")));
}

#[test]
fn test_get_class_name() {
    let (rules_list, _) = get_mock_rules(&[".intro.foo", "#top"]);
    assert_eq!(SelectorMap::get_class_name(&rules_list[0][0]), Some(Atom::from("foo")));
    assert_eq!(SelectorMap::get_class_name(&rules_list[1][0]), None);
}

#[test]
fn test_get_local_name() {
    let (rules_list, _) = get_mock_rules(&["img.foo", "#top", "IMG", "ImG"]);
    let check = |i: usize, names: Option<(&str, &str)>| {
        assert!(SelectorMap::get_local_name(&rules_list[i][0])
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

    let guard = shared_lock.read();
    let decls = map.get_universal_rules(
        &guard, CascadeLevel::UserNormal, CascadeLevel::UserImportant);

    assert_eq!(decls.len(), 1);
}
