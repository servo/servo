/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Style sheets and their CSS rules.

use cssparser::{AtRuleParser, Parser, QualifiedRuleParser, decode_stylesheet_bytes};
use cssparser::{AtRuleType, RuleListParser, Token};
use encoding::EncodingRef;
use error_reporting::ParseErrorReporter;
use font_face::{FontFaceRule, parse_font_face_block};
use keyframes::{Keyframe, parse_keyframe_list};
use media_queries::{Device, MediaQueryList, parse_media_query_list};
use parking_lot::RwLock;
use parser::{ParserContext, ParserContextExtraData, log_css_error};
use properties::{PropertyDeclarationBlock, parse_property_declaration_list};
use selector_impl::TheSelectorImpl;
use selectors::parser::{Selector, parse_selector_list};
use smallvec::SmallVec;
use std::cell::Cell;
use std::iter::Iterator;
use std::slice;
use std::sync::Arc;
use string_cache::{Atom, Namespace};
use url::Url;
use viewport::ViewportRule;


/// Each style rule has an origin, which determines where it enters the cascade.
///
/// http://dev.w3.org/csswg/css-cascade/#cascading-origins
#[derive(Clone, PartialEq, Eq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Origin {
    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-ua
    UserAgent,

    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-author
    Author,

    /// http://dev.w3.org/csswg/css-cascade/#cascade-origin-user
    User,
}


#[derive(Debug)]
pub struct Stylesheet {
    /// List of rules in the order they were found (important for
    /// cascading order)
    pub rules: Vec<CSSRule>,
    /// List of media associated with the Stylesheet, if any.
    pub media: Option<MediaQueryList>,
    pub origin: Origin,
    pub dirty_on_viewport_size_change: bool,
}


/// This structure holds the user-agent and user stylesheets.
pub struct UserAgentStylesheets {
    pub user_or_user_agent_stylesheets: Vec<Stylesheet>,
    pub quirks_mode_stylesheet: Stylesheet,
}


#[derive(Debug)]
pub enum CSSRule {
    // No Charset here, CSSCharsetRule has been removed from CSSOM
    // https://drafts.csswg.org/cssom/#changes-from-5-december-2013

    Namespace(Arc<NamespaceRule>),
    Style(Arc<StyleRule>),
    Media(Arc<MediaRule>),
    FontFace(Arc<FontFaceRule>),
    Viewport(Arc<ViewportRule>),
    Keyframes(Arc<KeyframesRule>),
}


#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct NamespaceRule {
    /// `None` for the default Namespace
    pub prefix: Option<Atom>,
    pub url: Namespace,
}

#[derive(Debug)]
pub struct KeyframesRule {
    pub name: Atom,
    pub keyframes: Vec<Arc<Keyframe>>,
}

#[derive(Debug)]
pub struct MediaRule {
    pub media_queries: Arc<MediaQueryList>,
    pub rules: Vec<CSSRule>,
}


impl MediaRule {
    #[inline]
    pub fn evaluate(&self, device: &Device) -> bool {
        self.media_queries.evaluate(device)
    }
}

#[derive(Debug)]
pub struct StyleRule {
    pub selectors: Vec<Selector<TheSelectorImpl>>,
    pub block: Arc<RwLock<PropertyDeclarationBlock>>,
}


impl Stylesheet {
    pub fn from_bytes_iter<I: Iterator<Item=Vec<u8>>>(
            input: I, base_url: Url, protocol_encoding_label: Option<&str>,
            environment_encoding: Option<EncodingRef>, origin: Origin,
            error_reporter: Box<ParseErrorReporter + Send>,
            extra_data: ParserContextExtraData) -> Stylesheet {
        let mut bytes = vec![];
        // TODO: incremental decoding and tokenization/parsing
        for chunk in input {
            bytes.extend_from_slice(&chunk)
        }
        Stylesheet::from_bytes(&bytes, base_url, protocol_encoding_label,
                               environment_encoding, origin, error_reporter,
                               extra_data)
    }

    pub fn from_bytes(bytes: &[u8],
                      base_url: Url,
                      protocol_encoding_label: Option<&str>,
                      environment_encoding: Option<EncodingRef>,
                      origin: Origin, error_reporter: Box<ParseErrorReporter + Send>,
                      extra_data: ParserContextExtraData)
                      -> Stylesheet {
        // TODO: bytes.as_slice could be bytes.container_as_bytes()
        let (string, _) = decode_stylesheet_bytes(
            bytes, protocol_encoding_label, environment_encoding);
        Stylesheet::from_str(&string, base_url, origin, error_reporter, extra_data)
    }

