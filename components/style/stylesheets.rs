/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ascii::AsciiExt;
use std::cell::Cell;
use std::iter::Iterator;
use std::slice;
use url::Url;

use encoding::EncodingRef;

use cssparser::{Parser, decode_stylesheet_bytes, QualifiedRuleParser, AtRuleParser};
use cssparser::{RuleListParser, AtRuleType};
use font_face::{FontFaceRule, parse_font_face_block};
use media_queries::{Device, MediaQueryList, parse_media_query_list};
use parser::{ParserContext, log_css_error};
use properties::{PropertyDeclarationBlock, parse_property_declaration_list};
use selectors::parser::{Selector, parse_selector_list};
use smallvec::SmallVec;
use string_cache::{Atom, Namespace};
use viewport::ViewportRule;


/// Each style rule has an origin, which determines where it enters the cascade.
///
/// http://dev.w3.org/csswg/css-cascade/#cascading-origins
#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Origin {
    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-ua
    UserAgent,

    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-author
    Author,

    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-user
    User,
}


#[derive(Debug, PartialEq)]
pub struct Stylesheet {
    /// List of rules in the order they were found (important for
    /// cascading order)
    pub rules: Vec<CSSRule>,
    pub origin: Origin,
}


#[derive(Debug, PartialEq)]
pub enum CSSRule {
    Charset(String),
    Namespace(Option<String>, Namespace),
    Style(StyleRule),
    Media(MediaRule),
    FontFace(FontFaceRule),
    Viewport(ViewportRule),
}

#[derive(Debug, PartialEq)]
pub struct MediaRule {
    pub media_queries: MediaQueryList,
    pub rules: Vec<CSSRule>,
}

impl MediaRule {
    #[inline]
    pub fn evaluate(&self, device: &Device) -> bool {
        self.media_queries.evaluate(device)
    }
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
                            iter.parser.context.selector_context.namespace_prefixes.insert(
                                prefix.clone(), namespace.clone());
                        } else {
                            iter.parser.context.selector_context.default_namespace =
                                Some(namespace.clone());
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

    /// Return an iterator over all the rules within the style-sheet.
    #[inline]
    pub fn rules<'a>(&'a self) -> Rules<'a> {
        Rules::new(self.rules.iter(), None)
    }

    /// Return an iterator over the effective rules within the style-sheet, as
    /// according to the supplied `Device`.
    ///
    /// If a condition does not hold, its associated conditional group rule and
    /// nested rules will be skipped. Use `rules` if all rules need to be
    /// examined.
    #[inline]
    pub fn effective_rules<'a>(&'a self, device: &'a Device) -> Rules<'a> {
        Rules::new(self.rules.iter(), Some(device))
    }
}

/// `CSSRule` iterator.
///
/// The iteration order is pre-order. Specifically, this implies that a
/// conditional group rule will come before its nested rules.
pub struct Rules<'a> {
    // 2 because normal case is likely to be just one level of nesting (@media)
    stack: SmallVec<[slice::Iter<'a, CSSRule>; 2]>,
    device: Option<&'a Device>
}

impl<'a> Rules<'a> {
    fn new(iter: slice::Iter<'a, CSSRule>, device: Option<&'a Device>) -> Rules<'a> {
        let mut stack: SmallVec<[slice::Iter<'a, CSSRule>; 2]> = SmallVec::new();
        stack.push(iter);

        Rules { stack: stack, device: device }
    }
}

impl<'a> Iterator for Rules<'a> {
    type Item = &'a CSSRule;

    fn next(&mut self) -> Option<&'a CSSRule> {
        while !self.stack.is_empty() {
            let top = self.stack.len() - 1;
            while let Some(rule) = self.stack[top].next() {
                // handle conditional group rules
                match rule {
                    &CSSRule::Media(ref rule) => {
                        if let Some(device) = self.device {
                            if rule.evaluate(device) {
                                self.stack.push(rule.rules.iter());
                            } else {
                                continue
                            }
                        } else {
                            self.stack.push(rule.rules.iter());
                        }
                    }
                    _ => {}
                }

                return Some(rule)
            }

            self.stack.pop();
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // TODO: track total number of rules in style-sheet for upper bound?
        (0, None)
    }
}

