/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Selector matching.

use {Atom, LocalName, Namespace};
use applicable_declarations::{ApplicableDeclarationBlock, ApplicableDeclarationList};
use bit_vec::BitVec;
use context::{CascadeInputs, QuirksMode};
use dom::TElement;
use element_state::ElementState;
use font_metrics::FontMetricsProvider;
#[cfg(feature = "gecko")]
use gecko_bindings::structs::{nsIAtom, ServoStyleSetSizes, StyleRuleInclusion};
use invalidation::element::invalidation_map::InvalidationMap;
use invalidation::media_queries::{EffectiveMediaQueryResults, ToMediaListKey};
use media_queries::Device;
use properties::{self, CascadeFlags, ComputedValues};
use properties::{AnimationRules, PropertyDeclarationBlock};
#[cfg(feature = "servo")]
use properties::INHERIT_ALL;
use properties::IS_LINK;
use rule_tree::{CascadeLevel, RuleTree, StyleSource};
use selector_map::{PrecomputedHashMap, SelectorMap, SelectorMapEntry};
use selector_parser::{SelectorImpl, PerPseudoElementMap, PseudoElement};
use selectors::attr::NamespaceConstraint;
use selectors::bloom::{BloomFilter, NonCountingBloomFilter};
use selectors::matching::{ElementSelectorFlags, matches_selector, MatchingContext, MatchingMode};
use selectors::matching::VisitedHandlingMode;
use selectors::parser::{AncestorHashes, Combinator, Component, Selector};
use selectors::parser::{SelectorIter, SelectorMethods};
use selectors::sink::Push;
use selectors::visitor::SelectorVisitor;
use servo_arc::{Arc, ArcBorrow};
use shared_lock::{Locked, SharedRwLockReadGuard, StylesheetGuards};
use smallvec::VecLike;
use std::fmt::Debug;
use std::ops;
use style_traits::viewport::ViewportConstraints;
use stylesheet_set::{OriginValidity, SheetRebuildKind, StylesheetSet, StylesheetIterator, StylesheetFlusher};
#[cfg(feature = "gecko")]
use stylesheets::{CounterStyleRule, FontFaceRule, FontFeatureValuesRule};
use stylesheets::{CssRule, Origin, OriginSet, PerOrigin, PerOriginIter};
#[cfg(feature = "gecko")]
use stylesheets::{MallocEnclosingSizeOfFn, MallocSizeOf, MallocSizeOfBox, MallocSizeOfFn};
#[cfg(feature = "gecko")]
use stylesheets::{MallocSizeOfHash, MallocSizeOfVec};
use stylesheets::StyleRule;
use stylesheets::StylesheetInDocument;
use stylesheets::UserAgentStylesheets;
use stylesheets::keyframes_rule::KeyframesAnimation;
use stylesheets::viewport_rule::{self, MaybeNew, ViewportRule};
use thread_state;

pub use ::fnv::FnvHashMap;

/// The type of the stylesheets that the stylist contains.
#[cfg(feature = "servo")]
pub type StylistSheet = ::stylesheets::DocumentStyleSheet;

/// The type of the stylesheets that the stylist contains.
#[cfg(feature = "gecko")]
pub type StylistSheet = ::gecko::data::GeckoStyleSheet;

/// All the computed information for a stylesheet.
#[derive(Default)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct DocumentCascadeData {
    /// Common data for all the origins.
    per_origin: PerOrigin<CascadeData>,

    /// Applicable declarations for a given non-eagerly cascaded pseudo-element.
    ///
    /// These are eagerly computed once, and then used to resolve the new
    /// computed values on the fly on layout.
    ///
    /// These are only filled from UA stylesheets.
    ///
    /// FIXME(emilio): Use the rule tree!
    precomputed_pseudo_element_decls: PerPseudoElementMap<Vec<ApplicableDeclarationBlock>>,
}

impl DocumentCascadeData {
    fn iter_origins(&self) -> PerOriginIter<CascadeData> {
        self.per_origin.iter_origins()
    }

    /// Rebuild the cascade data for the given document stylesheets, and
    /// optionally with a set of user agent stylesheets.
    fn rebuild<'a, 'b, S>(
        &mut self,
        device: &Device,
        quirks_mode: QuirksMode,
        flusher: StylesheetFlusher<'a, 'b, S>,
        guards: &StylesheetGuards,
        ua_stylesheets: Option<&UserAgentStylesheets>,
        extra_data: &mut PerOrigin<ExtraStyleData>,
    )
    where
        'b: 'a,
        S: StylesheetInDocument + ToMediaListKey + PartialEq + 'static,
    {
        debug_assert!(!flusher.nothing_to_do());

        for (cascade_data, origin) in self.per_origin.iter_mut_origins() {
            let validity = flusher.origin_validity(origin);

            if validity == OriginValidity::Valid {
                continue;
            }

            if origin == Origin::UserAgent {
                self.precomputed_pseudo_element_decls.clear();
            }

            extra_data.borrow_mut_for_origin(&origin).clear();
            if validity == OriginValidity::CascadeInvalid {
                cascade_data.clear_cascade_data()
            } else {
                debug_assert_eq!(validity, OriginValidity::FullyInvalid);
                cascade_data.clear();
            }
        }

        if let Some(ua_stylesheets) = ua_stylesheets {
            debug_assert!(cfg!(feature = "servo"));

            for stylesheet in &ua_stylesheets.user_or_user_agent_stylesheets {
                let sheet_origin =
                    stylesheet.contents(guards.ua_or_user).origin;

                debug_assert!(matches!(
                    sheet_origin,
                    Origin::UserAgent | Origin::User
                ));

                let validity = flusher.origin_validity(sheet_origin);

                // Servo doesn't support to incrementally mutate UA sheets.
                debug_assert!(matches!(
                    validity,
                    OriginValidity::Valid | OriginValidity::FullyInvalid
                ));

                if validity == OriginValidity::Valid {
                    continue;
                }

                self.add_stylesheet(
                    device,
                    quirks_mode,
                    stylesheet,
                    guards.ua_or_user,
                    extra_data,
                    SheetRebuildKind::Full,
                );
            }

            if quirks_mode != QuirksMode::NoQuirks {
                let stylesheet = &ua_stylesheets.quirks_mode_stylesheet;
                let sheet_origin =
                    stylesheet.contents(guards.ua_or_user).origin;

                debug_assert!(matches!(
                    sheet_origin,
                    Origin::UserAgent | Origin::User
                ));

                let validity = flusher.origin_validity(sheet_origin);

                // Servo doesn't support to incrementally mutate UA sheets.
                debug_assert!(matches!(
                    validity,
                    OriginValidity::Valid | OriginValidity::FullyInvalid
                ));

                if validity != OriginValidity::Valid {
                    self.add_stylesheet(
                        device,
                        quirks_mode,
                        &ua_stylesheets.quirks_mode_stylesheet,
                        guards.ua_or_user,
                        extra_data,
                        SheetRebuildKind::Full,
                    );
                }
            }
        }

        for (stylesheet, rebuild_kind) in flusher {
            self.add_stylesheet(
                device,
                quirks_mode,
                stylesheet,
                guards.author,
                extra_data,
                rebuild_kind,
            );
        }
    }

