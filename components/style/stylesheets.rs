/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Style sheets and their CSS rules.

#![deny(missing_docs)]

use {Atom, Prefix, Namespace};
use cssparser::{AtRuleParser, Parser, QualifiedRuleParser, decode_stylesheet_bytes};
use cssparser::{AtRuleType, RuleListParser, SourcePosition, Token, parse_one_rule};
use cssparser::ToCss as ParserToCss;
use encoding::EncodingRef;
use error_reporting::ParseErrorReporter;
use font_face::{FontFaceRule, parse_font_face_block};
use keyframes::{Keyframe, parse_keyframe_list};
use media_queries::{Device, MediaList, parse_media_query_list};
use parking_lot::RwLock;
use parser::{ParserContext, ParserContextExtraData, log_css_error};
use properties::{PropertyDeclarationBlock, parse_property_declaration_list};
use selector_parser::{SelectorImpl, SelectorParser};
use selectors::parser::SelectorList;
use servo_config::prefs::PREFS;
use servo_url::ServoUrl;
use std::cell::Cell;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use style_traits::ToCss;
use stylist::FnvHashMap;
use supports::SupportsCondition;
use values::specified::url::SpecifiedUrl;
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

/// A set of namespaces applying to a given stylesheet.
#[derive(Default, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Namespaces {
    pub default: Option<Namespace>,
    pub prefixes: FnvHashMap<Prefix , Namespace>,
}

/// A list of CSS rules.
#[derive(Debug)]
pub struct CssRules(pub Vec<CssRule>);

impl CssRules {
    /// Whether this CSS rules is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[allow(missing_docs)]
pub enum RulesMutateError {
    Syntax,
    IndexSize,
    HierarchyRequest,
    InvalidState,
}

impl From<SingleRuleParseError> for RulesMutateError {
    fn from(other: SingleRuleParseError) -> Self {
        match other {
            SingleRuleParseError::Syntax => RulesMutateError::Syntax,
            SingleRuleParseError::Hierarchy => RulesMutateError::HierarchyRequest,
        }
    }
}

impl CssRules {
    #[allow(missing_docs)]
    pub fn new(rules: Vec<CssRule>) -> Arc<RwLock<CssRules>> {
        Arc::new(RwLock::new(CssRules(rules)))
    }

    fn only_ns_or_import(&self) -> bool {
        self.0.iter().all(|r| {
            match *r {
                CssRule::Namespace(..) |
                CssRule::Import(..) => true,
                _ => false
            }
        })
    }

    /// https://drafts.csswg.org/cssom/#insert-a-css-rule
    pub fn insert_rule(&mut self, rule: &str, parent_stylesheet: &Stylesheet, index: usize, nested: bool)
                       -> Result<CssRule, RulesMutateError> {
        // Step 1, 2
        if index > self.0.len() {
            return Err(RulesMutateError::IndexSize);
        }

        // Computes the parser state at the given index
        let state = if nested {
            None
        } else if index == 0 {
            Some(State::Start)
        } else {
            self.0.get(index - 1).map(CssRule::rule_state)
        };

        // Step 3, 4
        // XXXManishearth should we also store the namespace map?
        let (new_rule, new_state) =
            try!(CssRule::parse(&rule, parent_stylesheet,
                                ParserContextExtraData::default(), state));

        // Step 5
        // Computes the maximum allowed parser state at a given index.
        let rev_state = self.0.get(index).map_or(State::Body, CssRule::rule_state);
        if new_state > rev_state {
            // We inserted a rule too early, e.g. inserting
            // a regular style rule before @namespace rules
            return Err(RulesMutateError::HierarchyRequest);
        }

        // Step 6
        if let CssRule::Namespace(..) = new_rule {
            if !self.only_ns_or_import() {
                return Err(RulesMutateError::InvalidState);
            }
        }

        self.0.insert(index, new_rule.clone());
        Ok(new_rule)
    }