    pub fn from_str(css: &str, base_url: Url, origin: Origin,
                    error_reporter: Box<ParseErrorReporter + Send>,
                    extra_data: ParserContextExtraData) -> Stylesheet {
        let rule_parser = TopLevelRuleParser {
            context: ParserContext::new_with_extra_data(origin, &base_url, error_reporter.clone(),
                                                        extra_data),
            state: Cell::new(State::Start),
        };
        let mut input = Parser::new(css);
        input.look_for_viewport_percentages();

        let mut rules = vec![];
        {
            let mut iter = RuleListParser::new_for_stylesheet(&mut input, rule_parser);
            while let Some(result) = iter.next() {
                match result {
                    Ok(rule) => rules.push(rule),
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
            dirty_on_viewport_size_change:
                input.seen_viewport_percentages(),
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
    pub fn rules(&self) -> Rules {
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

    use std::marker::PhantomData;
    use super::{CSSRule, KeyframesRule, MediaRule, StyleRule};
    use super::super::font_face::FontFaceRule;
    use super::super::viewport::ViewportRule;

    macro_rules! rule_filter {
        ($variant:ident -> $value:ty) => {
            /// An iterator that only yields rules that are of the synonymous `CSSRule` variant.
            #[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
            pub struct $variant<'a, I> {
                iter: I,
                _lifetime: PhantomData<&'a ()>
            }

            impl<'a, I> $variant<'a, I>
                where I: Iterator<Item=&'a CSSRule> {
                #[inline]
                pub fn new(iter: I) -> $variant<'a, I> {
                    $variant {
                        iter: iter,
                        _lifetime: PhantomData
                    }
                }
            }

            impl<'a, I> Iterator for $variant<'a, I>
                where I: Iterator<Item=&'a CSSRule> {
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

    rule_filter!(Media -> MediaRule);
    rule_filter!(Style -> StyleRule);
    rule_filter!(FontFace -> FontFaceRule);
    rule_filter!(Viewport -> ViewportRule);
    rule_filter!(Keyframes -> KeyframesRule);
}

/// Extension methods for `CSSRule` iterators.
pub trait CSSRuleIteratorExt<'a>: Iterator<Item=&'a CSSRule> + Sized {
    /// Yield only @font-face rules.
    fn font_face(self) -> rule_filter::FontFace<'a, Self>;

    /// Yield only @media rules.
    fn media(self) -> rule_filter::Media<'a, Self>;

    /// Yield only style rules.
    fn style(self) -> rule_filter::Style<'a, Self>;

    /// Yield only @viewport rules.
    fn viewport(self) -> rule_filter::Viewport<'a, Self>;

    /// Yield only @keyframes rules.
    fn keyframes(self) -> rule_filter::Keyframes<'a, Self>;
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

    #[inline]
    fn keyframes(self) -> rule_filter::Keyframes<'a, I> {
        rule_filter::Keyframes::new(self)
    }
}

fn parse_nested_rules(context: &ParserContext, input: &mut Parser) -> Vec<CSSRule> {
    let mut iter = RuleListParser::new_for_nested_rule(input,
                                                       NestedRuleParser { context: context });
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
    /// A @font-face rule prelude.
    FontFace,
    /// A @media rule prelude, with its media queries.
    Media(Arc<MediaQueryList>),
    /// A @viewport rule prelude.
    Viewport,
    /// A @keyframes rule, with its animation name.
    Keyframes(Atom),
}


impl<'a> AtRuleParser for TopLevelRuleParser<'a> {
    type Prelude = AtRulePrelude;
    type AtRule = CSSRule;

    fn parse_prelude(&mut self, name: &str, input: &mut Parser)
                     -> Result<AtRuleType<AtRulePrelude, CSSRule>, ()> {
        match_ignore_ascii_case! { name,
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

                    let prefix_result = input.try(|input| input.expect_ident());
                    let url = Namespace(Atom::from(try!(input.expect_url_or_string())));

                    let opt_prefix = if let Ok(prefix) = prefix_result {
                        let prefix: Atom = prefix.into();
                        self.context.selector_context.namespace_prefixes.insert(
                            prefix.clone(), url.clone());
                        Some(prefix)
                    } else {
                        self.context.selector_context.default_namespace = Some(url.clone());
                        None
                    };

                    return Ok(AtRuleType::WithoutBlock(CSSRule::Namespace(Arc::new(NamespaceRule {
                        prefix: opt_prefix,
                        url: url,
                    }))))
                } else {
                    return Err(())  // "@namespace must be before any rule but @charset and @import"
                }
            },
            // @charset is removed by rust-cssparser if it’s the first rule in the stylesheet
            // anything left is invalid.
            "charset" => return Err(()), // (insert appropriate error message)
            _ => {}
        }

        self.state.set(State::Body);
        AtRuleParser::parse_prelude(&mut NestedRuleParser { context: &self.context }, name, input)
    }

    #[inline]
    fn parse_block(&mut self, prelude: AtRulePrelude, input: &mut Parser) -> Result<CSSRule, ()> {
        AtRuleParser::parse_block(&mut NestedRuleParser { context: &self.context }, prelude, input)
    }
}


impl<'a> QualifiedRuleParser for TopLevelRuleParser<'a> {
    type Prelude = Vec<Selector<TheSelectorImpl>>;
    type QualifiedRule = CSSRule;