    fn add_stylesheet<S>(
        &mut self,
        device: &Device,
        quirks_mode: QuirksMode,
        stylesheet: &S,
        guard: &SharedRwLockReadGuard,
        _extra_data: &mut PerOrigin<ExtraStyleData>,
        rebuild_kind: SheetRebuildKind,
    )
    where
        S: StylesheetInDocument + ToMediaListKey + 'static,
    {
        if !stylesheet.enabled() ||
           !stylesheet.is_effective_for_device(device, guard) {
            return;
        }

        let origin = stylesheet.origin(guard);
        let origin_cascade_data =
            self.per_origin.borrow_mut_for_origin(&origin);

        if rebuild_kind.should_rebuild_invalidation() {
            origin_cascade_data
                .effective_media_query_results
                .saw_effective(stylesheet);
        }

        for rule in stylesheet.effective_rules(device, guard) {
            match *rule {
                CssRule::Style(ref locked) => {
                    let style_rule = locked.read_with(&guard);
                    origin_cascade_data.num_declarations +=
                        style_rule.block.read_with(&guard).len();
                    for selector in &style_rule.selectors.0 {
                        origin_cascade_data.num_selectors += 1;

                        let map = match selector.pseudo_element() {
                            Some(pseudo) if pseudo.is_precomputed() => {
                                if !selector.is_universal() ||
                                   !matches!(origin, Origin::UserAgent) {
                                    // ::-moz-tree selectors may appear in
                                    // non-UA sheets (even though they never
                                    // match).
                                    continue;
                                }

                                self.precomputed_pseudo_element_decls
                                    .get_or_insert_with(&pseudo.canonical(), Vec::new)
                                    .expect("Unexpected tree pseudo-element?")
                                    .push(ApplicableDeclarationBlock::new(
                                        StyleSource::Style(locked.clone()),
                                        origin_cascade_data.rules_source_order,
                                        CascadeLevel::UANormal,
                                        selector.specificity()
                                    ));

                                continue;
                            }
                            None => &mut origin_cascade_data.element_map,
                            Some(pseudo) => {
                                origin_cascade_data
                                    .pseudos_map
                                    .get_or_insert_with(&pseudo.canonical(), || Box::new(SelectorMap::new()))
                                    .expect("Unexpected tree pseudo-element?")
                            }
                        };

                        let hashes =
                            AncestorHashes::new(&selector, quirks_mode);

                        let rule = Rule::new(
                            selector.clone(),
                            hashes.clone(),
                            locked.clone(),
                            origin_cascade_data.rules_source_order
                        );

                        map.insert(rule, quirks_mode);

                        if rebuild_kind.should_rebuild_invalidation() {
                            origin_cascade_data
                                .invalidation_map
                                .note_selector(selector, quirks_mode);
                            let mut visitor = StylistSelectorVisitor {
                                needs_revalidation: false,
                                passed_rightmost_selector: false,
                                attribute_dependencies: &mut origin_cascade_data.attribute_dependencies,
                                style_attribute_dependency: &mut origin_cascade_data.style_attribute_dependency,
                                state_dependencies: &mut origin_cascade_data.state_dependencies,
                                mapped_ids: &mut origin_cascade_data.mapped_ids,
                            };

                            selector.visit(&mut visitor);

                            if visitor.needs_revalidation {
                                origin_cascade_data.selectors_for_cache_revalidation.insert(
                                    RevalidationSelectorAndHashes::new(selector.clone(), hashes),
                                    quirks_mode
                                );
                            }
                        }
                    }
                    origin_cascade_data.rules_source_order += 1;
                }
                CssRule::Import(ref lock) => {
                    if rebuild_kind.should_rebuild_invalidation() {
                        let import_rule = lock.read_with(guard);
                        origin_cascade_data
                            .effective_media_query_results
                            .saw_effective(import_rule);
                    }

                    // NOTE: effective_rules visits the inner stylesheet if
                    // appropriate.
                }
                CssRule::Media(ref lock) => {
                    if rebuild_kind.should_rebuild_invalidation() {
                        let media_rule = lock.read_with(guard);
                        origin_cascade_data
                            .effective_media_query_results
                            .saw_effective(media_rule);
                    }
                }
                CssRule::Keyframes(ref keyframes_rule) => {
                    let keyframes_rule = keyframes_rule.read_with(guard);
                    debug!("Found valid keyframes rule: {:?}", *keyframes_rule);

                    // Don't let a prefixed keyframes animation override a non-prefixed one.
                    let needs_insertion =
                        keyframes_rule.vendor_prefix.is_none() ||
                        origin_cascade_data.animations.get(keyframes_rule.name.as_atom())
                            .map_or(true, |rule| rule.vendor_prefix.is_some());
                    if needs_insertion {
                        let animation = KeyframesAnimation::from_keyframes(
                            &keyframes_rule.keyframes, keyframes_rule.vendor_prefix.clone(), guard);
                        debug!("Found valid keyframe animation: {:?}", animation);
                        origin_cascade_data.animations.insert(keyframes_rule.name.as_atom().clone(), animation);
                    }
                }
                #[cfg(feature = "gecko")]
                CssRule::FontFace(ref rule) => {
                    _extra_data
                        .borrow_mut_for_origin(&origin)
                        .add_font_face(rule);
                }
                #[cfg(feature = "gecko")]
                CssRule::FontFeatureValues(ref rule) => {
                    _extra_data
                        .borrow_mut_for_origin(&origin)
                        .add_font_feature_values(rule);
                }
                #[cfg(feature = "gecko")]
                CssRule::CounterStyle(ref rule) => {
                    _extra_data
                        .borrow_mut_for_origin(&origin)
                        .add_counter_style(guard, rule);
                }
                // We don't care about any other rule.
                _ => {}
            }
        }
    }

    /// Measures heap usage.
    #[cfg(feature = "gecko")]
    pub fn malloc_add_size_of_children(&self, malloc_size_of: MallocSizeOfFn,
                                       malloc_enclosing_size_of: MallocEnclosingSizeOfFn,
                                       sizes: &mut ServoStyleSetSizes) {
        self.per_origin.user_agent.malloc_add_size_of_children(malloc_size_of,
                                                               malloc_enclosing_size_of, sizes);
        self.per_origin.user.malloc_add_size_of_children(malloc_size_of,
                                                         malloc_enclosing_size_of, sizes);
        self.per_origin.author.malloc_add_size_of_children(malloc_size_of,
                                                           malloc_enclosing_size_of, sizes);

        for elem in self.precomputed_pseudo_element_decls.iter() {
            if let Some(ref elem) = *elem {
                sizes.mStylistPrecomputedPseudos += elem.malloc_shallow_size_of_vec(malloc_size_of);
            }
        }
    }
}

/// A wrapper over a StylesheetSet that can be `Sync`, since it's only used and
/// exposed via mutable methods in the `Stylist`.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct StylistStylesheetSet(StylesheetSet<StylistSheet>);
// Read above to see why this is fine.
unsafe impl Sync for StylistStylesheetSet {}

impl StylistStylesheetSet {
    fn new() -> Self {
        StylistStylesheetSet(StylesheetSet::new())
    }
}

impl ops::Deref for StylistStylesheetSet {
    type Target = StylesheetSet<StylistSheet>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for StylistStylesheetSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// This structure holds all the selectors and device characteristics
/// for a given document. The selectors are converted into `Rule`s
/// and sorted into `SelectorMap`s keyed off stylesheet origin and
/// pseudo-element (see `CascadeData`).
///
/// This structure is effectively created once per pipeline, in the
/// LayoutThread corresponding to that pipeline.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Stylist {
    /// Device that the stylist is currently evaluating against.
    ///
    /// This field deserves a bigger comment due to the different use that Gecko
    /// and Servo give to it (that we should eventually unify).
    ///
    /// With Gecko, the device is never changed. Gecko manually tracks whether
    /// the device data should be reconstructed, and "resets" the state of the
    /// device.
    ///
    /// On Servo, on the other hand, the device is a really cheap representation
    /// that is recreated each time some constraint changes and calling
    /// `set_device`.
    device: Device,

    /// Viewport constraints based on the current device.
    viewport_constraints: Option<ViewportConstraints>,

    /// The list of stylesheets.
    stylesheets: StylistStylesheetSet,

    /// If true, the quirks-mode stylesheet is applied.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "defined in selectors")]
    quirks_mode: QuirksMode,

    /// Selector maps for all of the style sheets in the stylist, after
    /// evalutaing media rules against the current device, split out per
    /// cascade level.
    cascade_data: DocumentCascadeData,

    /// The rule tree, that stores the results of selector matching.
    rule_tree: RuleTree,

    /// The total number of times the stylist has been rebuilt.
    num_rebuilds: usize,
}

/// What cascade levels to include when styling elements.
#[derive(Clone, Copy, PartialEq)]
pub enum RuleInclusion {
    /// Include rules for style sheets at all cascade levels.  This is the
    /// normal rule inclusion mode.
    All,
    /// Only include rules from UA and user level sheets.  Used to implement
    /// `getDefaultComputedStyle`.
    DefaultOnly,
}

#[cfg(feature = "gecko")]
impl From<StyleRuleInclusion> for RuleInclusion {
    fn from(value: StyleRuleInclusion) -> Self {
        match value {
            StyleRuleInclusion::All => RuleInclusion::All,
            StyleRuleInclusion::DefaultOnly => RuleInclusion::DefaultOnly,
        }
    }
}

impl Stylist {
    /// Construct a new `Stylist`, using given `Device` and `QuirksMode`.
    /// If more members are added here, think about whether they should
    /// be reset in clear().
    #[inline]
    pub fn new(device: Device, quirks_mode: QuirksMode) -> Self {
        Self {
            viewport_constraints: None,
            device,
            quirks_mode,
            stylesheets: StylistStylesheetSet::new(),
            cascade_data: Default::default(),
            rule_tree: RuleTree::new(),
            num_rebuilds: 0,
        }
    }

