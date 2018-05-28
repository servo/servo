/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use {Namespace, Prefix};
use context::QuirksMode;
use cssparser::{Parser, ParserInput, RuleListParser};
use error_reporting::{ContextualParseError, ParseErrorReporter};
use fallible::FallibleVec;
use fnv::FnvHashMap;
use invalidation::media_queries::{MediaListKey, ToMediaListKey};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use media_queries::{Device, MediaList};
use parking_lot::RwLock;
use parser::{ParserContext, ParserErrorContext};
use servo_arc::Arc;
use shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard};
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use style_traits::ParsingMode;
use stylesheets::{CssRule, CssRules, Origin, UrlExtraData};
use stylesheets::loader::StylesheetLoader;
use stylesheets::rule_parser::{State, TopLevelRuleParser};
use stylesheets::rules_iterator::{EffectiveRules, EffectiveRulesIterator};
use stylesheets::rules_iterator::{NestedRuleIterationCondition, RulesIterator};

/// This structure holds the user-agent and user stylesheets.
pub struct UserAgentStylesheets {
    /// The lock used for user-agent stylesheets.
    pub shared_lock: SharedRwLock,
    /// The user or user agent stylesheets.
    pub user_or_user_agent_stylesheets: Vec<DocumentStyleSheet>,
    /// The quirks mode stylesheet.
    pub quirks_mode_stylesheet: DocumentStyleSheet,
}

/// A set of namespaces applying to a given stylesheet.
///
/// The namespace id is used in gecko
#[derive(Clone, Debug, Default, MallocSizeOf)]
#[allow(missing_docs)]
pub struct Namespaces {
    pub default: Option<Namespace>,
    pub prefixes: FnvHashMap<Prefix, Namespace>,
}

/// The contents of a given stylesheet. This effectively maps to a
/// StyleSheetInner in Gecko.
#[derive(Debug)]
pub struct StylesheetContents {
    /// List of rules in the order they were found (important for
    /// cascading order)
    pub rules: Arc<Locked<CssRules>>,
    /// The origin of this stylesheet.
    pub origin: Origin,
    /// The url data this stylesheet should use.
    pub url_data: RwLock<UrlExtraData>,
    /// The namespaces that apply to this stylesheet.
    pub namespaces: RwLock<Namespaces>,
    /// The quirks mode of this stylesheet.
    pub quirks_mode: QuirksMode,
    /// This stylesheet's source map URL.
    pub source_map_url: RwLock<Option<String>>,
    /// This stylesheet's source URL.
    pub source_url: RwLock<Option<String>>,
}

impl StylesheetContents {
    /// Parse a given CSS string, with a given url-data, origin, and
    /// quirks mode.
    pub fn from_str<R: ParseErrorReporter>(
        css: &str,
        url_data: UrlExtraData,
        origin: Origin,
        shared_lock: &SharedRwLock,
        stylesheet_loader: Option<&StylesheetLoader>,
        error_reporter: &R,
        quirks_mode: QuirksMode,
        line_number_offset: u32,
    ) -> Self {
        let namespaces = RwLock::new(Namespaces::default());
        let (rules, source_map_url, source_url) = Stylesheet::parse_rules(
            css,
            &url_data,
            origin,
            &mut *namespaces.write(),
            &shared_lock,
            stylesheet_loader,
            error_reporter,
            quirks_mode,
            line_number_offset,
        );

        Self {
            rules: CssRules::new(rules, &shared_lock),
            origin: origin,
            url_data: RwLock::new(url_data),
            namespaces: namespaces,
            quirks_mode: quirks_mode,
            source_map_url: RwLock::new(source_map_url),
            source_url: RwLock::new(source_url),
        }
    }

    /// Returns a reference to the list of rules.
    #[inline]
    pub fn rules<'a, 'b: 'a>(&'a self, guard: &'b SharedRwLockReadGuard) -> &'a [CssRule] {
        &self.rules.read_with(guard).0
    }

    /// Measure heap usage.
    #[cfg(feature = "gecko")]
    pub fn size_of(&self, guard: &SharedRwLockReadGuard, ops: &mut MallocSizeOfOps) -> usize {
        // Measurement of other fields may be added later.
        self.rules.unconditional_shallow_size_of(ops) +
            self.rules.read_with(guard).size_of(guard, ops)
    }
}

impl DeepCloneWithLock for StylesheetContents {
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        // Make a deep clone of the rules, using the new lock.
        let rules = self.rules
            .read_with(guard)
            .deep_clone_with_lock(lock, guard, params);

        Self {
            rules: Arc::new(lock.wrap(rules)),
            quirks_mode: self.quirks_mode,
            origin: self.origin,
            url_data: RwLock::new((*self.url_data.read()).clone()),
            namespaces: RwLock::new((*self.namespaces.read()).clone()),
            source_map_url: RwLock::new((*self.source_map_url.read()).clone()),
            source_url: RwLock::new((*self.source_map_url.read()).clone()),
        }
    }
}

