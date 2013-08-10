/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::iterator::Iterator;
use std::ascii::to_ascii_lower;
use cssparser::*;
use selectors;
use properties;
use errors::{ErrorLoggerIterator, log_css_error};
use namespaces::{NamespaceMap, parse_namespace_rule};
use media_queries::{MediaRule, parse_media_rule};


pub struct Stylesheet {
    style_rules: ~[CSSRule],
    namespaces: NamespaceMap,
}


pub enum CSSRule {
    CSSStyleRule(StyleRule),
    CSSMediaRule(MediaRule),
}


pub struct StyleRule {
    selectors: ~[selectors::Selector],
    declarations: ~[properties::PropertyDeclaration],
}


fn parse_stylesheet(css: &str) -> Stylesheet {
    static STATE_CHARSET: uint = 1;
    static STATE_IMPORTS: uint = 2;
    static STATE_NAMESPACES: uint = 3;
    static STATE_BODY: uint = 4;
    let mut state: uint = STATE_CHARSET;

    let mut rules = ~[];
    let mut namespaces = NamespaceMap::new();

    for rule in ErrorLoggerIterator(parse_stylesheet_rules(tokenize(css))) {
        let next_state;  // Unitialized to force each branch to set it.
        match rule {
            QualifiedRule(rule) => {
                next_state = STATE_BODY;
                parse_style_rule(rule, &mut rules, &namespaces)
            },
            AtRule(rule) => {
                let lower_name: &str = to_ascii_lower(rule.name);
                match lower_name {
                    "charset" => {
                        if state > STATE_CHARSET {
                            log_css_error(rule.location, "@charset must be the first rule")
                        }
                        // Valid @charset rules are just ignored
                        next_state = state;
                    },
                    "import" => {
                        if state > STATE_IMPORTS {
                            next_state = state;
                            log_css_error(rule.location,
                                          "@import must be before any rule but @charset")
                        } else {
                            next_state = STATE_IMPORTS;
                            log_css_error(rule.location, "@import is not supported yet")  // TODO
                        }
                    },
                    "namespace" => {
                        if state > STATE_NAMESPACES {
                            next_state = state;
                            log_css_error(
                                rule.location,
                                "@namespace must be before any rule but @charset and @import"
                            )
                        } else {
                            next_state = STATE_NAMESPACES;
                            parse_namespace_rule(rule, &mut namespaces)
                        }
                    },
                    _ => {
                        next_state = STATE_BODY;
                        parse_nested_at_rule(lower_name, rule, &mut rules, &namespaces)
                    },
                }
            },
        }
        state = next_state;
    }
    Stylesheet{ style_rules: rules, namespaces: namespaces }
}


pub fn parse_style_rule(rule: QualifiedRule, parent_rules: &mut ~[CSSRule],
                        namespaces: &NamespaceMap) {
    let QualifiedRule{location: location, prelude: prelude, block: block} = rule;
    match selectors::parse_selector_list(prelude, namespaces) {
        Some(selectors) => parent_rules.push(CSSStyleRule(StyleRule{
            selectors: selectors,
            declarations: properties::parse_property_declaration_list(block)
        })),
        None => log_css_error(location, "Unsupported CSS selector."),
    }
}


// lower_name is passed explicitly to avoid computing it twice.
pub fn parse_nested_at_rule(lower_name: &str, rule: AtRule,
                            parent_rules: &mut ~[CSSRule], namespaces: &NamespaceMap) {
    match lower_name {
        "media" => parse_media_rule(rule, parent_rules, namespaces),
        _ => log_css_error(rule.location, fmt!("Unsupported at-rule: @%s", lower_name))
    }
}