    /// https://drafts.csswg.org/cssom/#remove-a-css-rule
    pub fn remove_rule(&mut self, index: usize) -> Result<(), RulesMutateError> {
        // Step 1, 2
        if index >= self.0.len() {
            return Err(RulesMutateError::IndexSize);
        }

        {
            // Step 3
            let ref rule = self.0[index];

            // Step 4
            if let CssRule::Namespace(..) = *rule {
                if !self.only_ns_or_import() {
                    return Err(RulesMutateError::InvalidState);
                }
            }
        }

        // Step 5, 6
        self.0.remove(index);
        Ok(())
    }
}

/// The structure servo uses to represent a stylesheet.
#[derive(Debug)]
pub struct Stylesheet {
    /// List of rules in the order they were found (important for
    /// cascading order)
    pub rules: Arc<RwLock<CssRules>>,
    /// List of media associated with the Stylesheet.
    pub media: Arc<RwLock<MediaList>>,
    /// The origin of this stylesheet.
    pub origin: Origin,
    /// The base url this stylesheet should use.
    pub base_url: ServoUrl,
    /// The namespaces that apply to this stylesheet.
    pub namespaces: RwLock<Namespaces>,
    /// Whether this stylesheet would be dirty when the viewport size changes.
    pub dirty_on_viewport_size_change: AtomicBool,
    /// Whether this stylesheet should be disabled.
    pub disabled: AtomicBool,
}


/// This structure holds the user-agent and user stylesheets.
pub struct UserAgentStylesheets {
    /// The user or user agent stylesheets.
    pub user_or_user_agent_stylesheets: Vec<Stylesheet>,
    /// The quirks mode stylesheet.
    pub quirks_mode_stylesheet: Stylesheet,
}


/// A CSS rule.
///
/// TODO(emilio): Lots of spec links should be around.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum CssRule {
    // No Charset here, CSSCharsetRule has been removed from CSSOM
    // https://drafts.csswg.org/cssom/#changes-from-5-december-2013

    Namespace(Arc<RwLock<NamespaceRule>>),
    Import(Arc<RwLock<ImportRule>>),
    Style(Arc<RwLock<StyleRule>>),
    Media(Arc<RwLock<MediaRule>>),
    FontFace(Arc<RwLock<FontFaceRule>>),
    Viewport(Arc<RwLock<ViewportRule>>),
    Keyframes(Arc<RwLock<KeyframesRule>>),
    Supports(Arc<RwLock<SupportsRule>>),
}

#[allow(missing_docs)]
pub enum CssRuleType {
    // https://drafts.csswg.org/cssom/#the-cssrule-interface
    Style               = 1,
    Charset             = 2,
    Import              = 3,
    Media               = 4,
    FontFace            = 5,
    Page                = 6,
    // https://drafts.csswg.org/css-animations-1/#interface-cssrule-idl
    Keyframes           = 7,
    Keyframe            = 8,
    // https://drafts.csswg.org/cssom/#the-cssrule-interface
    Margin              = 9,
    Namespace           = 10,
    // https://drafts.csswg.org/css-counter-styles-3/#extentions-to-cssrule-interface
    CounterStyle        = 11,
    // https://drafts.csswg.org/css-conditional-3/#extentions-to-cssrule-interface
    Supports            = 12,
    // https://drafts.csswg.org/css-fonts-3/#om-fontfeaturevalues
    FontFeatureValues   = 14,
    // https://drafts.csswg.org/css-device-adapt/#css-rule-interface
    Viewport            = 15,
}

/// Error reporter which silently forgets errors
pub struct MemoryHoleReporter;

impl ParseErrorReporter for MemoryHoleReporter {
    fn report_error(&self,
            _: &mut Parser,
            _: SourcePosition,
            _: &str) {
        // do nothing
    }
    fn clone(&self) -> Box<ParseErrorReporter + Send + Sync> {
        Box::new(MemoryHoleReporter)
    }
}

#[allow(missing_docs)]
pub enum SingleRuleParseError {
    Syntax,
    Hierarchy,
}

