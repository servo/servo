/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::iter::Iterator;
use std::ascii::AsciiExt;
use url::Url;

use encoding::EncodingRef;

use cssparser::{Parser, decode_stylesheet_bytes,
                QualifiedRuleParser, AtRuleParser, RuleListParser, AtRuleType};
use string_cache::{Atom, Namespace};
use selectors::{Selector, parse_selector_list};
use parser::{ParserContext, log_css_error};
use properties::{PropertyDeclarationBlock, parse_property_declaration_list};
use media_queries::{self, Device, MediaQueryList, parse_media_query_list};
use font_face::{FontFaceRule, Source, parse_font_face_block, iter_font_face_rules_inner};


#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Origin {
    UserAgent,
    Author,
    User,
}


#[derive(Debug, PartialEq)]
pub struct Stylesheet {
    /// List of rules in the order they were found (important for
    /// cascading order)
    rules: Vec<CSSRule>,
    pub origin: Origin,
}


#[derive(Debug, PartialEq)]
pub enum CSSRule {
    Charset(String),
    Namespace(Option<String>, Namespace),
    Style(StyleRule),
    Media(MediaRule),
    FontFace(FontFaceRule),
}

#[derive(Debug, PartialEq)]
pub struct MediaRule {
    pub media_queries: MediaQueryList,
    pub rules: Vec<CSSRule>,
}


#[derive(Debug, PartialEq)]
pub struct StyleRule {
    pub selectors: Vec<Selector>,
    pub declarations: PropertyDeclarationBlock,
}


impl Stylesheet {
    pub fn from_bytes_iter<I: Iterator<Item=Vec<u8>>>(
            input: I, base_url: Url, protocol_encoding_label: Option<&str>,
            environment_encoding: Option<EncodingRef>, origin: Origin) -> Stylesheet {
        let mut bytes = vec![];
        // TODO: incremental decoding and tokenization/parsing
        for chunk in input {
            bytes.push_all(&chunk)
        }
        Stylesheet::from_bytes(&bytes, base_url, protocol_encoding_label,
                               environment_encoding, origin)
    }

    pub fn from_bytes(bytes: &[u8],
                      base_url: Url,
                      protocol_encoding_label: Option<&str>,
                      environment_encoding: Option<EncodingRef>,
                      origin: Origin)
                      -> Stylesheet {
        // TODO: bytes.as_slice could be bytes.container_as_bytes()
        let (string, _) = decode_stylesheet_bytes(
            bytes, protocol_encoding_label, environment_encoding);
        Stylesheet::from_str(&string, base_url, origin)
    }

    pub fn from_str<'i>(css: &'i str, base_url: Url, origin: Origin) -> Stylesheet {
        let rule_parser = TopLevelRuleParser {
            context: ParserContext::new(origin, &base_url),
            state: Cell::new(State::Start),
        };
        let mut input = Parser::new(css);
        let mut iter = RuleListParser::new_for_stylesheet(&mut input, rule_parser);
        let mut rules = Vec::new();
        while let Some(result) = iter.next() {
            match result {
                Ok(rule) => {
                    if let CSSRule::Namespace(ref prefix, ref namespace) = rule {
                        if let Some(prefix) = prefix.as_ref() {
                            iter.parser.context.namespaces.prefix_map.insert(
                                prefix.clone(), namespace.clone());
                        } else {
                            iter.parser.context.namespaces.default = Some(namespace.clone());
                        }
                    }
                    rules.push(rule);
                }
                Err(range) => {
                    let pos = range.start;
                    let message = format!("Invalid rule: '{}'", iter.input.slice(range));
                    log_css_error(iter.input, pos, &*message);
                }
            }
        }
        Stylesheet {
            origin: origin,
            rules: rules,
        }
    }
}


fn parse_nested_rules(context: &ParserContext, input: &mut Parser) -> Vec<CSSRule> {
    let mut iter = RuleListParser::new_for_nested_rule(input, NestedRuleParser { context: context });
    let mut rules = Vec::new();
    while let Some(result) = iter.next() {
        match result {
            Ok(rule) => rules.push(rule),
            Err(range) => {
                let pos = range.start;
                let message = format!("Unsupported rule: '{}'", iter.input.slice(range));
                log_css_error(iter.input, pos, &*message);
            }
        }
    }
    rules
}


struct TopLevelRuleParser<'a> {
    context: ParserContext<'a>,
    state: Cell<State>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy)]
enum State {
    Start = 1,
    Imports = 2,
    Namespaces = 3,
    Body = 4,
}


enum AtRulePrelude {
    FontFace,
    Media(MediaQueryList),
}


impl<'a> AtRuleParser for TopLevelRuleParser<'a> {
    type Prelude = AtRulePrelude;
    type AtRule = CSSRule;

