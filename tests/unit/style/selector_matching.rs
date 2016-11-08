/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use html5ever_atoms::LocalName;
use parking_lot::RwLock;
use selectors::parser::{LocalName as LocalNameSelector, ParserContext, parse_selector_list};
use servo_atoms::Atom;
use std::sync::Arc;
use style::properties::{PropertyDeclarationBlock, PropertyDeclaration, DeclaredValue};
use style::properties::{longhands, Importance};
use style::selector_matching::{Rule, SelectorMap};
use style::stylesheets::StyleRule;
use style::thread_state;

/// Helper method to get some Rules from selector strings.
/// Each sublist of the result contains the Rules for one StyleRule.
fn get_mock_rules(css_selectors: &[&str]) -> Vec<Vec<Rule>> {
    css_selectors.iter().enumerate().map(|(i, selectors)| {
        let context = ParserContext::new();
        let selectors =
            parse_selector_list(&context, &mut Parser::new(*selectors)).unwrap();

        let rule = Arc::new(RwLock::new(StyleRule {
            selectors: selectors,
            block: Arc::new(RwLock::new(PropertyDeclarationBlock {
                declarations: vec![
                    (PropertyDeclaration::Display(DeclaredValue::Value(
                        longhands::display::SpecifiedValue::block)),
                     Importance::Normal),
                ],
                important_count: 0,
            })),
        }));

        let guard = rule.read();
        guard.selectors.iter().map(|s| {
            Rule {
                selector: s.complex_selector.clone(),
                style_rule: rule.clone(),
                specificity: s.specificity,
                source_order: i,
            }
        }).collect()
    }).collect()
}

fn get_mock_map(selectors: &[&str]) -> SelectorMap {
    let mut map = SelectorMap::new();
    let selector_rules = get_mock_rules(selectors);

    for rules in selector_rules.into_iter() {
        for rule in rules.into_iter() {
            map.insert(rule)
        }
    }

    map
}

#[test]
fn test_rule_ordering_same_specificity() {
    let rules_list = get_mock_rules(&["a.intro", "img.sidebar"]);
    let a = &rules_list[0][0];
    let b = &rules_list[1][0];
    assert!((a.specificity, a.source_order) < ((b.specificity, b.source_order)),
            "The rule that comes later should win.");
}


#[test]
fn test_get_id_name() {
    let rules_list = get_mock_rules(&[".intro", "#top"]);
    assert_eq!(SelectorMap::get_id_name(&rules_list[0][0]), None);
    assert_eq!(SelectorMap::get_id_name(&rules_list[1][0]), Some(Atom::from("top")));
}

#[test]
fn test_get_class_name() {
    let rules_list = get_mock_rules(&[".intro.foo", "#top"]);
    assert_eq!(SelectorMap::get_class_name(&rules_list[0][0]), Some(Atom::from("intro")));
    assert_eq!(SelectorMap::get_class_name(&rules_list[1][0]), None);
}

#[test]
fn test_get_local_name() {
    let rules_list = get_mock_rules(&["img.foo", "#top", "IMG", "ImG"]);
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
    let rules_list = get_mock_rules(&[".intro.foo", "#top"]);
    let mut selector_map = SelectorMap::new();
    selector_map.insert(rules_list[1][0].clone());
    assert_eq!(1, selector_map.id_hash.get(&atom!("top")).unwrap()[0].source_order);
    selector_map.insert(rules_list[0][0].clone());
    assert_eq!(0, selector_map.class_hash.get(&Atom::from("intro")).unwrap()[0].source_order);
    assert!(selector_map.class_hash.get(&Atom::from("foo")).is_none());
}

#[test]
fn test_get_universal_rules() {
    thread_state::initialize(thread_state::LAYOUT);
    let map = get_mock_map(&["*|*", "#foo > *|*", ".klass", "#id"]);
    let mut decls = vec![];

    map.get_universal_rules(&mut decls);

    assert_eq!(decls.len(), 1);
}
