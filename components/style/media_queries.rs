/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ascii::StrAsciiExt;
use cssparser::parse_rule_list;
use cssparser::ast::*;

use errors::{ErrorLoggerIterator, log_css_error};
use stylesheets::{CSSRule, CSSMediaRule, parse_style_rule, parse_nested_at_rule};
use namespaces::NamespaceMap;
use url::Url;


pub struct MediaRule {
    pub media_queries: MediaQueryList,
    pub rules: Vec<CSSRule>,
}


pub struct MediaQueryList {
    // "not all" is omitted from the list.
    // An empty list never matches.
    media_queries: Vec<MediaQuery>
}

// For now, this is a "Level 2 MQ", ie. a media type.
pub struct MediaQuery {
    media_type: MediaQueryType,
    // TODO: Level 3 MQ expressions
}


pub enum MediaQueryType {
    All,  // Always true
    MediaType(MediaType),
}

#[deriving(PartialEq)]
pub enum MediaType {
    Screen,
    Print,
}

pub struct Device {
    pub media_type: MediaType,
    // TODO: Level 3 MQ data: viewport size, etc.
}


pub fn parse_media_rule(rule: AtRule, parent_rules: &mut Vec<CSSRule>,
                        namespaces: &NamespaceMap, base_url: &Url) {
    let media_queries = parse_media_query_list(rule.prelude.as_slice());
    let block = match rule.block {
        Some(block) => block,
        None => {
            log_css_error(rule.location, "Invalid @media rule");
            return
        }
    };
    let mut rules = vec!();
    for rule in ErrorLoggerIterator(parse_rule_list(block.into_iter())) {
        match rule {
            QualifiedRule(rule) => parse_style_rule(rule, &mut rules, namespaces, base_url),
            AtRule(rule) => parse_nested_at_rule(
                rule.name.as_slice().to_ascii_lower().as_slice(), rule, &mut rules, namespaces, base_url),
        }
    }
    parent_rules.push(CSSMediaRule(MediaRule {
        media_queries: media_queries,
        rules: rules,
    }))
}


pub fn parse_media_query_list(input: &[ComponentValue]) -> MediaQueryList {
    let iter = &mut input.skip_whitespace();
    let mut next = iter.next();
    if next.is_none() {
        return MediaQueryList{ media_queries: vec!(MediaQuery{media_type: All}) }
    }
    let mut queries = vec!();
    loop {
        let mq = match next {
            Some(&Ident(ref value)) => {
                match value.as_slice().to_ascii_lower().as_slice() {
                    "screen" => Some(MediaQuery{ media_type: MediaType(Screen) }),
                    "print" => Some(MediaQuery{ media_type: MediaType(Print) }),
                    "all" => Some(MediaQuery{ media_type: All }),
                    _ => None
                }
            },
            _ => None
        };
        match iter.next() {
            None => {
                for mq in mq.into_iter() {
                    queries.push(mq);
                }
                return MediaQueryList{ media_queries: queries }
            },
            Some(&Comma) => {
                for mq in mq.into_iter() {
                    queries.push(mq);
                }
            },
            // Ingnore this comma-separated part
            _ => loop {
                match iter.next() {
                    Some(&Comma) => break,
                    None => return MediaQueryList{ media_queries: queries },
                    _ => (),
                }
            },
        }
        next = iter.next();
    }
}


impl MediaQueryList {
    pub fn evaluate(&self, device: &Device) -> bool {
        self.media_queries.iter().any(|mq| {
            match mq.media_type {
                MediaType(media_type) => media_type == device.media_type,
                All => true,
            }
            // TODO: match Level 3 expressions
        })
    }
}
