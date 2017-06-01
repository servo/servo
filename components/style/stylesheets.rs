/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Style sheets and their CSS rules.

#![deny(missing_docs)]

use {Atom, Prefix, Namespace};
use context::QuirksMode;
use counter_style::{parse_counter_style_name, parse_counter_style_body};
#[cfg(feature = "servo")]
use counter_style::CounterStyleRuleData;
use cssparser::{AtRuleParser, Parser, QualifiedRuleParser};
use cssparser::{AtRuleType, RuleListParser, parse_one_rule, SourceLocation};
use cssparser::ToCss as ParserToCss;
use document_condition::DocumentCondition;
use error_reporting::{ParseErrorReporter, NullReporter};
#[cfg(feature = "servo")]
use font_face::FontFaceRuleData;
use font_face::parse_font_face_block;
#[cfg(feature = "gecko")]
pub use gecko::rules::{CounterStyleRule, FontFaceRule};
#[cfg(feature = "gecko")]
use gecko_bindings::structs::URLExtraData;
#[cfg(feature = "gecko")]
use gecko_bindings::sugar::refptr::RefPtr;
use keyframes::{Keyframe, KeyframeSelector, parse_keyframe_list};
use media_queries::{Device, MediaList, parse_media_query_list};
use parking_lot::RwLock;
use parser::{PARSING_MODE_DEFAULT, Parse, ParserContext, log_css_error};
use properties::{PropertyDeclarationBlock, parse_property_declaration_list};
use selector_parser::{SelectorImpl, SelectorParser};
use selectors::parser::SelectorList;
#[cfg(feature = "servo")]
use servo_config::prefs::PREFS;
#[cfg(not(feature = "gecko"))]
use servo_url::ServoUrl;
use shared_lock::{SharedRwLock, Locked, ToCssWithGuard, SharedRwLockReadGuard};
use smallvec::SmallVec;
use std::{fmt, mem};
use std::borrow::Borrow;
use std::cell::Cell;
use std::mem::align_of;
use std::os::raw::c_void;
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use str::starts_with_ignore_ascii_case;
use style_traits::ToCss;
use stylearc::Arc;
use stylist::FnvHashMap;
use supports::SupportsCondition;
use values::{CustomIdent, KeyframesName};
use values::specified::NamespaceId;
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
///
/// The namespace id is used in gecko
#[derive(Clone, Default, Debug)]
#[allow(missing_docs)]
pub struct Namespaces {
    pub default: Option<(Namespace, NamespaceId)>,
    pub prefixes: FnvHashMap<Prefix, (Namespace, NamespaceId)>,
}

/// Like gecko_bindings::structs::MallocSizeOf, but without the Option<> wrapper. Note that
/// functions of this type should not be called via do_malloc_size_of(), rather than directly.
pub type MallocSizeOfFn = unsafe extern "C" fn(ptr: *const c_void) -> usize;

/// Call malloc_size_of on ptr, first checking that the allocation isn't empty.
pub unsafe fn do_malloc_size_of<T>(malloc_size_of: MallocSizeOfFn, ptr: *const T) -> usize {
    if ptr as usize <= align_of::<T>() {
        0
    } else {
        malloc_size_of(ptr as *const c_void)
    }
}

/// Trait for measuring the size of heap data structures.
pub trait MallocSizeOf {
    /// Measure the size of any heap-allocated structures that hang off this value, but not the
    /// space taken up by the value itself.
    fn malloc_size_of_children(&self, malloc_size_of: MallocSizeOfFn) -> usize;
}

/// Like MallocSizeOf, but operates with the global SharedRwLockReadGuard locked.
pub trait MallocSizeOfWithGuard {
    /// Like MallocSizeOf::malloc_size_of_children, but with a |guard| argument.
    fn malloc_size_of_children(&self, guard: &SharedRwLockReadGuard,
                               malloc_size_of: MallocSizeOfFn) -> usize;
}

impl<A: MallocSizeOf, B: MallocSizeOf> MallocSizeOf for (A, B) {
    fn malloc_size_of_children(&self, malloc_size_of: MallocSizeOfFn) -> usize {
        self.0.malloc_size_of_children(malloc_size_of) +
            self.1.malloc_size_of_children(malloc_size_of)
    }
}

impl<T: MallocSizeOf> MallocSizeOf for Vec<T> {
    fn malloc_size_of_children(&self, malloc_size_of: MallocSizeOfFn) -> usize {
        self.iter().fold(
            unsafe { do_malloc_size_of(malloc_size_of, self.as_ptr()) },
            |n, elem| n + elem.malloc_size_of_children(malloc_size_of))
    }
}

