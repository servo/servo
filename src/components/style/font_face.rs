/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::ast::*;
use cssparser::parse_declaration_list;
use errors::{ErrorLoggerIterator, log_css_error};
use std::ascii::StrAsciiExt;
use parsing_utils::one_component_value;
use stylesheets::{CSSRule, CSSFontFaceRule};
use url::Url;
use servo_util::url::parse_url;

#[deriving(PartialEq)]
pub enum FontFaceFormat {
    UnknownFormat,
    WoffFormat,
    TtfFormat,
    SvgFormat,
    EotFormat,
}

pub struct FontFaceSource {
    pub url: Url,
    pub format: FontFaceFormat,
}

pub struct FontFaceRule {
    pub family: String,
    pub sources: Vec<FontFaceSource>,
}

pub fn parse_font_face_rule(rule: AtRule, parent_rules: &mut Vec<CSSRule>, base_url: &Url) {
    let mut font_face_rule = FontFaceRule {
        family: "".to_string(),
        sources: vec!()
    };

    let block = match rule.block {
        Some(block) => block,
        None => {
            log_css_error(rule.location, "Invalid @font-face rule");
            return
        }
    };

    for item in ErrorLoggerIterator(parse_declaration_list(block.move_iter())) {
        match item {
            DeclAtRule(rule) => log_css_error(
                rule.location, format!("Unsupported at-rule in declaration list: @{:s}", rule.name).as_slice()),
            Declaration(Declaration{ location: location, name: name, value: value, important: _}) => {

                let name_lower = name.as_slice().to_ascii_lower();
                match name_lower.as_slice() {
                    "font-family" => {
                        match one_component_value(value.as_slice()) {
                            Some(&String(ref string_value)) => {
                                font_face_rule.family = string_value.clone();
                            },
                            _ => {
                                log_css_error(location, format!("Unsupported font-family string {:s}", name).as_slice());
                            }
                        }
                    },
                    "src" => {
                        for component_value in value.as_slice().skip_whitespace() {
                            match component_value {
                                &URL(ref string_value) => {
                                    let font_url = parse_url(string_value.as_slice(), Some(base_url.clone()));
                                    let src = FontFaceSource { url: font_url, format: UnknownFormat };
                                    font_face_rule.sources.push(src);
                                },
                                &Function(ref string_value, ref values) => {
                                    match string_value.as_slice() {
                                        "format" => {
                                            let format = one_component_value(values.as_slice()).and_then(|c| {
                                                match c {
                                                    &String(ref format_string) => Some(format_string.as_slice().to_ascii_lower()),
                                                    _ => None,
                                                }
                                            });
                                            match font_face_rule.sources.mut_last() {
                                                Some(source) => {
                                                    source.format = match format.unwrap_or("".to_string()).as_slice() {
                                                        "embedded-opentype" => EotFormat,
                                                        "woff" => WoffFormat,
                                                        "truetype" | "opentype" => TtfFormat,
                                                        "svg" => SvgFormat,
                                                        _ => UnknownFormat,
                                                    }
                                                }
                                                None => {}
                                            };
                                        },
                                        "local" => {
                                            log_css_error(location, "local font face not supported yet!");
                                        },
                                        _ => {
                                            log_css_error(location, format!("Unsupported declaration {}", string_value).as_slice());
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    },
                    _ => {
                        log_css_error(location, format!("Unsupported declaration {:s}", name).as_slice());
                    }
                }
            }
        }
    }

    parent_rules.push(CSSFontFaceRule(font_face_rule));
}