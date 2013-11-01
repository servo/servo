/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::str;
use std::iter::Iterator;
use std::ascii::StrAsciiExt;
use cssparser::*;
use selectors;
use properties;
use errors::{ErrorLoggerIterator, log_css_error};
use namespaces::{NamespaceMap, parse_namespace_rule};
use media_queries::{MediaRule, parse_media_rule};
use media_queries;


pub struct Stylesheet {
    rules: ~[CSSRule],
    namespaces: NamespaceMap,
}


pub enum CSSRule {
    CSSStyleRule(StyleRule),
    CSSMediaRule(MediaRule),
}


pub struct StyleRule {
    selectors: ~[selectors::Selector],
    declarations: properties::PropertyDeclarationBlock,
}


impl Stylesheet {
    pub fn from_iter<I: Iterator<~[u8]>>(input: I) -> Stylesheet {
        let mut string = ~"";
        let mut input = input;
        // TODO: incremental tokinization/parsing
        for chunk in input {
            // Assume UTF-8. This fails on invalid UTF-8
            // TODO: support character encodings (use rust-encodings in rust-cssparser)
            string.push_str(str::from_utf8_owned(chunk))
        }
        Stylesheet::from_str(string)
    }

    pub fn from_str(css: &str) -> Stylesheet {
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
                                // TODO: support @import
                                log_css_error(rule.location, "@import is not supported yet")
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
}


pub fn parse_style_rule(rule: QualifiedRule, parent_rules: &mut ~[CSSRule],
                        namespaces: &NamespaceMap) {
    let QualifiedRule{location: location, prelude: prelude, block: block} = rule;
    // FIXME: avoid doing this for valid selectors
    let serialized = prelude.iter().to_css();
    match selectors::parse_selector_list(prelude, namespaces) {
        Some(selectors) => parent_rules.push(CSSStyleRule(StyleRule{
            selectors: selectors,
            declarations: properties::parse_property_declaration_list(block.move_iter())
        })),
        None => log_css_error(location, format!(
            "Invalid/unsupported selector: {}", serialized)),
    }
}


// lower_name is passed explicitly to avoid computing it twice.
pub fn parse_nested_at_rule(lower_name: &str, rule: AtRule,
                            parent_rules: &mut ~[CSSRule], namespaces: &NamespaceMap) {
    match lower_name {
        "media" => parse_media_rule(rule, parent_rules, namespaces),
        _ => log_css_error(rule.location, format!("Unsupported at-rule: @{:s}", lower_name))
    }
}


pub fn iter_style_rules<'a>(rules: &[CSSRule], device: &media_queries::Device,
                            callback: &fn(&StyleRule)) {
    for rule in rules.iter() {
        match *rule {
            CSSStyleRule(ref rule) => callback(rule),
            CSSMediaRule(ref rule) => if rule.media_queries.evaluate(device) {
                iter_style_rules(rule.rules.as_slice(), device, |s| callback(s))
            }
        }
    }
}
