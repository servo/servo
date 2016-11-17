/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Style sheets and their CSS rules.

use {Atom, Prefix, Namespace};
use cssparser::{AtRuleParser, Parser, QualifiedRuleParser, decode_stylesheet_bytes};
use cssparser::{AtRuleType, RuleListParser, Token};
use cssparser::ToCss as ParserToCss;
use encoding::EncodingRef;
use error_reporting::ParseErrorReporter;
use font_face::{FontFaceRule, parse_font_face_block};
use keyframes::{Keyframe, parse_keyframe_list};
use media_queries::{Device, MediaList, parse_media_query_list};
use parking_lot::RwLock;
use parser::{ParserContext, ParserContextExtraData, log_css_error};
use properties::{PropertyDeclarationBlock, parse_property_declaration_list};
use selector_impl::TheSelectorImpl;
use selectors::parser::{Selector, parse_selector_list};
use servo_url::ServoUrl;
use std::cell::Cell;
use std::fmt;
use std::sync::Arc;
use style_traits::ToCss;
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

#[derive(Debug, Clone)]
pub struct CssRules(pub Arc<RwLock<Vec<CssRule>>>);

impl From<Vec<CssRule>> for CssRules {
    fn from(other: Vec<CssRule>) -> Self {
        CssRules(Arc::new(RwLock::new(other)))
    }
}

#[derive(Debug)]
pub struct Stylesheet {
    /// List of rules in the order they were found (important for
    /// cascading order)
    pub rules: CssRules,
    /// List of media associated with the Stylesheet.
    pub media: MediaList,
    pub origin: Origin,
    pub dirty_on_viewport_size_change: bool,
}


/// This structure holds the user-agent and user stylesheets.
pub struct UserAgentStylesheets {
    pub user_or_user_agent_stylesheets: Vec<Stylesheet>,
    pub quirks_mode_stylesheet: Stylesheet,
}


#[derive(Debug, Clone)]
pub enum CssRule {
    // No Charset here, CSSCharsetRule has been removed from CSSOM
    // https://drafts.csswg.org/cssom/#changes-from-5-december-2013

    Namespace(Arc<RwLock<NamespaceRule>>),
    Style(Arc<RwLock<StyleRule>>),
    Media(Arc<RwLock<MediaRule>>),
    FontFace(Arc<RwLock<FontFaceRule>>),
    Viewport(Arc<RwLock<ViewportRule>>),
    Keyframes(Arc<RwLock<KeyframesRule>>),
}