    /// Returns the number of selectors.
    pub fn num_selectors(&self) -> usize {
        self.cascade_data.iter_origins().map(|(d, _)| d.num_selectors).sum()
    }

    /// Returns the number of declarations.
    pub fn num_declarations(&self) -> usize {
        self.cascade_data.iter_origins().map(|(d, _)| d.num_declarations).sum()
    }

    /// Returns the number of times the stylist has been rebuilt.
    pub fn num_rebuilds(&self) -> usize {
        self.num_rebuilds
    }

    /// Returns the number of revalidation_selectors.
    pub fn num_revalidation_selectors(&self) -> usize {
        self.cascade_data.iter_origins()
            .map(|(d, _)| d.selectors_for_cache_revalidation.len()).sum()
    }

    /// Returns the number of entries in invalidation maps.
    pub fn num_invalidations(&self) -> usize {
        self.cascade_data.iter_origins()
            .map(|(d, _)| d.invalidation_map.len()).sum()
    }

    /// Invokes `f` with the `InvalidationMap` for each origin.
    ///
    /// NOTE(heycam) This might be better as an `iter_invalidation_maps`, once
    /// we have `impl trait` and can return that easily without bothering to
    /// create a whole new iterator type.
    pub fn each_invalidation_map<F>(&self, mut f: F)
        where F: FnMut(&InvalidationMap)
    {
        for (data, _) in self.cascade_data.iter_origins() {
            f(&data.invalidation_map)
        }
    }

    /// Flush the list of stylesheets if they changed, ensuring the stylist is
    /// up-to-date.
    ///
    /// FIXME(emilio): Move the `ua_sheets` to the Stylist too?
    pub fn flush<E>(
        &mut self,
        guards: &StylesheetGuards,
        ua_sheets: Option<&UserAgentStylesheets>,
        extra_data: &mut PerOrigin<ExtraStyleData>,
        document_element: Option<E>,
    ) -> bool
    where
        E: TElement,
    {
        if !self.stylesheets.has_changed() {
            return false;
        }

        self.num_rebuilds += 1;

        // Update viewport_constraints regardless of which origins'
        // `CascadeData` we're updating.
        self.viewport_constraints = None;
        if viewport_rule::enabled() {
            // TODO(emilio): This doesn't look so efficient.
            //
            // Presumably when we properly implement this we can at least have a
            // bit on the stylesheet that says whether it contains viewport
            // rules to skip it entirely?
            //
            // Processing it with the rest of rules seems tricky since it
            // overrides the viewport size which may change the evaluation of
            // media queries (or may not? how are viewport units in media
            // queries defined?)
            let cascaded_rule = ViewportRule {
                declarations: viewport_rule::Cascade::from_stylesheets(
                    self.stylesheets.iter(),
                    guards.author,
                    &self.device,
                ).finish()
            };

            self.viewport_constraints =
                ViewportConstraints::maybe_new(
                    &self.device,
                    &cascaded_rule,
                    self.quirks_mode,
                );

            if let Some(ref constraints) = self.viewport_constraints {
                self.device.account_for_viewport_rule(constraints);
            }
        }

        let flusher = self.stylesheets.flush(document_element, &guards.author);

        let had_invalidations = flusher.had_invalidations();

        self.cascade_data.rebuild(
            &self.device,
            self.quirks_mode,
            flusher,
            guards,
            ua_sheets,
            extra_data,
        );

        had_invalidations
    }

    /// Insert a given stylesheet before another stylesheet in the document.
    pub fn insert_stylesheet_before(
        &mut self,
        sheet: StylistSheet,
        before_sheet: StylistSheet,
        guard: &SharedRwLockReadGuard,
    ) {
        self.stylesheets.insert_stylesheet_before(
            Some(&self.device),
            sheet,
            before_sheet,
            guard,
        )
    }

    /// Marks a given stylesheet origin as dirty, due to, for example, changes
    /// in the declarations that affect a given rule.
    ///
    /// FIXME(emilio): Eventually it'd be nice for this to become more
    /// fine-grained.
    pub fn force_stylesheet_origins_dirty(&mut self, origins: OriginSet) {
        self.stylesheets.force_dirty(origins)
    }

    /// Iterate over the given set of stylesheets.
    ///
    /// This is very intentionally exposed only on `&mut self`, since we don't
    /// want to give access to the stylesheet list from worker threads.
    pub fn iter_stylesheets(&mut self) -> StylesheetIterator<StylistSheet> {
        self.stylesheets.iter()
    }

    /// Sets whether author style is enabled or not.
    pub fn set_author_style_disabled(&mut self, disabled: bool) {
        self.stylesheets.set_author_style_disabled(disabled);
    }

    /// Returns whether we've recorded any stylesheet change so far.
    pub fn stylesheets_have_changed(&self) -> bool {
        self.stylesheets.has_changed()
    }

    /// Appends a new stylesheet to the current set.
    pub fn append_stylesheet(&mut self, sheet: StylistSheet, guard: &SharedRwLockReadGuard) {
        self.stylesheets.append_stylesheet(Some(&self.device), sheet, guard)
    }

    /// Appends a new stylesheet to the current set.
    pub fn prepend_stylesheet(&mut self, sheet: StylistSheet, guard: &SharedRwLockReadGuard) {
        self.stylesheets.prepend_stylesheet(Some(&self.device), sheet, guard)
    }

    /// Remove a given stylesheet to the current set.
    pub fn remove_stylesheet(&mut self, sheet: StylistSheet, guard: &SharedRwLockReadGuard) {
        self.stylesheets.remove_stylesheet(Some(&self.device), sheet, guard)
    }

    /// Returns whether the given attribute might appear in an attribute
    /// selector of some rule in the stylist.
    pub fn might_have_attribute_dependency(
        &self,
        local_name: &LocalName,
    ) -> bool {
        if *local_name == local_name!("style") {
            self.cascade_data
                .iter_origins()
                .any(|(d, _)| d.style_attribute_dependency)
        } else {
            self.cascade_data
                .iter_origins()
                .any(|(d, _)| {
                    d.attribute_dependencies
                        .might_contain_hash(local_name.get_hash())
                })
        }
    }

    /// Returns whether the given ElementState bit might be relied upon by a
    /// selector of some rule in the stylist.
    pub fn might_have_state_dependency(&self, state: ElementState) -> bool {
        self.has_state_dependency(state)
    }

    /// Returns whether the given ElementState bit is relied upon by a selector
    /// of some rule in the stylist.
    pub fn has_state_dependency(&self, state: ElementState) -> bool {
        self.cascade_data
            .iter_origins()
            .any(|(d, _)| d.state_dependencies.intersects(state))
    }

    /// Computes the style for a given "precomputed" pseudo-element, taking the
    /// universal rules and applying them.
    ///
    /// If `inherit_all` is true, then all properties are inherited from the
    /// parent; otherwise, non-inherited properties are reset to their initial
    /// values. The flow constructor uses this flag when constructing anonymous
    /// flows.
    pub fn precomputed_values_for_pseudo(
        &self,
        guards: &StylesheetGuards,
        pseudo: &PseudoElement,
        parent: Option<&ComputedValues>,
        cascade_flags: CascadeFlags,
        font_metrics: &FontMetricsProvider
    ) -> Arc<ComputedValues> {
        debug_assert!(pseudo.is_precomputed());

        let rule_node =
            match self.cascade_data.precomputed_pseudo_element_decls.get(pseudo) {
                Some(declarations) => {
                    self.rule_tree.insert_ordered_rules_with_important(
                        declarations.into_iter().map(|a| (a.source.clone(), a.level())),
                        guards
                    )
                }
                None => self.rule_tree.root().clone(),
            };

        // NOTE(emilio): We skip calculating the proper layout parent style
        // here.
        //
        // It'd be fine to assert that this isn't called with a parent style
        // where display contents is in effect, but in practice this is hard to
        // do for stuff like :-moz-fieldset-content with a
        // <fieldset style="display: contents">. That is, the computed value of
        // display for the fieldset is "contents", even though it's not the used
        // value, so we don't need to adjust in a different way anyway.
        //
        // In practice, I don't think any anonymous content can be a direct
        // descendant of a display: contents element where display: contents is
        // the actual used value, and the computed value of it would need
        // blockification.
        properties::cascade(
            &self.device,
            Some(pseudo),
            &rule_node,
            guards,
            parent,
            parent,
            parent,
            None,
            font_metrics,
            cascade_flags,
            self.quirks_mode,
        )
    }

