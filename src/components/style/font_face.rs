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
    pub format_hint: FontFaceFormat,
}

pub struct FontFaceSourceLine {
    pub sources: Vec<FontFaceSource>
}

pub struct FontFaceRule {
    pub family: String,
    pub source_lines: Vec<FontFaceSourceLine>,
}

pub fn parse_font_face_rule(rule: AtRule, parent_rules: &mut Vec<CSSRule>, base_url: &Url) {

    let mut maybe_family = None;
    let mut source_lines = vec!();

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

    for item in ErrorLoggerIterator(parse_declaration_list(block.move_iter())) {
        match item {
            DeclAtRule(rule) => log_css_error(
                rule.location, format!("Unsupported at-rule in declaration list: @{:s}", rule.name).as_slice()),
            Declaration(Declaration{ location: location, name: name, value: value, important: _}) => {

                let name_lower = name.as_slice().to_ascii_lower();
                match name_lower.as_slice() {
                    "font-family" => {
                        // FIXME(#2802): Share code with the font-family parser.
                        match one_component_value(value.as_slice()) {
                            Some(&String(ref string_value)) => {
                                maybe_family = Some(string_value.clone());
                            },
                            _ => {
                                log_css_error(location, format!("Unsupported font-family string {:s}", name).as_slice());
                            }
                        }
                    },
                    "src" => {
                        let mut iter = value.as_slice().skip_whitespace();
                        let mut sources = vec!();
                        let mut syntax_error = false;

                        'outer: loop {

                            // url() or local() should be next
                            let url = match iter.next() {
                                Some(&URL(ref string_value)) => {
                                    parse_url(string_value.as_slice(), Some(base_url.clone()))
                                },
                                _ => {
                                    log_css_error(location, "Unsupported declaration (local font face is not supported yet)");
                                    syntax_error = true;
                                    break;
                                }
                            };

                            // optional format, or comma to start loop again
                            let mut next_token = iter.next();
                            match next_token {
                                Some(&Function(ref string_value, ref values)) => {
                                    match string_value.as_slice() {
                                        "format" => {
                                            let maybe_format_hint_string = one_component_value(
                                                                            values.as_slice()).and_then(|c| {
                                                match c {
                                                    &String(ref s) => Some(s.as_slice().to_ascii_lower()),
                                                    _ => None,
                                                }
                                            });
                                            
                                            match maybe_format_hint_string {
                                                Some(ref format_hint_string) => {
                                                    let format_hints: Vec<&str> = format_hint_string.as_slice().split(',').collect();

                                                    for format_hint in format_hints.iter() {
                                                        let format_hint = format_hint.trim();
                                                        
                                                        let hint = match format_hint.as_slice() {
                                                            "embedded-opentype" => EotFormat,
                                                            "woff" => WoffFormat,
                                                            "truetype" | "opentype" => TtfFormat,
                                                            "svg" => SvgFormat,
                                                            _ => UnknownFormat,
                                                        };

                                                        if hint == UnknownFormat {
                                                            log_css_error(location, 
                                                                    format!("Unknown font format {}", format_hint).as_slice());
                                                            syntax_error = true;
                                                            break 'outer;
                                                        }

                                                        let source = FontFaceSource {
                                                            url: url.clone(),
                                                            format_hint: hint,
                                                        };
                                                        sources.push(source);
                                                    }
                                                },
                                                None => {
                                                    log_css_error(location, 
                                                                    format!("Unsupported token {}", string_value).as_slice());
                                                    syntax_error = true;
                                                    break;                                                   
                                                }
                                            }
                                        },
                                        _ => {
                                            log_css_error(location, 
                                                            format!("Unsupported token {}", string_value).as_slice());
                                            syntax_error = true;
                                            break;
                                        }
                                    }
                                    next_token = iter.next();
                                },
                                _ => {
                                    let source = FontFaceSource {
                                        url: url,
                                        format_hint: UnknownFormat,
                                    };
                                    sources.push(source);
                                }
                            }

                            // after url or optional format, comes comma or end
                            match next_token {
                                Some(&Comma) => {},
                                None => break,
                                _ => {
                                    log_css_error(location, "Unexpected token type");
                                    syntax_error = true;
                                    break;
                                }
                            }
                        }

                        if !syntax_error {
                            assert!(sources.len() > 0);

                            let source_line = FontFaceSourceLine {
                                sources: sources
                            };
                            source_lines.push(source_line);
                        }
                    },
                    _ => {
                        log_css_error(location, format!("Unsupported declaration {:s}", name).as_slice());
                    }
                }
            }
        }
    }

    if maybe_family.is_some() && source_lines.len() > 0 {
        let font_face_rule = FontFaceRule {
            family: maybe_family.unwrap(),
            source_lines: source_lines,
        };
        parent_rules.push(CSSFontFaceRule(font_face_rule));
    }
}