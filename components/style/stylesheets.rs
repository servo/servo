/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Style sheets and their CSS rules.

#![deny(missing_docs)]

use {Atom, Prefix, Namespace};
use cssparser::{AtRuleParser, Parser, QualifiedRuleParser};
use cssparser::{AtRuleType, RuleListParser, SourcePosition, Token, parse_one_rule};
use cssparser::ToCss as ParserToCss;
use error_reporting::ParseErrorReporter;
#[cfg(feature = "servo")]
use font_face::FontFaceRuleData;
use font_face::parse_font_face_block;
#[cfg(feature = "gecko")]
pub use gecko::rules::FontFaceRule;
#[cfg(feature = "gecko")]
use gecko_bindings::structs::URLExtraData;
#[cfg(feature = "gecko")]
use gecko_bindings::sugar::refptr::RefPtr;
use keyframes::{Keyframe, parse_keyframe_list};
use media_queries::{Device, MediaList, parse_media_query_list};
use parking_lot::RwLock;
use parser::{LengthParsingMode, Parse, ParserContext, log_css_error};
use properties::{PropertyDeclarationBlock, parse_property_declaration_list};
use selector_parser::{SelectorImpl, SelectorParser};
use selectors::parser::SelectorList;
#[cfg(feature = "servo")]
use servo_config::prefs::PREFS;
#[cfg(not(feature = "gecko"))]
use servo_url::ServoUrl;
use shared_lock::{SharedRwLock, Locked, ToCssWithGuard, SharedRwLockReadGuard};
use std::cell::Cell;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use style_traits::ToCss;
use stylist::FnvHashMap;
use supports::SupportsCondition;
use values::specified::url::SpecifiedUrl;
use viewport::ViewportRule;


/// Extra data that the backend may need to resolve url values.
#[cfg(not(feature = "gecko"))]
pub type UrlExtraData = ServoUrl;

/// Extra data that the backend may need to resolve url values.
#[cfg(feature = "gecko")]
pub type UrlExtraData = RefPtr<URLExtraData>;

#[cfg(feature = "gecko")]
impl UrlExtraData {
    /// Returns a string for the url.
    ///
    /// Unimplemented currently.
    pub fn as_str(&self) -> &str {
        // TODO
        "(stylo: not supported)"
    }
}

