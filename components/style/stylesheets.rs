/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{AtRuleParser, Parser, QualifiedRuleParser, decode_stylesheet_bytes};
use cssparser::{AtRuleType, RuleListParser};
use encoding::EncodingRef;
use error_reporting::ParseErrorReporter;
use font_face::{FontFaceRule, parse_font_face_block};
use media_queries::{Device, MediaQueryList, parse_media_query_list};
use parser::{ParserContext, log_css_error};
use properties::{PropertyDeclarationBlock, parse_property_declaration_list};
use selectors::parser::{Selector, SelectorImpl, parse_selector_list};
use smallvec::SmallVec;
use std::ascii::AsciiExt;
use std::cell::Cell;
use std::iter::Iterator;
use std::marker::PhantomData;
use std::slice;
use string_cache::{Atom, Namespace};
use url::Url;
use viewport::ViewportRule;


/// Each style rule has an origin, which determines where it enters the cascade.
///
/// http://dev.w3.org/csswg/css-cascade/#cascading-origins
#[derive(Clone, PartialEq, Eq, Copy, Debug, HeapSizeOf)]
pub enum Origin {
    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-ua
    UserAgent,

    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-author
    Author,

    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-user
    User,
}


#[derive(Debug, HeapSizeOf, PartialEq)]
pub struct Stylesheet<Impl: SelectorImpl> {
    /// List of rules in the order they were found (important for
    /// cascading order)
    pub rules: Vec<CSSRule<Impl>>,
    /// List of media associated with the Stylesheet, if any.
    pub media: Option<MediaQueryList>,
    pub origin: Origin,
    pub dirty_on_viewport_size_change: bool,
}


#[derive(Debug, HeapSizeOf, PartialEq)]
pub enum CSSRule<Impl: SelectorImpl> {
    Charset(String),
    Namespace(Option<String>, Namespace),
    Style(StyleRule<Impl>),
    Media(MediaRule<Impl>),
    FontFace(FontFaceRule),
    Viewport(ViewportRule),
}

#[derive(Debug, HeapSizeOf, PartialEq)]
pub struct MediaRule<Impl: SelectorImpl> {
    pub media_queries: MediaQueryList,
    pub rules: Vec<CSSRule<Impl>>,
}

impl<Impl: SelectorImpl> MediaRule<Impl> {
    #[inline]
    pub fn evaluate(&self, device: &Device) -> bool {
        self.media_queries.evaluate(device)
    }
}

#[derive(Debug, HeapSizeOf, PartialEq)]
pub struct StyleRule<Impl: SelectorImpl> {
    pub selectors: Vec<Selector<Impl>>,
    pub declarations: PropertyDeclarationBlock,
}


impl<Impl: SelectorImpl> Stylesheet<Impl> {
    pub fn from_bytes_iter<I: Iterator<Item=Vec<u8>>>(
            input: I, base_url: Url, protocol_encoding_label: Option<&str>,
            environment_encoding: Option<EncodingRef>, origin: Origin,
            error_reporter: Box<ParseErrorReporter + Send>) -> Stylesheet<Impl> {
        let mut bytes = vec![];
        // TODO: incremental decoding and tokenization/parsing
        for chunk in input {
            bytes.extend_from_slice(&chunk)
        }
        Stylesheet::from_bytes(&bytes, base_url, protocol_encoding_label,
                               environment_encoding, origin, error_reporter)
    }

    pub fn from_bytes(bytes: &[u8],
                      base_url: Url,
                      protocol_encoding_label: Option<&str>,
                      environment_encoding: Option<EncodingRef>,
                      origin: Origin, error_reporter: Box<ParseErrorReporter + Send>)
                      -> Stylesheet<Impl> {
        // TODO: bytes.as_slice could be bytes.container_as_bytes()
        let (string, _) = decode_stylesheet_bytes(
            bytes, protocol_encoding_label, environment_encoding);
        Stylesheet::from_str(&string, base_url, origin, error_reporter)
    }

