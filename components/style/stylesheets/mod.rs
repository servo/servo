/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Style sheets and their CSS rules.

mod counter_style_rule;
mod document_rule;
mod font_face_rule;
mod import_rule;
pub mod keyframes_rule;
mod loader;
mod media_rule;
mod memory;
mod namespace_rule;
mod page_rule;
mod rule_list;
mod rule_parser;
mod rules_iterator;
mod style_rule;
mod stylesheet;
pub mod supports_rule;
pub mod viewport_rule;

use cssparser::{parse_one_rule, Parser, ParserInput};
use error_reporting::NullReporter;
use parser::ParserContext;
use shared_lock::{DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt;
use style_traits::PARSING_MODE_DEFAULT;
use stylearc::Arc;

pub use self::counter_style_rule::CounterStyleRule;
pub use self::document_rule::DocumentRule;
pub use self::font_face_rule::FontFaceRule;
pub use self::import_rule::ImportRule;
pub use self::keyframes_rule::KeyframesRule;
pub use self::loader::StylesheetLoader;
pub use self::media_rule::MediaRule;
pub use self::memory::{MallocSizeOf, MallocSizeOfFn, MallocSizeOfWithGuard};
pub use self::namespace_rule::NamespaceRule;
pub use self::page_rule::PageRule;
pub use self::rule_parser::{State, TopLevelRuleParser};
pub use self::rule_list::{CssRules, CssRulesHelpers};
pub use self::rules_iterator::{AllRules, EffectiveRules, NestedRuleIterationCondition, RulesIterator};
pub use self::stylesheet::{Namespaces, Stylesheet, UserAgentStylesheets};
pub use self::style_rule::StyleRule;
pub use self::supports_rule::SupportsRule;
pub use self::viewport_rule::ViewportRule;

/// Extra data that the backend may need to resolve url values.
#[cfg(not(feature = "gecko"))]
pub type UrlExtraData = ::servo_url::ServoUrl;

/// Extra data that the backend may need to resolve url values.
#[cfg(feature = "gecko")]
pub type UrlExtraData =
    ::gecko_bindings::sugar::refptr::RefPtr<::gecko_bindings::structs::URLExtraData>;

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
    CounterStyle(Arc<Locked<CounterStyleRule>>),
    Viewport(Arc<Locked<ViewportRule>>),
    Keyframes(Arc<Locked<KeyframesRule>>),
    Supports(Arc<Locked<SupportsRule>>),
    Page(Arc<Locked<PageRule>>),
    Document(Arc<Locked<DocumentRule>>),
}

impl MallocSizeOfWithGuard for CssRule {
    fn malloc_size_of_children(
        &self,
        guard: &SharedRwLockReadGuard,
        malloc_size_of: MallocSizeOfFn
    ) -> usize {
        match *self {
            CssRule::Style(ref lock) => {
                lock.read_with(guard).malloc_size_of_children(guard, malloc_size_of)
            },
            // Measurement of these fields may be added later.
            CssRule::Import(_) => 0,
            CssRule::Media(_) => 0,
            CssRule::FontFace(_) => 0,
            CssRule::CounterStyle(_) => 0,
            CssRule::Keyframes(_) => 0,
            CssRule::Namespace(_) => 0,
            CssRule::Viewport(_) => 0,
            CssRule::Supports(_) => 0,
            CssRule::Page(_) => 0,
            CssRule::Document(_)  => 0,
        }
    }
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
    // https://www.w3.org/TR/2012/WD-css3-conditional-20120911/#extentions-to-cssrule-interface
    Document            = 13,
    // https://drafts.csswg.org/css-fonts-3/#om-fontfeaturevalues
    FontFeatureValues   = 14,
    // https://drafts.csswg.org/css-device-adapt/#css-rule-interface
    Viewport            = 15,
}

#[allow(missing_docs)]
pub enum SingleRuleParseError {
    Syntax,
    Hierarchy,
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

impl CssRule {
    /// Returns the CSSOM rule type of this rule.
    pub fn rule_type(&self) -> CssRuleType {
        match *self {
            CssRule::Style(_) => CssRuleType::Style,
            CssRule::Import(_) => CssRuleType::Import,
            CssRule::Media(_) => CssRuleType::Media,
            CssRule::FontFace(_) => CssRuleType::FontFace,
            CssRule::CounterStyle(_) => CssRuleType::CounterStyle,
            CssRule::Keyframes(_) => CssRuleType::Keyframes,
            CssRule::Namespace(_) => CssRuleType::Namespace,
            CssRule::Viewport(_) => CssRuleType::Viewport,
            CssRule::Supports(_) => CssRuleType::Supports,
            CssRule::Page(_) => CssRuleType::Page,
            CssRule::Document(_)  => CssRuleType::Document,
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

    /// Parse a CSS rule.
    ///
    /// Returns a parsed CSS rule and the final state of the parser.
    ///
    /// Input state is None for a nested rule
    pub fn parse(
        css: &str,
        parent_stylesheet: &Stylesheet,
        state: Option<State>,
        loader: Option<&StylesheetLoader>
    ) -> Result<(Self, State), SingleRuleParseError> {
        let url_data = parent_stylesheet.url_data.read();
        let error_reporter = NullReporter;
        let context = ParserContext::new(
            parent_stylesheet.origin,
            &url_data,
            &error_reporter,
            None,
            PARSING_MODE_DEFAULT,
            parent_stylesheet.quirks_mode
        );

        let mut input = ParserInput::new(css);
        let mut input = Parser::new(&mut input);

        let mut guard = parent_stylesheet.namespaces.write();

        // nested rules are in the body state
        let state = state.unwrap_or(State::Body);
        let mut rule_parser = TopLevelRuleParser {
            stylesheet_origin: parent_stylesheet.origin,
            context: context,
            shared_lock: &parent_stylesheet.shared_lock,
            loader: loader,
            state: state,
            namespaces: Some(&mut *guard),
        };

        match parse_one_rule(&mut input, &mut rule_parser) {
            Ok(result) => Ok((result, rule_parser.state)),
            Err(_) => {
                Err(match rule_parser.state {
                    State::Invalid => SingleRuleParseError::Hierarchy,
                    _ => SingleRuleParseError::Syntax,
                })
            }
        }
    }
}

impl DeepCloneWithLock for CssRule {
    /// Deep clones this CssRule.
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
    ) -> CssRule {
        match *self {
            CssRule::Namespace(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Namespace(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::Import(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Import(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::Style(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Style(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard))))
            },
            CssRule::Media(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Media(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard))))
            },
            CssRule::FontFace(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::FontFace(Arc::new(lock.wrap(
                    rule.clone_conditionally_gecko_or_servo())))
            },
            CssRule::CounterStyle(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::CounterStyle(Arc::new(lock.wrap(
                    rule.clone_conditionally_gecko_or_servo())))
            },
            CssRule::Viewport(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Viewport(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::Keyframes(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Keyframes(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard))))
            },
            CssRule::Supports(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Supports(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard))))
            },
            CssRule::Page(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Page(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard))))
            },
            CssRule::Document(ref arc) => {
                let rule = arc.read_with(guard);
                CssRule::Document(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock, guard))))
            },
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
            CssRule::CounterStyle(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Viewport(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Keyframes(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Media(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Supports(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Page(ref lock) => lock.read_with(guard).to_css(guard, dest),
            CssRule::Document(ref lock) => lock.read_with(guard).to_css(guard, dest),
        }
    }
}