// XXX We probably need to figure out whether we should mark Eq here.
// It is currently marked so because properties::UnparsedValue wants Eq.
#[cfg(feature = "gecko")]
impl Eq for UrlExtraData {}

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
    pub fn new(rules: Vec<CssRule>, shared_lock: &SharedRwLock) -> Arc<Locked<CssRules>> {
        Arc::new(shared_lock.wrap(CssRules(rules)))
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

/// A trait to implement helpers for `Arc<Locked<CssRules>>`.
pub trait CssRulesHelpers {
    /// https://drafts.csswg.org/cssom/#insert-a-css-rule
    ///
    /// Written in this funky way because parsing an @import rule may cause us
    /// to clone a stylesheet from the same document due to caching in the CSS
    /// loader.
    ///
    /// TODO(emilio): We could also pass the write guard down into the loader
    /// instead, but that seems overkill.
    fn insert_rule(&self,
                   lock: &SharedRwLock,
                   rule: &str,
                   parent_stylesheet: &Stylesheet,
                   index: usize,
                   nested: bool,
                   loader: Option<&StylesheetLoader>)
                   -> Result<CssRule, RulesMutateError>;
}

impl CssRulesHelpers for Arc<Locked<CssRules>> {
    fn insert_rule(&self,
                   lock: &SharedRwLock,
                   rule: &str,
                   parent_stylesheet: &Stylesheet,
                   index: usize,
                   nested: bool,
                   loader: Option<&StylesheetLoader>)
                   -> Result<CssRule, RulesMutateError> {
        let state = {
            let read_guard = lock.read();
            let rules = self.read_with(&read_guard);

            // Step 1, 2
            if index > rules.0.len() {
                return Err(RulesMutateError::IndexSize);
            }

            // Computes the parser state at the given index
            if nested {
                None
            } else if index == 0 {
                Some(State::Start)
            } else {
                rules.0.get(index - 1).map(CssRule::rule_state)
            }
        };

        // Step 3, 4
        // XXXManishearth should we also store the namespace map?
        let (new_rule, new_state) =
            try!(CssRule::parse(&rule, parent_stylesheet, state, loader));

        {
            let mut write_guard = lock.write();
            let mut rules = self.write_with(&mut write_guard);
            // Step 5
            // Computes the maximum allowed parser state at a given index.
            let rev_state = rules.0.get(index).map_or(State::Body, CssRule::rule_state);
            if new_state > rev_state {
                // We inserted a rule too early, e.g. inserting
                // a regular style rule before @namespace rules
                return Err(RulesMutateError::HierarchyRequest);
            }

            // Step 6
            if let CssRule::Namespace(..) = new_rule {
                if !rules.only_ns_or_import() {
                    return Err(RulesMutateError::InvalidState);
                }
            }

            rules.0.insert(index, new_rule.clone());
        }

        Ok(new_rule)
    }

}

/// The structure servo uses to represent a stylesheet.
#[derive(Debug)]
pub struct Stylesheet {
    /// List of rules in the order they were found (important for
    /// cascading order)
    pub rules: Arc<Locked<CssRules>>,
    /// List of media associated with the Stylesheet.
    pub media: Arc<Locked<MediaList>>,
    /// The origin of this stylesheet.
    pub origin: Origin,
    /// The url data this stylesheet should use.
    pub url_data: UrlExtraData,
    /// The lock used for objects inside this stylesheet
    pub shared_lock: SharedRwLock,
    /// The namespaces that apply to this stylesheet.
    pub namespaces: RwLock<Namespaces>,
    /// Whether this stylesheet would be dirty when the viewport size changes.
    pub dirty_on_viewport_size_change: AtomicBool,
    /// Whether this stylesheet should be disabled.
    pub disabled: AtomicBool,
}


/// This structure holds the user-agent and user stylesheets.
pub struct UserAgentStylesheets {
    /// The lock used for user-agent stylesheets.
    pub shared_lock: SharedRwLock,
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

    Namespace(Arc<Locked<NamespaceRule>>),
    Import(Arc<Locked<ImportRule>>),
    Style(Arc<Locked<StyleRule>>),
    Media(Arc<Locked<MediaRule>>),
    FontFace(Arc<Locked<FontFaceRule>>),
    Viewport(Arc<Locked<ViewportRule>>),
    Keyframes(Arc<Locked<KeyframesRule>>),
    Supports(Arc<Locked<SupportsRule>>),
    Page(Arc<Locked<PageRule>>),
}

#[allow(missing_docs)]
#[derive(PartialEq, Eq, Copy, Clone)]
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
            _: &str,
            _: &UrlExtraData,
            _: u64) {
        // do nothing
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
            CssRule::Page(_)      => CssRuleType::Page,
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
    pub fn with_nested_rules_and_mq<F, R>(&self, guard: &SharedRwLockReadGuard, mut f: F) -> R
    where F: FnMut(&[CssRule], Option<&MediaList>) -> R {
        match *self {
            CssRule::Import(ref lock) => {
                let rule = lock.read_with(guard);
                let media = rule.stylesheet.media.read_with(guard);
                let rules = rule.stylesheet.rules.read_with(guard);
                // FIXME(emilio): Include the nested rules if the stylesheet is
                // loaded.
                f(&rules.0, Some(&media))
            }
            CssRule::Namespace(_) |
            CssRule::Style(_) |
            CssRule::FontFace(_) |
            CssRule::Viewport(_) |
            CssRule::Keyframes(_) |
            CssRule::Page(_) => {
                f(&[], None)
            }
            CssRule::Media(ref lock) => {
                let media_rule = lock.read_with(guard);
                let mq = media_rule.media_queries.read_with(guard);
                let rules = &media_rule.rules.read_with(guard).0;
                f(rules, Some(&mq))
            }
            CssRule::Supports(ref lock) => {
                let supports_rule = lock.read_with(guard);
                let enabled = supports_rule.enabled;
                if enabled {
                    let rules = &supports_rule.rules.read_with(guard).0;
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
                 state: Option<State>,
                 loader: Option<&StylesheetLoader>)
                 -> Result<(Self, State), SingleRuleParseError> {
        let error_reporter = MemoryHoleReporter;
        let mut namespaces = parent_stylesheet.namespaces.write();
        let context = ParserContext::new(parent_stylesheet.origin,
                                         &parent_stylesheet.url_data,
                                         &error_reporter,
                                         None,
                                         LengthParsingMode::Default);
        let mut input = Parser::new(css);

        // nested rules are in the body state
        let state = state.unwrap_or(State::Body);
        let mut rule_parser = TopLevelRuleParser {
            stylesheet_origin: parent_stylesheet.origin,
            context: context,
            shared_lock: &parent_stylesheet.shared_lock,
            loader: loader,
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

impl ToCssWithGuard for CssRule {
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        match *self {
            CssRule::Namespace(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Import(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Style(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::FontFace(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Viewport(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Keyframes(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Media(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Supports(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Page(ref lock) => lock.read_with(guard).to_css(guard, dest),
        }
    }
}

#[derive(Debug, PartialEq)]
#[allow(missing_docs)]
pub struct NamespaceRule {
    /// `None` for the default Namespace
    pub prefix: Option<Prefix>,
    pub url: Namespace,
}

impl ToCssWithGuard for NamespaceRule {
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSNamespaceRule
    fn to_css<W>(&self, _guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
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

impl ToCssWithGuard for ImportRule {
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        try!(dest.write_str("@import "));
        try!(self.url.to_css(dest));
        let media = self.stylesheet.media.read_with(guard);
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
    pub keyframes: Vec<Arc<Locked<Keyframe>>>,
}

impl ToCssWithGuard for KeyframesRule {
    // Serialization of KeyframesRule is not specced.
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
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
            let keyframe = lock.read_with(&guard);
            try!(keyframe.to_css(guard, dest));
        }
        dest.write_str(" }")
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct MediaRule {
    pub media_queries: Arc<Locked<MediaList>>,
    pub rules: Arc<Locked<CssRules>>,
}

impl ToCssWithGuard for MediaRule {
    // Serialization of MediaRule is not specced.
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSMediaRule
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        try!(dest.write_str("@media "));
        try!(self.media_queries.read_with(guard).to_css(dest));
        try!(dest.write_str(" {"));
        for rule in self.rules.read_with(guard).0.iter() {
            try!(dest.write_str(" "));
            try!(rule.to_css(guard, dest));
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
    pub rules: Arc<Locked<CssRules>>,
    /// The result of evaluating the condition
    pub enabled: bool,
}

impl ToCssWithGuard for SupportsRule {
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        try!(dest.write_str("@supports "));
        try!(self.condition.to_css(dest));
        try!(dest.write_str(" {"));
        for rule in self.rules.read_with(guard).0.iter() {
            try!(dest.write_str(" "));
            try!(rule.to_css(guard, dest));
        }
        dest.write_str(" }")
    }
}

/// A [`@page`][page] rule.  This implements only a limited subset of the CSS 2.2 syntax.  In this
/// subset, [page selectors][page-selectors] are not implemented.
///
/// [page]: https://drafts.csswg.org/css2/page.html#page-box
/// [page-selectors]: https://drafts.csswg.org/css2/page.html#page-selectors
#[derive(Debug)]
pub struct PageRule(pub Arc<Locked<PropertyDeclarationBlock>>);

impl ToCssWithGuard for PageRule {
    // Serialization of PageRule is not specced, adapted from steps for StyleRule.
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        dest.write_str("@page { ")?;
        let declaration_block = self.0.read_with(guard);
        declaration_block.to_css(dest)?;
        if declaration_block.declarations().len() > 0 {
            write!(dest, " ")?;
        }
        dest.write_str("}")
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct StyleRule {
    pub selectors: SelectorList<SelectorImpl>,
    pub block: Arc<Locked<PropertyDeclarationBlock>>,
}

impl ToCssWithGuard for StyleRule {
    // https://drafts.csswg.org/cssom/#serialize-a-css-rule CSSStyleRule
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        // Step 1
        try!(self.selectors.to_css(dest));
        // Step 2
        try!(dest.write_str(" { "));
        // Step 3
        let declaration_block = self.block.read_with(guard);
        try!(declaration_block.to_css(dest));
        // Step 4
        if declaration_block.declarations().len() > 0 {
            try!(write!(dest, " "));
        }
        // Step 5
        try!(dest.write_str("}"));
        Ok(())
    }
}

/// A @font-face rule
#[cfg(feature = "servo")]
pub type FontFaceRule = FontFaceRuleData;

impl Stylesheet {
    /// Updates an empty stylesheet from a given string of text.
    pub fn update_from_str(existing: &Stylesheet,
                           css: &str,
                           url_data: &UrlExtraData,
                           stylesheet_loader: Option<&StylesheetLoader>,
                           error_reporter: &ParseErrorReporter) {
        let mut namespaces = Namespaces::default();
        // FIXME: we really should update existing.url_data with the given url_data,
        // otherwise newly inserted rule may not have the right base url.
        let (rules, dirty_on_viewport_size_change) = Stylesheet::parse_rules(
            css, url_data, existing.origin, &mut namespaces,
            &existing.shared_lock, stylesheet_loader, error_reporter,
            0u64);

        *existing.namespaces.write() = namespaces;
        existing.dirty_on_viewport_size_change
            .store(dirty_on_viewport_size_change, Ordering::Release);

        // Acquire the lock *after* parsing, to minimize the exclusive section.
        let mut guard = existing.shared_lock.write();
        *existing.rules.write_with(&mut guard) = CssRules(rules);
    }

    fn parse_rules(css: &str,
                   url_data: &UrlExtraData,
                   origin: Origin,
                   namespaces: &mut Namespaces,
                   shared_lock: &SharedRwLock,
                   stylesheet_loader: Option<&StylesheetLoader>,
                   error_reporter: &ParseErrorReporter,
                   line_number_offset: u64)
                   -> (Vec<CssRule>, bool) {
        let mut rules = Vec::new();
        let mut input = Parser::new(css);
        let rule_parser = TopLevelRuleParser {
            stylesheet_origin: origin,
            namespaces: namespaces,
            shared_lock: shared_lock,
            loader: stylesheet_loader,
            context: ParserContext::new_with_line_number_offset(origin, url_data, error_reporter,
                                                                line_number_offset, LengthParsingMode::Default),
            state: Cell::new(State::Start),
        };

        input.look_for_viewport_percentages();

        {
            let mut iter = RuleListParser::new_for_stylesheet(&mut input, rule_parser);
            while let Some(result) = iter.next() {
                match result {
                    Ok(rule) => rules.push(rule),
                    Err(range) => {
                        let pos = range.start;
                        let message = format!("Invalid rule: '{}'", iter.input.slice(range));
                        log_css_error(iter.input, pos, &*message, &iter.parser.context);
                    }
                }
            }
        }

        (rules, input.seen_viewport_percentages())
    }

    /// Creates an empty stylesheet and parses it with a given base url, origin
    /// and media.
    ///
    /// Effectively creates a new stylesheet and forwards the hard work to
    /// `Stylesheet::update_from_str`.
    pub fn from_str(css: &str,
                    url_data: UrlExtraData,
                    origin: Origin,
                    media: Arc<Locked<MediaList>>,
                    shared_lock: SharedRwLock,
                    stylesheet_loader: Option<&StylesheetLoader>,
                    error_reporter: &ParseErrorReporter,
                    line_number_offset: u64) -> Stylesheet {
        let mut namespaces = Namespaces::default();
        let (rules, dirty_on_viewport_size_change) = Stylesheet::parse_rules(
            css, &url_data, origin, &mut namespaces,
            &shared_lock, stylesheet_loader, error_reporter, line_number_offset
        );
        Stylesheet {
            origin: origin,
            url_data: url_data,
            namespaces: RwLock::new(namespaces),
            rules: CssRules::new(rules, &shared_lock),
            media: media,
            shared_lock: shared_lock,
            dirty_on_viewport_size_change: AtomicBool::new(dirty_on_viewport_size_change),
            disabled: AtomicBool::new(false),
        }
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
    pub fn is_effective_for_device(&self, device: &Device, guard: &SharedRwLockReadGuard) -> bool {
        self.media.read_with(guard).evaluate(device)
    }

    /// Return an iterator over the effective rules within the style-sheet, as
    /// according to the supplied `Device`.
    ///
    /// If a condition does not hold, its associated conditional group rule and
    /// nested rules will be skipped. Use `rules` if all rules need to be
    /// examined.
    #[inline]
    pub fn effective_rules<F>(&self, device: &Device, guard: &SharedRwLockReadGuard, mut f: F)
    where F: FnMut(&CssRule) {
        effective_rules(&self.rules.read_with(guard).0, device, guard, &mut f);
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

fn effective_rules<F>(rules: &[CssRule], device: &Device, guard: &SharedRwLockReadGuard, f: &mut F)
where F: FnMut(&CssRule) {
    for rule in rules {
        f(rule);
        rule.with_nested_rules_and_mq(guard, |rules, mq| {
            if let Some(media_queries) = mq {
                if !media_queries.evaluate(device) {
                    return
                }
            }
            effective_rules(rules, device, guard, f)
        })
    }
}

macro_rules! rule_filter {
    ($( $method: ident($variant:ident => $rule_type: ident), )+) => {
        impl Stylesheet {
            $(
                #[allow(missing_docs)]
                pub fn $method<F>(&self, device: &Device, guard: &SharedRwLockReadGuard, mut f: F)
                where F: FnMut(&$rule_type) {
                    self.effective_rules(device, guard, |rule| {
                        if let CssRule::$variant(ref lock) = *rule {
                            let rule = lock.read_with(guard);
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
    effective_page_rules(Page => PageRule),
}

/// The stylesheet loader is the abstraction used to trigger network requests
/// for `@import` rules.
pub trait StylesheetLoader {
    /// Request a stylesheet after parsing a given `@import` rule.
    ///
    /// The called code is responsible to update the `stylesheet` rules field
    /// when the sheet is done loading.
    ///
    /// The convoluted signature allows impls to look at MediaList and ImportRule
    /// before they’re locked, while keeping the trait object-safe.
    fn request_stylesheet(
        &self,
        media: Arc<Locked<MediaList>>,
        make_import: &mut FnMut(Arc<Locked<MediaList>>) -> ImportRule,
        make_arc: &mut FnMut(ImportRule) -> Arc<Locked<ImportRule>>,
    ) -> Arc<Locked<ImportRule>>;
}

struct NoOpLoader;

impl StylesheetLoader for NoOpLoader {
    fn request_stylesheet(
        &self,
        media: Arc<Locked<MediaList>>,
        make_import: &mut FnMut(Arc<Locked<MediaList>>) -> ImportRule,
        make_arc: &mut FnMut(ImportRule) -> Arc<Locked<ImportRule>>,
    ) -> Arc<Locked<ImportRule>> {
        make_arc(make_import(media))
    }
}


struct TopLevelRuleParser<'a> {
    stylesheet_origin: Origin,
    namespaces: &'a mut Namespaces,
    shared_lock: &'a SharedRwLock,
    loader: Option<&'a StylesheetLoader>,
    context: ParserContext<'a>,
    state: Cell<State>,
}

impl<'b> TopLevelRuleParser<'b> {
    fn nested<'a: 'b>(&'a self) -> NestedRuleParser<'a, 'b> {
        NestedRuleParser {
            stylesheet_origin: self.stylesheet_origin,
            shared_lock: self.shared_lock,
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
    Media(Arc<Locked<MediaList>>),
    /// An @supports rule, with its conditional
    Supports(SupportsCondition),
    /// A @viewport rule prelude.
    Viewport,
    /// A @keyframes rule, with its animation name.
    Keyframes(Atom),
    /// A @page rule prelude.
    Page,
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
                    let url_string = input.expect_url_or_string()?;
                    let specified_url = SpecifiedUrl::parse_from_string(url_string, &self.context)?;

                    let media = parse_media_query_list(&self.context, input);
                    let media = Arc::new(self.shared_lock.wrap(media));

                    let noop_loader = NoOpLoader;
                    let loader = if !specified_url.is_invalid() {
                        self.loader.expect("Expected a stylesheet loader for @import")
                    } else {
                        &noop_loader
                    };

                    let mut specified_url = Some(specified_url);
                    let arc = loader.request_stylesheet(media, &mut |media| {
                        ImportRule {
                            url: specified_url.take().unwrap(),
                            stylesheet: Arc::new(Stylesheet {
                                rules: CssRules::new(Vec::new(), self.shared_lock),
                                media: media,
                                shared_lock: self.shared_lock.clone(),
                                origin: self.context.stylesheet_origin,
                                url_data: self.context.url_data.clone(),
                                namespaces: RwLock::new(Namespaces::default()),
                                dirty_on_viewport_size_change: AtomicBool::new(false),
                                disabled: AtomicBool::new(false),
                            })
                        }
                    }, &mut |import_rule| {
                        Arc::new(self.shared_lock.wrap(import_rule))
                    });
                    return Ok(AtRuleType::WithoutBlock(CssRule::Import(arc)))
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

                    return Ok(AtRuleType::WithoutBlock(CssRule::Namespace(Arc::new(
                        self.shared_lock.wrap(NamespaceRule {
                            prefix: opt_prefix,
                            url: url,
                        })
                    ))))
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

#[derive(Clone)]  // shallow, relatively cheap .clone
struct NestedRuleParser<'a, 'b: 'a> {
    stylesheet_origin: Origin,
    shared_lock: &'a SharedRwLock,
    context: &'a ParserContext<'b>,
    namespaces: &'b Namespaces,
}

impl<'a, 'b> NestedRuleParser<'a, 'b> {
    fn parse_nested_rules(&self, input: &mut Parser, rule_type: CssRuleType) -> Arc<Locked<CssRules>> {
        let context = ParserContext::new_with_rule_type(self.context, Some(rule_type));
        let nested_parser = NestedRuleParser {
            stylesheet_origin: self.stylesheet_origin,
            shared_lock: self.shared_lock,
            context: &context,
            namespaces: self.namespaces,
        };
        let mut iter = RuleListParser::new_for_nested_rule(input, nested_parser);
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
        CssRules::new(rules, self.shared_lock)
    }
}

#[cfg(feature = "servo")]
fn is_viewport_enabled() -> bool {
    PREFS.get("layout.viewport.enabled").as_boolean().unwrap_or(false)
}

#[cfg(not(feature = "servo"))]
fn is_viewport_enabled() -> bool {
    true
}

impl<'a, 'b> AtRuleParser for NestedRuleParser<'a, 'b> {
    type Prelude = AtRulePrelude;
    type AtRule = CssRule;

    fn parse_prelude(&mut self, name: &str, input: &mut Parser)
                     -> Result<AtRuleType<AtRulePrelude, CssRule>, ()> {
        match_ignore_ascii_case! { name,
            "media" => {
                let media_queries = parse_media_query_list(self.context, input);
                let arc = Arc::new(self.shared_lock.wrap(media_queries));
                Ok(AtRuleType::WithBlock(AtRulePrelude::Media(arc)))
            },
            "supports" => {
                let cond = SupportsCondition::parse(input)?;
                Ok(AtRuleType::WithBlock(AtRulePrelude::Supports(cond)))
            },
            "font-face" => {
                Ok(AtRuleType::WithBlock(AtRulePrelude::FontFace))
            },
            "viewport" => {
                if is_viewport_enabled() {
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
            "page" => {
                if cfg!(feature = "gecko") {
                    Ok(AtRuleType::WithBlock(AtRulePrelude::Page))
                } else {
                    Err(())
                }
            },
            _ => Err(())
        }
    }

    fn parse_block(&mut self, prelude: AtRulePrelude, input: &mut Parser) -> Result<CssRule, ()> {
        match prelude {
            AtRulePrelude::FontFace => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::FontFace));
                Ok(CssRule::FontFace(Arc::new(self.shared_lock.wrap(
                   parse_font_face_block(&context, input).into()))))
            }
            AtRulePrelude::Media(media_queries) => {
                Ok(CssRule::Media(Arc::new(self.shared_lock.wrap(MediaRule {
                    media_queries: media_queries,
                    rules: self.parse_nested_rules(input, CssRuleType::Media),
                }))))
            }
            AtRulePrelude::Supports(cond) => {
                let enabled = cond.eval(self.context);
                Ok(CssRule::Supports(Arc::new(self.shared_lock.wrap(SupportsRule {
                    condition: cond,
                    rules: self.parse_nested_rules(input, CssRuleType::Supports),
                    enabled: enabled,
                }))))
            }
            AtRulePrelude::Viewport => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Viewport));
                Ok(CssRule::Viewport(Arc::new(self.shared_lock.wrap(
                   try!(ViewportRule::parse(&context, input))))))
            }
            AtRulePrelude::Keyframes(name) => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Keyframes));
                Ok(CssRule::Keyframes(Arc::new(self.shared_lock.wrap(KeyframesRule {
                    name: name,
                    keyframes: parse_keyframe_list(&context, input, self.shared_lock),
                }))))
            }
            AtRulePrelude::Page => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Page));
                let declarations = parse_property_declaration_list(&context, input);
                Ok(CssRule::Page(Arc::new(self.shared_lock.wrap(PageRule(
                    Arc::new(self.shared_lock.wrap(declarations))
                )))))
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
        let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Style));
        let declarations = parse_property_declaration_list(&context, input);
        Ok(CssRule::Style(Arc::new(self.shared_lock.wrap(StyleRule {
            selectors: prelude,
            block: Arc::new(self.shared_lock.wrap(declarations))
        }))))
    }
}