    pub fn from_str(css: &str, base_url: Url, origin: Origin,
                    error_reporter: Box<ParseErrorReporter + Send>) -> Stylesheet<Impl> {
        let rule_parser = TopLevelRuleParser {
            context: ParserContext::new(origin, &base_url, error_reporter.clone()),
            state: Cell::new(State::Start),
            _impl: PhantomData,
        };
        let mut input = Parser::new(css);
        input.look_for_viewport_percentages();

        let mut rules = Vec::new();
        {
            let mut iter = RuleListParser::new_for_stylesheet(&mut input, rule_parser);
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
                        let context = ParserContext::new(origin, &base_url, error_reporter.clone());
                        log_css_error(iter.input, pos, &*message, &context);
                    }
                }
            }
        }
        Stylesheet {
            origin: origin,
            rules: rules,
            media: None,
            dirty_on_viewport_size_change: input.seen_viewport_percentages(),
        }
    }

    /// Set the MediaQueryList associated with the style-sheet.
    pub fn set_media(&mut self, media: Option<MediaQueryList>) {
        self.media = media;
    }

    /// Returns whether the style-sheet applies for the current device depending
    /// on the associated MediaQueryList.
    ///
    /// Always true if no associated MediaQueryList exists.
    pub fn is_effective_for_device(&self, device: &Device) -> bool {
        self.media.as_ref().map_or(true, |ref media| media.evaluate(device))
    }

    /// Return an iterator over all the rules within the style-sheet.
    #[inline]
    pub fn rules(&self) -> Rules<Impl> {
        Rules::new(self.rules.iter(), None)
    }

    /// Return an iterator over the effective rules within the style-sheet, as
    /// according to the supplied `Device`.
    ///
    /// If a condition does not hold, its associated conditional group rule and
    /// nested rules will be skipped. Use `rules` if all rules need to be
    /// examined.
    #[inline]
    pub fn effective_rules<'a>(&'a self, device: &'a Device) -> Rules<'a, Impl> {
        Rules::new(self.rules.iter(), Some(device))
    }
}

/// `CSSRule` iterator.
///
/// The iteration order is pre-order. Specifically, this implies that a
/// conditional group rule will come before its nested rules.
pub struct Rules<'a, Impl: SelectorImpl + 'a> {
    // 2 because normal case is likely to be just one level of nesting (@media)
    stack: SmallVec<[slice::Iter<'a, CSSRule<Impl>>; 2]>,
    device: Option<&'a Device>
}

impl<'a, Impl: SelectorImpl + 'a> Rules<'a, Impl> {
    fn new(iter: slice::Iter<'a, CSSRule<Impl>>, device: Option<&'a Device>) -> Rules<'a, Impl> {
        let mut stack: SmallVec<[slice::Iter<'a, CSSRule<Impl>>; 2]> = SmallVec::new();
        stack.push(iter);

        Rules { stack: stack, device: device }
    }
}

impl<'a, Impl: SelectorImpl + 'a> Iterator for Rules<'a, Impl> {
    type Item = &'a CSSRule<Impl>;

    fn next(&mut self) -> Option<&'a CSSRule<Impl>> {
        while !self.stack.is_empty() {
            let top = self.stack.len() - 1;
            while let Some(rule) = self.stack[top].next() {
                // handle conditional group rules
                if let &CSSRule::Media(ref rule) = rule {
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

    use selectors::parser::SelectorImpl;
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

            impl<'a, I, Impl: SelectorImpl + 'a> $variant<'a, I>
                where I: Iterator<Item=&'a CSSRule<Impl>> {
                pub fn new(iter: I) -> $variant<'a, I> {
                    $variant {
                        iter: iter,
                        _lifetime: PhantomData
                    }
                }
            }

            impl<'a, I, Impl: SelectorImpl + 'a> Iterator for $variant<'a, I>
                where I: Iterator<Item=&'a CSSRule<Impl>> {
                type Item = &'a $value;

                fn next(&mut self) -> Option<&'a $value> {
                    while let Some(rule) = self.iter.next() {
                        match *rule {
                            CSSRule::$variant(ref value) => return Some(value),
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

    rule_filter!(Media -> MediaRule<Impl>);
    rule_filter!(Style -> StyleRule<Impl>);
    rule_filter!(FontFace -> FontFaceRule);
    rule_filter!(Viewport -> ViewportRule);
}

/// Extension methods for `CSSRule` iterators.
pub trait CSSRuleIteratorExt<'a, Impl: SelectorImpl + 'a>: Iterator<Item=&'a CSSRule<Impl>> + Sized {
    /// Yield only @font-face rules.
    fn font_face(self) -> rule_filter::FontFace<'a, Self>;

    /// Yield only @media rules.
    fn media(self) -> rule_filter::Media<'a, Self>;

    /// Yield only style rules.
    fn style(self) -> rule_filter::Style<'a, Self>;

    /// Yield only @viewport rules.
    fn viewport(self) -> rule_filter::Viewport<'a, Self>;
}

impl<'a, I, Impl: SelectorImpl + 'a> CSSRuleIteratorExt<'a, Impl> for I where I: Iterator<Item=&'a CSSRule<Impl>> {
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

fn parse_nested_rules<Impl: SelectorImpl>(context: &ParserContext, input: &mut Parser) -> Vec<CSSRule<Impl>> {
    let mut iter = RuleListParser::new_for_nested_rule(input,
                                                       NestedRuleParser {
                                                           context: context,
                                                           _impl: PhantomData
                                                       });
    let mut rules = Vec::new();
    while let Some(result) = iter.next() {
        match result {
            Ok(rule) => rules.push(rule),
            Err(range) => {
                let pos = range.start;
                let message = format!("Unsupported rule: '{}'", iter.input.slice(range));
                log_css_error(iter.input, pos, &*message, &context);
            }
        }
    }
    rules
}


struct TopLevelRuleParser<'a, Impl: SelectorImpl> {
    context: ParserContext<'a>,
    state: Cell<State>,
    _impl: PhantomData<Impl>
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


impl<'a, Impl: SelectorImpl> AtRuleParser for TopLevelRuleParser<'a, Impl> {
    type Prelude = AtRulePrelude;
    type AtRule = CSSRule<Impl>;

    fn parse_prelude(&self, name: &str, input: &mut Parser)
                     -> Result<AtRuleType<AtRulePrelude, CSSRule<Impl>>, ()> {
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
                    let url = Namespace(Atom::from(try!(input.expect_url_or_string())));
                    return Ok(AtRuleType::WithoutBlock(CSSRule::Namespace(prefix, url)))
                } else {
                    return Err(())  // "@namespace must be before any rule but @charset and @import"
                }
            },
            _ => {}
        }

        self.state.set(State::Body);
        AtRuleParser::parse_prelude(&NestedRuleParser { context: &self.context, _impl: PhantomData }, name, input)
    }

    #[inline]
    fn parse_block(&self, prelude: AtRulePrelude, input: &mut Parser) -> Result<CSSRule<Impl>, ()> {
        AtRuleParser::parse_block(&NestedRuleParser { context: &self.context, _impl: PhantomData }, prelude, input)
    }
}