    /// Returns the style for an anonymous box of the given type.
    #[cfg(feature = "servo")]
    pub fn style_for_anonymous(
        &self,
        guards: &StylesheetGuards,
        pseudo: &PseudoElement,
        parent_style: &ComputedValues
    ) -> Arc<ComputedValues> {
        use font_metrics::ServoMetricsProvider;

        // For most (but not all) pseudo-elements, we inherit all values from the parent.
        let inherit_all = match *pseudo {
            PseudoElement::ServoText |
            PseudoElement::ServoInputText => false,
            PseudoElement::ServoAnonymousBlock |
            PseudoElement::ServoAnonymousTable |
            PseudoElement::ServoAnonymousTableCell |
            PseudoElement::ServoAnonymousTableRow |
            PseudoElement::ServoAnonymousTableWrapper |
            PseudoElement::ServoTableWrapper |
            PseudoElement::ServoInlineBlockWrapper |
            PseudoElement::ServoInlineAbsolute => true,
            PseudoElement::Before |
            PseudoElement::After |
            PseudoElement::Selection |
            PseudoElement::DetailsSummary |
            PseudoElement::DetailsContent => {
                unreachable!("That pseudo doesn't represent an anonymous box!")
            }
        };
        let mut cascade_flags = CascadeFlags::empty();
        if inherit_all {
            cascade_flags.insert(INHERIT_ALL);
        }
        self.precomputed_values_for_pseudo(
            guards,
            &pseudo,
            Some(parent_style),
            cascade_flags,
            &ServoMetricsProvider
        )
    }

    /// Computes a pseudo-element style lazily during layout.
    ///
    /// This can only be done for a certain set of pseudo-elements, like
    /// :selection.
    ///
    /// Check the documentation on lazy pseudo-elements in
    /// docs/components/style.md
    pub fn lazily_compute_pseudo_element_style<E>(
        &self,
        guards: &StylesheetGuards,
        element: &E,
        pseudo: &PseudoElement,
        rule_inclusion: RuleInclusion,
        parent_style: &ComputedValues,
        is_probe: bool,
        font_metrics: &FontMetricsProvider
    ) -> Option<Arc<ComputedValues>>
    where
        E: TElement,
    {
        let cascade_inputs =
            self.lazy_pseudo_rules(guards, element, pseudo, is_probe, rule_inclusion);
        self.compute_pseudo_element_style_with_inputs(
            &cascade_inputs,
            pseudo,
            guards,
            parent_style,
            font_metrics,
        )
    }

    /// Computes a pseudo-element style lazily using the given CascadeInputs.
    /// This can be used for truly lazy pseudo-elements or to avoid redoing
    /// selector matching for eager pseudo-elements when we need to recompute
    /// their style with a new parent style.
    pub fn compute_pseudo_element_style_with_inputs(
        &self,
        inputs: &CascadeInputs,
        pseudo: &PseudoElement,
        guards: &StylesheetGuards,
        parent_style: &ComputedValues,
        font_metrics: &FontMetricsProvider
    ) -> Option<Arc<ComputedValues>> {
        // We may have only visited rules in cases when we are actually
        // resolving, not probing, pseudo-element style.
        if inputs.rules.is_none() && inputs.visited_rules.is_none() {
            return None
        }

        // FIXME(emilio): The lack of layout_parent_style here could be
        // worrying, but we're probably dropping the display fixup for
        // pseudos other than before and after, so it's probably ok.
        //
        // (Though the flags don't indicate so!)
        Some(self.compute_style_with_inputs(
            inputs,
            Some(pseudo),
            guards,
            parent_style,
            parent_style,
            parent_style,
            font_metrics,
            CascadeFlags::empty(),
        ))
    }

    /// Computes a style using the given CascadeInputs.  This can be used to
    /// compute a style any time we know what rules apply and just need to use
    /// the given parent styles.
    ///
    /// parent_style is the style to inherit from for properties affected by
    /// first-line ancestors.
    ///
    /// parent_style_ignoring_first_line is the style to inherit from for
    /// properties not affected by first-line ancestors.
    ///
    /// layout_parent_style is the style used for some property fixups.  It's
    /// the style of the nearest ancestor with a layout box.
    ///
    /// is_link should be true if we're computing style for a link; that affects
    /// how :visited handling is done.
    pub fn compute_style_with_inputs(
        &self,
        inputs: &CascadeInputs,
        pseudo: Option<&PseudoElement>,
        guards: &StylesheetGuards,
        parent_style: &ComputedValues,
        parent_style_ignoring_first_line: &ComputedValues,
        layout_parent_style: &ComputedValues,
        font_metrics: &FontMetricsProvider,
        cascade_flags: CascadeFlags
    ) -> Arc<ComputedValues> {
        // We need to compute visited values if we have visited rules or if our
        // parent has visited values.
        let visited_values = if inputs.visited_rules.is_some() || parent_style.get_visited_style().is_some() {
            // Slightly annoying: we know that inputs has either rules or
            // visited rules, but we can't do inputs.rules() up front because
            // maybe it just has visited rules, so can't unwrap_or.
            let rule_node = match inputs.visited_rules.as_ref() {
                Some(rules) => rules,
                None => inputs.rules.as_ref().unwrap(),
            };
            let inherited_style;
            let inherited_style_ignoring_first_line;
            let layout_parent_style_for_visited;
            if cascade_flags.contains(IS_LINK) {
                // We just want to use our parent style as our parent.
                inherited_style = parent_style;
                inherited_style_ignoring_first_line = parent_style_ignoring_first_line;
                layout_parent_style_for_visited = layout_parent_style;
            } else {
                // We want to use the visited bits (if any) from our parent
                // style as our parent.
                inherited_style =
                    parent_style.get_visited_style().unwrap_or(parent_style);
                inherited_style_ignoring_first_line =
                    parent_style_ignoring_first_line.get_visited_style().unwrap_or(parent_style_ignoring_first_line);
                layout_parent_style_for_visited =
                    layout_parent_style.get_visited_style().unwrap_or(layout_parent_style);
            }

            Some(properties::cascade(
                &self.device,
                pseudo,
                rule_node,
                guards,
                Some(inherited_style),
                Some(inherited_style_ignoring_first_line),
                Some(layout_parent_style_for_visited),
                None,
                font_metrics,
                cascade_flags,
                self.quirks_mode,
            ))
        } else {
            None
        };

        // We may not have non-visited rules, if we only had visited ones.  In
        // that case we want to use the root rulenode for our non-visited rules.
        let rules = inputs.rules.as_ref().unwrap_or(self.rule_tree.root());

        // Read the comment on `precomputed_values_for_pseudo` to see why it's
        // difficult to assert that display: contents nodes never arrive here
        // (tl;dr: It doesn't apply for replaced elements and such, but the
        // computed value is still "contents").
        properties::cascade(
            &self.device,
            pseudo,
            rules,
            guards,
            Some(parent_style),
            Some(parent_style_ignoring_first_line),
            Some(layout_parent_style),
            visited_values,
            font_metrics,
            cascade_flags,
            self.quirks_mode,
        )
    }

    fn has_rules_for_pseudo(&self, pseudo: &PseudoElement) -> bool {
        self.cascade_data
            .iter_origins()
            .any(|(d, _)| d.has_rules_for_pseudo(pseudo))
    }

