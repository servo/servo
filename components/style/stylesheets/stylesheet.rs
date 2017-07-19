/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use {Prefix, Namespace};
use context::QuirksMode;
use cssparser::{Parser, RuleListParser, ParserInput};
use error_reporting::{ParseErrorReporter, ContextualParseError};
use fnv::FnvHashMap;
use media_queries::{MediaList, Device};
use parking_lot::RwLock;
use parser::{ParserContext, log_css_error};
use servo_arc::Arc;
use shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard};
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use style_traits::PARSING_MODE_DEFAULT;
use stylesheets::{CssRule, CssRules, Origin, UrlExtraData};
use stylesheets::loader::StylesheetLoader;
use stylesheets::memory::{MallocSizeOfFn, MallocSizeOfWithGuard};
use stylesheets::rule_parser::{State, TopLevelRuleParser};
use stylesheets::rules_iterator::{EffectiveRules, EffectiveRulesIterator, NestedRuleIterationCondition, RulesIterator};
use values::specified::NamespaceId;

/// This structure holds the user-agent and user stylesheets.
pub struct UserAgentStylesheets {
    /// The lock used for user-agent stylesheets.
    pub shared_lock: SharedRwLock,
    /// The user or user agent stylesheets.
    pub user_or_user_agent_stylesheets: Vec<Stylesheet>,
    /// The quirks mode stylesheet.
    pub quirks_mode_stylesheet: Stylesheet,
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
    /// Whether this stylesheet would be dirty when the viewport size changes.
    pub dirty_on_viewport_size_change: AtomicBool,
}

impl StylesheetContents {
    /// Parse a given CSS string, with a given url-data, origin, and
    /// quirks mode.
    pub fn from_str(
        css: &str,
        url_data: UrlExtraData,
        origin: Origin,
        shared_lock: &SharedRwLock,
        stylesheet_loader: Option<&StylesheetLoader>,
        error_reporter: &ParseErrorReporter,
        quirks_mode: QuirksMode,
        line_number_offset: u64
    ) -> Self {
        let namespaces = RwLock::new(Namespaces::default());
        let (rules, dirty_on_viewport_size_change) = Stylesheet::parse_rules(
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
            dirty_on_viewport_size_change: AtomicBool::new(dirty_on_viewport_size_change),
            quirks_mode: quirks_mode,
        }
    }

    /// Return an iterator using the condition `C`.
    #[inline]
    pub fn iter_rules<'a, 'b, C>(
        &'a self,
        device: &'a Device,
        guard: &'a SharedRwLockReadGuard<'b>
    ) -> RulesIterator<'a, 'b, C>
    where
        C: NestedRuleIterationCondition,
    {
        RulesIterator::new(
            device,
            self.quirks_mode,
            guard,
            &self.rules.read_with(guard)
        )
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
        let rules =
            self.rules.read_with(guard)
                .deep_clone_with_lock(lock, guard, params);

        let dirty_on_viewport_size_change =
            AtomicBool::new(self.dirty_on_viewport_size_change.load(Ordering::Relaxed));

        Self {
            rules: Arc::new(lock.wrap(rules)),
            dirty_on_viewport_size_change,
            quirks_mode: self.quirks_mode,
            origin: self.origin,
            url_data: RwLock::new((*self.url_data.read()).clone()),
            namespaces: RwLock::new((*self.namespaces.read()).clone()),
        }
    }
}

