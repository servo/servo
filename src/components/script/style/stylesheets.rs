/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::iterator::Iterator;
use std::ascii::StrAsciiExt;
use cssparser::*;
use style::selectors;
use style::properties;
use style::errors::{ErrorLoggerIterator, log_css_error};
use style::namespaces::{NamespaceMap, parse_namespace_rule};
use style::media_queries::{MediaRule, parse_media_rule};
use style::media_queries;


pub struct Stylesheet {
    rules: ~[CSSRule],
    namespaces: NamespaceMap,
}


pub enum CSSRule {
    CSSStyleRule(StyleRule),
    CSSMediaRule(MediaRule),
}


pub struct StyleRule {
    selectors: ~[@selectors::Selector],
    declarations: properties::PropertyDeclarationBlock,
}


pub fn parse_stylesheet(css: &str) -> Stylesheet {
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
                let lower_name = rule.name.to_ascii_lower();
                match lower_name.as_slice() {
                    "charset" => {
                        if state > STATE_CHARSET {
                            log_css_error(rule.location, "@charset must be the first rule")
                        }
                        // Valid @charset rules are just ignored
                        next_state = STATE_IMPORTS;
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
    Stylesheet{ rules: rules, namespaces: namespaces }
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


impl Stylesheet {
    pub fn iter_style_rules<'a>(&'a self, device: &'a media_queries::Device)
                                -> StyleRuleIterator<'a> {
        StyleRuleIterator { device: device, stack: ~[(self.rules.as_slice(), 0)] }
    }
}

struct StyleRuleIterator<'self> {
    device: &'self media_queries::Device,
    // FIXME: I couldn’t get this to borrow-check with a stack of VecIterator
    stack: ~[(&'self [CSSRule], uint)],
}

impl<'self> Iterator<&'self StyleRule> for StyleRuleIterator<'self> {
    fn next(&mut self) -> Option<&'self StyleRule> {
        loop {
            match self.stack.pop_opt() {
                None => return None,
                Some((rule_list, i)) => {
                    if i + 1 < rule_list.len() {
                        self.stack.push((rule_list, i + 1))
                    }
                    match rule_list[i] {
                        CSSStyleRule(ref rule) => return Some(rule),
                        CSSMediaRule(ref rule) => {
                            if rule.media_queries.evaluate(self.device) {
                                self.stack.push((rule.rules.as_slice(), 0))
                            }
                        }
                    }
                }
            }
        }
    }
}