    /// Computes the cascade inputs for a lazily-cascaded pseudo-element.
    ///
    /// See the documentation on lazy pseudo-elements in
    /// docs/components/style.md
    pub fn lazy_pseudo_rules<E>(
        &self,
        guards: &StylesheetGuards,
        element: &E,
        pseudo: &PseudoElement,
        is_probe: bool,
        rule_inclusion: RuleInclusion
    ) -> CascadeInputs
    where
        E: TElement
    {
        let pseudo = pseudo.canonical();
        debug_assert!(pseudo.is_lazy());

        if !self.has_rules_for_pseudo(&pseudo) {
            return CascadeInputs::default()
        }

        // Apply the selector flags. We should be in sequential mode
        // already, so we can directly apply the parent flags.
        let mut set_selector_flags = |element: &E, flags: ElementSelectorFlags| {
            if cfg!(feature = "servo") {
                // Servo calls this function from the worker, but only for internal
                // pseudos, so we should never generate selector flags here.
                unreachable!("internal pseudo generated slow selector flags?");
            }

            // No need to bother setting the selector flags when we're computing
            // default styles.
            if rule_inclusion == RuleInclusion::DefaultOnly {
                return;
            }

            // Gecko calls this from sequential mode, so we can directly apply
            // the flags.
            debug_assert!(thread_state::get() == thread_state::LAYOUT);
            let self_flags = flags.for_self();
            if !self_flags.is_empty() {
                unsafe { element.set_selector_flags(self_flags); }
            }
            let parent_flags = flags.for_parent();
            if !parent_flags.is_empty() {
                if let Some(p) = element.parent_element() {
                    unsafe { p.set_selector_flags(parent_flags); }
                }
            }
        };

        let mut inputs = CascadeInputs::default();
        let mut declarations = ApplicableDeclarationList::new();
        let mut matching_context =
            MatchingContext::new(MatchingMode::ForStatelessPseudoElement,
                                 None,
                                 self.quirks_mode);

        self.push_applicable_declarations(
            element,
            Some(&pseudo),
            None,
            None,
            AnimationRules(None, None),
            rule_inclusion,
            &mut declarations,
            &mut matching_context,
            &mut set_selector_flags
        );

        if !declarations.is_empty() {
            let rule_node =
                self.rule_tree.compute_rule_node(&mut declarations, guards);
            debug_assert!(rule_node != *self.rule_tree.root());
            inputs.rules = Some(rule_node);
        }

        if is_probe && inputs.rules.is_none() {
            // When probing, don't compute visited styles if we have no
            // unvisited styles.
            return inputs;
        }

        if matching_context.relevant_link_found {
            let mut declarations = ApplicableDeclarationList::new();
            let mut matching_context =
                MatchingContext::new_for_visited(
                    MatchingMode::ForStatelessPseudoElement,
                    None,
                    VisitedHandlingMode::RelevantLinkVisited,
                    self.quirks_mode,
                );

            self.push_applicable_declarations(
                element,
                Some(&pseudo),
                None,
                None,
                AnimationRules(None, None),
                rule_inclusion,
                &mut declarations,
                &mut matching_context,
                &mut set_selector_flags
            );
            if !declarations.is_empty() {
                let rule_node =
                    self.rule_tree.insert_ordered_rules_with_important(
                        declarations.drain().map(|a| a.order_and_level()),
                        guards);
                if rule_node != *self.rule_tree.root() {
                    inputs.visited_rules = Some(rule_node);
                }
            }
        }

        inputs
    }

    /// Set a given device, which may change the styles that apply to the
    /// document.
    ///
    /// Returns the sheet origins that were actually affected.
    ///
    /// This means that we may need to rebuild style data even if the
    /// stylesheets haven't changed.
    ///
    /// Also, the device that arrives here may need to take the viewport rules
    /// into account.
    ///
    /// For Gecko, this is called when XBL bindings are used by different
    /// documents.
    pub fn set_device(
        &mut self,
        mut device: Device,
        guard: &SharedRwLockReadGuard,
    ) -> OriginSet {
        if viewport_rule::enabled() {
            let cascaded_rule = {
                let stylesheets = self.stylesheets.iter();

                ViewportRule {
                    declarations: viewport_rule::Cascade::from_stylesheets(
                        stylesheets.clone(),
                        guard,
                        &device
                    ).finish(),
                }
            };

            self.viewport_constraints =
                ViewportConstraints::maybe_new(&device, &cascaded_rule, self.quirks_mode);

            if let Some(ref constraints) = self.viewport_constraints {
                device.account_for_viewport_rule(constraints);
            }
        }

        self.device = device;
        self.media_features_change_changed_style(guard)
    }

    /// Returns whether, given a media feature change, any previously-applicable
    /// style has become non-applicable, or vice-versa for each origin.
    pub fn media_features_change_changed_style(
        &self,
        guard: &SharedRwLockReadGuard,
    ) -> OriginSet {
        use invalidation::media_queries::PotentiallyEffectiveMediaRules;

        debug!("Stylist::media_features_change_changed_style");

        let mut origins = OriginSet::empty();
        let stylesheets = self.stylesheets.iter();

        'stylesheets_loop: for stylesheet in stylesheets {
            let effective_now =
                stylesheet.is_effective_for_device(&self.device, guard);

            let origin = stylesheet.origin(guard);

            if origins.contains(origin.into()) {
                continue;
            }

            let origin_cascade_data =
                self.cascade_data.per_origin.borrow_for_origin(&origin);

            let effective_then =
                origin_cascade_data
                    .effective_media_query_results
                    .was_effective(stylesheet);

            if effective_now != effective_then {
                debug!(" > Stylesheet changed -> {}, {}",
                       effective_then, effective_now);
                origins |= origin;
                continue;
            }

            if !effective_now {
                continue;
            }

            let mut iter =
                stylesheet.iter_rules::<PotentiallyEffectiveMediaRules>(
                    &self.device,
                    guard
                );

            while let Some(rule) = iter.next() {
                match *rule {
                    CssRule::Style(..) |
                    CssRule::Namespace(..) |
                    CssRule::FontFace(..) |
                    CssRule::CounterStyle(..) |
                    CssRule::Supports(..) |
                    CssRule::Keyframes(..) |
                    CssRule::Page(..) |
                    CssRule::Viewport(..) |
                    CssRule::Document(..) |
                    CssRule::FontFeatureValues(..) => {
                        // Not affected by device changes.
                        continue;
                    }
                    CssRule::Import(ref lock) => {
                        let import_rule = lock.read_with(guard);
                        let effective_now =
                            import_rule.stylesheet
                                .is_effective_for_device(&self.device, guard);
                        let effective_then =
                            origin_cascade_data
                                .effective_media_query_results
                                .was_effective(import_rule);
                        if effective_now != effective_then {
                            debug!(" > @import rule changed {} -> {}",
                                   effective_then, effective_now);
                            origins |= origin;
                            continue 'stylesheets_loop;
                        }

                        if !effective_now {
                            iter.skip_children();
                        }
                    }
                    CssRule::Media(ref lock) => {
                        let media_rule = lock.read_with(guard);
                        let mq = media_rule.media_queries.read_with(guard);
                        let effective_now =
                            mq.evaluate(&self.device, self.quirks_mode);
                        let effective_then =
                            origin_cascade_data
                                .effective_media_query_results
                                .was_effective(media_rule);
                        if effective_now != effective_then {
                            debug!(" > @media rule changed {} -> {}",
                                   effective_then, effective_now);
                            origins |= origin;
                            continue 'stylesheets_loop;
                        }

                        if !effective_now {
                            iter.skip_children();
                        }
                    }
                }
            }
        }