impl CssRule {
    #[allow(missing_docs)]
    pub fn rule_type(&self) -> CssRuleType {
        match *self {
            CssRule::Style(_)     => CssRuleType::Style,
            CssRule::Import(_)    => CssRuleType::Import,
            CssRule::Media(_)     => CssRuleType::Media,
            CssRule::FontFace(_)  => CssRuleType::FontFace,
            CssRule::Keyframes(_) => CssRuleType::Keyframes,
            CssRule::Namespace(_) => CssRuleType::Namespace,
            CssRule::Viewport(_)  => CssRuleType::Viewport,
            CssRule::Supports(_)  => CssRuleType::Supports,
        }
    }

    fn rule_state(&self) -> State {
        match *self {
            // CssRule::Charset(..) => State::Start,
            CssRule::Import(..) => State::Imports,
            CssRule::Namespace(..) => State::Namespaces,
            _ => State::Body,
        }
    }

    /// Call `f` with the slice of rules directly contained inside this rule.
    ///
    /// Note that only some types of rules can contain rules. An empty slice is
    /// used for others.
    ///
    /// This will not recurse down unsupported @supports rules
    pub fn with_nested_rules_and_mq<F, R>(&self, mut f: F) -> R
    where F: FnMut(&[CssRule], Option<&MediaList>) -> R {
        match *self {
            CssRule::Import(ref lock) => {
                let rule = lock.read();
                let media = rule.stylesheet.media.read();
                let rules = rule.stylesheet.rules.read();
                // FIXME(emilio): Include the nested rules if the stylesheet is
                // loaded.
                f(&rules.0, Some(&media))
            }
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
                let rules = &media_rule.rules.read().0;
                f(rules, Some(&mq))
            }
            CssRule::Supports(ref lock) => {
                let supports_rule = lock.read();
                let enabled = supports_rule.enabled;
                if enabled {
                    let rules = &supports_rule.rules.read().0;
                    f(rules, None)
                } else {
                    f(&[], None)
                }
            }
        }
    }

    // input state is None for a nested rule
    // Returns a parsed CSS rule and the final state of the parser
    #[allow(missing_docs)]
    pub fn parse(css: &str,
                 parent_stylesheet: &Stylesheet,
                 extra_data: ParserContextExtraData,
                 state: Option<State>)
                 -> Result<(Self, State), SingleRuleParseError> {
        let error_reporter = Box::new(MemoryHoleReporter);
        let mut namespaces = parent_stylesheet.namespaces.write();
        let context = ParserContext::new_with_extra_data(parent_stylesheet.origin,
                                                         &parent_stylesheet.base_url,
                                                         error_reporter.clone(),
                                                         extra_data);
        let mut input = Parser::new(css);

        // nested rules are in the body state
        let state = state.unwrap_or(State::Body);
        let mut rule_parser = TopLevelRuleParser {
            stylesheet_origin: parent_stylesheet.origin,
            context: context,
            loader: None,
            state: Cell::new(state),
            namespaces: &mut namespaces,
        };
        match parse_one_rule(&mut input, &mut rule_parser) {
            Ok(result) => Ok((result, rule_parser.state.get())),
            Err(_) => {
                if let State::Invalid = rule_parser.state.get() {
                    Err(SingleRuleParseError::Hierarchy)
                } else {
                    Err(SingleRuleParseError::Syntax)
                }
            }
        }
    }
}

impl ToCss for CssRule {
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            CssRule::Namespace(ref lock) => lock.read().to_css(dest),
            CssRule::Import(ref lock) => lock.read().to_css(dest),
            CssRule::Style(ref lock) => lock.read().to_css(dest),
            CssRule::FontFace(ref lock) => lock.read().to_css(dest),
            CssRule::Viewport(ref lock) => lock.read().to_css(dest),
            CssRule::Keyframes(ref lock) => lock.read().to_css(dest),
            CssRule::Media(ref lock) => lock.read().to_css(dest),
            CssRule::Supports(ref lock) => lock.read().to_css(dest),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct NamespaceRule {
    /// `None` for the default Namespace
    pub prefix: Option<Prefix>,
    pub url: Namespace,
}

impl ToCss for NamespaceRule {
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSNamespaceRule
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("@namespace "));
        if let Some(ref prefix) = self.prefix {
            try!(dest.write_str(&*prefix.to_string()));
            try!(dest.write_str(" "));
        }
        try!(dest.write_str("url(\""));
        try!(dest.write_str(&*self.url.to_string()));
        dest.write_str("\");")
    }
}


