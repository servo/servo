/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::iter::Iterator;
use std::ascii::StrAsciiExt;
use extra::url::Url;

use encoding::EncodingRef;

use cssparser::{decode_stylesheet_bytes, tokenize, parse_stylesheet_rules, ToCss};
use cssparser::ast::*;
use selectors;
use properties;
use errors::{ErrorLoggerIterator, log_css_error};
use namespaces::{NamespaceMap, parse_namespace_rule};
use media_queries::{MediaRule, parse_media_rule};
use media_queries;


pub struct Stylesheet {
    /// List of rules in the order they were found (important for
    /// cascading order)
    rules: ~[CSSRule],
    namespaces: NamespaceMap,
    encoding: EncodingRef,
    base_url: Url,
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
    pub fn from_bytes_iter<I: Iterator<~[u8]>>(
            mut input: I, base_url: Url, protocol_encoding_label: Option<&str>,
            environment_encoding: Option<EncodingRef>) -> Stylesheet {
        let mut bytes = ~[];
        // TODO: incremental decoding and tokinization/parsing
        for chunk in input {
            bytes.push_all(chunk)
        }
        Stylesheet::from_bytes(bytes, base_url, protocol_encoding_label, environment_encoding)
    }

    pub fn from_bytes(
            bytes: &[u8], base_url: Url, protocol_encoding_label: Option<&str>,
            environment_encoding: Option<EncodingRef>) -> Stylesheet {
        let (string, used_encoding) = decode_stylesheet_bytes(
            bytes, protocol_encoding_label, environment_encoding);
        Stylesheet::from_str(string, base_url, used_encoding)
    }

    pub fn from_str(css: &str, base_url: Url, encoding: EncodingRef) -> Stylesheet {
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
        Stylesheet{ rules: rules, namespaces: namespaces, encoding: encoding, base_url: base_url }
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
                            callback: |&StyleRule|) {
    for rule in rules.iter() {
        match *rule {
            CSSStyleRule(ref rule) => callback(rule),
            CSSMediaRule(ref rule) => if rule.media_queries.evaluate(device) {
                iter_style_rules(rule.rules.as_slice(), device, |s| callback(s))
            }
        }
    }
}
