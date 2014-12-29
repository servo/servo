/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::iter::Iterator;
use std::ascii::AsciiExt;
use url::Url;

use encoding::EncodingRef;

use cssparser::{decode_stylesheet_bytes, tokenize, parse_stylesheet_rules, ToCss};
use cssparser::ast::*;
use selectors::{mod, ParserContext};
use properties;
use errors::{ErrorLoggerIterator, log_css_error};
use namespaces::{NamespaceMap, parse_namespace_rule};
use media_queries::{mod, Device, MediaRule};
use font_face::{FontFaceRule, Source, parse_font_face_rule, iter_font_face_rules_inner};
use selector_matching::StylesheetOrigin;


pub struct Stylesheet {
    /// List of rules in the order they were found (important for
    /// cascading order)
    rules: Vec<CSSRule>,
    pub origin: StylesheetOrigin,
}


pub enum CSSRule {
    Style(StyleRule),
    Media(MediaRule),
    FontFace(FontFaceRule),
}


pub struct StyleRule {
    pub selectors: Vec<selectors::Selector>,
    pub declarations: properties::PropertyDeclarationBlock,
}


impl Stylesheet {
    pub fn from_bytes_iter<I: Iterator<Vec<u8>>>(
            mut input: I, base_url: Url, protocol_encoding_label: Option<&str>,
            environment_encoding: Option<EncodingRef>, origin: StylesheetOrigin) -> Stylesheet {
        let mut bytes = vec!();
        // TODO: incremental decoding and tokenization/parsing
        for chunk in input {
            bytes.push_all(chunk.as_slice())
        }
        Stylesheet::from_bytes(bytes.as_slice(), base_url, protocol_encoding_label, environment_encoding, origin)
    }

    pub fn from_bytes(bytes: &[u8],
                      base_url: Url,
                      protocol_encoding_label: Option<&str>,
                      environment_encoding: Option<EncodingRef>,
                      origin: StylesheetOrigin)
                      -> Stylesheet {
        // TODO: bytes.as_slice could be bytes.container_as_bytes()
        let (string, _) = decode_stylesheet_bytes(
            bytes.as_slice(), protocol_encoding_label, environment_encoding);
        Stylesheet::from_str(string.as_slice(), base_url, origin)
    }

    pub fn from_str(css: &str, base_url: Url, origin: StylesheetOrigin) -> Stylesheet {
        static STATE_CHARSET: uint = 1;
        static STATE_IMPORTS: uint = 2;
        static STATE_NAMESPACES: uint = 3;
        static STATE_BODY: uint = 4;

        let parser_context = ParserContext {
            origin: origin,
        };

        let mut state: uint = STATE_CHARSET;

        let mut rules = vec!();
        let mut namespaces = NamespaceMap::new();

        for rule in ErrorLoggerIterator(parse_stylesheet_rules(tokenize(css))) {
            let next_state;  // Unitialized to force each branch to set it.
            match rule {
                Rule::QualifiedRule(rule) => {
                    next_state = STATE_BODY;
                    parse_style_rule(&parser_context, rule, &mut rules, &namespaces, &base_url)
                },
                Rule::AtRule(rule) => {
                    let lower_name = rule.name.as_slice().to_ascii_lower();
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
                            parse_nested_at_rule(&parser_context,
                                                 lower_name.as_slice(),
                                                 rule,
                                                 &mut rules,
                                                 &namespaces,
                                                 &base_url)
                        },
                    }
                },
            }
            state = next_state;
        }
        Stylesheet {
            rules: rules,
            origin: origin,
        }
    }
}

// lower_name is passed explicitly to avoid computing it twice.
pub fn parse_nested_at_rule(context: &ParserContext,
                            lower_name: &str,
                            rule: AtRule,
                            parent_rules: &mut Vec<CSSRule>,
                            namespaces: &NamespaceMap,
                            base_url: &Url) {
    match lower_name {
        "media" => {
            media_queries::parse_media_rule(context, rule, parent_rules, namespaces, base_url)
        }
        "font-face" => parse_font_face_rule(rule, parent_rules, base_url),
        _ => log_css_error(rule.location,
                           format!("Unsupported at-rule: @{:s}", lower_name).as_slice())
    }
}

pub fn parse_style_rule(context: &ParserContext,
                        rule: QualifiedRule,
                        parent_rules: &mut Vec<CSSRule>,
                        namespaces: &NamespaceMap,
                        base_url: &Url) {
    let QualifiedRule {
        location,
        prelude,
        block
    } = rule;
    // FIXME: avoid doing this for valid selectors
    let serialized = prelude.to_css_string();
    match selectors::parse_selector_list(context, prelude.into_iter(), namespaces) {
        Ok(selectors) => parent_rules.push(CSSRule::Style(StyleRule{
            selectors: selectors,
            declarations: properties::parse_property_declaration_list(block.into_iter(), base_url)
        })),
        Err(()) => log_css_error(location, format!(
            "Invalid/unsupported selector: {}", serialized).as_slice()),
    }
}

pub fn iter_style_rules<'a>(rules: &[CSSRule], device: &media_queries::Device,
                            callback: |&StyleRule|) {
    for rule in rules.iter() {
        match *rule {
            CSSRule::Style(ref rule) => callback(rule),
            CSSRule::Media(ref rule) => if rule.media_queries.evaluate(device) {
                iter_style_rules(rule.rules.as_slice(), device, |s| callback(s))
            },
            CSSRule::FontFace(_) => {},
        }
    }
}

pub fn iter_stylesheet_media_rules(stylesheet: &Stylesheet, callback: |&MediaRule|) {
    for rule in stylesheet.rules.iter() {
        match *rule {
            CSSRule::Media(ref rule) => callback(rule),
            _ => {}
        }
    }
}

#[inline]
pub fn iter_stylesheet_style_rules(stylesheet: &Stylesheet, device: &media_queries::Device,
                                   callback: |&StyleRule|) {
    iter_style_rules(stylesheet.rules.as_slice(), device, callback)
}


#[inline]
pub fn iter_font_face_rules(stylesheet: &Stylesheet, device: &Device,
                            callback: |family: &str, source: &Source|) {
    iter_font_face_rules_inner(stylesheet.rules.as_slice(), device, callback)
}