/// The [`@import`][import] at-rule.
///
/// [import]: https://drafts.csswg.org/css-cascade-3/#at-import
#[derive(Debug)]
pub struct ImportRule {
    /// The `<url>` this `@import` rule is loading.
    pub url: SpecifiedUrl,

    /// The stylesheet is always present.
    ///
    /// It contains an empty list of rules and namespace set that is updated
    /// when it loads.
    pub stylesheet: Arc<Stylesheet>,
}

impl ToCss for ImportRule {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("@import "));
        try!(self.url.to_css(dest));
        let media = self.stylesheet.media.read();
        if !media.is_empty() {
            try!(dest.write_str(" "));
            try!(media.to_css(dest));
        }
        dest.write_str(";")
    }
}

/// A [`@keyframes`][keyframes] rule.
///
/// [keyframes]: https://drafts.csswg.org/css-animations/#keyframes
#[derive(Debug)]
pub struct KeyframesRule {
    /// The name of the current animation.
    pub name: Atom,
    /// The keyframes specified for this CSS rule.
    pub keyframes: Vec<Arc<RwLock<Keyframe>>>,
}

impl ToCss for KeyframesRule {
    // Serialization of KeyframesRule is not specced.
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("@keyframes "));
        try!(dest.write_str(&*self.name.to_string()));
        try!(dest.write_str(" { "));
        let iter = self.keyframes.iter();
        let mut first = true;
        for lock in iter {
            if !first {
                try!(dest.write_str(" "));
            }
            first = false;
            let keyframe = lock.read();
            try!(keyframe.to_css(dest));
        }
        dest.write_str(" }")
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct MediaRule {
    pub media_queries: Arc<RwLock<MediaList>>,
    pub rules: Arc<RwLock<CssRules>>,
}

impl ToCss for MediaRule {
    // Serialization of MediaRule is not specced.
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSMediaRule
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("@media "));
        try!(self.media_queries.read().to_css(dest));
        try!(dest.write_str(" {"));
        for rule in self.rules.read().0.iter() {
            try!(dest.write_str(" "));
            try!(rule.to_css(dest));
        }
        dest.write_str(" }")
    }
}


#[derive(Debug)]
/// An @supports rule
pub struct SupportsRule {
    /// The parsed condition
    pub condition: SupportsCondition,
    /// Child rules
    pub rules: Arc<RwLock<CssRules>>,
    /// The result of evaluating the condition
    pub enabled: bool,
}

impl ToCss for SupportsRule {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("@supports "));
        try!(self.condition.to_css(dest));
        try!(dest.write_str(" {"));
        for rule in self.rules.read().0.iter() {
            try!(dest.write_str(" "));
            try!(rule.to_css(dest));
        }
        dest.write_str(" }")
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct StyleRule {
    pub selectors: SelectorList<SelectorImpl>,
    pub block: Arc<RwLock<PropertyDeclarationBlock>>,
}

impl ToCss for StyleRule {
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSStyleRule
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        // Step 1
        try!(self.selectors.to_css(dest));
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
    /// Parse a stylesheet from a set of bytes, potentially received over the
    /// network.
    ///
    /// Takes care of decoding the network bytes and forwards the resulting
    /// string to `Stylesheet::from_str`.
    pub fn from_bytes(bytes: &[u8],
                      base_url: ServoUrl,
                      protocol_encoding_label: Option<&str>,
                      environment_encoding: Option<EncodingRef>,
                      origin: Origin,
                      media: MediaList,
                      stylesheet_loader: Option<&StylesheetLoader>,
                      error_reporter: Box<ParseErrorReporter + Send>,
                      extra_data: ParserContextExtraData)
                      -> Stylesheet {
        let (string, _) = decode_stylesheet_bytes(
            bytes, protocol_encoding_label, environment_encoding);
        Stylesheet::from_str(&string,
                             base_url,
                             origin,
                             media,
                             stylesheet_loader,
                             error_reporter,
                             extra_data)
    }