impl CssRule {
    /// Call `f` with the slice of rules directly contained inside this rule.
    ///
    /// Note that only some types of rules can contain rules. An empty slice is used for others.
    pub fn with_nested_rules_and_mq<F, R>(&self, mut f: F) -> R
    where F: FnMut(&[CssRule], Option<&MediaList>) -> R {
        match *self {
            CssRule::Namespace(_) |
            CssRule::Style(_) |
            CssRule::FontFace(_) |
            CssRule::Viewport(_) |
            CssRule::Keyframes(_) => {
                f(&[], None)
            }
            CssRule::Media(ref lock) => {
                let media_rule = lock.read();
                let mq = media_rule.media_queries.read();
                let rules = media_rule.rules.0.read();
                f(&rules, Some(&mq))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct NamespaceRule {
    /// `None` for the default Namespace
    pub prefix: Option<Prefix>,
    pub url: Namespace,
}

#[derive(Debug)]
pub struct KeyframesRule {
    pub name: Atom,
    pub keyframes: Vec<Arc<RwLock<Keyframe>>>,
}

#[derive(Debug)]
pub struct MediaRule {
    pub media_queries: Arc<RwLock<MediaList>>,
    pub rules: CssRules,
}

#[derive(Debug)]
pub struct StyleRule {
    pub selectors: Vec<Selector<TheSelectorImpl>>,
    pub block: Arc<RwLock<PropertyDeclarationBlock>>,
}

impl StyleRule {
    /// Serialize the group of selectors for this rule.
    ///
    /// https://drafts.csswg.org/cssom/#serialize-a-group-of-selectors
    pub fn selectors_to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut iter = self.selectors.iter();
        try!(iter.next().unwrap().to_css(dest));
        for selector in iter {
            try!(write!(dest, ", "));
            try!(selector.to_css(dest));
        }
        Ok(())
    }
}

impl ToCss for StyleRule {
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSStyleRule
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        // Step 1
        try!(self.selectors_to_css(dest));
        // Step 2
        try!(dest.write_str(" { "));
        // Step 3
        let declaration_block = self.block.read();
        try!(declaration_block.to_css(dest));
        // Step 4
        if declaration_block.declarations.len() > 0 {
            try!(write!(dest, " "));
        }
        // Step 5
        try!(dest.write_str("}"));
        Ok(())
    }
}


impl Stylesheet {
    pub fn from_bytes_iter<I: Iterator<Item=Vec<u8>>>(
            input: I, base_url: ServoUrl, protocol_encoding_label: Option<&str>,
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
                      base_url: ServoUrl,
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

    pub fn from_str(css: &str, base_url: ServoUrl, origin: Origin,
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
            rules: rules.into(),
            media: Default::default(),
            dirty_on_viewport_size_change:
                input.seen_viewport_percentages(),
        }
    }

    /// Set the MediaList associated with the style-sheet.
    pub fn set_media(&mut self, media: MediaList) {
        self.media = media;
    }

    /// Returns whether the style-sheet applies for the current device depending
    /// on the associated MediaList.
    ///
    /// Always true if no associated MediaList exists.
    pub fn is_effective_for_device(&self, device: &Device) -> bool {
        self.media.evaluate(device)
    }

    /// Return an iterator over the effective rules within the style-sheet, as
    /// according to the supplied `Device`.
    ///
    /// If a condition does not hold, its associated conditional group rule and
    /// nested rules will be skipped. Use `rules` if all rules need to be
    /// examined.
    #[inline]
    pub fn effective_rules<F>(&self, device: &Device, mut f: F) where F: FnMut(&CssRule) {
        effective_rules(&self.rules.0.read(), device, &mut f);
    }
}

fn effective_rules<F>(rules: &[CssRule], device: &Device, f: &mut F) where F: FnMut(&CssRule) {
    for rule in rules {
        f(rule);
        rule.with_nested_rules_and_mq(|rules, mq| {
            if let Some(media_queries) = mq {
                if !media_queries.evaluate(device) {
                    return
                }
            }
            effective_rules(rules, device, f)
        })
    }
}

macro_rules! rule_filter {
    ($( $method: ident($variant:ident => $rule_type: ident), )+) => {
        impl Stylesheet {
            $(
                pub fn $method<F>(&self, device: &Device, mut f: F) where F: FnMut(&$rule_type) {
                    self.effective_rules(device, |rule| {
                        if let CssRule::$variant(ref lock) = *rule {
                            let rule = lock.read();
                            f(&rule)
                        }
                    })
                }
            )+
        }
    }
}

rule_filter! {
    effective_style_rules(Style => StyleRule),
    effective_media_rules(Media => MediaRule),
    effective_font_face_rules(FontFace => FontFaceRule),
    effective_viewport_rules(Viewport => ViewportRule),
    effective_keyframes_rules(Keyframes => KeyframesRule),
}

fn parse_nested_rules(context: &ParserContext, input: &mut Parser) -> CssRules {
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
    rules.into()
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
    Media(Arc<RwLock<MediaList>>),
    /// A @viewport rule prelude.
    Viewport,
    /// A @keyframes rule, with its animation name.
    Keyframes(Atom),
}


impl<'a> AtRuleParser for TopLevelRuleParser<'a> {
    type Prelude = AtRulePrelude;
    type AtRule = CssRule;

    fn parse_prelude(&mut self, name: &str, input: &mut Parser)
                     -> Result<AtRuleType<AtRulePrelude, CssRule>, ()> {
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
                    let url = Namespace::from(try!(input.expect_url_or_string()));

                    let opt_prefix = if let Ok(prefix) = prefix_result {
                        let prefix = Prefix::from(prefix);
                        self.context.selector_context.namespace_prefixes.insert(
                            prefix.clone(), url.clone());
                        Some(prefix)
                    } else {
                        self.context.selector_context.default_namespace = Some(url.clone());
                        None
                    };

                    return Ok(AtRuleType::WithoutBlock(CssRule::Namespace(Arc::new(RwLock::new(
                        NamespaceRule {
                            prefix: opt_prefix,
                            url: url,
                        }
                    )))))
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
    fn parse_block(&mut self, prelude: AtRulePrelude, input: &mut Parser) -> Result<CssRule, ()> {
        AtRuleParser::parse_block(&mut NestedRuleParser { context: &self.context }, prelude, input)
    }
}


impl<'a> QualifiedRuleParser for TopLevelRuleParser<'a> {
    type Prelude = Vec<Selector<TheSelectorImpl>>;
    type QualifiedRule = CssRule;

    #[inline]
    fn parse_prelude(&mut self, input: &mut Parser) -> Result<Vec<Selector<TheSelectorImpl>>, ()> {
        self.state.set(State::Body);
        QualifiedRuleParser::parse_prelude(&mut NestedRuleParser { context: &self.context }, input)
    }

    #[inline]
    fn parse_block(&mut self, prelude: Vec<Selector<TheSelectorImpl>>, input: &mut Parser)
                   -> Result<CssRule, ()> {
        QualifiedRuleParser::parse_block(&mut NestedRuleParser { context: &self.context },
                                         prelude, input)
    }
}


struct NestedRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
}


impl<'a, 'b> AtRuleParser for NestedRuleParser<'a, 'b> {
    type Prelude = AtRulePrelude;
    type AtRule = CssRule;

    fn parse_prelude(&mut self, name: &str, input: &mut Parser)
                     -> Result<AtRuleType<AtRulePrelude, CssRule>, ()> {
        match_ignore_ascii_case! { name,
            "media" => {
                let media_queries = parse_media_query_list(input);
                Ok(AtRuleType::WithBlock(AtRulePrelude::Media(Arc::new(RwLock::new(media_queries)))))
            },
            "font-face" => {
                Ok(AtRuleType::WithBlock(AtRulePrelude::FontFace))
            },
            "viewport" => {
                if ::util::prefs::PREFS.get("layout.viewport.enabled").as_boolean().unwrap_or(false) ||
                   cfg!(feature = "gecko") {
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

    fn parse_block(&mut self, prelude: AtRulePrelude, input: &mut Parser) -> Result<CssRule, ()> {
        match prelude {
            AtRulePrelude::FontFace => {
                Ok(CssRule::FontFace(Arc::new(RwLock::new(
                    try!(parse_font_face_block(self.context, input))))))
            }
            AtRulePrelude::Media(media_queries) => {
                Ok(CssRule::Media(Arc::new(RwLock::new(MediaRule {
                    media_queries: media_queries,
                    rules: parse_nested_rules(self.context, input),
                }))))
            }
            AtRulePrelude::Viewport => {
                Ok(CssRule::Viewport(Arc::new(RwLock::new(
                    try!(ViewportRule::parse(input, self.context))))))
            }
            AtRulePrelude::Keyframes(name) => {
                Ok(CssRule::Keyframes(Arc::new(RwLock::new(KeyframesRule {
                    name: name,
                    keyframes: parse_keyframe_list(&self.context, input),
                }))))
            }
        }
    }
}

impl<'a, 'b> QualifiedRuleParser for NestedRuleParser<'a, 'b> {
    type Prelude = Vec<Selector<TheSelectorImpl>>;
    type QualifiedRule = CssRule;

    fn parse_prelude(&mut self, input: &mut Parser) -> Result<Vec<Selector<TheSelectorImpl>>, ()> {
        parse_selector_list(&self.context.selector_context, input)
    }

    fn parse_block(&mut self, prelude: Vec<Selector<TheSelectorImpl>>, input: &mut Parser)
                   -> Result<CssRule, ()> {
        Ok(CssRule::Style(Arc::new(RwLock::new(StyleRule {
            selectors: prelude,
            block: Arc::new(RwLock::new(parse_property_declaration_list(self.context, input)))
        }))))
    }
}
