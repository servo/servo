/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::ast::*;
use cssparser::ast::ComponentValue::*;
use cssparser::parse_declaration_list;
use errors::{ErrorLoggerIterator, log_css_error};
use std::ascii::AsciiExt;
use parsing_utils::{BufferedIter, ParserIter, parse_slice_comma_separated};
use properties::longhands::font_family::parse_one_family;
use properties::computed_values::font_family::FontFamily::FamilyName;
use stylesheets::CSSRule;
use media_queries::Device;
use url::{Url, UrlParser};


pub fn iter_font_face_rules_inner(rules: &[CSSRule], device: &Device,
                                    callback: |family: &str, source: &Source|) {
    for rule in rules.iter() {
        match *rule {
            CSSRule::Style(_) => {},
            CSSRule::Media(ref rule) => if rule.media_queries.evaluate(device) {
                iter_font_face_rules_inner(rule.rules.as_slice(), device, |f, s| callback(f, s))
            },
            CSSRule::FontFace(ref rule) => {
                for source in rule.sources.iter() {
                    callback(rule.family.as_slice(), source)
                }
            },
        }
    }
}

#[deriving(Clone)]
pub enum Source {
    Url(UrlSource),
    Local(String),
}

#[deriving(Clone)]
pub struct UrlSource {
    pub url: Url,
    pub format_hints: Vec<String>,
}

pub struct FontFaceRule {
    pub family: String,
    pub sources: Vec<Source>,
}

pub fn parse_font_face_rule(rule: AtRule, parent_rules: &mut Vec<CSSRule>, base_url: &Url) {
    if rule.prelude.as_slice().skip_whitespace().next().is_some() {
        log_css_error(rule.location, "@font-face prelude contains unexpected characters");
        return;
    }

    let block = match rule.block {
        Some(block) => block,
        None => {
            log_css_error(rule.location, "Invalid @font-face rule");
            return
        }
    };

    let mut maybe_family = None;
    let mut maybe_sources = None;

    for item in ErrorLoggerIterator(parse_declaration_list(block.into_iter())) {
        match item {
            DeclarationListItem::AtRule(rule) => log_css_error(
                rule.location, format!("Unsupported at-rule in declaration list: @{}", rule.name).as_slice()),
            DeclarationListItem::Declaration(Declaration{ location, name, value, important }) => {
                if important {
                    log_css_error(location, "!important is not allowed on @font-face descriptors");
                    continue
                }
                let name_lower = name.as_slice().to_ascii_lower();
                match name_lower.as_slice() {
                    "font-family" => {
                        let iter = &mut BufferedIter::new(value.as_slice().skip_whitespace());
                        match parse_one_family(iter) {
                            Ok(FamilyName(name)) => {
                                maybe_family = Some(name);
                            },
                            // This also includes generic family names:
                            _ => log_css_error(location, "Invalid font-family in @font-face"),
                        }
                    },
                    "src" => {
                        match parse_slice_comma_separated(
                                value.as_slice(), |iter| parse_one_src(iter, base_url)) {
                            Ok(sources) => maybe_sources = Some(sources),
                            Err(()) => log_css_error(location, "Invalid src in @font-face"),
                        };
                    },
                    _ => {
                        log_css_error(location, format!("Unsupported declaration {}", name).as_slice());
                    }
                }
            }
        }
    }

    match (maybe_family, maybe_sources) {
        (Some(family), Some(sources)) => parent_rules.push(CSSRule::FontFace(FontFaceRule {
            family: family,
            sources: sources,
        })),
        (None, _) => log_css_error(rule.location, "@font-face without a font-family descriptor"),
        _ => log_css_error(rule.location, "@font-face without an src descriptor"),
    }
}


fn parse_one_src(iter: ParserIter, base_url: &Url) -> Result<Source, ()> {
    let url = match iter.next() {
        // Parsing url()
        Some(&URL(ref url)) => {
            UrlParser::new().base_url(base_url).parse(url.as_slice()).unwrap_or_else(
                |_error| Url::parse("about:invalid").unwrap())
        },
        // Parsing local() with early return()
        Some(&Function(ref name, ref arguments)) => {
            if name.as_slice().eq_ignore_ascii_case("local") {
                let iter = &mut BufferedIter::new(arguments.as_slice().skip_whitespace());
                match parse_one_family(iter) {
                    Ok(FamilyName(name)) => return Ok(Source::Local(name)),
                    _ => return Err(())
                }
            }
            return Err(())
        },
        _ => return Err(())
    };

    // Parsing optional format()
    let format_hints = match iter.next() {
        Some(&Function(ref name, ref arguments)) => {
            if !name.as_slice().eq_ignore_ascii_case("format") {
                return Err(())
            }
            try!(parse_slice_comma_separated(arguments.as_slice(), parse_one_format))
        }
        Some(component_value) => {
            iter.push_back(component_value);
            vec![]
        }
        None => vec![],
    };

    Ok(Source::Url(UrlSource {
        url: url,
        format_hints: format_hints,
    }))
}


fn parse_one_format(iter: ParserIter) -> Result<String, ()> {
    match iter.next() {
        Some(&QuotedString(ref value)) => {
            if iter.next().is_none() {
                Ok(value.clone())
            } else {
                Err(())
            }
        }
        _ => Err(())
    }
}