    fn parse_prelude(&self, name: &str, input: &mut Parser)
                     -> Result<AtRuleType<AtRulePrelude, CSSRule>, ()> {
        match_ignore_ascii_case! { name,
            "charset" => {
                if self.state.get() <= State::Start {
                    // Valid @charset rules are just ignored
                    self.state.set(State::Imports);
                    let charset = try!(input.expect_string()).into_owned();
                    return Ok(AtRuleType::WithoutBlock(CSSRule::Charset(charset)))
                } else {
                    return Err(())  // "@charset must be the first rule"
                }
            },
            "import" => {
                if self.state.get() <= State::Imports {
                    self.state.set(State::Imports);
                    // TODO: support @import
                    return Err(())  // "@import is not supported yet"
                } else {
                    return Err(())  // "@import must be before any rule but @charset"
                }
            },
            "namespace" => {
                if self.state.get() <= State::Namespaces {
                    self.state.set(State::Namespaces);

                    let prefix = input.try(|input| input.expect_ident()).ok().map(|p| p.into_owned());
                    let url = Namespace(Atom::from_slice(&*try!(input.expect_url_or_string())));
                    return Ok(AtRuleType::WithoutBlock(CSSRule::Namespace(prefix, url)))
                } else {
                    return Err(())  // "@namespace must be before any rule but @charset and @import"
                }
            }
            _ => {}
        }

        self.state.set(State::Body);
        AtRuleParser::parse_prelude(&NestedRuleParser { context: &self.context }, name, input)
    }

    #[inline]
    fn parse_block(&self, prelude: AtRulePrelude, input: &mut Parser) -> Result<CSSRule, ()> {
        AtRuleParser::parse_block(&NestedRuleParser { context: &self.context }, prelude, input)
    }
}


impl<'a> QualifiedRuleParser for TopLevelRuleParser<'a> {
    type Prelude = Vec<Selector>;
    type QualifiedRule = CSSRule;

    #[inline]
    fn parse_prelude(&self, input: &mut Parser) -> Result<Vec<Selector>, ()> {
        self.state.set(State::Body);
        QualifiedRuleParser::parse_prelude(&NestedRuleParser { context: &self.context }, input)
    }

    #[inline]
    fn parse_block(&self, prelude: Vec<Selector>, input: &mut Parser) -> Result<CSSRule, ()> {
        QualifiedRuleParser::parse_block(&NestedRuleParser { context: &self.context },
                                         prelude, input)
    }
}


struct NestedRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
}


impl<'a, 'b> AtRuleParser for NestedRuleParser<'a, 'b> {
    type Prelude = AtRulePrelude;
    type AtRule = CSSRule;

    fn parse_prelude(&self, name: &str, input: &mut Parser)
                     -> Result<AtRuleType<AtRulePrelude, CSSRule>, ()> {
        match_ignore_ascii_case! { name,
            "media" => {
                let media_queries = parse_media_query_list(input);
                Ok(AtRuleType::WithBlock(AtRulePrelude::Media(media_queries)))
            },
            "font-face" => {
                Ok(AtRuleType::WithBlock(AtRulePrelude::FontFace))
            }
            _ => Err(())
        }
    }

    fn parse_block(&self, prelude: AtRulePrelude, input: &mut Parser) -> Result<CSSRule, ()> {
        match prelude {
            AtRulePrelude::FontFace => {
                parse_font_face_block(self.context, input).map(CSSRule::FontFace)
            }
            AtRulePrelude::Media(media_queries) => {
                Ok(CSSRule::Media(MediaRule {
                    media_queries: media_queries,
                    rules: parse_nested_rules(self.context, input),
                }))
            }
        }
    }
}


impl<'a, 'b> QualifiedRuleParser for NestedRuleParser<'a, 'b> {
    type Prelude = Vec<Selector>;
    type QualifiedRule = CSSRule;

    fn parse_prelude(&self, input: &mut Parser) -> Result<Vec<Selector>, ()> {
        parse_selector_list(self.context, input)
    }

    fn parse_block(&self, prelude: Vec<Selector>, input: &mut Parser) -> Result<CSSRule, ()> {
        Ok(CSSRule::Style(StyleRule {
            selectors: prelude,
            declarations: parse_property_declaration_list(self.context, input)
        }))
    }
}


pub fn iter_style_rules<'a, F>(rules: &[CSSRule], device: &media_queries::Device,
                               callback: &mut F) where F: FnMut(&StyleRule) {
    for rule in rules.iter() {
        match *rule {
            CSSRule::Style(ref rule) => callback(rule),
            CSSRule::Media(ref rule) => if rule.media_queries.evaluate(device) {
                iter_style_rules(&rule.rules, device, callback)
            },
            CSSRule::FontFace(..) |
            CSSRule::Charset(..) |
            CSSRule::Namespace(..) => {}
        }
    }
}

pub fn iter_stylesheet_media_rules<F>(stylesheet: &Stylesheet, mut callback: F) where F: FnMut(&MediaRule) {
    for rule in stylesheet.rules.iter() {
        match *rule {
            CSSRule::Media(ref rule) => callback(rule),
            CSSRule::Style(..) |
            CSSRule::FontFace(..) |
            CSSRule::Charset(..) |
            CSSRule::Namespace(..) => {}
        }
    }
}

#[inline]
pub fn iter_stylesheet_style_rules<F>(stylesheet: &Stylesheet, device: &media_queries::Device,
                                      mut callback: F) where F: FnMut(&StyleRule) {
    iter_style_rules(&stylesheet.rules, device, &mut callback)
}