    /// Updates an empty stylesheet with a set of bytes that reached over the
    /// network.
    pub fn update_from_bytes(existing: &Stylesheet,
                             bytes: &[u8],
                             protocol_encoding_label: Option<&str>,
                             environment_encoding: Option<EncodingRef>,
                             stylesheet_loader: Option<&StylesheetLoader>,
                             error_reporter: Box<ParseErrorReporter + Send>,
                             extra_data: ParserContextExtraData) {
        let (string, _) = decode_stylesheet_bytes(
            bytes, protocol_encoding_label, environment_encoding);
        Self::update_from_str(existing,
                              &string,
                              stylesheet_loader,
                              error_reporter,
                              extra_data)
    }

    /// Updates an empty stylesheet from a given string of text.
    pub fn update_from_str(existing: &Stylesheet,
                           css: &str,
                           stylesheet_loader: Option<&StylesheetLoader>,
                           error_reporter: Box<ParseErrorReporter + Send>,
                           extra_data: ParserContextExtraData) {
        let mut rules = existing.rules.write();
        let mut namespaces = existing.namespaces.write();

        assert!(rules.is_empty());

        let mut input = Parser::new(css);
        let rule_parser = TopLevelRuleParser {
            stylesheet_origin: existing.origin,
            namespaces: &mut namespaces,
            loader: stylesheet_loader,
            context: ParserContext::new_with_extra_data(existing.origin,
                                                        &existing.base_url,
                                                        error_reporter,
                                                        extra_data),
            state: Cell::new(State::Start),
        };

        input.look_for_viewport_percentages();

        {
            let mut iter = RuleListParser::new_for_stylesheet(&mut input, rule_parser);
            while let Some(result) = iter.next() {
                match result {
                    Ok(rule) => rules.0.push(rule),
                    Err(range) => {
                        let pos = range.start;
                        let message = format!("Invalid rule: '{}'", iter.input.slice(range));
                        log_css_error(iter.input, pos, &*message, &iter.parser.context);
                    }
                }
            }
        }

        existing.dirty_on_viewport_size_change
            .store(input.seen_viewport_percentages(), Ordering::Release);
    }

    /// Creates an empty stylesheet and parses it with a given base url, origin
    /// and media.
    ///
    /// Effectively creates a new stylesheet and forwards the hard work to
    /// `Stylesheet::update_from_str`.
    pub fn from_str(css: &str,
                    base_url: ServoUrl,
                    origin: Origin,
                    media: MediaList,
                    stylesheet_loader: Option<&StylesheetLoader>,
                    error_reporter: Box<ParseErrorReporter + Send>,
                    extra_data: ParserContextExtraData) -> Stylesheet {
        let s = Stylesheet {
            origin: origin,
            base_url: base_url,
            namespaces: RwLock::new(Namespaces::default()),
            rules: CssRules::new(vec![]),
            media: Arc::new(RwLock::new(media)),
            dirty_on_viewport_size_change: AtomicBool::new(false),
            disabled: AtomicBool::new(false),
        };

        Self::update_from_str(&s,
                              css,
                              stylesheet_loader,
                              error_reporter,
                              extra_data);

        s
    }

    /// Whether this stylesheet can be dirty on viewport size change.
    pub fn dirty_on_viewport_size_change(&self) -> bool {
        self.dirty_on_viewport_size_change.load(Ordering::SeqCst)
    }

    /// When CSSOM inserts a rule or declaration into this stylesheet, it needs to call this method
    /// with the return value of `cssparser::Parser::seen_viewport_percentages`.
    ///
    /// FIXME: actually make these calls
    ///
    /// Note: when *removing* a rule or declaration that contains a viewport percentage,
    /// to keep the flag accurate we’d need to iterator through the rest of the stylesheet to
    /// check for *other* such values.
    ///
    /// Instead, we conservatively assume there might be some.
    /// Restyling will some some more work than necessary, but give correct results.
    pub fn inserted_has_viewport_percentages(&self, has_viewport_percentages: bool) {
        self.dirty_on_viewport_size_change.fetch_or(has_viewport_percentages, Ordering::SeqCst);
    }