pub mod rule_filter {
    //! Specific `CSSRule` variant iterators.

    use std::marker::PhantomData;
    use super::super::font_face::FontFaceRule;
    use super::super::viewport::ViewportRule;
    use super::{CSSRule, MediaRule, StyleRule};

    macro_rules! rule_filter {
        ($variant:ident -> $value:ty) => {
            /// An iterator that only yields rules that are of the synonymous `CSSRule` variant.
            #[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
            pub struct $variant<'a, I> {
                iter: I,
                _lifetime: PhantomData<&'a ()>
            }

            impl<'a, I> $variant<'a, I> where I: Iterator<Item=&'a CSSRule> {
                pub fn new(iter: I) -> $variant<'a, I> {
                    $variant {
                        iter: iter,
                        _lifetime: PhantomData
                    }
                }
            }

            impl<'a, I> Iterator for $variant<'a, I> where I: Iterator<Item=&'a CSSRule> {
                type Item = &'a $value;

                fn next(&mut self) -> Option<&'a $value> {
                    while let Some(rule) = self.iter.next() {
                        match rule {
                            &CSSRule::$variant(ref value) => return Some(value),
                            _ => continue
                        }
                    }
                    None
                }

                #[inline]
                fn size_hint(&self) -> (usize, Option<usize>) {
                    (0, self.iter.size_hint().1)
                }
            }
        }
    }

    rule_filter!(FontFace -> FontFaceRule);
    rule_filter!(Media -> MediaRule);
    rule_filter!(Style -> StyleRule);
    rule_filter!(Viewport -> ViewportRule);
}

/// Extension methods for `CSSRule` iterators.
pub trait CSSRuleIteratorExt<'a>: Iterator<Item=&'a CSSRule> {
    /// Yield only @font-face rules.
    fn font_face(self) -> rule_filter::FontFace<'a, Self>;

    /// Yield only @media rules.
    fn media(self) -> rule_filter::Media<'a, Self>;

    /// Yield only style rules.
    fn style(self) -> rule_filter::Style<'a, Self>;

    /// Yield only @viewport rules.
    fn viewport(self) -> rule_filter::Viewport<'a, Self>;
}

impl<'a, I> CSSRuleIteratorExt<'a> for I where I: Iterator<Item=&'a CSSRule> {
    #[inline]
    fn font_face(self) -> rule_filter::FontFace<'a, I> {
        rule_filter::FontFace::new(self)
    }

    #[inline]
    fn media(self) -> rule_filter::Media<'a, I> {
        rule_filter::Media::new(self)
    }

    #[inline]
    fn style(self) -> rule_filter::Style<'a, I> {
        rule_filter::Style::new(self)
    }

    #[inline]
    fn viewport(self) -> rule_filter::Viewport<'a, I> {
        rule_filter::Viewport::new(self)
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

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
enum State {
    Start = 1,
    Imports = 2,
    Namespaces = 3,
    Body = 4,
}


enum AtRulePrelude {
    FontFace,
    Media(MediaQueryList),
    Viewport,
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
            },
            "viewport" => {
                if ::util::opts::experimental_enabled() {
                    Ok(AtRuleType::WithBlock(AtRulePrelude::Viewport))
                } else {
                    Err(())
                }
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
            AtRulePrelude::Viewport => {
                ViewportRule::parse(input, self.context).map(CSSRule::Viewport)
            }
        }
    }
}


impl<'a, 'b> QualifiedRuleParser for NestedRuleParser<'a, 'b> {
    type Prelude = Vec<Selector>;
    type QualifiedRule = CSSRule;

    fn parse_prelude(&self, input: &mut Parser) -> Result<Vec<Selector>, ()> {
        parse_selector_list(&self.context.selector_context, input)
    }

    fn parse_block(&self, prelude: Vec<Selector>, input: &mut Parser) -> Result<CSSRule, ()> {
        Ok(CSSRule::Style(StyleRule {
            selectors: prelude,
            declarations: parse_property_declaration_list(self.context, input)
        }))
    }
}