impl<T: MallocSizeOfWithGuard> MallocSizeOfWithGuard for Vec<T> {
    fn malloc_size_of_children(&self, guard: &SharedRwLockReadGuard,
                               malloc_size_of: MallocSizeOfFn) -> usize {
        self.iter().fold(
            unsafe { do_malloc_size_of(malloc_size_of, self.as_ptr()) },
            |n, elem| n + elem.malloc_size_of_children(guard, malloc_size_of))
    }
}

/// A list of CSS rules.
#[derive(Debug)]
pub struct CssRules(pub Vec<CssRule>);

impl CssRules {
    /// Whether this CSS rules is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Creates a deep clone where each CssRule has also been cloned with
    /// the provided lock.
    fn deep_clone_with_lock(&self,
                            lock: &SharedRwLock) -> CssRules {
        CssRules(
            self.0.iter().map(|ref x| x.deep_clone_with_lock(lock)).collect()
        )
    }
}

impl MallocSizeOfWithGuard for CssRules {
    fn malloc_size_of_children(&self, guard: &SharedRwLockReadGuard,
                               malloc_size_of: MallocSizeOfFn) -> usize {
        self.0.malloc_size_of_children(guard, malloc_size_of)
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
    /// The quirks mode of this stylesheet.
    pub quirks_mode: QuirksMode,
}

impl MallocSizeOfWithGuard for Stylesheet {
    fn malloc_size_of_children(&self, guard: &SharedRwLockReadGuard,
                               malloc_size_of: MallocSizeOfFn) -> usize {
        // Measurement of other fields may be added later.
        self.rules.read_with(guard).malloc_size_of_children(guard, malloc_size_of)
    }
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
    CounterStyle(Arc<Locked<CounterStyleRule>>),
    Viewport(Arc<Locked<ViewportRule>>),
    Keyframes(Arc<Locked<KeyframesRule>>),
    Supports(Arc<Locked<SupportsRule>>),
    Page(Arc<Locked<PageRule>>),
    Document(Arc<Locked<DocumentRule>>),
}

impl MallocSizeOfWithGuard for CssRule {
    fn malloc_size_of_children(&self, guard: &SharedRwLockReadGuard,
                               malloc_size_of: MallocSizeOfFn) -> usize {
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

impl CssRule {
    #[allow(missing_docs)]
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

    // input state is None for a nested rule
    // Returns a parsed CSS rule and the final state of the parser
    #[allow(missing_docs)]
    pub fn parse(css: &str,
                 parent_stylesheet: &Stylesheet,
                 state: Option<State>,
                 loader: Option<&StylesheetLoader>)
                 -> Result<(Self, State), SingleRuleParseError> {
        let error_reporter = NullReporter;
        let mut context = ParserContext::new(parent_stylesheet.origin,
                                             &parent_stylesheet.url_data,
                                             &error_reporter,
                                             None,
                                             PARSING_MODE_DEFAULT,
                                             parent_stylesheet.quirks_mode);
        context.namespaces = Some(&parent_stylesheet.namespaces);
        let mut input = Parser::new(css);

        // nested rules are in the body state
        let state = state.unwrap_or(State::Body);
        let mut rule_parser = TopLevelRuleParser {
            stylesheet_origin: parent_stylesheet.origin,
            context: context,
            shared_lock: &parent_stylesheet.shared_lock,
            loader: loader,
            state: Cell::new(state),
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

    /// Deep clones this CssRule.
    fn deep_clone_with_lock(&self,
                            lock: &SharedRwLock) -> CssRule {
        let guard = lock.read();
        match *self {
            CssRule::Namespace(ref arc) => {
                let rule = arc.read_with(&guard);
                CssRule::Namespace(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::Import(ref arc) => {
                let rule = arc.read_with(&guard);
                CssRule::Import(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::Style(ref arc) => {
                let rule = arc.read_with(&guard);
                CssRule::Style(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock))))
            },
            CssRule::Media(ref arc) => {
                let rule = arc.read_with(&guard);
                CssRule::Media(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock))))
            },
            CssRule::FontFace(ref arc) => {
                let rule = arc.read_with(&guard);
                CssRule::FontFace(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::CounterStyle(ref arc) => {
                let rule = arc.read_with(&guard);
                CssRule::CounterStyle(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::Viewport(ref arc) => {
                let rule = arc.read_with(&guard);
                CssRule::Viewport(Arc::new(lock.wrap(rule.clone())))
            },
            CssRule::Keyframes(ref arc) => {
                let rule = arc.read_with(&guard);
                CssRule::Keyframes(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock))))
            },
            CssRule::Supports(ref arc) => {
                let rule = arc.read_with(&guard);
                CssRule::Supports(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock))))
            },
            CssRule::Page(ref arc) => {
                let rule = arc.read_with(&guard);
                CssRule::Page(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock))))
            },
            CssRule::Document(ref arc) => {
                let rule = arc.read_with(&guard);
                CssRule::Document(Arc::new(
                    lock.wrap(rule.deep_clone_with_lock(lock))))
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

/// Calculates the location of a rule's source given an offset.
fn get_location_with_offset(location: SourceLocation, offset: u64)
    -> SourceLocation {
    SourceLocation {
        line: location.line + offset as usize - 1,
        column: location.column,
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub struct NamespaceRule {
    /// `None` for the default Namespace
    pub prefix: Option<Prefix>,
    pub url: Namespace,
    pub source_location: SourceLocation,
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

    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl Clone for ImportRule {
    fn clone(&self) -> ImportRule {
        let stylesheet: &Stylesheet = self.stylesheet.borrow();
        ImportRule {
            url: self.url.clone(),
            stylesheet: Arc::new(stylesheet.clone()),
            source_location: self.source_location.clone(),
        }
    }
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
    pub name: KeyframesName,
    /// The keyframes specified for this CSS rule.
    pub keyframes: Vec<Arc<Locked<Keyframe>>>,
    /// Vendor prefix type the @keyframes has.
    pub vendor_prefix: Option<VendorPrefix>,
    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl ToCssWithGuard for KeyframesRule {
    // Serialization of KeyframesRule is not specced.
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        try!(dest.write_str("@keyframes "));
        try!(self.name.to_css(dest));
        try!(dest.write_str(" {"));
        let iter = self.keyframes.iter();
        for lock in iter {
            try!(dest.write_str("\n"));
            let keyframe = lock.read_with(&guard);
            try!(keyframe.to_css(guard, dest));
        }
        dest.write_str("\n}")
    }
}

impl KeyframesRule {
    /// Returns the index of the last keyframe that matches the given selector.
    /// If the selector is not valid, or no keyframe is found, returns None.
    ///
    /// Related spec:
    /// https://drafts.csswg.org/css-animations-1/#interface-csskeyframesrule-findrule
    pub fn find_rule(&self, guard: &SharedRwLockReadGuard, selector: &str) -> Option<usize> {
        if let Ok(selector) = Parser::new(selector).parse_entirely(KeyframeSelector::parse) {
            for (i, keyframe) in self.keyframes.iter().enumerate().rev() {
                if keyframe.read_with(guard).selector == selector {
                    return Some(i);
                }
            }
        }
        None
    }
}

impl KeyframesRule {
    /// Deep clones this KeyframesRule.
    fn deep_clone_with_lock(&self,
                            lock: &SharedRwLock) -> KeyframesRule {
        let guard = lock.read();
        KeyframesRule {
            name: self.name.clone(),
            keyframes: self.keyframes.iter()
                .map(|ref x| Arc::new(lock.wrap(
                    x.read_with(&guard).deep_clone_with_lock(lock))))
                .collect(),
            vendor_prefix: self.vendor_prefix.clone(),
            source_location: self.source_location.clone(),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct MediaRule {
    pub media_queries: Arc<Locked<MediaList>>,
    pub rules: Arc<Locked<CssRules>>,
    pub source_location: SourceLocation,
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

impl MediaRule {
    /// Deep clones this MediaRule.
    fn deep_clone_with_lock(&self,
                            lock: &SharedRwLock) -> MediaRule {
        let guard = lock.read();
        let media_queries = self.media_queries.read_with(&guard);
        let rules = self.rules.read_with(&guard);
        MediaRule {
            media_queries: Arc::new(lock.wrap(media_queries.clone())),
            rules: Arc::new(lock.wrap(rules.deep_clone_with_lock(lock))),
            source_location: self.source_location.clone(),
        }
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
    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
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

impl SupportsRule {
    fn deep_clone_with_lock(&self,
                            lock: &SharedRwLock) -> SupportsRule {
        let guard = lock.read();
        let rules = self.rules.read_with(&guard);
        SupportsRule {
            condition: self.condition.clone(),
            rules: Arc::new(lock.wrap(rules.deep_clone_with_lock(lock))),
            enabled: self.enabled,
            source_location: self.source_location.clone(),
        }
    }
}

/// A [`@page`][page] rule.  This implements only a limited subset of the CSS 2.2 syntax.  In this
/// subset, [page selectors][page-selectors] are not implemented.
///
/// [page]: https://drafts.csswg.org/css2/page.html#page-box
/// [page-selectors]: https://drafts.csswg.org/css2/page.html#page-selectors
#[allow(missing_docs)]
#[derive(Debug)]
pub struct PageRule {
    pub block: Arc<Locked<PropertyDeclarationBlock>>,
    pub source_location: SourceLocation,
}

impl ToCssWithGuard for PageRule {
    // Serialization of PageRule is not specced, adapted from steps for StyleRule.
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        dest.write_str("@page { ")?;
        let declaration_block = self.block.read_with(guard);
        declaration_block.to_css(dest)?;
        if declaration_block.declarations().len() > 0 {
            write!(dest, " ")?;
        }
        dest.write_str("}")
    }
}

impl PageRule {
    fn deep_clone_with_lock(&self,
                            lock: &SharedRwLock) -> PageRule {
        let guard = lock.read();
        PageRule {
            block: Arc::new(lock.wrap(self.block.read_with(&guard).clone())),
            source_location: self.source_location.clone(),
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct StyleRule {
    pub selectors: SelectorList<SelectorImpl>,
    pub block: Arc<Locked<PropertyDeclarationBlock>>,
    pub source_location: SourceLocation,
}

impl MallocSizeOfWithGuard for StyleRule {
    fn malloc_size_of_children(&self, guard: &SharedRwLockReadGuard,
                               malloc_size_of: MallocSizeOfFn) -> usize {
        // Measurement of other fields may be added later.
        self.block.read_with(guard).malloc_size_of_children(malloc_size_of)
    }
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

impl StyleRule {
    /// Deep clones this StyleRule.
    fn deep_clone_with_lock(&self,
                            lock: &SharedRwLock) -> StyleRule {
        let guard = lock.read();
        StyleRule {
            selectors: self.selectors.clone(),
            block: Arc::new(lock.wrap(self.block.read_with(&guard).clone())),
            source_location: self.source_location.clone(),
        }
    }
}

/// A @font-face rule
#[cfg(feature = "servo")]
pub type FontFaceRule = FontFaceRuleData;

/// A @counter-style rule
#[cfg(feature = "servo")]
pub type CounterStyleRule = CounterStyleRuleData;

#[derive(Debug)]
/// A @-moz-document rule
pub struct DocumentRule {
    /// The parsed condition
    pub condition: DocumentCondition,
    /// Child rules
    pub rules: Arc<Locked<CssRules>>,
    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl ToCssWithGuard for DocumentRule {
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        try!(dest.write_str("@-moz-document "));
        try!(self.condition.to_css(dest));
        try!(dest.write_str(" {"));
        for rule in self.rules.read_with(guard).0.iter() {
            try!(dest.write_str(" "));
            try!(rule.to_css(guard, dest));
        }
        dest.write_str(" }")
    }
}

impl DocumentRule {
    /// Deep clones this DocumentRule.
    fn deep_clone_with_lock(&self,
                            lock: &SharedRwLock) -> DocumentRule {
        let guard = lock.read();
        let rules = self.rules.read_with(&guard);
        DocumentRule {
            condition: self.condition.clone(),
            rules: Arc::new(lock.wrap(rules.deep_clone_with_lock(lock))),
            source_location: self.source_location.clone(),
        }
    }
}

/// A trait that describes statically which rules are iterated for a given
/// RulesIterator.
pub trait NestedRuleIterationCondition {
    /// Whether we should process the nested rules in a given `@import` rule.
    fn process_import(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &ImportRule)
        -> bool;

    /// Whether we should process the nested rules in a given `@media` rule.
    fn process_media(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &MediaRule)
        -> bool;

    /// Whether we should process the nested rules in a given `@-moz-document` rule.
    fn process_document(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &DocumentRule)
        -> bool;

    /// Whether we should process the nested rules in a given `@supports` rule.
    fn process_supports(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &SupportsRule)
        -> bool;
}

/// A struct that represents the condition that a rule applies to the document.
pub struct EffectiveRules;

impl NestedRuleIterationCondition for EffectiveRules {
    fn process_import(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &ImportRule)
        -> bool
    {
        rule.stylesheet.media.read_with(guard).evaluate(device, quirks_mode)
    }

    fn process_media(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &MediaRule)
        -> bool
    {
        rule.media_queries.read_with(guard).evaluate(device, quirks_mode)
    }

    fn process_document(
        _: &SharedRwLockReadGuard,
        device: &Device,
        _: QuirksMode,
        rule: &DocumentRule)
        -> bool
    {
        rule.condition.evaluate(device)
    }

    fn process_supports(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        rule: &SupportsRule)
        -> bool
    {
        rule.enabled
    }
}

/// A filter that processes all the rules in a rule list.
pub struct AllRules;

impl NestedRuleIterationCondition for AllRules {
    fn process_import(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &ImportRule)
        -> bool
    {
        true
    }

    /// Whether we should process the nested rules in a given `@media` rule.
    fn process_media(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &MediaRule)
        -> bool
    {
        true
    }

    /// Whether we should process the nested rules in a given `@-moz-document` rule.
    fn process_document(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &DocumentRule)
        -> bool
    {
        true
    }

    /// Whether we should process the nested rules in a given `@supports` rule.
    fn process_supports(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &SupportsRule)
        -> bool
    {
        true
    }
}

/// An iterator over all the effective rules of a stylesheet.
///
/// NOTE: This iterator recurses into `@import` rules.
pub type EffectiveRulesIterator<'a, 'b> = RulesIterator<'a, 'b, EffectiveRules>;

/// An iterator over a list of rules.
pub struct RulesIterator<'a, 'b, C>
    where 'b: 'a,
          C: NestedRuleIterationCondition + 'static,
{
    device: &'a Device,
    quirks_mode: QuirksMode,
    guard: &'a SharedRwLockReadGuard<'b>,
    stack: SmallVec<[slice::Iter<'a, CssRule>; 3]>,
    _phantom: ::std::marker::PhantomData<C>,
}

impl<'a, 'b, C> RulesIterator<'a, 'b, C>
    where 'b: 'a,
          C: NestedRuleIterationCondition + 'static,
{
    /// Creates a new `RulesIterator` to iterate over `rules`.
    pub fn new(
        device: &'a Device,
        quirks_mode: QuirksMode,
        guard: &'a SharedRwLockReadGuard<'b>,
        rules: &'a CssRules)
        -> Self
    {
        let mut stack = SmallVec::new();
        stack.push(rules.0.iter());
        Self {
            device: device,
            quirks_mode: quirks_mode,
            guard: guard,
            stack: stack,
            _phantom: ::std::marker::PhantomData,
        }
    }

    /// Skips all the remaining children of the last nested rule processed.
    pub fn skip_children(&mut self) {
        self.stack.pop();
    }
}

impl<'a, 'b, C> Iterator for RulesIterator<'a, 'b, C>
    where 'b: 'a,
          C: NestedRuleIterationCondition + 'static,
{
    type Item = &'a CssRule;

    fn next(&mut self) -> Option<Self::Item> {
        let mut nested_iter_finished = false;
        while !self.stack.is_empty() {
            if nested_iter_finished {
                self.stack.pop();
                nested_iter_finished = false;
                continue;
            }

            let rule;
            let sub_iter;
            {
                let mut nested_iter = self.stack.last_mut().unwrap();
                rule = match nested_iter.next() {
                    Some(r) => r,
                    None => {
                        nested_iter_finished = true;
                        continue
                    }
                };

                sub_iter = match *rule {
                    CssRule::Import(ref import_rule) => {
                        let import_rule = import_rule.read_with(self.guard);

                        if C::process_import(self.guard, self.device, self.quirks_mode, import_rule) {
                            Some(import_rule.stylesheet.rules.read_with(self.guard).0.iter())
                        } else {
                            None
                        }
                    }
                    CssRule::Document(ref doc_rule) => {
                        let doc_rule = doc_rule.read_with(self.guard);
                        if C::process_document(self.guard, self.device, self.quirks_mode, doc_rule) {
                            Some(doc_rule.rules.read_with(self.guard).0.iter())
                        } else {
                            None
                        }
                    }
                    CssRule::Media(ref lock) => {
                        let media_rule = lock.read_with(self.guard);
                        if C::process_media(self.guard, self.device, self.quirks_mode, media_rule) {
                            Some(media_rule.rules.read_with(self.guard).0.iter())
                        } else {
                            None
                        }
                    }
                    CssRule::Supports(ref lock) => {
                        let supports_rule = lock.read_with(self.guard);
                        if C::process_supports(self.guard, self.device, self.quirks_mode, supports_rule) {
                            Some(supports_rule.rules.read_with(self.guard).0.iter())
                        } else {
                            None
                        }
                    }
                    CssRule::Namespace(_) |
                    CssRule::Style(_) |
                    CssRule::FontFace(_) |
                    CssRule::CounterStyle(_) |
                    CssRule::Viewport(_) |
                    CssRule::Keyframes(_) |
                    CssRule::Page(_) => None,
                };
            }

            if let Some(sub_iter) = sub_iter {
                self.stack.push(sub_iter);
            }

            return Some(rule);
        }

        None
    }
}

impl Stylesheet {
    /// Updates an empty stylesheet from a given string of text.
    pub fn update_from_str(existing: &Stylesheet,
                           css: &str,
                           url_data: &UrlExtraData,
                           stylesheet_loader: Option<&StylesheetLoader>,
                           error_reporter: &ParseErrorReporter,
                           line_number_offset: u64) {
        let namespaces = RwLock::new(Namespaces::default());
        // FIXME: we really should update existing.url_data with the given url_data,
        // otherwise newly inserted rule may not have the right base url.
        let (rules, dirty_on_viewport_size_change) = Stylesheet::parse_rules(
            css, url_data, existing.origin, &namespaces,
            &existing.shared_lock, stylesheet_loader, error_reporter,
            existing.quirks_mode, line_number_offset);
        mem::swap(&mut *existing.namespaces.write(), &mut *namespaces.write());
        existing.dirty_on_viewport_size_change
            .store(dirty_on_viewport_size_change, Ordering::Release);

        // Acquire the lock *after* parsing, to minimize the exclusive section.
        let mut guard = existing.shared_lock.write();
        *existing.rules.write_with(&mut guard) = CssRules(rules);
    }

    fn parse_rules(css: &str,
                   url_data: &UrlExtraData,
                   origin: Origin,
                   namespaces: &RwLock<Namespaces>,
                   shared_lock: &SharedRwLock,
                   stylesheet_loader: Option<&StylesheetLoader>,
                   error_reporter: &ParseErrorReporter,
                   quirks_mode: QuirksMode,
                   line_number_offset: u64)
                   -> (Vec<CssRule>, bool) {
        let mut rules = Vec::new();
        let mut input = Parser::new(css);
        let mut context = ParserContext::new_with_line_number_offset(origin, url_data, error_reporter,
                                                                     line_number_offset,
                                                                     PARSING_MODE_DEFAULT,
                                                                     quirks_mode);
        context.namespaces = Some(namespaces);
        let rule_parser = TopLevelRuleParser {
            stylesheet_origin: origin,
            shared_lock: shared_lock,
            loader: stylesheet_loader,
            context: context,
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
                    quirks_mode: QuirksMode,
                    line_number_offset: u64)
                    -> Stylesheet {
        let namespaces = RwLock::new(Namespaces::default());
        let (rules, dirty_on_viewport_size_change) = Stylesheet::parse_rules(
            css, &url_data, origin, &namespaces,
            &shared_lock, stylesheet_loader, error_reporter, quirks_mode, line_number_offset,
        );
        Stylesheet {
            origin: origin,
            url_data: url_data,
            namespaces: namespaces,
            rules: CssRules::new(rules, &shared_lock),
            media: media,
            shared_lock: shared_lock,
            dirty_on_viewport_size_change: AtomicBool::new(dirty_on_viewport_size_change),
            disabled: AtomicBool::new(false),
            quirks_mode: quirks_mode,
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
        self.media.read_with(guard).evaluate(device, self.quirks_mode)
    }

    /// Return an iterator over the effective rules within the style-sheet, as
    /// according to the supplied `Device`.
    #[inline]
    pub fn effective_rules<'a, 'b>(
        &'a self,
        device: &'a Device,
        guard: &'a SharedRwLockReadGuard<'b>)
        -> EffectiveRulesIterator<'a, 'b>
    {
        self.iter_rules::<'a, 'b, EffectiveRules>(device, guard)
    }

    /// Return an iterator using the condition `C`.
    #[inline]
    pub fn iter_rules<'a, 'b, C>(
        &'a self,
        device: &'a Device,
        guard: &'a SharedRwLockReadGuard<'b>)
        -> RulesIterator<'a, 'b, C>
        where C: NestedRuleIterationCondition,
    {
        RulesIterator::new(
            device,
            self.quirks_mode,
            guard,
            &self.rules.read_with(guard))
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

impl Clone for Stylesheet {
    fn clone(&self) -> Stylesheet {
        // Create a new lock for our clone.
        let lock = self.shared_lock.clone();
        let guard = self.shared_lock.read();

        // Make a deep clone of the rules, using the new lock.
        let rules = self.rules.read_with(&guard);
        let cloned_rules = rules.deep_clone_with_lock(&lock);

        // Make a deep clone of the media, using the new lock.
        let media = self.media.read_with(&guard);
        let cloned_media = media.clone();

        Stylesheet {
            rules: Arc::new(lock.wrap(cloned_rules)),
            media: Arc::new(lock.wrap(cloned_media)),
            origin: self.origin,
            url_data: self.url_data.clone(),
            shared_lock: lock,
            namespaces: RwLock::new((*self.namespaces.read()).clone()),
            dirty_on_viewport_size_change: AtomicBool::new(
                self.dirty_on_viewport_size_change.load(Ordering::SeqCst)),
            disabled: AtomicBool::new(self.disabled.load(Ordering::SeqCst)),
            quirks_mode: self.quirks_mode,
        }
    }
}

macro_rules! rule_filter {
    ($( $method: ident($variant:ident => $rule_type: ident), )+) => {
        impl Stylesheet {
            $(
                #[allow(missing_docs)]
                pub fn $method<F>(&self, device: &Device, guard: &SharedRwLockReadGuard, mut f: F)
                    where F: FnMut(&$rule_type),
                {
                    for rule in self.effective_rules(device, guard) {
                        if let CssRule::$variant(ref lock) = *rule {
                            let rule = lock.read_with(guard);
                            f(&rule)
                        }
                    }
                }
            )+
        }
    }
}

rule_filter! {
    effective_style_rules(Style => StyleRule),
    effective_media_rules(Media => MediaRule),
    effective_font_face_rules(FontFace => FontFaceRule),
    effective_counter_style_rules(CounterStyle => CounterStyleRule),
    effective_viewport_rules(Viewport => ViewportRule),
    effective_keyframes_rules(Keyframes => KeyframesRule),
    effective_supports_rules(Supports => SupportsRule),
    effective_page_rules(Page => PageRule),
    effective_document_rules(Document => DocumentRule),
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// Vendor prefix.
pub enum VendorPrefix {
    /// -moz prefix.
    Moz,
    /// -webkit prefix.
    WebKit,
}

enum AtRulePrelude {
    /// A @font-face rule prelude.
    FontFace(SourceLocation),
    /// A @counter-style rule prelude, with its counter style name.
    CounterStyle(CustomIdent),
    /// A @media rule prelude, with its media queries.
    Media(Arc<Locked<MediaList>>, SourceLocation),
    /// An @supports rule, with its conditional
    Supports(SupportsCondition, SourceLocation),
    /// A @viewport rule prelude.
    Viewport,
    /// A @keyframes rule, with its animation name and vendor prefix if exists.
    Keyframes(KeyframesName, Option<VendorPrefix>, SourceLocation),
    /// A @page rule prelude.
    Page(SourceLocation),
    /// A @document rule, with its conditional.
    Document(DocumentCondition, SourceLocation),
}


#[cfg(feature = "gecko")]
fn register_namespace(ns: &Namespace) -> Result<i32, ()> {
    let id = unsafe { ::gecko_bindings::bindings::Gecko_RegisterNamespace(ns.0.as_ptr()) };
    if id == -1 {
        Err(())
    } else {
        Ok(id)
    }
}

#[cfg(feature = "servo")]
fn register_namespace(_: &Namespace) -> Result<(), ()> {
    Ok(()) // servo doesn't use namespace ids
}

impl<'a> AtRuleParser for TopLevelRuleParser<'a> {
    type Prelude = AtRulePrelude;
    type AtRule = CssRule;

    fn parse_prelude(&mut self, name: &str, input: &mut Parser)
                     -> Result<AtRuleType<AtRulePrelude, CssRule>, ()> {
        let location = get_location_with_offset(input.current_source_location(),
                                                self.context.line_number_offset);
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
                                quirks_mode: self.context.quirks_mode,
                            }),
                            source_location: location,
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

                    let id = register_namespace(&url)?;

                    let opt_prefix = if let Ok(prefix) = prefix_result {
                        let prefix = Prefix::from(prefix);
                        self.context.namespaces.expect("namespaces must be set whilst parsing rules")
                                               .write().prefixes.insert(prefix.clone(), (url.clone(), id));
                        Some(prefix)
                    } else {
                        self.context.namespaces.expect("namespaces must be set whilst parsing rules")
                                               .write().default = Some((url.clone(), id));
                        None
                    };

                    return Ok(AtRuleType::WithoutBlock(CssRule::Namespace(Arc::new(
                        self.shared_lock.wrap(NamespaceRule {
                            prefix: opt_prefix,
                            url: url,
                            source_location: location,
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
}

impl<'a, 'b> NestedRuleParser<'a, 'b> {
    fn parse_nested_rules(&self, input: &mut Parser, rule_type: CssRuleType) -> Arc<Locked<CssRules>> {
        let context = ParserContext::new_with_rule_type(self.context, Some(rule_type));
        let nested_parser = NestedRuleParser {
            stylesheet_origin: self.stylesheet_origin,
            shared_lock: self.shared_lock,
            context: &context,
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
        let location = get_location_with_offset(input.current_source_location(),
                                                self.context.line_number_offset);
        match_ignore_ascii_case! { name,
            "media" => {
                let media_queries = parse_media_query_list(self.context, input);
                let arc = Arc::new(self.shared_lock.wrap(media_queries));
                Ok(AtRuleType::WithBlock(AtRulePrelude::Media(arc, location)))
            },
            "supports" => {
                let cond = SupportsCondition::parse(input)?;
                Ok(AtRuleType::WithBlock(AtRulePrelude::Supports(cond, location)))
            },
            "font-face" => {
                Ok(AtRuleType::WithBlock(AtRulePrelude::FontFace(location)))
            },
            "counter-style" => {
                if !cfg!(feature = "gecko") {
                    // Support for this rule is not fully implemented in Servo yet.
                    return Err(())
                }
                let name = parse_counter_style_name(input)?;
                // ASCII-case-insensitive matches for "decimal" are already lower-cased
                // by `parse_counter_style_name`, so we can use == here.
                // FIXME: https://bugzilla.mozilla.org/show_bug.cgi?id=1359323 use atom!("decimal")
                if name.0 == Atom::from("decimal") {
                    return Err(())
                }
                Ok(AtRuleType::WithBlock(AtRulePrelude::CounterStyle(name)))
            },
            "viewport" => {
                if is_viewport_enabled() {
                    Ok(AtRuleType::WithBlock(AtRulePrelude::Viewport))
                } else {
                    Err(())
                }
            },
            "keyframes" | "-webkit-keyframes" | "-moz-keyframes" => {
                let prefix = if starts_with_ignore_ascii_case(name, "-webkit-") {
                    Some(VendorPrefix::WebKit)
                } else if starts_with_ignore_ascii_case(name, "-moz-") {
                    Some(VendorPrefix::Moz)
                } else {
                    None
                };
                if cfg!(feature = "servo") &&
                   prefix.as_ref().map_or(false, |p| matches!(*p, VendorPrefix::Moz)) {
                    // Servo should not support @-moz-keyframes.
                    return Err(())
                }
                let name = KeyframesName::parse(self.context, input)?;

                Ok(AtRuleType::WithBlock(AtRulePrelude::Keyframes(name, prefix, location)))
            },
            "page" => {
                if cfg!(feature = "gecko") {
                    Ok(AtRuleType::WithBlock(AtRulePrelude::Page(location)))
                } else {
                    Err(())
                }
            },
            "-moz-document" => {
                if cfg!(feature = "gecko") {
                    let cond = DocumentCondition::parse(self.context, input)?;
                    Ok(AtRuleType::WithBlock(AtRulePrelude::Document(cond, location)))
                } else {
                    Err(())
                }
            },
            _ => Err(())
        }
    }

    fn parse_block(&mut self, prelude: AtRulePrelude, input: &mut Parser) -> Result<CssRule, ()> {
        match prelude {
            AtRulePrelude::FontFace(location) => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::FontFace));
                Ok(CssRule::FontFace(Arc::new(self.shared_lock.wrap(
                   parse_font_face_block(&context, input, location).into()))))
            }
            AtRulePrelude::CounterStyle(name) => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::CounterStyle));
                Ok(CssRule::CounterStyle(Arc::new(self.shared_lock.wrap(
                   parse_counter_style_body(name, &context, input)?.into()))))
            }
            AtRulePrelude::Media(media_queries, location) => {
                Ok(CssRule::Media(Arc::new(self.shared_lock.wrap(MediaRule {
                    media_queries: media_queries,
                    rules: self.parse_nested_rules(input, CssRuleType::Media),
                    source_location: location,
                }))))
            }
            AtRulePrelude::Supports(cond, location) => {
                let enabled = cond.eval(self.context);
                Ok(CssRule::Supports(Arc::new(self.shared_lock.wrap(SupportsRule {
                    condition: cond,
                    rules: self.parse_nested_rules(input, CssRuleType::Supports),
                    enabled: enabled,
                    source_location: location,
                }))))
            }
            AtRulePrelude::Viewport => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Viewport));
                Ok(CssRule::Viewport(Arc::new(self.shared_lock.wrap(
                   try!(ViewportRule::parse(&context, input))))))
            }
            AtRulePrelude::Keyframes(name, prefix, location) => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Keyframes));
                Ok(CssRule::Keyframes(Arc::new(self.shared_lock.wrap(KeyframesRule {
                    name: name,
                    keyframes: parse_keyframe_list(&context, input, self.shared_lock),
                    vendor_prefix: prefix,
                    source_location: location,
                }))))
            }
            AtRulePrelude::Page(location) => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Page));
                let declarations = parse_property_declaration_list(&context, input);
                Ok(CssRule::Page(Arc::new(self.shared_lock.wrap(PageRule {
                    block: Arc::new(self.shared_lock.wrap(declarations)),
                    source_location: location,
                }))))
            }
            AtRulePrelude::Document(cond, location) => {
                if cfg!(feature = "gecko") {
                    Ok(CssRule::Document(Arc::new(self.shared_lock.wrap(DocumentRule {
                        condition: cond,
                        rules: self.parse_nested_rules(input, CssRuleType::Document),
                        source_location: location,
                    }))))
                } else {
                    unreachable!()
                }
            }
        }
    }
}

impl<'a, 'b> QualifiedRuleParser for NestedRuleParser<'a, 'b> {
    type Prelude = SelectorList<SelectorImpl>;
    type QualifiedRule = CssRule;

    fn parse_prelude(&mut self, input: &mut Parser) -> Result<SelectorList<SelectorImpl>, ()> {
        let ns = self.context.namespaces.expect("namespaces must be set when parsing rules").read();
        let selector_parser = SelectorParser {
            stylesheet_origin: self.stylesheet_origin,
            namespaces: &*ns,
        };
        SelectorList::parse(&selector_parser, input)
    }

    fn parse_block(&mut self, prelude: SelectorList<SelectorImpl>, input: &mut Parser)
                   -> Result<CssRule, ()> {
        let location = get_location_with_offset(input.current_source_location(),
                                                self.context.line_number_offset);
        let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Style));
        let declarations = parse_property_declaration_list(&context, input);
        Ok(CssRule::Style(Arc::new(self.shared_lock.wrap(StyleRule {
            selectors: prelude,
            block: Arc::new(self.shared_lock.wrap(declarations)),
            source_location: location,
        }))))
    }
}