    /// Returns whether the style-sheet applies for the current device depending
    /// on the associated MediaList.
    ///
    /// Always true if no associated MediaList exists.
    pub fn is_effective_for_device(&self, device: &Device) -> bool {
        self.media.read().evaluate(device)
    }

    /// Return an iterator over the effective rules within the style-sheet, as
    /// according to the supplied `Device`.
    ///
    /// If a condition does not hold, its associated conditional group rule and
    /// nested rules will be skipped. Use `rules` if all rules need to be
    /// examined.
    #[inline]
    pub fn effective_rules<F>(&self, device: &Device, mut f: F) where F: FnMut(&CssRule) {
        effective_rules(&self.rules.read().0, device, &mut f);
    }

    /// Returns whether the stylesheet has been explicitly disabled through the
    /// CSSOM.
    pub fn disabled(&self) -> bool {
        self.disabled.load(Ordering::SeqCst)
    }

    /// Records that the stylesheet has been explicitly disabled through the
    /// CSSOM.
    ///
    /// Returns whether the the call resulted in a change in disabled state.
    ///
    /// Disabled stylesheets remain in the document, but their rules are not
    /// added to the Stylist.
    pub fn set_disabled(&self, disabled: bool) -> bool {
        self.disabled.swap(disabled, Ordering::SeqCst) != disabled
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
                #[allow(missing_docs)]
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
    effective_supports_rules(Supports => SupportsRule),
}

/// The stylesheet loader is the abstraction used to trigger network requests
/// for `@import` rules.
pub trait StylesheetLoader {
    /// Request a stylesheet after parsing a given `@import` rule.
    ///
    /// The called code is responsible to update the `stylesheet` rules field
    /// when the sheet is done loading.
    fn request_stylesheet(&self, import: &Arc<RwLock<ImportRule>>);
}

struct TopLevelRuleParser<'a> {
    stylesheet_origin: Origin,
    namespaces: &'a mut Namespaces,
    loader: Option<&'a StylesheetLoader>,
    context: ParserContext<'a>,
    state: Cell<State>,
}

impl<'b> TopLevelRuleParser<'b> {
    fn nested<'a: 'b>(&'a self) -> NestedRuleParser<'a, 'b> {
        NestedRuleParser {
            stylesheet_origin: self.stylesheet_origin,
            context: &self.context,
            namespaces: self.namespaces,
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[allow(missing_docs)]
pub enum State {
    Start = 1,
    Imports = 2,
    Namespaces = 3,
    Body = 4,
    Invalid = 5,
}


enum AtRulePrelude {
    /// A @font-face rule prelude.
    FontFace,
    /// A @media rule prelude, with its media queries.
    Media(Arc<RwLock<MediaList>>),
    /// An @supports rule, with its conditional
    Supports(SupportsCondition),
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
                    let url = try!(input.expect_url_or_string());
                    let url =
                        try!(SpecifiedUrl::parse_from_string(url,
                                                             &self.context));

                    let media =
                        Arc::new(RwLock::new(parse_media_query_list(input)));

                    let is_valid_url = url.url().is_some();

                    let import_rule = Arc::new(RwLock::new(
                        ImportRule {
                            url: url,
                            stylesheet: Arc::new(Stylesheet {
                                rules: Arc::new(RwLock::new(CssRules(vec![]))),
                                media: media,
                                origin: self.context.stylesheet_origin,
                                base_url: self.context.base_url.clone(),
                                namespaces: RwLock::new(Namespaces::default()),
                                dirty_on_viewport_size_change: AtomicBool::new(false),
                                disabled: AtomicBool::new(false),
                            })
                        }
                    ));

                    if is_valid_url {
                        let loader = self.loader
                            .expect("Expected a stylesheet loader for @import");
                        loader.request_stylesheet(&import_rule);
                    }

                    return Ok(AtRuleType::WithoutBlock(CssRule::Import(import_rule)))
                } else {
                    self.state.set(State::Invalid);
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
                        self.namespaces.prefixes.insert(prefix.clone(), url.clone());
                        Some(prefix)
                    } else {
                        self.namespaces.default = Some(url.clone());
                        None
                    };

                    return Ok(AtRuleType::WithoutBlock(CssRule::Namespace(Arc::new(RwLock::new(
                        NamespaceRule {
                            prefix: opt_prefix,
                            url: url,
                        }
                    )))))
                } else {
                    self.state.set(State::Invalid);
                    return Err(())  // "@namespace must be before any rule but @charset and @import"
                }
            },
            // @charset is removed by rust-cssparser if it’s the first rule in the stylesheet
            // anything left is invalid.
            "charset" => return Err(()), // (insert appropriate error message)
            _ => {}
        }
        // Don't allow starting with an invalid state
        if self.state.get() > State::Body {
            self.state.set(State::Invalid);
            return Err(());
        }
        self.state.set(State::Body);
        AtRuleParser::parse_prelude(&mut self.nested(), name, input)
    }

    #[inline]
    fn parse_block(&mut self, prelude: AtRulePrelude, input: &mut Parser) -> Result<CssRule, ()> {
        AtRuleParser::parse_block(&mut self.nested(), prelude, input)
    }
}