impl<'a, Impl: SelectorImpl> QualifiedRuleParser for TopLevelRuleParser<'a, Impl> {
    type Prelude = Vec<Selector<Impl>>;
    type QualifiedRule = CSSRule<Impl>;

    #[inline]
    fn parse_prelude(&self, input: &mut Parser) -> Result<Vec<Selector<Impl>>, ()> {
        self.state.set(State::Body);
        QualifiedRuleParser::parse_prelude(&NestedRuleParser { context: &self.context, _impl: PhantomData }, input)
    }

    #[inline]
    fn parse_block(&self, prelude: Vec<Selector<Impl>>, input: &mut Parser) -> Result<CSSRule<Impl>, ()> {
        QualifiedRuleParser::parse_block(&NestedRuleParser { context: &self.context, _impl: PhantomData },
                                         prelude, input)
    }
}


struct NestedRuleParser<'a, 'b: 'a, Impl: SelectorImpl> {
    context: &'a ParserContext<'b>,
    _impl: PhantomData<Impl>,
}


impl<'a, 'b, Impl: SelectorImpl> AtRuleParser for NestedRuleParser<'a, 'b, Impl> {
    type Prelude = AtRulePrelude;
    type AtRule = CSSRule<Impl>;

    fn parse_prelude(&self, name: &str, input: &mut Parser)
                     -> Result<AtRuleType<AtRulePrelude, CSSRule<Impl>>, ()> {
        match_ignore_ascii_case! { name,
            "media" => {
                let media_queries = parse_media_query_list(input);
                Ok(AtRuleType::WithBlock(AtRulePrelude::Media(media_queries)))
            },
            "font-face" => {
                Ok(AtRuleType::WithBlock(AtRulePrelude::FontFace))
            },
            "viewport" => {
                if ::util::prefs::get_pref("layout.viewport.enabled").as_boolean().unwrap_or(false) {
                    Ok(AtRuleType::WithBlock(AtRulePrelude::Viewport))
                } else {
                    Err(())
                }
            },
            _ => Err(())
        }
    }

    fn parse_block(&self, prelude: AtRulePrelude, input: &mut Parser) -> Result<CSSRule<Impl>, ()> {
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


impl<'a, 'b, Impl: SelectorImpl> QualifiedRuleParser for NestedRuleParser<'a, 'b, Impl> {
    type Prelude = Vec<Selector<Impl>>;
    type QualifiedRule = CSSRule<Impl>;

    fn parse_prelude(&self, input: &mut Parser) -> Result<Vec<Selector<Impl>>, ()> {
        parse_selector_list(&self.context.selector_context, input)
    }

    fn parse_block(&self, prelude: Vec<Selector<Impl>>, input: &mut Parser) -> Result<CSSRule<Impl>, ()> {
        Ok(CSSRule::Style(StyleRule {
            selectors: prelude,
            declarations: parse_property_declaration_list(self.context, input)
        }))
    }
}