/// The structure servo uses to represent a stylesheet.
#[derive(Debug)]
pub struct Stylesheet {
    /// The contents of this stylesheet.
    pub contents: StylesheetContents,
    /// The lock used for objects inside this stylesheet
    pub shared_lock: SharedRwLock,
    /// List of media associated with the Stylesheet.
    pub media: Arc<Locked<MediaList>>,
    /// Whether this stylesheet should be disabled.
    pub disabled: AtomicBool,
}

macro_rules! rule_filter {
    ($( $method: ident($variant:ident => $rule_type: ident), )+) => {
        $(
            #[allow(missing_docs)]
            fn $method<F>(&self, device: &Device, guard: &SharedRwLockReadGuard, mut f: F)
                where F: FnMut(&::stylesheets::$rule_type),
            {
                use stylesheets::CssRule;

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

/// A trait to represent a given stylesheet in a document.
pub trait StylesheetInDocument {
    /// Get the stylesheet origin.
    fn origin(&self, guard: &SharedRwLockReadGuard) -> Origin;

    /// Get the stylesheet quirks mode.
    fn quirks_mode(&self, guard: &SharedRwLockReadGuard) -> QuirksMode;

    /// Get whether this stylesheet is enabled.
    fn enabled(&self) -> bool;

    /// Get the media associated with this stylesheet.
    fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList>;

    /// Returns a reference to the list of rules in this stylesheet.
    fn rules<'a, 'b: 'a>(&'a self, guard: &'b SharedRwLockReadGuard) -> &'a [CssRule];

    /// Return an iterator using the condition `C`.
    #[inline]
    fn iter_rules<'a, 'b, C>(
        &'a self,
        device: &'a Device,
        guard: &'a SharedRwLockReadGuard<'b>,
    ) -> RulesIterator<'a, 'b, C>
    where
        C: NestedRuleIterationCondition,
    {
        RulesIterator::new(device, self.quirks_mode(guard), guard, self.rules(guard))
    }

    /// Returns whether the style-sheet applies for the current device.
    fn is_effective_for_device(&self, device: &Device, guard: &SharedRwLockReadGuard) -> bool {
        match self.media(guard) {
            Some(medialist) => medialist.evaluate(device, self.quirks_mode(guard)),
            None => true,
        }
    }

    /// Return an iterator over the effective rules within the style-sheet, as
    /// according to the supplied `Device`.
    #[inline]
    fn effective_rules<'a, 'b>(
        &'a self,
        device: &'a Device,
        guard: &'a SharedRwLockReadGuard<'b>,
    ) -> EffectiveRulesIterator<'a, 'b> {
        self.iter_rules::<EffectiveRules>(device, guard)
    }

    rule_filter! {
        effective_style_rules(Style => StyleRule),
        effective_media_rules(Media => MediaRule),
        effective_font_face_rules(FontFace => FontFaceRule),
        effective_font_face_feature_values_rules(FontFeatureValues => FontFeatureValuesRule),
        effective_counter_style_rules(CounterStyle => CounterStyleRule),
        effective_viewport_rules(Viewport => ViewportRule),
        effective_keyframes_rules(Keyframes => KeyframesRule),
        effective_supports_rules(Supports => SupportsRule),
        effective_page_rules(Page => PageRule),
        effective_document_rules(Document => DocumentRule),
    }
}

impl StylesheetInDocument for Stylesheet {
    fn origin(&self, _guard: &SharedRwLockReadGuard) -> Origin {
        self.contents.origin
    }

    fn quirks_mode(&self, _guard: &SharedRwLockReadGuard) -> QuirksMode {
        self.contents.quirks_mode
    }

    fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        Some(self.media.read_with(guard))
    }

    fn enabled(&self) -> bool {
        !self.disabled()
    }

    #[inline]
    fn rules<'a, 'b: 'a>(&'a self, guard: &'b SharedRwLockReadGuard) -> &'a [CssRule] {
        self.contents.rules(guard)
    }
}

/// A simple wrapper over an `Arc<Stylesheet>`, with pointer comparison, and
/// suitable for its use in a `StylesheetSet`.
#[derive(Clone)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct DocumentStyleSheet(
    #[cfg_attr(feature = "servo", ignore_malloc_size_of = "Arc")] pub Arc<Stylesheet>,
);

impl PartialEq for DocumentStyleSheet {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl ToMediaListKey for DocumentStyleSheet {
    fn to_media_list_key(&self) -> MediaListKey {
        self.0.to_media_list_key()
    }
}

impl StylesheetInDocument for DocumentStyleSheet {
    fn origin(&self, guard: &SharedRwLockReadGuard) -> Origin {
        self.0.origin(guard)
    }

    fn quirks_mode(&self, guard: &SharedRwLockReadGuard) -> QuirksMode {
        self.0.quirks_mode(guard)
    }

    fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        self.0.media(guard)
    }

    fn enabled(&self) -> bool {
        self.0.enabled()
    }

    #[inline]
    fn rules<'a, 'b: 'a>(&'a self, guard: &'b SharedRwLockReadGuard) -> &'a [CssRule] {
        self.0.rules(guard)
    }
}

impl Stylesheet {
    /// Updates an empty stylesheet from a given string of text.
    pub fn update_from_str<R>(
        existing: &Stylesheet,
        css: &str,
        url_data: UrlExtraData,
        stylesheet_loader: Option<&StylesheetLoader>,
        error_reporter: &R,
        line_number_offset: u32,
    ) where
        R: ParseErrorReporter,
    {
        let namespaces = RwLock::new(Namespaces::default());
        let (rules, source_map_url, source_url) = Stylesheet::parse_rules(
            css,
            &url_data,
            existing.contents.origin,
            &mut *namespaces.write(),
            &existing.shared_lock,
            stylesheet_loader,
            error_reporter,
            existing.contents.quirks_mode,
            line_number_offset,
        );

        *existing.contents.url_data.write() = url_data;
        mem::swap(
            &mut *existing.contents.namespaces.write(),
            &mut *namespaces.write(),
        );

        // Acquire the lock *after* parsing, to minimize the exclusive section.
        let mut guard = existing.shared_lock.write();
        *existing.contents.rules.write_with(&mut guard) = CssRules(rules);
        *existing.contents.source_map_url.write() = source_map_url;
        *existing.contents.source_url.write() = source_url;
    }

    fn parse_rules<R: ParseErrorReporter>(
        css: &str,
        url_data: &UrlExtraData,
        origin: Origin,
        namespaces: &mut Namespaces,
        shared_lock: &SharedRwLock,
        stylesheet_loader: Option<&StylesheetLoader>,
        error_reporter: &R,
        quirks_mode: QuirksMode,
        line_number_offset: u32,
    ) -> (Vec<CssRule>, Option<String>, Option<String>) {
        let mut rules = Vec::new();
        let mut input = ParserInput::new_with_line_number_offset(css, line_number_offset);
        let mut input = Parser::new(&mut input);

        let context = ParserContext::new(origin, url_data, None, ParsingMode::DEFAULT, quirks_mode);

        let error_context = ParserErrorContext { error_reporter };

        let rule_parser = TopLevelRuleParser {
            stylesheet_origin: origin,
            shared_lock,
            loader: stylesheet_loader,
            context,
            error_context,
            state: State::Start,
            dom_error: None,
            insert_rule_context: None,
            namespaces,
        };

        {
            let mut iter = RuleListParser::new_for_stylesheet(&mut input, rule_parser);

            while let Some(result) = iter.next() {
                match result {
                    Ok(rule) => {
                        // Use a fallible push here, and if it fails, just
                        // fall out of the loop.  This will cause the page to
                        // be shown incorrectly, but it's better than OOMing.
                        if rules.try_push(rule).is_err() {
                            break;
                        }
                    },
                    Err((error, slice)) => {
                        let location = error.location;
                        let error = ContextualParseError::InvalidRule(slice, error);
                        iter.parser.context.log_css_error(
                            &iter.parser.error_context,
                            location,
                            error,
                        );
                    },
                }
            }
        }

        let source_map_url = input.current_source_map_url().map(String::from);
        let source_url = input.current_source_url().map(String::from);
        (rules, source_map_url, source_url)
    }

    /// Creates an empty stylesheet and parses it with a given base url, origin
    /// and media.
    ///
    /// Effectively creates a new stylesheet and forwards the hard work to
    /// `Stylesheet::update_from_str`.
    pub fn from_str<R: ParseErrorReporter>(
        css: &str,
        url_data: UrlExtraData,
        origin: Origin,
        media: Arc<Locked<MediaList>>,
        shared_lock: SharedRwLock,
        stylesheet_loader: Option<&StylesheetLoader>,
        error_reporter: &R,
        quirks_mode: QuirksMode,
        line_number_offset: u32,
    ) -> Stylesheet {
        let contents = StylesheetContents::from_str(
            css,
            url_data,
            origin,
            &shared_lock,
            stylesheet_loader,
            error_reporter,
            quirks_mode,
            line_number_offset,
        );

        Stylesheet {
            contents,
            shared_lock,
            media,
            disabled: AtomicBool::new(false),
        }
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

#[cfg(feature = "servo")]
impl Clone for Stylesheet {
    fn clone(&self) -> Self {
        // Create a new lock for our clone.
        let lock = self.shared_lock.clone();
        let guard = self.shared_lock.read();

        // Make a deep clone of the media, using the new lock.
        let media = self.media.read_with(&guard).clone();
        let media = Arc::new(lock.wrap(media));
        let contents = self.contents
            .deep_clone_with_lock(&lock, &guard, &DeepCloneParams);

        Stylesheet {
            contents,
            media: media,
            shared_lock: lock,
            disabled: AtomicBool::new(self.disabled.load(Ordering::SeqCst)),
        }
    }
}