        return origins
    }

    /// Returns the viewport constraints that apply to this document because of
    /// a @viewport rule.
    pub fn viewport_constraints(&self) -> Option<&ViewportConstraints> {
        self.viewport_constraints.as_ref()
    }

    /// Returns the Quirks Mode of the document.
    pub fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode
    }

    /// Sets the quirks mode of the document.
    pub fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        // FIXME(emilio): We don't seem to change the quirks mode dynamically
        // during multiple layout passes, but this is totally bogus, in the
        // sense that it's updated asynchronously.
        //
        // This should probably be an argument to `update`, and use the quirks
        // mode info in the `SharedLayoutContext`.
        self.quirks_mode = quirks_mode;
    }

    /// Returns the applicable CSS declarations for the given element by
    /// treating us as an XBL stylesheet-only stylist.
    pub fn push_applicable_declarations_as_xbl_only_stylist<E, V>(
        &self,
        element: &E,
        pseudo_element: Option<&PseudoElement>,
        applicable_declarations: &mut V
    )
    where
        E: TElement,
        V: Push<ApplicableDeclarationBlock> + VecLike<ApplicableDeclarationBlock>,
    {
        let mut matching_context =
            MatchingContext::new(MatchingMode::Normal, None, self.quirks_mode);
        let mut dummy_flag_setter = |_: &E, _: ElementSelectorFlags| {};

        let rule_hash_target = element.rule_hash_target();

        // nsXBLPrototypeResources::LoadResources() loads Chrome XBL style
        // sheets under eAuthorSheetFeatures level.
        if let Some(map) = self.cascade_data.per_origin.author.borrow_for_pseudo(pseudo_element) {
            map.get_all_matching_rules(
                element,
                &rule_hash_target,
                applicable_declarations,
                &mut matching_context,
                self.quirks_mode,
                &mut dummy_flag_setter,
                CascadeLevel::XBL,
            );
        }
    }

    /// Returns the applicable CSS declarations for the given element.
    ///
    /// This corresponds to `ElementRuleCollector` in WebKit.
    pub fn push_applicable_declarations<E, V, F>(
        &self,
        element: &E,
        pseudo_element: Option<&PseudoElement>,
        style_attribute: Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>,
        smil_override: Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>,
        animation_rules: AnimationRules,
        rule_inclusion: RuleInclusion,
        applicable_declarations: &mut V,
        context: &mut MatchingContext,
        flags_setter: &mut F,
    )
    where
        E: TElement,
        V: Push<ApplicableDeclarationBlock> + VecLike<ApplicableDeclarationBlock> + Debug,
        F: FnMut(&E, ElementSelectorFlags),
    {
        // Gecko definitely has pseudo-elements with style attributes, like
        // ::-moz-color-swatch.
        debug_assert!(cfg!(feature = "gecko") ||
                      style_attribute.is_none() || pseudo_element.is_none(),
                      "Style attributes do not apply to pseudo-elements");
        debug_assert!(pseudo_element.map_or(true, |p| !p.is_precomputed()));

        let rule_hash_target = element.rule_hash_target();

        debug!("Determining if style is shareable: pseudo: {}",
               pseudo_element.is_some());

        let only_default_rules = rule_inclusion == RuleInclusion::DefaultOnly;

        // Step 1: Normal user-agent rules.
        if let Some(map) = self.cascade_data.per_origin.user_agent.borrow_for_pseudo(pseudo_element) {
            map.get_all_matching_rules(
                element,
                &rule_hash_target,
                applicable_declarations,
                context,
                self.quirks_mode,
                flags_setter,
                CascadeLevel::UANormal
            );
        }

        if pseudo_element.is_none() && !only_default_rules {
            // Step 2: Presentational hints.
            let length_before_preshints = applicable_declarations.len();
            element.synthesize_presentational_hints_for_legacy_attributes(
                context.visited_handling,
                applicable_declarations
            );
            if applicable_declarations.len() != length_before_preshints {
                if cfg!(debug_assertions) {
                    for declaration in &applicable_declarations[length_before_preshints..] {
                        assert_eq!(declaration.level(), CascadeLevel::PresHints);
                    }
                }
            }
        }

        // NB: the following condition, although it may look somewhat
        // inaccurate, would be equivalent to something like:
        //
        //     element.matches_user_and_author_rules() ||
        //     (is_implemented_pseudo &&
        //      rule_hash_target.matches_user_and_author_rules())
        //
        // Which may be more what you would probably expect.
        if rule_hash_target.matches_user_and_author_rules() {
            // Step 3a: User normal rules.
            if let Some(map) = self.cascade_data.per_origin.user.borrow_for_pseudo(pseudo_element) {
                map.get_all_matching_rules(
                    element,
                    &rule_hash_target,
                    applicable_declarations,
                    context,
                    self.quirks_mode,
                    flags_setter,
                    CascadeLevel::UserNormal,
                );
            }
        } else {
            debug!("skipping user rules");
        }

        // Step 3b: XBL rules.
        let cut_off_inheritance =
            element.get_declarations_from_xbl_bindings(
                pseudo_element,
                applicable_declarations,
            );

        if rule_hash_target.matches_user_and_author_rules() && !only_default_rules {
            // Gecko skips author normal rules if cutting off inheritance.
            // See nsStyleSet::FileRules().
            if !cut_off_inheritance {
                // Step 3c: Author normal rules.
                if let Some(map) = self.cascade_data.per_origin.author.borrow_for_pseudo(pseudo_element) {
                    map.get_all_matching_rules(
                        element,
                        &rule_hash_target,
                        applicable_declarations,
                        context,
                        self.quirks_mode,
                        flags_setter,
                        CascadeLevel::AuthorNormal
                    );
                }
            } else {
                debug!("skipping author normal rules due to cut off inheritance");
            }
        } else {
            debug!("skipping author normal rules");
        }

        if !only_default_rules {
            // Step 4: Normal style attributes.
            if let Some(sa) = style_attribute {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(
                        sa.clone_arc(),
                        CascadeLevel::StyleAttributeNormal
                    )
                );
            }

            // Step 5: SMIL override.
            // Declarations from SVG SMIL animation elements.
            if let Some(so) = smil_override {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(
                        so.clone_arc(),
                        CascadeLevel::SMILOverride
                    )
                );
            }

            // Step 6: Animations.
            // The animations sheet (CSS animations, script-generated animations,
            // and CSS transitions that are no longer tied to CSS markup)
            if let Some(anim) = animation_rules.0 {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(
                        anim.clone(),
                        CascadeLevel::Animations
                    )
                );
            }
        } else {
            debug!("skipping style attr and SMIL & animation rules");
        }

        //
        // Steps 7-10 correspond to !important rules, and are handled during
        // rule tree insertion.
        //

        if !only_default_rules {
            // Step 11: Transitions.
            // The transitions sheet (CSS transitions that are tied to CSS markup)
            if let Some(anim) = animation_rules.1 {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(
                        anim.clone(),
                        CascadeLevel::Transitions
                    )
                );
            }
        } else {
            debug!("skipping transition rules");
        }
    }

    /// Given an id, returns whether there might be any rules for that id in any
    /// of our rule maps.
    #[inline]
    pub fn may_have_rules_for_id(&self, id: &Atom) -> bool {
        self.cascade_data
            .iter_origins()
            .any(|(d, _)| d.mapped_ids.might_contain_hash(id.get_hash()))
    }

    /// Returns the registered `@keyframes` animation for the specified name.
    #[inline]
    pub fn get_animation(&self, name: &Atom) -> Option<&KeyframesAnimation> {
        self.cascade_data
            .iter_origins()
            .filter_map(|(d, _)| d.animations.get(name))
            .next()
    }

    /// Computes the match results of a given element against the set of
    /// revalidation selectors.
    pub fn match_revalidation_selectors<E, F>(
        &self,
        element: &E,
        bloom: Option<&BloomFilter>,
        flags_setter: &mut F
    ) -> BitVec
    where
        E: TElement,
        F: FnMut(&E, ElementSelectorFlags),
    {
        // NB: `MatchingMode` doesn't really matter, given we don't share style
        // between pseudos.
        let mut matching_context =
            MatchingContext::new(MatchingMode::Normal, bloom, self.quirks_mode);

        // Note that, by the time we're revalidating, we're guaranteed that the
        // candidate and the entry have the same id, classes, and local name.
        // This means we're guaranteed to get the same rulehash buckets for all
        // the lookups, which means that the bitvecs are comparable. We verify
        // this in the caller by asserting that the bitvecs are same-length.
        let mut results = BitVec::new();
        for (data, _) in self.cascade_data.iter_origins() {
            data.selectors_for_cache_revalidation.lookup(
                *element,
                self.quirks_mode,
                &mut |selector_and_hashes| {
                    results.push(matches_selector(
                        &selector_and_hashes.selector,
                        selector_and_hashes.selector_offset,
                        Some(&selector_and_hashes.hashes),
                        element,
                        &mut matching_context,
                        flags_setter
                    ));
                    true
                }
            );
        }

        results
    }

    /// Computes styles for a given declaration with parent_style.
    pub fn compute_for_declarations(
        &self,
        guards: &StylesheetGuards,
        parent_style: &ComputedValues,
        declarations: Arc<Locked<PropertyDeclarationBlock>>,
    ) -> Arc<ComputedValues> {
        use font_metrics::get_metrics_provider_for_product;

        let v = vec![ApplicableDeclarationBlock::from_declarations(
            declarations.clone(),
            CascadeLevel::StyleAttributeNormal
        )];

        let rule_node =
            self.rule_tree.insert_ordered_rules(v.into_iter().map(|a| a.order_and_level()));

        // This currently ignores visited styles.  It appears to be used for
        // font styles in <canvas> via Servo_StyleSet_ResolveForDeclarations.
        // It is unclear if visited styles are meaningful for this case.
        let metrics = get_metrics_provider_for_product();

        // FIXME(emilio): the pseudo bit looks quite dubious!
        properties::cascade(
            &self.device,
            /* pseudo = */ None,
            &rule_node,
            guards,
            Some(parent_style),
            Some(parent_style),
            Some(parent_style),
            None,
            &metrics,
            CascadeFlags::empty(),
            self.quirks_mode,
        )
    }

    /// Accessor for a shared reference to the device.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Accessor for a mutable reference to the device.
    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    /// Accessor for a shared reference to the rule tree.
    pub fn rule_tree(&self) -> &RuleTree {
        &self.rule_tree
    }

    /// Measures heap usage.
    #[cfg(feature = "gecko")]
    pub fn malloc_add_size_of_children(&self, malloc_size_of: MallocSizeOfFn,
                                       malloc_enclosing_size_of: MallocEnclosingSizeOfFn,
                                       sizes: &mut ServoStyleSetSizes) {
        self.cascade_data.malloc_add_size_of_children(malloc_size_of, malloc_enclosing_size_of,
                                                      sizes);
        sizes.mStylistRuleTree += self.rule_tree.malloc_size_of_children(malloc_size_of);

        // We may measure other fields in the future if DMD says it's worth it.
    }
}