impl<'a> QualifiedRuleParser for TopLevelRuleParser<'a> {
    type Prelude = SelectorList<SelectorImpl>;
    type QualifiedRule = CssRule;

    #[inline]
    fn parse_prelude(&mut self, input: &mut Parser) -> Result<SelectorList<SelectorImpl>, ()> {
        self.state.set(State::Body);
        QualifiedRuleParser::parse_prelude(&mut self.nested(), input)
    }

    #[inline]
    fn parse_block(&mut self, prelude: SelectorList<SelectorImpl>, input: &mut Parser)
                   -> Result<CssRule, ()> {
        QualifiedRuleParser::parse_block(&mut self.nested(), prelude, input)
    }
}

#[derive(Clone)]  // shallow, relatively cheap clone
struct NestedRuleParser<'a, 'b: 'a> {
    stylesheet_origin: Origin,
    context: &'a ParserContext<'b>,
    namespaces: &'b Namespaces,
}

impl<'a, 'b> NestedRuleParser<'a, 'b> {
    fn parse_nested_rules(&self, input: &mut Parser) -> Arc<RwLock<CssRules>> {
        let mut iter = RuleListParser::new_for_nested_rule(input, self.clone());
        let mut rules = Vec::new();
        while let Some(result) = iter.next() {
            match result {
                Ok(rule) => rules.push(rule),
                Err(range) => {
                    let pos = range.start;
                    let message = format!("Unsupported rule: '{}'", iter.input.slice(range));
                    log_css_error(iter.input, pos, &*message, self.context);
                }
            }
        }
        CssRules::new(rules)
    }
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
            "supports" => {
                let cond = SupportsCondition::parse(input)?;
                Ok(AtRuleType::WithBlock(AtRulePrelude::Supports(cond)))
            },
            "font-face" => {
                Ok(AtRuleType::WithBlock(AtRulePrelude::FontFace))
            },
            "viewport" => {
                if PREFS.get("layout.viewport.enabled").as_boolean().unwrap_or(false) ||
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
                    rules: self.parse_nested_rules(input),
                }))))
            }
            AtRulePrelude::Supports(cond) => {
                let enabled = cond.eval(self.context);
                Ok(CssRule::Supports(Arc::new(RwLock::new(SupportsRule {
                    condition: cond,
                    rules: self.parse_nested_rules(input),
                    enabled: enabled,
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
    type Prelude = SelectorList<SelectorImpl>;
    type QualifiedRule = CssRule;

    fn parse_prelude(&mut self, input: &mut Parser) -> Result<SelectorList<SelectorImpl>, ()> {
        let selector_parser = SelectorParser {
            stylesheet_origin: self.stylesheet_origin,
            namespaces: self.namespaces,
        };
        SelectorList::parse(&selector_parser, input)
    }

    fn parse_block(&mut self, prelude: SelectorList<SelectorImpl>, input: &mut Parser)
                   -> Result<CssRule, ()> {
        Ok(CssRule::Style(Arc::new(RwLock::new(StyleRule {
            selectors: prelude,
            block: Arc::new(RwLock::new(parse_property_declaration_list(self.context, input)))
        }))))
    }
}