#[inline]
pub fn iter_font_face_rules<F>(stylesheet: &Stylesheet, device: &Device,
                               callback: &F) where F: Fn(&str, &Source) {
    iter_font_face_rules_inner(&stylesheet.rules, device, callback)
}


#[test]
fn test_parse_stylesheet() {
    use std::sync::Arc;
    use cssparser;
    use selectors::*;
    use string_cache::Atom;
    use properties::{PropertyDeclaration, DeclaredValue, longhands};
    use std::borrow::ToOwned;

    let css = r"
        @namespace url(http://www.w3.org/1999/xhtml);
        /* FIXME: only if scripting is enabled */
        input[type=hidden i] { display: none !important; }
        html , body /**/ { display: block; }
        #d1 > .ok { background: blue; }
    ";
    let url = Url::parse("about::test").unwrap();
    let stylesheet = Stylesheet::from_str(css, url, Origin::UserAgent);
    assert_eq!(stylesheet, Stylesheet {
        origin: Origin::UserAgent,
        rules: vec![
            CSSRule::Namespace(None, ns!(HTML)),
            CSSRule::Style(StyleRule {
                selectors: vec![
                    Selector {
                        compound_selectors: Arc::new(CompoundSelector {
                            simple_selectors: vec![
                                SimpleSelector::Namespace(ns!(HTML)),
                                SimpleSelector::LocalName(LocalName {
                                    name: atom!(input),
                                    lower_name: atom!(input),
                                }),
                                SimpleSelector::AttrEqual(AttrSelector {
                                    name: atom!(type),
                                    lower_name: atom!(type),
                                    namespace: NamespaceConstraint::Specific(ns!("")),
                                }, "hidden".to_owned(), CaseSensitivity::CaseInsensitive)
                            ],
                            next: None,
                        }),
                        pseudo_element: None,
                        specificity: (0 << 20) + (1 << 10) + (1 << 0),
                    },
                ],
                declarations: PropertyDeclarationBlock {
                    normal: Arc::new(vec![]),
                    important: Arc::new(vec![
                        PropertyDeclaration::Display(DeclaredValue::SpecifiedValue(
                            longhands::display::SpecifiedValue::none)),
                    ]),
                },
            }),
            CSSRule::Style(StyleRule {
                selectors: vec![
                    Selector {
                        compound_selectors: Arc::new(CompoundSelector {
                            simple_selectors: vec![
                                SimpleSelector::Namespace(ns!(HTML)),
                                SimpleSelector::LocalName(LocalName {
                                    name: atom!(html),
                                    lower_name: atom!(html),
                                }),
                            ],
                            next: None,
                        }),
                        pseudo_element: None,
                        specificity: (0 << 20) + (0 << 10) + (1 << 0),
                    },
                    Selector {
                        compound_selectors: Arc::new(CompoundSelector {
                            simple_selectors: vec![
                                SimpleSelector::Namespace(ns!(HTML)),
                                SimpleSelector::LocalName(LocalName {
                                    name: atom!(body),
                                    lower_name: atom!(body),
                                }),
                            ],
                            next: None,
                        }),
                        pseudo_element: None,
                        specificity: (0 << 20) + (0 << 10) + (1 << 0),
                    },
                ],
                declarations: PropertyDeclarationBlock {
                    normal: Arc::new(vec![
                        PropertyDeclaration::Display(DeclaredValue::SpecifiedValue(
                            longhands::display::SpecifiedValue::block)),
                    ]),
                    important: Arc::new(vec![]),
                },
            }),
            CSSRule::Style(StyleRule {
                selectors: vec![
                    Selector {
                        compound_selectors: Arc::new(CompoundSelector {
                            simple_selectors: vec![
                                SimpleSelector::Class(Atom::from_slice("ok")),
                            ],
                            next: Some((box CompoundSelector {
                                simple_selectors: vec![
                                    SimpleSelector::ID(Atom::from_slice("d1")),
                                ],
                                next: None,
                            }, Combinator::Child)),
                        }),
                        pseudo_element: None,
                        specificity: (1 << 20) + (1 << 10) + (0 << 0),
                    },
                ],
                declarations: PropertyDeclarationBlock {
                    normal: Arc::new(vec![
                        PropertyDeclaration::BackgroundImage(DeclaredValue::Initial),
                        PropertyDeclaration::BackgroundAttachment(DeclaredValue::Initial),
                        PropertyDeclaration::BackgroundRepeat(DeclaredValue::Initial),
                        PropertyDeclaration::BackgroundPosition(DeclaredValue::Initial),
                        PropertyDeclaration::BackgroundColor(DeclaredValue::SpecifiedValue(
                            longhands::background_color::SpecifiedValue {
                                authored: Some("blue".to_owned()),
                                parsed: cssparser::Color::RGBA(cssparser::RGBA {
                                    red: 0., green: 0., blue: 1., alpha: 1.
                                }),
                            }
                        )),
                    ]),
                    important: Arc::new(vec![]),
                },
            }),
        ],
    });
}