/// This struct holds data which users of Stylist may want to extract
/// from stylesheets which can be done at the same time as updating.
#[derive(Default)]
pub struct ExtraStyleData {
    /// A list of effective font-face rules and their origin.
    #[cfg(feature = "gecko")]
    pub font_faces: Vec<Arc<Locked<FontFaceRule>>>,

    /// A list of effective font-feature-values rules.
    #[cfg(feature = "gecko")]
    pub font_feature_values: Vec<Arc<Locked<FontFeatureValuesRule>>>,

    /// A map of effective counter-style rules.
    #[cfg(feature = "gecko")]
    pub counter_styles: PrecomputedHashMap<Atom, Arc<Locked<CounterStyleRule>>>,
}

#[cfg(feature = "gecko")]
impl ExtraStyleData {
    /// Add the given @font-face rule.
    fn add_font_face(&mut self, rule: &Arc<Locked<FontFaceRule>>) {
        self.font_faces.push(rule.clone());
    }

    /// Add the given @font-feature-values rule.
    fn add_font_feature_values(&mut self, rule: &Arc<Locked<FontFeatureValuesRule>>) {
        self.font_feature_values.push(rule.clone());
    }

    /// Add the given @counter-style rule.
    fn add_counter_style(
        &mut self,
        guard: &SharedRwLockReadGuard,
        rule: &Arc<Locked<CounterStyleRule>>,
    ) {
        let name = rule.read_with(guard).mName.raw::<nsIAtom>().into();
        self.counter_styles.insert(name, rule.clone());
    }
}

impl ExtraStyleData {
    fn clear(&mut self) {
        #[cfg(feature = "gecko")]
        {
            self.font_faces.clear();
            self.font_feature_values.clear();
            self.counter_styles.clear();
        }
    }

    /// Measure heap usage.
    #[cfg(feature = "gecko")]
    pub fn malloc_size_of_children(&self, malloc_size_of: MallocSizeOfFn,
                                   malloc_enclosing_size_of: MallocEnclosingSizeOfFn) -> usize {
        let mut n = 0;
        n += self.font_faces.malloc_shallow_size_of_vec(malloc_size_of);
        n += self.font_feature_values.malloc_shallow_size_of_vec(malloc_size_of);
        n += self.counter_styles.malloc_shallow_size_of_hash(malloc_enclosing_size_of);
        n
    }
}

/// SelectorMapEntry implementation for use in our revalidation selector map.
#[derive(Clone, Debug)]
struct RevalidationSelectorAndHashes {
    selector: Selector<SelectorImpl>,
    selector_offset: usize,
    hashes: AncestorHashes,
}

impl RevalidationSelectorAndHashes {
    fn new(selector: Selector<SelectorImpl>, hashes: AncestorHashes) -> Self {
        let selector_offset = {
            // We basically want to check whether the first combinator is a
            // pseudo-element combinator.  If it is, we want to use the offset
            // one past it.  Otherwise, our offset is 0.
            let mut index = 0;
            let mut iter = selector.iter();

            // First skip over the first ComplexSelector.
            //
            // We can't check what sort of what combinator we have until we do
            // that.
            for _ in &mut iter {
                index += 1; // Simple selector
            }

            match iter.next_sequence() {
                Some(Combinator::PseudoElement) => index + 1, // +1 for the combinator
                _ => 0
            }
        };

        RevalidationSelectorAndHashes { selector, selector_offset, hashes, }
    }
}

impl SelectorMapEntry for RevalidationSelectorAndHashes {
    fn selector(&self) -> SelectorIter<SelectorImpl> {
        self.selector.iter_from(self.selector_offset)
    }
}

/// A selector visitor implementation that collects all the state the Stylist
/// cares about a selector.
struct StylistSelectorVisitor<'a> {
    /// Whether the selector needs revalidation for the style sharing cache.
    needs_revalidation: bool,
    /// Whether we've past the rightmost compound selector, not counting
    /// pseudo-elements.
    passed_rightmost_selector: bool,
    /// The filter with all the id's getting referenced from rightmost
    /// selectors.
    mapped_ids: &'a mut NonCountingBloomFilter,
    /// The filter with the local names of attributes there are selectors for.
    attribute_dependencies: &'a mut NonCountingBloomFilter,
    /// Whether there's any attribute selector for the [style] attribute.
    style_attribute_dependency: &'a mut bool,
    /// All the states selectors in the page reference.
    state_dependencies: &'a mut ElementState,
}

fn component_needs_revalidation(
    c: &Component<SelectorImpl>,
    passed_rightmost_selector: bool,
) -> bool {
    match *c {
        Component::ID(_) => {
            // TODO(emilio): This could also check that the ID is not already in
            // the rule hash. In that case, we could avoid making this a
            // revalidation selector too.
            //
            // See https://bugzilla.mozilla.org/show_bug.cgi?id=1369611
            passed_rightmost_selector
        }
        Component::AttributeInNoNamespaceExists { .. } |
        Component::AttributeInNoNamespace { .. } |
        Component::AttributeOther(_) |
        Component::Empty |
        Component::FirstChild |
        Component::LastChild |
        Component::OnlyChild |
        Component::NthChild(..) |
        Component::NthLastChild(..) |
        Component::NthOfType(..) |
        Component::NthLastOfType(..) |
        Component::FirstOfType |
        Component::LastOfType |
        Component::OnlyOfType => {
            true
        },
        Component::NonTSPseudoClass(ref p) => {
            p.needs_cache_revalidation()
        },
        _ => {
            false
        }
    }
}

impl<'a> SelectorVisitor for StylistSelectorVisitor<'a> {
    type Impl = SelectorImpl;

    fn visit_complex_selector(
        &mut self,
        combinator: Option<Combinator>
    ) -> bool {
        self.needs_revalidation =
            self.needs_revalidation || combinator.map_or(false, |c| c.is_sibling());

        // NOTE(emilio): This works properly right now because we can't store
        // complex selectors in nested selectors, otherwise we may need to
        // rethink this.
        //
        // Also, note that this call happens before we visit any of the simple
        // selectors in the next ComplexSelector, so we can use this to skip
        // looking at them.
        self.passed_rightmost_selector =
            self.passed_rightmost_selector ||
            !matches!(combinator, None | Some(Combinator::PseudoElement));

        true
    }

    fn visit_attribute_selector(
        &mut self,
        _ns: &NamespaceConstraint<&Namespace>,
        name: &LocalName,
        lower_name: &LocalName
    ) -> bool {
        if *lower_name == local_name!("style") {
            *self.style_attribute_dependency = true;
        } else {
            self.attribute_dependencies.insert_hash(name.get_hash());
            self.attribute_dependencies.insert_hash(lower_name.get_hash());
        }
        true
    }

    fn visit_simple_selector(&mut self, s: &Component<SelectorImpl>) -> bool {
        self.needs_revalidation =
            self.needs_revalidation ||
            component_needs_revalidation(s, self.passed_rightmost_selector);

        match *s {
            Component::NonTSPseudoClass(ref p) => {
                self.state_dependencies.insert(p.state_flag());
            }
            Component::ID(ref id) if !self.passed_rightmost_selector => {
                // We want to stop storing mapped ids as soon as we've moved off
                // the rightmost ComplexSelector that is not a pseudo-element.
                //
                // That can be detected by a visit_complex_selector call with a
                // combinator other than None and PseudoElement.
                //
                // Importantly, this call happens before we visit any of the
                // simple selectors in that ComplexSelector.
                //
                // NOTE(emilio): See the comment regarding on when this may
                // break in visit_complex_selector.
                self.mapped_ids.insert_hash(id.get_hash());
            }
            _ => {},
        }

        true
    }
}

/// Data resulting from performing the CSS cascade that is specific to a given
/// origin.
///
/// FIXME(emilio): Consider renaming and splitting in `CascadeData` and
/// `InvalidationData`? That'd make `clear_cascade_data()` clearer.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Debug)]
struct CascadeData {
    /// Rules from stylesheets at this `CascadeData`'s origin.
    element_map: SelectorMap<Rule>,