    #[inline]
    fn parse_prelude(&mut self, input: &mut Parser) -> Result<Vec<Selector<TheSelectorImpl>>, ()> {
        self.state.set(State::Body);
        QualifiedRuleParser::parse_prelude(&mut NestedRuleParser { context: &self.context }, input)
    }

    #[inline]
    fn parse_block(&mut self, prelude: Vec<Selector<TheSelectorImpl>>, input: &mut Parser)
                   -> Result<CSSRule, ()> {
        QualifiedRuleParser::parse_block(&mut NestedRuleParser { context: &self.context },
                                         prelude, input)
    }
}


struct NestedRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
}


impl<'a, 'b> AtRuleParser for NestedRuleParser<'a, 'b> {
    type Prelude = AtRulePrelude;
    type AtRule = CSSRule;

    fn parse_prelude(&mut self, name: &str, input: &mut Parser)
                     -> Result<AtRuleType<AtRulePrelude, CSSRule>, ()> {
        match_ignore_ascii_case! { name,
            "media" => {
                let media_queries = parse_media_query_list(input);
                Ok(AtRuleType::WithBlock(AtRulePrelude::Media(Arc::new(media_queries))))
            },
            "font-face" => {
                Ok(AtRuleType::WithBlock(AtRulePrelude::FontFace))
            },
            "viewport" => {
                if ::util::prefs::PREFS.get("layout.viewport.enabled").as_boolean().unwrap_or(false) {
                    Ok(AtRuleType::WithBlock(AtRulePrelude::Viewport))
                } else {
                    Err(())
                }
            },
            "keyframes" => {
                let name = match input.next() {
                    Ok(Token::Ident(ref value)) if value != "none" => Atom::from(&**value),
                    Ok(Token::QuotedString(value)) => Atom::from(&*value),
                    _ => return Err(())
                };

                Ok(AtRuleType::WithBlock(AtRulePrelude::Keyframes(Atom::from(name))))
            },
            _ => Err(())
        }
    }

    fn parse_block(&mut self, prelude: AtRulePrelude, input: &mut Parser) -> Result<CSSRule, ()> {
        match prelude {
            AtRulePrelude::FontFace => {
                Ok(CSSRule::FontFace(Arc::new(try!(parse_font_face_block(self.context, input)))))
            }
            AtRulePrelude::Media(media_queries) => {
                Ok(CSSRule::Media(Arc::new(MediaRule {
                    media_queries: media_queries,
                    rules: parse_nested_rules(self.context, input),
                })))
            }
            AtRulePrelude::Viewport => {
                Ok(CSSRule::Viewport(Arc::new(try!(ViewportRule::parse(input, self.context)))))
            }
            AtRulePrelude::Keyframes(name) => {
                Ok(CSSRule::Keyframes(Arc::new(KeyframesRule {
                    name: name,
                    keyframes: parse_keyframe_list(&self.context, input),
                })))
            }
        }
    }
}

impl<'a, 'b> QualifiedRuleParser for NestedRuleParser<'a, 'b> {
    type Prelude = Vec<Selector<TheSelectorImpl>>;
    type QualifiedRule = CSSRule;

    fn parse_prelude(&mut self, input: &mut Parser) -> Result<Vec<Selector<TheSelectorImpl>>, ()> {
        parse_selector_list(&self.context.selector_context, input)
    }

    fn parse_block(&mut self, prelude: Vec<Selector<TheSelectorImpl>>, input: &mut Parser)
                   -> Result<CSSRule, ()> {
        Ok(CSSRule::Style(Arc::new(StyleRule {
            selectors: prelude,
            block: Arc::new(RwLock::new(parse_property_declaration_list(self.context, input)))
        })))
    }
}