impl MallocSizeOfWithGuard for StylesheetContents {
    fn malloc_size_of_children(
        &self,
        guard: &SharedRwLockReadGuard,
        malloc_size_of: MallocSizeOfFn
    ) -> usize {
        // Measurement of other fields may be added later.
        self.rules.read_with(guard).malloc_size_of_children(guard, malloc_size_of)
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
    /// Get the contents of this stylesheet.
    fn contents(&self, guard: &SharedRwLockReadGuard) -> &StylesheetContents;

    /// Get the stylesheet origin.
    fn origin(&self, guard: &SharedRwLockReadGuard) -> Origin {
        self.contents(guard).origin
    }

    /// Get the stylesheet quirks mode.
    fn quirks_mode(&self, guard: &SharedRwLockReadGuard) -> QuirksMode {
        self.contents(guard).quirks_mode
    }

    /// Get the media associated with this stylesheet.
    fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList>;

    /// Returns whether the style-sheet applies for the current device.
    fn is_effective_for_device(
        &self,
        device: &Device,
        guard: &SharedRwLockReadGuard
    ) -> bool {
        match self.media(guard) {
            Some(medialist) => medialist.evaluate(device, self.quirks_mode(guard)),
            None => true,
        }
    }

    /// Get whether this stylesheet is enabled.
    fn enabled(&self) -> bool;

    /// Return an iterator using the condition `C`.
    #[inline]
    fn iter_rules<'a, 'b, C>(
        &'a self,
        device: &'a Device,
        guard: &'a SharedRwLockReadGuard<'b>
    ) -> RulesIterator<'a, 'b, C>
    where
        C: NestedRuleIterationCondition,
    {
        self.contents(guard).iter_rules(device, guard)
    }

    /// Return an iterator over the effective rules within the style-sheet, as
    /// according to the supplied `Device`.
    #[inline]
    fn effective_rules<'a, 'b>(
        &'a self,
        device: &'a Device,
        guard: &'a SharedRwLockReadGuard<'b>
    ) -> EffectiveRulesIterator<'a, 'b> {
        self.iter_rules::<EffectiveRules>(device, guard)
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
}

impl StylesheetInDocument for Stylesheet {
    fn contents(&self, _: &SharedRwLockReadGuard) -> &StylesheetContents {
        &self.contents
    }

    fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        Some(self.media.read_with(guard))
    }

    fn enabled(&self) -> bool {
        !self.disabled()
    }
}

impl Stylesheet {
    /// Updates an empty stylesheet from a given string of text.
    pub fn update_from_str(existing: &Stylesheet,
                           css: &str,
                           url_data: UrlExtraData,
                           stylesheet_loader: Option<&StylesheetLoader>,
                           error_reporter: &ParseErrorReporter,
                           line_number_offset: u64) {
        let namespaces = RwLock::new(Namespaces::default());
        let (rules, dirty_on_viewport_size_change) =
            Stylesheet::parse_rules(
                css,
                &url_data,
                existing.contents.origin,
                &mut *namespaces.write(),
                &existing.shared_lock,
                stylesheet_loader,
                error_reporter,
                existing.contents.quirks_mode,
                line_number_offset
            );

        *existing.contents.url_data.write() = url_data;
        mem::swap(
            &mut *existing.contents.namespaces.write(),
            &mut *namespaces.write()
        );
        existing.contents.dirty_on_viewport_size_change
            .store(dirty_on_viewport_size_change, Ordering::Release);

        // Acquire the lock *after* parsing, to minimize the exclusive section.
        let mut guard = existing.shared_lock.write();
        *existing.contents.rules.write_with(&mut guard) = CssRules(rules);
    }

    fn parse_rules(
        css: &str,
        url_data: &UrlExtraData,
        origin: Origin,
        namespaces: &mut Namespaces,
        shared_lock: &SharedRwLock,
        stylesheet_loader: Option<&StylesheetLoader>,
        error_reporter: &ParseErrorReporter,
        quirks_mode: QuirksMode,
        line_number_offset: u64
    ) -> (Vec<CssRule>, bool) {
        let mut rules = Vec::new();
        let mut input = ParserInput::new(css);
        let mut input = Parser::new(&mut input);

        let context =
            ParserContext::new_with_line_number_offset(
                origin,
                url_data,
                error_reporter,
                line_number_offset,
                PARSING_MODE_DEFAULT,
                quirks_mode
            );

        let rule_parser = TopLevelRuleParser {
            stylesheet_origin: origin,
            shared_lock: shared_lock,
            loader: stylesheet_loader,
            context: context,
            state: State::Start,
            namespaces: Some(namespaces),
        };

        input.look_for_viewport_percentages();

        {
            let mut iter =
                RuleListParser::new_for_stylesheet(&mut input, rule_parser);

            while let Some(result) = iter.next() {
                match result {
                    Ok(rule) => rules.push(rule),
                    Err(err) => {
                        let pos = err.span.start;
                        let error = ContextualParseError::InvalidRule(
                            iter.input.slice(err.span), err.error);
                        log_css_error(iter.input, pos, error, iter.parser.context());
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
        let contents = StylesheetContents::from_str(
            css,
            url_data,
            origin,
            &shared_lock,
            stylesheet_loader,
            error_reporter,
            quirks_mode,
            line_number_offset
        );

        Stylesheet {
            contents,
            shared_lock,
            media,
            disabled: AtomicBool::new(false),
        }
    }

    /// Whether this stylesheet can be dirty on viewport size change.
    pub fn dirty_on_viewport_size_change(&self) -> bool {
        self.contents.dirty_on_viewport_size_change.load(Ordering::SeqCst)
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
        self.contents.dirty_on_viewport_size_change.fetch_or(
            has_viewport_percentages,
            Ordering::SeqCst
        );
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
        let contents = self.contents.deep_clone_with_lock(
            &lock,
            &guard,
            &DeepCloneParams
        );

        Stylesheet {
            contents,
            media: media,
            shared_lock: lock,
            disabled: AtomicBool::new(self.disabled.load(Ordering::SeqCst)),
        }
    }
}