    /// Rules from stylesheets at this `CascadeData`'s origin that correspond
    /// to a given pseudo-element.
    ///
    /// FIXME(emilio): There are a bunch of wasted entries here in practice.
    /// Figure out a good way to do a `PerNonAnonBox` and `PerAnonBox` (for
    /// `precomputed_values_for_pseudo`) without duplicating a lot of code.
    pseudos_map: PerPseudoElementMap<Box<SelectorMap<Rule>>>,

    /// A map with all the animations at this `CascadeData`'s origin, indexed
    /// by name.
    animations: PrecomputedHashMap<Atom, KeyframesAnimation>,

    /// The invalidation map for the rules at this origin.
    invalidation_map: InvalidationMap,

    /// The attribute local names that appear in attribute selectors.  Used
    /// to avoid taking element snapshots when an irrelevant attribute changes.
    /// (We don't bother storing the namespace, since namespaced attributes
    /// are rare.)
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "just an array")]
    attribute_dependencies: NonCountingBloomFilter,

    /// Whether `"style"` appears in an attribute selector.  This is not common,
    /// and by tracking this explicitly, we can avoid taking an element snapshot
    /// in the common case of style=""` changing due to modifying
    /// `element.style`.  (We could track this in `attribute_dependencies`, like
    /// all other attributes, but we should probably not risk incorrectly
    /// returning `true` for `"style"` just due to a hash collision.)
    style_attribute_dependency: bool,

    /// The element state bits that are relied on by selectors.  Like
    /// `attribute_dependencies`, this is used to avoid taking element snapshots
    /// when an irrelevant element state bit changes.
    state_dependencies: ElementState,

    /// The ids that appear in the rightmost complex selector of selectors (and
    /// hence in our selector maps).  Used to determine when sharing styles is
    /// safe: we disallow style sharing for elements whose id matches this
    /// filter, and hence might be in one of our selector maps.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "just an array")]
    mapped_ids: NonCountingBloomFilter,

    /// Selectors that require explicit cache revalidation (i.e. which depend
    /// on state that is not otherwise visible to the cache, like attributes or
    /// tree-structural state like child index and pseudos).
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    selectors_for_cache_revalidation: SelectorMap<RevalidationSelectorAndHashes>,

    /// Effective media query results cached from the last rebuild.
    effective_media_query_results: EffectiveMediaQueryResults,

    /// A monotonically increasing counter to represent the order on which a
    /// style rule appears in a stylesheet, needed to sort them by source order.
    rules_source_order: u32,

    /// The total number of selectors.
    num_selectors: usize,

    /// The total number of declarations.
    num_declarations: usize,
}

impl CascadeData {
    fn new() -> Self {
        Self {
            element_map: SelectorMap::new(),
            pseudos_map: PerPseudoElementMap::default(),
            animations: Default::default(),
            invalidation_map: InvalidationMap::new(),
            attribute_dependencies: NonCountingBloomFilter::new(),
            style_attribute_dependency: false,
            state_dependencies: ElementState::empty(),
            mapped_ids: NonCountingBloomFilter::new(),
            selectors_for_cache_revalidation: SelectorMap::new(),
            effective_media_query_results: EffectiveMediaQueryResults::new(),
            rules_source_order: 0,
            num_selectors: 0,
            num_declarations: 0,
        }
    }

    #[inline]
    fn borrow_for_pseudo(&self, pseudo: Option<&PseudoElement>) -> Option<&SelectorMap<Rule>> {
        match pseudo {
            Some(pseudo) => self.pseudos_map.get(&pseudo.canonical()).map(|p| &**p),
            None => Some(&self.element_map),
        }
    }

    fn has_rules_for_pseudo(&self, pseudo: &PseudoElement) -> bool {
        self.pseudos_map.get(pseudo).is_some()
    }

    /// Clears the cascade data, but not the invalidation data.
    fn clear_cascade_data(&mut self) {
        self.element_map.clear();
        self.pseudos_map.clear();
        self.animations.clear();
        self.rules_source_order = 0;
        self.num_selectors = 0;
        self.num_declarations = 0;
    }

    fn clear(&mut self) {
        self.clear_cascade_data();
        self.effective_media_query_results.clear();
        self.invalidation_map.clear();
        self.attribute_dependencies.clear();
        self.style_attribute_dependency = false;
        self.state_dependencies = ElementState::empty();
        self.mapped_ids.clear();
        self.selectors_for_cache_revalidation.clear();
    }

    /// Measures heap usage.
    #[cfg(feature = "gecko")]
    pub fn malloc_add_size_of_children(&self, malloc_size_of: MallocSizeOfFn,
                                       malloc_enclosing_size_of: MallocEnclosingSizeOfFn,
                                       sizes: &mut ServoStyleSetSizes) {
        sizes.mStylistElementAndPseudosMaps +=
            self.element_map.malloc_size_of_children(malloc_enclosing_size_of);

        for elem in self.pseudos_map.iter() {
            if let Some(ref elem) = *elem {
                sizes.mStylistElementAndPseudosMaps +=
                    elem.malloc_shallow_size_of_box(malloc_size_of) +
                    elem.malloc_size_of_children(malloc_enclosing_size_of)
            }
        }

        sizes.mStylistOther +=
            self.animations.malloc_shallow_size_of_hash(malloc_enclosing_size_of);
        for val in self.animations.values() {
            sizes.mStylistOther += val.malloc_size_of_children(malloc_size_of);
        }

        sizes.mStylistInvalidationMap +=
            self.invalidation_map.malloc_size_of_children(malloc_enclosing_size_of);

        sizes.mStylistRevalidationSelectors +=
            self.selectors_for_cache_revalidation.malloc_size_of_children(malloc_enclosing_size_of);

        sizes.mStylistOther +=
            self.effective_media_query_results.malloc_size_of_children(malloc_enclosing_size_of);
    }
}

impl Default for CascadeData {
    fn default() -> Self {
        CascadeData::new()
    }
}

/// A rule, that wraps a style rule, but represents a single selector of the
/// rule.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug)]
pub struct Rule {
    /// The selector this struct represents. We store this and the
    /// any_{important,normal} booleans inline in the Rule to avoid
    /// pointer-chasing when gathering applicable declarations, which
    /// can ruin performance when there are a lot of rules.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub selector: Selector<SelectorImpl>,
    /// The ancestor hashes associated with the selector.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "No heap data")]
    pub hashes: AncestorHashes,
    /// The source order this style rule appears in. Note that we only use
    /// three bytes to store this value in ApplicableDeclarationsBlock, so
    /// we could repurpose that storage here if we needed to.
    pub source_order: u32,
    /// The actual style rule.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub style_rule: Arc<Locked<StyleRule>>,
}

impl SelectorMapEntry for Rule {
    fn selector(&self) -> SelectorIter<SelectorImpl> {
        self.selector.iter()
    }
}

impl Rule {
    /// Returns the specificity of the rule.
    pub fn specificity(&self) -> u32 {
        self.selector.specificity()
    }

    /// Turns this rule into an `ApplicableDeclarationBlock` for the given
    /// cascade level.
    pub fn to_applicable_declaration_block(
        &self,
        level: CascadeLevel
    ) -> ApplicableDeclarationBlock {
        let source = StyleSource::Style(self.style_rule.clone());
        ApplicableDeclarationBlock::new(
            source,
            self.source_order,
            level,
            self.specificity()
        )
    }

    /// Creates a new Rule.
    pub fn new(selector: Selector<SelectorImpl>,
               hashes: AncestorHashes,
               style_rule: Arc<Locked<StyleRule>>,
               source_order: u32)
               -> Self
    {
        Rule {
            selector: selector,
            hashes: hashes,
            style_rule: style_rule,
            source_order: source_order,
        }
    }
}

/// A function to be able to test the revalidation stuff.
pub fn needs_revalidation_for_testing(s: &Selector<SelectorImpl>) -> bool {
    let mut attribute_dependencies = NonCountingBloomFilter::new();
    let mut mapped_ids = NonCountingBloomFilter::new();
    let mut style_attribute_dependency = false;
    let mut state_dependencies = ElementState::empty();
    let mut visitor = StylistSelectorVisitor {
        needs_revalidation: false,
        passed_rightmost_selector: false,
        attribute_dependencies: &mut attribute_dependencies,
        style_attribute_dependency: &mut style_attribute_dependency,
        state_dependencies: &mut state_dependencies,
        mapped_ids: &mut mapped_ids,
    };
    s.visit(&mut visitor);
    visitor.needs_revalidation
}
