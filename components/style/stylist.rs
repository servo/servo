/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Selector matching.

use crate::applicable_declarations::{
    ApplicableDeclarationBlock, ApplicableDeclarationList, CascadePriority,
};
use crate::context::{CascadeInputs, QuirksMode};
use crate::dom::{TElement, TShadowRoot};
#[cfg(feature = "gecko")]
use crate::gecko_bindings::structs::{ServoStyleSetSizes, StyleRuleInclusion};
use crate::invalidation::element::invalidation_map::InvalidationMap;
use crate::invalidation::media_queries::{
    EffectiveMediaQueryResults, MediaListKey, ToMediaListKey,
};
use crate::invalidation::stylesheets::RuleChangeKind;
use crate::media_queries::Device;
use crate::properties::{self, CascadeMode, ComputedValues};
use crate::properties::{AnimationDeclarations, PropertyDeclarationBlock};
use crate::rule_cache::{RuleCache, RuleCacheConditions};
use crate::rule_collector::{containing_shadow_ignoring_svg_use, RuleCollector};
use crate::rule_tree::{CascadeLevel, RuleTree, StrongRuleNode, StyleSource};
use crate::selector_map::{PrecomputedHashMap, PrecomputedHashSet, SelectorMap, SelectorMapEntry};
use crate::selector_parser::{PerPseudoElementMap, PseudoElement, SelectorImpl, SnapshotMap};
use crate::shared_lock::{Locked, SharedRwLockReadGuard, StylesheetGuards};
use crate::stylesheet_set::{DataValidity, DocumentStylesheetSet, SheetRebuildKind};
use crate::stylesheet_set::{DocumentStylesheetFlusher, SheetCollectionFlusher};
use crate::stylesheets::container_rule::ContainerCondition;
use crate::stylesheets::import_rule::ImportLayer;
use crate::stylesheets::keyframes_rule::KeyframesAnimation;
use crate::stylesheets::layer_rule::{LayerName, LayerOrder};
#[cfg(feature = "gecko")]
use crate::stylesheets::{
    CounterStyleRule, FontFaceRule, FontFeatureValuesRule, FontPaletteValuesRule, PageRule,
};
use crate::stylesheets::{
    CssRule, EffectiveRulesIterator, Origin, OriginSet, PageRule, PerOrigin, PerOriginIter,
};
use crate::stylesheets::{StyleRule, StylesheetContents, StylesheetInDocument};
use crate::AllocErr;
use crate::{Atom, LocalName, Namespace, ShrinkIfNeeded, WeakAtom};
use fxhash::FxHashMap;
use malloc_size_of::{MallocSizeOf, MallocShallowSizeOf, MallocSizeOfOps};
#[cfg(feature = "gecko")]
use malloc_size_of::MallocUnconditionalShallowSizeOf;
use selectors::attr::{CaseSensitivity, NamespaceConstraint};
use selectors::bloom::BloomFilter;
use selectors::matching::VisitedHandlingMode;
use selectors::matching::{matches_selector, MatchingContext, MatchingMode, NeedsSelectorFlags};
use selectors::parser::{
    AncestorHashes, Combinator, Component, Selector, SelectorIter, SelectorList,
};
use selectors::visitor::{SelectorListKind, SelectorVisitor};
use selectors::NthIndexCache;
use servo_arc::{Arc, ArcBorrow};
use smallbitvec::SmallBitVec;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::{mem, ops};
use style_traits::dom::{DocumentState, ElementState};

/// The type of the stylesheets that the stylist contains.
#[cfg(feature = "servo")]
pub type StylistSheet = crate::stylesheets::DocumentStyleSheet;

/// The type of the stylesheets that the stylist contains.
#[cfg(feature = "gecko")]
pub type StylistSheet = crate::gecko::data::GeckoStyleSheet;

#[derive(Debug, Clone)]
struct StylesheetContentsPtr(Arc<StylesheetContents>);

impl PartialEq for StylesheetContentsPtr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for StylesheetContentsPtr {}

impl Hash for StylesheetContentsPtr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let contents: &StylesheetContents = &*self.0;
        (contents as *const StylesheetContents).hash(state)
    }
}

type StyleSheetContentList = Vec<StylesheetContentsPtr>;

/// A key in the cascade data cache.
#[derive(Debug, Hash, Default, PartialEq, Eq)]
struct CascadeDataCacheKey {
    media_query_results: Vec<MediaListKey>,
    contents: StyleSheetContentList,
}

unsafe impl Send for CascadeDataCacheKey {}
unsafe impl Sync for CascadeDataCacheKey {}

trait CascadeDataCacheEntry: Sized {
    /// Returns a reference to the cascade data.
    fn cascade_data(&self) -> &CascadeData;
    /// Rebuilds the cascade data for the new stylesheet collection. The
    /// collection is guaranteed to be dirty.
    fn rebuild<S>(
        device: &Device,
        quirks_mode: QuirksMode,
        collection: SheetCollectionFlusher<S>,
        guard: &SharedRwLockReadGuard,
        old_entry: &Self,
    ) -> Result<Arc<Self>, AllocErr>
    where
        S: StylesheetInDocument + PartialEq + 'static;
    /// Measures heap memory usage.
    #[cfg(feature = "gecko")]
    fn add_size_of(&self, ops: &mut MallocSizeOfOps, sizes: &mut ServoStyleSetSizes);
}

struct CascadeDataCache<Entry> {
    entries: FxHashMap<CascadeDataCacheKey, Arc<Entry>>,
}

impl<Entry> CascadeDataCache<Entry>
where
    Entry: CascadeDataCacheEntry,
{
    fn new() -> Self {
        Self {
            entries: Default::default(),
        }
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    // FIXME(emilio): This may need to be keyed on quirks-mode too, though for
    // UA sheets there aren't class / id selectors on those sheets, usually, so
    // it's probably ok... For the other cache the quirks mode shouldn't differ
    // so also should be fine.
    fn lookup<'a, S>(
        &'a mut self,
        device: &Device,
        quirks_mode: QuirksMode,
        collection: SheetCollectionFlusher<S>,
        guard: &SharedRwLockReadGuard,
        old_entry: &Entry,
    ) -> Result<Option<Arc<Entry>>, AllocErr>
    where
        S: StylesheetInDocument + PartialEq + 'static,
    {
        use std::collections::hash_map::Entry as HashMapEntry;
        debug!("StyleSheetCache::lookup({})", self.len());

        if !collection.dirty() {
            return Ok(None);
        }

        let mut key = CascadeDataCacheKey::default();
        for sheet in collection.sheets() {
            CascadeData::collect_applicable_media_query_results_into(
                device,
                sheet,
                guard,
                &mut key.media_query_results,
                &mut key.contents,
            )
        }

        let new_entry;
        match self.entries.entry(key) {
            HashMapEntry::Vacant(e) => {
                debug!("> Picking the slow path (not in the cache)");
                new_entry = Entry::rebuild(device, quirks_mode, collection, guard, old_entry)?;
                e.insert(new_entry.clone());
            },
            HashMapEntry::Occupied(mut e) => {
                // Avoid reusing our old entry (this can happen if we get
                // invalidated due to CSSOM mutations and our old stylesheet
                // contents were already unique, for example).
                if !std::ptr::eq(&**e.get(), old_entry) {
                    if log_enabled!(log::Level::Debug) {
                        debug!("cache hit for:");
                        for sheet in collection.sheets() {
                            debug!(" > {:?}", sheet);
                        }
                    }
                    // The line below ensures the "committed" bit is updated
                    // properly.
                    collection.each(|_, _| true);
                    return Ok(Some(e.get().clone()));
                }

                debug!("> Picking the slow path due to same entry as old");
                new_entry = Entry::rebuild(device, quirks_mode, collection, guard, old_entry)?;
                e.insert(new_entry.clone());
            },
        }

        Ok(Some(new_entry))
    }

    /// Returns all the cascade datas that are not being used (that is, that are
    /// held alive just by this cache).
    ///
    /// We return them instead of dropping in place because some of them may
    /// keep alive some other documents (like the SVG documents kept alive by
    /// URL references), and thus we don't want to drop them while locking the
    /// cache to not deadlock.
    fn take_unused(&mut self) -> SmallVec<[Arc<Entry>; 3]> {
        let mut unused = SmallVec::new();
        self.entries.retain(|_key, value| {
            // is_unique() returns false for static references, but we never
            // have static references to UserAgentCascadeDatas.  If we did, it
            // may not make sense to put them in the cache in the first place.
            if !value.is_unique() {
                return true;
            }
            unused.push(value.clone());
            false
        });
        unused
    }

    fn take_all(&mut self) -> FxHashMap<CascadeDataCacheKey, Arc<Entry>> {
        mem::take(&mut self.entries)
    }

    #[cfg(feature = "gecko")]
    fn add_size_of(&self, ops: &mut MallocSizeOfOps, sizes: &mut ServoStyleSetSizes) {
        sizes.mOther += self.entries.shallow_size_of(ops);
        for (_key, arc) in self.entries.iter() {
            // These are primary Arc references that can be measured
            // unconditionally.
            sizes.mOther += arc.unconditional_shallow_size_of(ops);
            arc.add_size_of(ops, sizes);
        }
    }
}

/// Measure heap usage of UA_CASCADE_DATA_CACHE.
#[cfg(feature = "gecko")]
pub fn add_size_of_ua_cache(ops: &mut MallocSizeOfOps, sizes: &mut ServoStyleSetSizes) {
    UA_CASCADE_DATA_CACHE
        .lock()
        .unwrap()
        .add_size_of(ops, sizes);
}

lazy_static! {
    /// A cache of computed user-agent data, to be shared across documents.
    static ref UA_CASCADE_DATA_CACHE: Mutex<UserAgentCascadeDataCache> =
        Mutex::new(UserAgentCascadeDataCache::new());
}

impl CascadeDataCacheEntry for UserAgentCascadeData {
    fn cascade_data(&self) -> &CascadeData {
        &self.cascade_data
    }

    fn rebuild<S>(
        device: &Device,
        quirks_mode: QuirksMode,
        collection: SheetCollectionFlusher<S>,
        guard: &SharedRwLockReadGuard,
        _old: &Self,
    ) -> Result<Arc<Self>, AllocErr>
    where
        S: StylesheetInDocument + PartialEq + 'static,
    {
        // TODO: Maybe we should support incremental rebuilds, though they seem
        // uncommon and rebuild() doesn't deal with
        // precomputed_pseudo_element_decls for now so...
        let mut new_data = Self {
            cascade_data: CascadeData::new(),
            precomputed_pseudo_element_decls: PrecomputedPseudoElementDeclarations::default(),
        };

        for sheet in collection.sheets() {
            new_data.cascade_data.add_stylesheet(
                device,
                quirks_mode,
                sheet,
                guard,
                SheetRebuildKind::Full,
                Some(&mut new_data.precomputed_pseudo_element_decls),
            )?;
        }

        new_data.cascade_data.did_finish_rebuild();

        Ok(Arc::new(new_data))
    }

    #[cfg(feature = "gecko")]
    fn add_size_of(&self, ops: &mut MallocSizeOfOps, sizes: &mut ServoStyleSetSizes) {
        self.cascade_data.add_size_of(ops, sizes);
        sizes.mPrecomputedPseudos += self.precomputed_pseudo_element_decls.size_of(ops);
    }
}

type UserAgentCascadeDataCache = CascadeDataCache<UserAgentCascadeData>;

type PrecomputedPseudoElementDeclarations = PerPseudoElementMap<Vec<ApplicableDeclarationBlock>>;

#[derive(Default)]
struct UserAgentCascadeData {
    cascade_data: CascadeData,

    /// Applicable declarations for a given non-eagerly cascaded pseudo-element.
    ///
    /// These are eagerly computed once, and then used to resolve the new
    /// computed values on the fly on layout.
    ///
    /// These are only filled from UA stylesheets.
    precomputed_pseudo_element_decls: PrecomputedPseudoElementDeclarations,
}

/// All the computed information for all the stylesheets that apply to the
/// document.
#[derive(Default)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct DocumentCascadeData {
    #[cfg_attr(
        feature = "servo",
        ignore_malloc_size_of = "Arc, owned by UserAgentCascadeDataCache"
    )]
    user_agent: Arc<UserAgentCascadeData>,
    user: CascadeData,
    author: CascadeData,
    per_origin: PerOrigin<()>,
}

/// An iterator over the cascade data of a given document.
pub struct DocumentCascadeDataIter<'a> {
    iter: PerOriginIter<'a, ()>,
    cascade_data: &'a DocumentCascadeData,
}

impl<'a> Iterator for DocumentCascadeDataIter<'a> {
    type Item = (&'a CascadeData, Origin);

    fn next(&mut self) -> Option<Self::Item> {
        let (_, origin) = self.iter.next()?;
        Some((self.cascade_data.borrow_for_origin(origin), origin))
    }
}

impl DocumentCascadeData {
    /// Borrows the cascade data for a given origin.
    #[inline]
    pub fn borrow_for_origin(&self, origin: Origin) -> &CascadeData {
        match origin {
            Origin::UserAgent => &self.user_agent.cascade_data,
            Origin::Author => &self.author,
            Origin::User => &self.user,
        }
    }

    fn iter_origins(&self) -> DocumentCascadeDataIter {
        DocumentCascadeDataIter {
            iter: self.per_origin.iter_origins(),
            cascade_data: self,
        }
    }

    fn iter_origins_rev(&self) -> DocumentCascadeDataIter {
        DocumentCascadeDataIter {
            iter: self.per_origin.iter_origins_rev(),
            cascade_data: self,
        }
    }

    /// Rebuild the cascade data for the given document stylesheets, and
    /// optionally with a set of user agent stylesheets.  Returns Err(..)
    /// to signify OOM.
    fn rebuild<'a, S>(
        &mut self,
        device: &Device,
        quirks_mode: QuirksMode,
        mut flusher: DocumentStylesheetFlusher<'a, S>,
        guards: &StylesheetGuards,
    ) -> Result<(), AllocErr>
    where
        S: StylesheetInDocument + PartialEq + 'static,
    {
        // First do UA sheets.
        {
            let origin_flusher = flusher.flush_origin(Origin::UserAgent);
            // Dirty check is just a minor optimization (no need to grab the
            // lock if nothing has changed).
            if origin_flusher.dirty() {
                let mut ua_cache = UA_CASCADE_DATA_CACHE.lock().unwrap();
                let new_data = ua_cache.lookup(
                    device,
                    quirks_mode,
                    origin_flusher,
                    guards.ua_or_user,
                    &self.user_agent,
                )?;
                if let Some(new_data) = new_data {
                    self.user_agent = new_data;
                }
                let _unused_entries = ua_cache.take_unused();
                // See the comments in take_unused() as for why the following
                // line.
                std::mem::drop(ua_cache);
            }
        }

        // Now do the user sheets.
        self.user.rebuild(
            device,
            quirks_mode,
            flusher.flush_origin(Origin::User),
            guards.ua_or_user,
        )?;

        // And now the author sheets.
        self.author.rebuild(
            device,
            quirks_mode,
            flusher.flush_origin(Origin::Author),
            guards.author,
        )?;

        Ok(())
    }

    /// Measures heap usage.
    #[cfg(feature = "gecko")]
    pub fn add_size_of(&self, ops: &mut MallocSizeOfOps, sizes: &mut ServoStyleSetSizes) {
        self.user.add_size_of(ops, sizes);
        self.author.add_size_of(ops, sizes);
    }
}

/// Whether author styles are enabled.
///
/// This is used to support Gecko.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq)]
pub enum AuthorStylesEnabled {
    Yes,
    No,
}

/// A wrapper over a DocumentStylesheetSet that can be `Sync`, since it's only
/// used and exposed via mutable methods in the `Stylist`.
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
struct StylistStylesheetSet(DocumentStylesheetSet<StylistSheet>);
// Read above to see why this is fine.
unsafe impl Sync for StylistStylesheetSet {}

impl StylistStylesheetSet {
    fn new() -> Self {
        StylistStylesheetSet(DocumentStylesheetSet::new())
    }
}

impl ops::Deref for StylistStylesheetSet {
    type Target = DocumentStylesheetSet<StylistSheet>;

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
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
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

    /// The list of stylesheets.
    stylesheets: StylistStylesheetSet,

    /// A cache of CascadeDatas for AuthorStylesheetSets (i.e., shadow DOM).
    #[cfg_attr(feature = "servo", ignore_malloc_size_of = "XXX: how to handle this?")]
    author_data_cache: CascadeDataCache<CascadeData>,

    /// If true, the quirks-mode stylesheet is applied.
    #[cfg_attr(feature = "servo", ignore_malloc_size_of = "defined in selectors")]
    quirks_mode: QuirksMode,

    /// Selector maps for all of the style sheets in the stylist, after
    /// evalutaing media rules against the current device, split out per
    /// cascade level.
    cascade_data: DocumentCascadeData,

    /// Whether author styles are enabled.
    author_styles_enabled: AuthorStylesEnabled,

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

type AncestorSelectorList<'a> = Cow<'a, SelectorList<SelectorImpl>>;

/// A struct containing state from ancestor rules like @layer / @import /
/// @container / nesting.
struct ContainingRuleState<'a> {
    layer_name: LayerName,
    layer_id: LayerId,
    container_condition_id: ContainerConditionId,
    ancestor_selector_lists: SmallVec<[AncestorSelectorList<'a>; 2]>,
}

impl<'a> Default for ContainingRuleState<'a> {
    fn default() -> Self {
        Self {
            layer_name: LayerName::new_empty(),
            layer_id: LayerId::root(),
            container_condition_id: ContainerConditionId::none(),
            ancestor_selector_lists: Default::default(),
        }
    }
}

struct SavedContainingRuleState {
    ancestor_selector_lists_len: usize,
    layer_name_len: usize,
    layer_id: LayerId,
    container_condition_id: ContainerConditionId,
}

impl<'a> ContainingRuleState<'a> {
    fn save(&self) -> SavedContainingRuleState {
        SavedContainingRuleState {
            ancestor_selector_lists_len: self.ancestor_selector_lists.len(),
            layer_name_len: self.layer_name.0.len(),
            layer_id: self.layer_id,
            container_condition_id: self.container_condition_id,
        }
    }

    fn restore(&mut self, saved: &SavedContainingRuleState) {
        debug_assert!(self.layer_name.0.len() >= saved.layer_name_len);
        debug_assert!(self.ancestor_selector_lists.len() >= saved.ancestor_selector_lists_len);
        self.ancestor_selector_lists
            .truncate(saved.ancestor_selector_lists_len);
        self.layer_name.0.truncate(saved.layer_name_len);
        self.layer_id = saved.layer_id;
        self.container_condition_id = saved.container_condition_id;
    }
}

impl Stylist {
    /// Construct a new `Stylist`, using given `Device` and `QuirksMode`.
    /// If more members are added here, think about whether they should
    /// be reset in clear().
    #[inline]
    pub fn new(device: Device, quirks_mode: QuirksMode) -> Self {
        Self {
            device,
            quirks_mode,
            stylesheets: StylistStylesheetSet::new(),
            author_data_cache: CascadeDataCache::new(),
            cascade_data: Default::default(),
            author_styles_enabled: AuthorStylesEnabled::Yes,
            rule_tree: RuleTree::new(),
            num_rebuilds: 0,
        }
    }

    /// Returns the document cascade data.
    #[inline]
    pub fn cascade_data(&self) -> &DocumentCascadeData {
        &self.cascade_data
    }

    /// Returns whether author styles are enabled or not.
    #[inline]
    pub fn author_styles_enabled(&self) -> AuthorStylesEnabled {
        self.author_styles_enabled
    }

    /// Iterate through all the cascade datas from the document.
    #[inline]
    pub fn iter_origins(&self) -> DocumentCascadeDataIter {
        self.cascade_data.iter_origins()
    }

    /// Does what the name says, to prevent author_data_cache to grow without
    /// bound.
    pub fn remove_unique_author_data_cache_entries(&mut self) {
        self.author_data_cache.take_unused();
    }

    /// Rebuilds (if needed) the CascadeData given a sheet collection.
    pub fn rebuild_author_data<S>(
        &mut self,
        old_data: &CascadeData,
        collection: SheetCollectionFlusher<S>,
        guard: &SharedRwLockReadGuard,
    ) -> Result<Option<Arc<CascadeData>>, AllocErr>
    where
        S: StylesheetInDocument + PartialEq + 'static,
    {
        self.author_data_cache
            .lookup(&self.device, self.quirks_mode, collection, guard, old_data)
    }

    /// Iterate over the extra data in origin order.
    #[inline]
    pub fn iter_extra_data_origins(&self) -> ExtraStyleDataIterator {
        ExtraStyleDataIterator(self.cascade_data.iter_origins())
    }

    /// Iterate over the extra data in reverse origin order.
    #[inline]
    pub fn iter_extra_data_origins_rev(&self) -> ExtraStyleDataIterator {
        ExtraStyleDataIterator(self.cascade_data.iter_origins_rev())
    }

    /// Returns the number of selectors.
    pub fn num_selectors(&self) -> usize {
        self.cascade_data
            .iter_origins()
            .map(|(d, _)| d.num_selectors)
            .sum()
    }

    /// Returns the number of declarations.
    pub fn num_declarations(&self) -> usize {
        self.cascade_data
            .iter_origins()
            .map(|(d, _)| d.num_declarations)
            .sum()
    }

    /// Returns the number of times the stylist has been rebuilt.
    pub fn num_rebuilds(&self) -> usize {
        self.num_rebuilds
    }

    /// Returns the number of revalidation_selectors.
    pub fn num_revalidation_selectors(&self) -> usize {
        self.cascade_data
            .iter_origins()
            .map(|(data, _)| data.selectors_for_cache_revalidation.len())
            .sum()
    }

    /// Returns the number of entries in invalidation maps.
    pub fn num_invalidations(&self) -> usize {
        self.cascade_data
            .iter_origins()
            .map(|(data, _)| data.invalidation_map.len())
            .sum()
    }

    /// Returns whether the given DocumentState bit is relied upon by a selector
    /// of some rule.
    pub fn has_document_state_dependency(&self, state: DocumentState) -> bool {
        self.cascade_data
            .iter_origins()
            .any(|(d, _)| d.document_state_dependencies.intersects(state))
    }

    /// Flush the list of stylesheets if they changed, ensuring the stylist is
    /// up-to-date.
    pub fn flush<E>(
        &mut self,
        guards: &StylesheetGuards,
        document_element: Option<E>,
        snapshots: Option<&SnapshotMap>,
    ) -> bool
    where
        E: TElement,
    {
        if !self.stylesheets.has_changed() {
            return false;
        }

        self.num_rebuilds += 1;

        let flusher = self.stylesheets.flush(document_element, snapshots);

        let had_invalidations = flusher.had_invalidations();

        self.cascade_data
            .rebuild(&self.device, self.quirks_mode, flusher, guards)
            .unwrap_or_else(|_| warn!("OOM in Stylist::flush"));

        had_invalidations
    }

    /// Insert a given stylesheet before another stylesheet in the document.
    pub fn insert_stylesheet_before(
        &mut self,
        sheet: StylistSheet,
        before_sheet: StylistSheet,
        guard: &SharedRwLockReadGuard,
    ) {
        self.stylesheets
            .insert_stylesheet_before(Some(&self.device), sheet, before_sheet, guard)
    }

    /// Marks a given stylesheet origin as dirty, due to, for example, changes
    /// in the declarations that affect a given rule.
    ///
    /// FIXME(emilio): Eventually it'd be nice for this to become more
    /// fine-grained.
    pub fn force_stylesheet_origins_dirty(&mut self, origins: OriginSet) {
        self.stylesheets.force_dirty(origins)
    }

    /// Sets whether author style is enabled or not.
    pub fn set_author_styles_enabled(&mut self, enabled: AuthorStylesEnabled) {
        self.author_styles_enabled = enabled;
    }

    /// Returns whether we've recorded any stylesheet change so far.
    pub fn stylesheets_have_changed(&self) -> bool {
        self.stylesheets.has_changed()
    }

    /// Appends a new stylesheet to the current set.
    pub fn append_stylesheet(&mut self, sheet: StylistSheet, guard: &SharedRwLockReadGuard) {
        self.stylesheets
            .append_stylesheet(Some(&self.device), sheet, guard)
    }

    /// Remove a given stylesheet to the current set.
    pub fn remove_stylesheet(&mut self, sheet: StylistSheet, guard: &SharedRwLockReadGuard) {
        self.stylesheets
            .remove_stylesheet(Some(&self.device), sheet, guard)
    }

    /// Notify of a change of a given rule.
    pub fn rule_changed(
        &mut self,
        sheet: &StylistSheet,
        rule: &CssRule,
        guard: &SharedRwLockReadGuard,
        change_kind: RuleChangeKind,
    ) {
        self.stylesheets
            .rule_changed(Some(&self.device), sheet, rule, guard, change_kind)
    }

    /// Appends a new stylesheet to the current set.
    #[inline]
    pub fn sheet_count(&self, origin: Origin) -> usize {
        self.stylesheets.sheet_count(origin)
    }

    /// Appends a new stylesheet to the current set.
    #[inline]
    pub fn sheet_at(&self, origin: Origin, index: usize) -> Option<&StylistSheet> {
        self.stylesheets.get(origin, index)
    }

    /// Returns whether for any of the applicable style rule data a given
    /// condition is true.
    pub fn any_applicable_rule_data<E, F>(&self, element: E, mut f: F) -> bool
    where
        E: TElement,
        F: FnMut(&CascadeData) -> bool,
    {
        if f(&self.cascade_data.user_agent.cascade_data) {
            return true;
        }

        let mut maybe = false;

        let doc_author_rules_apply =
            element.each_applicable_non_document_style_rule_data(|data, _| {
                maybe = maybe || f(&*data);
            });

        if maybe || f(&self.cascade_data.user) {
            return true;
        }

        doc_author_rules_apply && f(&self.cascade_data.author)
    }

    /// Computes the style for a given "precomputed" pseudo-element, taking the
    /// universal rules and applying them.
    pub fn precomputed_values_for_pseudo<E>(
        &self,
        guards: &StylesheetGuards,
        pseudo: &PseudoElement,
        parent: Option<&ComputedValues>,
    ) -> Arc<ComputedValues>
    where
        E: TElement,
    {
        debug_assert!(pseudo.is_precomputed());

        let rule_node = self.rule_node_for_precomputed_pseudo(guards, pseudo, vec![]);

        self.precomputed_values_for_pseudo_with_rule_node::<E>(guards, pseudo, parent, rule_node)
    }

    /// Computes the style for a given "precomputed" pseudo-element with
    /// given rule node.
    ///
    /// TODO(emilio): The type parameter could go away with a void type
    /// implementing TElement.
    pub fn precomputed_values_for_pseudo_with_rule_node<E>(
        &self,
        guards: &StylesheetGuards,
        pseudo: &PseudoElement,
        parent: Option<&ComputedValues>,
        rules: StrongRuleNode,
    ) -> Arc<ComputedValues>
    where
        E: TElement,
    {
        self.compute_pseudo_element_style_with_inputs::<E>(
            CascadeInputs {
                rules: Some(rules),
                visited_rules: None,
                flags: Default::default(),
            },
            pseudo,
            guards,
            /* originating_element_style */ None,
            parent,
            /* element */ None,
        )
    }

    /// Returns the rule node for a given precomputed pseudo-element.
    ///
    /// If we want to include extra declarations to this precomputed
    /// pseudo-element, we can provide a vector of ApplicableDeclarationBlocks
    /// to extra_declarations. This is useful for @page rules.
    pub fn rule_node_for_precomputed_pseudo(
        &self,
        guards: &StylesheetGuards,
        pseudo: &PseudoElement,
        mut extra_declarations: Vec<ApplicableDeclarationBlock>,
    ) -> StrongRuleNode {
        let mut declarations_with_extra;
        let declarations = match self
            .cascade_data
            .user_agent
            .precomputed_pseudo_element_decls
            .get(pseudo)
        {
            Some(declarations) => {
                if !extra_declarations.is_empty() {
                    declarations_with_extra = declarations.clone();
                    declarations_with_extra.append(&mut extra_declarations);
                    &*declarations_with_extra
                } else {
                    &**declarations
                }
            },
            None => &[],
        };

        self.rule_tree.insert_ordered_rules_with_important(
            declarations.into_iter().map(|a| a.clone().for_rule_tree()),
            guards,
        )
    }

    /// Returns the style for an anonymous box of the given type.
    ///
    /// TODO(emilio): The type parameter could go away with a void type
    /// implementing TElement.
    #[cfg(feature = "servo")]
    pub fn style_for_anonymous<E>(
        &self,
        guards: &StylesheetGuards,
        pseudo: &PseudoElement,
        parent_style: &ComputedValues,
    ) -> Arc<ComputedValues>
    where
        E: TElement,
    {
        self.precomputed_values_for_pseudo::<E>(guards, &pseudo, Some(parent_style))
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
        element: E,
        pseudo: &PseudoElement,
        rule_inclusion: RuleInclusion,
        originating_element_style: &ComputedValues,
        parent_style: &Arc<ComputedValues>,
        is_probe: bool,
        matching_fn: Option<&dyn Fn(&PseudoElement) -> bool>,
    ) -> Option<Arc<ComputedValues>>
    where
        E: TElement,
    {
        let cascade_inputs = self.lazy_pseudo_rules(
            guards,
            element,
            originating_element_style,
            parent_style,
            pseudo,
            is_probe,
            rule_inclusion,
            matching_fn,
        )?;

        Some(self.compute_pseudo_element_style_with_inputs(
            cascade_inputs,
            pseudo,
            guards,
            Some(originating_element_style),
            Some(parent_style),
            Some(element),
        ))
    }

    /// Computes a pseudo-element style lazily using the given CascadeInputs.
    /// This can be used for truly lazy pseudo-elements or to avoid redoing
    /// selector matching for eager pseudo-elements when we need to recompute
    /// their style with a new parent style.
    pub fn compute_pseudo_element_style_with_inputs<E>(
        &self,
        inputs: CascadeInputs,
        pseudo: &PseudoElement,
        guards: &StylesheetGuards,
        originating_element_style: Option<&ComputedValues>,
        parent_style: Option<&ComputedValues>,
        element: Option<E>,
    ) -> Arc<ComputedValues>
    where
        E: TElement,
    {
        // FIXME(emilio): The lack of layout_parent_style here could be
        // worrying, but we're probably dropping the display fixup for
        // pseudos other than before and after, so it's probably ok.
        //
        // (Though the flags don't indicate so!)
        //
        // It'd be fine to assert that this isn't called with a parent style
        // where display contents is in effect, but in practice this is hard to
        // do for stuff like :-moz-fieldset-content with a
        // <fieldset style="display: contents">. That is, the computed value of
        // display for the fieldset is "contents", even though it's not the used
        // value, so we don't need to adjust in a different way anyway.
        self.cascade_style_and_visited(
            element,
            Some(pseudo),
            inputs,
            guards,
            originating_element_style,
            parent_style,
            parent_style,
            parent_style,
            /* rule_cache = */ None,
            &mut RuleCacheConditions::default(),
        )
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
    pub fn cascade_style_and_visited<E>(
        &self,
        element: Option<E>,
        pseudo: Option<&PseudoElement>,
        inputs: CascadeInputs,
        guards: &StylesheetGuards,
        originating_element_style: Option<&ComputedValues>,
        parent_style: Option<&ComputedValues>,
        parent_style_ignoring_first_line: Option<&ComputedValues>,
        layout_parent_style: Option<&ComputedValues>,
        rule_cache: Option<&RuleCache>,
        rule_cache_conditions: &mut RuleCacheConditions,
    ) -> Arc<ComputedValues>
    where
        E: TElement,
    {
        debug_assert!(pseudo.is_some() || element.is_some(), "Huh?");

        // We need to compute visited values if we have visited rules or if our
        // parent has visited values.
        let visited_rules = match inputs.visited_rules.as_ref() {
            Some(rules) => Some(rules),
            None => {
                if parent_style.and_then(|s| s.visited_style()).is_some() {
                    Some(inputs.rules.as_ref().unwrap_or(self.rule_tree.root()))
                } else {
                    None
                }
            },
        };

        // Read the comment on `precomputed_values_for_pseudo` to see why it's
        // difficult to assert that display: contents nodes never arrive here
        // (tl;dr: It doesn't apply for replaced elements and such, but the
        // computed value is still "contents").
        //
        // FIXME(emilio): We should assert that it holds if pseudo.is_none()!
        properties::cascade::<E>(
            &self.device,
            pseudo,
            inputs.rules.as_ref().unwrap_or(self.rule_tree.root()),
            guards,
            originating_element_style,
            parent_style,
            parent_style_ignoring_first_line,
            layout_parent_style,
            visited_rules,
            inputs.flags,
            self.quirks_mode,
            rule_cache,
            rule_cache_conditions,
            element,
        )
    }

    /// Computes the cascade inputs for a lazily-cascaded pseudo-element.
    ///
    /// See the documentation on lazy pseudo-elements in
    /// docs/components/style.md
    fn lazy_pseudo_rules<E>(
        &self,
        guards: &StylesheetGuards,
        element: E,
        originating_element_style: &ComputedValues,
        parent_style: &Arc<ComputedValues>,
        pseudo: &PseudoElement,
        is_probe: bool,
        rule_inclusion: RuleInclusion,
        matching_fn: Option<&dyn Fn(&PseudoElement) -> bool>,
    ) -> Option<CascadeInputs>
    where
        E: TElement,
    {
        debug_assert!(pseudo.is_lazy());

        let mut nth_index_cache = Default::default();
        // No need to bother setting the selector flags when we're computing
        // default styles.
        let needs_selector_flags = if rule_inclusion == RuleInclusion::DefaultOnly {
            NeedsSelectorFlags::No
        } else {
            NeedsSelectorFlags::Yes
        };

        let mut declarations = ApplicableDeclarationList::new();
        let mut matching_context = MatchingContext::<'_, E::Impl>::new(
            MatchingMode::ForStatelessPseudoElement,
            None,
            &mut nth_index_cache,
            self.quirks_mode,
            needs_selector_flags,
        );

        matching_context.pseudo_element_matching_fn = matching_fn;
        matching_context.extra_data.originating_element_style = Some(originating_element_style);

        self.push_applicable_declarations(
            element,
            Some(&pseudo),
            None,
            None,
            /* animation_declarations = */ Default::default(),
            rule_inclusion,
            &mut declarations,
            &mut matching_context,
        );

        if declarations.is_empty() && is_probe {
            return None;
        }

        let rules = self.rule_tree.compute_rule_node(&mut declarations, guards);

        let mut visited_rules = None;
        if parent_style.visited_style().is_some() {
            let mut declarations = ApplicableDeclarationList::new();
            let mut nth_index_cache = Default::default();

            let mut matching_context = MatchingContext::<'_, E::Impl>::new_for_visited(
                MatchingMode::ForStatelessPseudoElement,
                None,
                &mut nth_index_cache,
                VisitedHandlingMode::RelevantLinkVisited,
                self.quirks_mode,
                needs_selector_flags,
            );
            matching_context.pseudo_element_matching_fn = matching_fn;
            matching_context.extra_data.originating_element_style = Some(originating_element_style);

            self.push_applicable_declarations(
                element,
                Some(&pseudo),
                None,
                None,
                /* animation_declarations = */ Default::default(),
                rule_inclusion,
                &mut declarations,
                &mut matching_context,
            );
            if !declarations.is_empty() {
                let rule_node = self.rule_tree.insert_ordered_rules_with_important(
                    declarations.drain(..).map(|a| a.for_rule_tree()),
                    guards,
                );
                if rule_node != *self.rule_tree.root() {
                    visited_rules = Some(rule_node);
                }
            }
        }

        Some(CascadeInputs {
            rules: Some(rules),
            visited_rules,
            flags: matching_context.extra_data.cascade_input_flags,
        })
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
    pub fn set_device(&mut self, device: Device, guards: &StylesheetGuards) -> OriginSet {
        self.device = device;
        self.media_features_change_changed_style(guards, &self.device)
    }

    /// Returns whether, given a media feature change, any previously-applicable
    /// style has become non-applicable, or vice-versa for each origin, using
    /// `device`.
    pub fn media_features_change_changed_style(
        &self,
        guards: &StylesheetGuards,
        device: &Device,
    ) -> OriginSet {
        debug!("Stylist::media_features_change_changed_style {:?}", device);

        let mut origins = OriginSet::empty();
        let stylesheets = self.stylesheets.iter();

        for (stylesheet, origin) in stylesheets {
            if origins.contains(origin.into()) {
                continue;
            }

            let guard = guards.for_origin(origin);
            let origin_cascade_data = self.cascade_data.borrow_for_origin(origin);

            let affected_changed = !origin_cascade_data.media_feature_affected_matches(
                stylesheet,
                guard,
                device,
                self.quirks_mode,
            );

            if affected_changed {
                origins |= origin;
            }
        }

        origins
    }

    /// Returns the Quirks Mode of the document.
    pub fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode
    }

    /// Sets the quirks mode of the document.
    pub fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        if self.quirks_mode == quirks_mode {
            return;
        }
        self.quirks_mode = quirks_mode;
        self.force_stylesheet_origins_dirty(OriginSet::all());
    }

    /// Returns the applicable CSS declarations for the given element.
    pub fn push_applicable_declarations<E>(
        &self,
        element: E,
        pseudo_element: Option<&PseudoElement>,
        style_attribute: Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>,
        smil_override: Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>,
        animation_declarations: AnimationDeclarations,
        rule_inclusion: RuleInclusion,
        applicable_declarations: &mut ApplicableDeclarationList,
        context: &mut MatchingContext<E::Impl>,
    ) where
        E: TElement,
    {
        RuleCollector::new(
            self,
            element,
            pseudo_element,
            style_attribute,
            smil_override,
            animation_declarations,
            rule_inclusion,
            applicable_declarations,
            context,
        )
        .collect_all();
    }

    /// Given an id, returns whether there might be any rules for that id in any
    /// of our rule maps.
    #[inline]
    pub fn may_have_rules_for_id<E>(&self, id: &WeakAtom, element: E) -> bool
    where
        E: TElement,
    {
        // If id needs to be compared case-insensitively, the logic below
        // wouldn't work. Just conservatively assume it may have such rules.
        match self.quirks_mode().classes_and_ids_case_sensitivity() {
            CaseSensitivity::AsciiCaseInsensitive => return true,
            CaseSensitivity::CaseSensitive => {},
        }

        self.any_applicable_rule_data(element, |data| data.mapped_ids.contains(id))
    }

    /// Returns the registered `@keyframes` animation for the specified name.
    #[inline]
    pub fn get_animation<'a, E>(&'a self, name: &Atom, element: E) -> Option<&'a KeyframesAnimation>
    where
        E: TElement + 'a,
    {
        macro_rules! try_find_in {
            ($data:expr) => {
                if let Some(animation) = $data.animations.get(name) {
                    return Some(animation);
                }
            };
        }

        // NOTE(emilio): We implement basically what Blink does for this case,
        // which is [1] as of this writing.
        //
        // See [2] for the spec discussion about what to do about this. WebKit's
        // behavior makes a bit more sense off-hand, but it's way more complex
        // to implement, and it makes value computation having to thread around
        // the cascade level, which is not great. Also, it breaks if you inherit
        // animation-name from an element in a different tree.
        //
        // See [3] for the bug to implement whatever gets resolved, and related
        // bugs for a bit more context.
        //
        // FIXME(emilio): This should probably work for pseudo-elements (i.e.,
        // use rule_hash_target().shadow_root() instead of
        // element.shadow_root()).
        //
        // [1]: https://cs.chromium.org/chromium/src/third_party/blink/renderer/
        //        core/css/resolver/style_resolver.cc?l=1267&rcl=90f9f8680ebb4a87d177f3b0833372ae4e0c88d8
        // [2]: https://github.com/w3c/csswg-drafts/issues/1995
        // [3]: https://bugzil.la/1458189
        if let Some(shadow) = element.shadow_root() {
            if let Some(data) = shadow.style_data() {
                try_find_in!(data);
            }
        }

        // Use the same rules to look for the containing host as we do for rule
        // collection.
        if let Some(shadow) = containing_shadow_ignoring_svg_use(element) {
            if let Some(data) = shadow.style_data() {
                try_find_in!(data);
            }
        } else {
            try_find_in!(self.cascade_data.author);
        }

        try_find_in!(self.cascade_data.user);
        try_find_in!(self.cascade_data.user_agent.cascade_data);

        None
    }

    /// Computes the match results of a given element against the set of
    /// revalidation selectors.
    pub fn match_revalidation_selectors<E>(
        &self,
        element: E,
        bloom: Option<&BloomFilter>,
        nth_index_cache: &mut NthIndexCache,
        needs_selector_flags: NeedsSelectorFlags,
    ) -> SmallBitVec
    where
        E: TElement,
    {
        // NB: `MatchingMode` doesn't really matter, given we don't share style
        // between pseudos.
        let mut matching_context = MatchingContext::new(
            MatchingMode::Normal,
            bloom,
            nth_index_cache,
            self.quirks_mode,
            needs_selector_flags,
        );

        // Note that, by the time we're revalidating, we're guaranteed that the
        // candidate and the entry have the same id, classes, and local name.
        // This means we're guaranteed to get the same rulehash buckets for all
        // the lookups, which means that the bitvecs are comparable. We verify
        // this in the caller by asserting that the bitvecs are same-length.
        let mut results = SmallBitVec::new();

        let matches_document_rules =
            element.each_applicable_non_document_style_rule_data(|data, host| {
                matching_context.with_shadow_host(Some(host), |matching_context| {
                    data.selectors_for_cache_revalidation.lookup(
                        element,
                        self.quirks_mode,
                        |selector_and_hashes| {
                            results.push(matches_selector(
                                &selector_and_hashes.selector,
                                selector_and_hashes.selector_offset,
                                Some(&selector_and_hashes.hashes),
                                &element,
                                matching_context,
                            ));
                            true
                        },
                    );
                })
            });

        for (data, origin) in self.cascade_data.iter_origins() {
            if origin == Origin::Author && !matches_document_rules {
                continue;
            }

            data.selectors_for_cache_revalidation.lookup(
                element,
                self.quirks_mode,
                |selector_and_hashes| {
                    results.push(matches_selector(
                        &selector_and_hashes.selector,
                        selector_and_hashes.selector_offset,
                        Some(&selector_and_hashes.hashes),
                        &element,
                        &mut matching_context,
                    ));
                    true
                },
            );
        }

        results
    }

    /// Computes styles for a given declaration with parent_style.
    ///
    /// FIXME(emilio): the lack of pseudo / cascade flags look quite dubious,
    /// hopefully this is only used for some canvas font stuff.
    ///
    /// TODO(emilio): The type parameter can go away when
    /// https://github.com/rust-lang/rust/issues/35121 is fixed.
    pub fn compute_for_declarations<E>(
        &self,
        guards: &StylesheetGuards,
        parent_style: &ComputedValues,
        declarations: Arc<Locked<PropertyDeclarationBlock>>,
    ) -> Arc<ComputedValues>
    where
        E: TElement,
    {
        let block = declarations.read_with(guards.author);

        // We don't bother inserting these declarations in the rule tree, since
        // it'd be quite useless and slow.
        //
        // TODO(emilio): Now that we fixed bug 1493420, we should consider
        // reversing this as it shouldn't be slow anymore, and should avoid
        // generating two instantiations of apply_declarations.
        properties::apply_declarations::<E, _>(
            &self.device,
            /* pseudo = */ None,
            self.rule_tree.root(),
            guards,
            block.declaration_importance_iter().map(|(declaration, _)| {
                (
                    declaration,
                    CascadePriority::new(
                        CascadeLevel::same_tree_author_normal(),
                        LayerOrder::root(),
                    ),
                )
            }),
            /* originating_element_style */ None,
            Some(parent_style),
            Some(parent_style),
            Some(parent_style),
            CascadeMode::Unvisited {
                visited_rules: None,
            },
            Default::default(),
            self.quirks_mode,
            /* rule_cache = */ None,
            &mut Default::default(),
            /* element = */ None,
        )
    }

    /// Accessor for a shared reference to the device.
    #[inline]
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Accessor for a mutable reference to the device.
    #[inline]
    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    /// Accessor for a shared reference to the rule tree.
    #[inline]
    pub fn rule_tree(&self) -> &RuleTree {
        &self.rule_tree
    }

    /// Measures heap usage.
    #[cfg(feature = "gecko")]
    pub fn add_size_of(&self, ops: &mut MallocSizeOfOps, sizes: &mut ServoStyleSetSizes) {
        self.cascade_data.add_size_of(ops, sizes);
        self.author_data_cache.add_size_of(ops, sizes);
        sizes.mRuleTree += self.rule_tree.size_of(ops);

        // We may measure other fields in the future if DMD says it's worth it.
    }

    /// Shutdown the static data that this module stores.
    pub fn shutdown() {
        let _entries = UA_CASCADE_DATA_CACHE.lock().unwrap().take_all();
    }
}

/// A vector that is sorted in layer order.
#[derive(Clone, Debug, Deref, MallocSizeOf)]
pub struct LayerOrderedVec<T>(Vec<(T, LayerId)>);
impl<T> Default for LayerOrderedVec<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// A map that is sorted in layer order.
#[derive(Clone, Debug, Deref, MallocSizeOf)]
pub struct LayerOrderedMap<T>(PrecomputedHashMap<Atom, SmallVec<[(T, LayerId); 1]>>);
impl<T> Default for LayerOrderedMap<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[cfg(feature = "gecko")]
impl<T: 'static> LayerOrderedVec<T> {
    fn clear(&mut self) {
        self.0.clear();
    }
    fn push(&mut self, v: T, id: LayerId) {
        self.0.push((v, id));
    }
    fn sort(&mut self, layers: &[CascadeLayer]) {
        self.0
            .sort_by_key(|&(_, ref id)| layers[id.0 as usize].order)
    }
}

impl<T: 'static> LayerOrderedMap<T> {
    fn clear(&mut self) {
        self.0.clear();
    }
    #[cfg(feature = "gecko")]
    fn try_insert(&mut self, name: Atom, v: T, id: LayerId) -> Result<(), AllocErr> {
        self.try_insert_with(name, v, id, |_, _| Ordering::Equal)
    }
    fn try_insert_with(
        &mut self,
        name: Atom,
        v: T,
        id: LayerId,
        cmp: impl Fn(&T, &T) -> Ordering,
    ) -> Result<(), AllocErr> {
        self.0.try_reserve(1)?;
        let vec = self.0.entry(name).or_default();
        if let Some(&mut (ref mut val, ref last_id)) = vec.last_mut() {
            if *last_id == id {
                if cmp(&val, &v) != Ordering::Greater {
                    *val = v;
                }
                return Ok(());
            }
        }
        vec.push((v, id));
        Ok(())
    }
    #[cfg(feature = "gecko")]
    fn sort(&mut self, layers: &[CascadeLayer]) {
        self.sort_with(layers, |_, _| Ordering::Equal)
    }
    fn sort_with(&mut self, layers: &[CascadeLayer], cmp: impl Fn(&T, &T) -> Ordering) {
        for (_, v) in self.0.iter_mut() {
            v.sort_by(|&(ref v1, ref id1), &(ref v2, ref id2)| {
                let order1 = layers[id1.0 as usize].order;
                let order2 = layers[id2.0 as usize].order;
                order1.cmp(&order2).then_with(|| cmp(v1, v2))
            })
        }
    }
    /// Get an entry on the LayerOrderedMap by name.
    pub fn get(&self, name: &Atom) -> Option<&T> {
        let vec = self.0.get(name)?;
        Some(&vec.last()?.0)
    }
}

/// Wrapper to allow better tracking of memory usage by page rule lists.
///
/// This includes the layer ID for use with the named page table.
#[derive(Clone, Debug, MallocSizeOf)]
pub struct PageRuleData {
    /// Layer ID for sorting page rules after matching.
    pub layer: LayerId,
    /// Page rule
    #[ignore_malloc_size_of = "Arc, stylesheet measures as primary ref"]
    pub rule: Arc<Locked<PageRule>>,
}

/// Wrapper to allow better tracking of memory usage by page rule lists.
///
/// This is meant to be used by the global page rule list which are already
/// sorted by layer ID, since all global page rules are less specific than all
/// named page rules that match a certain page.
#[derive(Clone, Debug, Deref, MallocSizeOf)]
pub struct PageRuleDataNoLayer(
    #[ignore_malloc_size_of = "Arc, stylesheet measures as primary ref"] pub Arc<Locked<PageRule>>,
);

/// Stores page rules indexed by page names.
#[derive(Clone, Debug, Default, MallocSizeOf)]
pub struct PageRuleMap {
    /// Global, unnamed page rules.
    pub global: LayerOrderedVec<PageRuleDataNoLayer>,
    /// Named page rules
    pub named: PrecomputedHashMap<Atom, SmallVec<[PageRuleData; 1]>>,
}

#[cfg(feature = "gecko")]
impl PageRuleMap {
    #[inline]
    fn clear(&mut self) {
        self.global.clear();
        self.named.clear();
    }
}

impl MallocShallowSizeOf for PageRuleMap {
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.global.size_of(ops) + self.named.shallow_size_of(ops)
    }
}

/// This struct holds data which users of Stylist may want to extract
/// from stylesheets which can be done at the same time as updating.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct ExtraStyleData {
    /// A list of effective font-face rules and their origin.
    #[cfg(feature = "gecko")]
    pub font_faces: LayerOrderedVec<Arc<Locked<FontFaceRule>>>,

    /// A list of effective font-feature-values rules.
    #[cfg(feature = "gecko")]
    pub font_feature_values: LayerOrderedVec<Arc<FontFeatureValuesRule>>,

    /// A list of effective font-palette-values rules.
    #[cfg(feature = "gecko")]
    pub font_palette_values: LayerOrderedVec<Arc<FontPaletteValuesRule>>,

    /// A map of effective counter-style rules.
    #[cfg(feature = "gecko")]
    pub counter_styles: LayerOrderedMap<Arc<Locked<CounterStyleRule>>>,

    /// A map of effective page rules.
    #[cfg(feature = "gecko")]
    pub pages: PageRuleMap,
}

#[cfg(feature = "gecko")]
impl ExtraStyleData {
    /// Add the given @font-face rule.
    fn add_font_face(&mut self, rule: &Arc<Locked<FontFaceRule>>, layer: LayerId) {
        self.font_faces.push(rule.clone(), layer);
    }

    /// Add the given @font-feature-values rule.
    fn add_font_feature_values(&mut self, rule: &Arc<FontFeatureValuesRule>, layer: LayerId) {
        self.font_feature_values.push(rule.clone(), layer);
    }

    /// Add the given @font-palette-values rule.
    fn add_font_palette_values(&mut self, rule: &Arc<FontPaletteValuesRule>, layer: LayerId) {
        self.font_palette_values.push(rule.clone(), layer);
    }

    /// Add the given @counter-style rule.
    fn add_counter_style(
        &mut self,
        guard: &SharedRwLockReadGuard,
        rule: &Arc<Locked<CounterStyleRule>>,
        layer: LayerId,
    ) -> Result<(), AllocErr> {
        let name = rule.read_with(guard).name().0.clone();
        self.counter_styles.try_insert(name, rule.clone(), layer)
    }

    /// Add the given @page rule.
    fn add_page(
        &mut self,
        guard: &SharedRwLockReadGuard,
        rule: &Arc<Locked<PageRule>>,
        layer: LayerId,
    ) -> Result<(), AllocErr> {
        let page_rule = rule.read_with(guard);
        if page_rule.selectors.0.is_empty() {
            self.pages
                .global
                .push(PageRuleDataNoLayer(rule.clone()), layer);
        } else {
            // TODO: Handle pseudo-classes
            self.pages.named.try_reserve(page_rule.selectors.0.len())?;
            for name in page_rule.selectors.as_slice() {
                let vec = self.pages.named.entry(name.0 .0.clone()).or_default();
                vec.try_reserve(1)?;
                vec.push(PageRuleData {
                    layer,
                    rule: rule.clone(),
                });
            }
        }
        Ok(())
    }

    fn sort_by_layer(&mut self, layers: &[CascadeLayer]) {
        self.font_faces.sort(layers);
        self.font_feature_values.sort(layers);
        self.font_palette_values.sort(layers);
        self.counter_styles.sort(layers);
        self.pages.global.sort(layers);
    }

    fn clear(&mut self) {
        #[cfg(feature = "gecko")]
        {
            self.font_faces.clear();
            self.font_feature_values.clear();
            self.font_palette_values.clear();
            self.counter_styles.clear();
            self.pages.clear();
        }
    }
}

// Don't let a prefixed keyframes animation override
// a non-prefixed one.
fn compare_keyframes_in_same_layer(v1: &KeyframesAnimation, v2: &KeyframesAnimation) -> Ordering {
    if v1.vendor_prefix.is_some() == v2.vendor_prefix.is_some() {
        Ordering::Equal
    } else if v2.vendor_prefix.is_some() {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

/// An iterator over the different ExtraStyleData.
pub struct ExtraStyleDataIterator<'a>(DocumentCascadeDataIter<'a>);

impl<'a> Iterator for ExtraStyleDataIterator<'a> {
    type Item = (&'a ExtraStyleData, Origin);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|d| (&d.0.extra_data, d.1))
    }
}

#[cfg(feature = "gecko")]
impl MallocSizeOf for ExtraStyleData {
    /// Measure heap usage.
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = 0;
        n += self.font_faces.shallow_size_of(ops);
        n += self.font_feature_values.shallow_size_of(ops);
        n += self.font_palette_values.shallow_size_of(ops);
        n += self.counter_styles.shallow_size_of(ops);
        n += self.pages.shallow_size_of(ops);
        n
    }
}

/// SelectorMapEntry implementation for use in our revalidation selector map.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug)]
struct RevalidationSelectorAndHashes {
    #[cfg_attr(
        feature = "gecko",
        ignore_malloc_size_of = "CssRules have primary refs, we measure there"
    )]
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
                _ => 0,
            }
        };

        RevalidationSelectorAndHashes {
            selector,
            selector_offset,
            hashes,
        }
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
    /// Whether we've past the rightmost compound selector, not counting
    /// pseudo-elements.
    passed_rightmost_selector: bool,

    /// Whether the selector needs revalidation for the style sharing cache.
    needs_revalidation: &'a mut bool,

    /// Flags for which selector list-containing components the visitor is
    /// inside of, if any
    in_selector_list_of: SelectorListKind,

    /// The filter with all the id's getting referenced from rightmost
    /// selectors.
    mapped_ids: &'a mut PrecomputedHashSet<Atom>,

    /// The filter with the IDs getting referenced from the selector list of
    /// :nth-child(... of <selector list>) selectors.
    nth_of_mapped_ids: &'a mut PrecomputedHashSet<Atom>,

    /// The filter with the local names of attributes there are selectors for.
    attribute_dependencies: &'a mut PrecomputedHashSet<LocalName>,

    /// The filter with the classes getting referenced from the selector list of
    /// :nth-child(... of <selector list>) selectors.
    nth_of_class_dependencies: &'a mut PrecomputedHashSet<Atom>,

    /// The filter with the local names of attributes there are selectors for
    /// within the selector list of :nth-child(... of <selector list>)
    /// selectors.
    nth_of_attribute_dependencies: &'a mut PrecomputedHashSet<LocalName>,

    /// All the states selectors in the page reference.
    state_dependencies: &'a mut ElementState,

    /// All the state selectors in the page reference within the selector list
    /// of :nth-child(... of <selector list>) selectors.
    nth_of_state_dependencies: &'a mut ElementState,

    /// All the document states selectors in the page reference.
    document_state_dependencies: &'a mut DocumentState,
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
        },
        Component::AttributeInNoNamespaceExists { .. } |
        Component::AttributeInNoNamespace { .. } |
        Component::AttributeOther(_) |
        Component::Empty |
        Component::Nth(_) |
        Component::NthOf(_) => true,
        Component::NonTSPseudoClass(ref p) => p.needs_cache_revalidation(),
        _ => false,
    }
}

impl<'a> SelectorVisitor for StylistSelectorVisitor<'a> {
    type Impl = SelectorImpl;

    fn visit_complex_selector(&mut self, combinator: Option<Combinator>) -> bool {
        *self.needs_revalidation =
            *self.needs_revalidation || combinator.map_or(false, |c| c.is_sibling());

        // NOTE(emilio): this call happens before we visit any of the simple
        // selectors in the next ComplexSelector, so we can use this to skip
        // looking at them.
        self.passed_rightmost_selector = self.passed_rightmost_selector ||
            !matches!(combinator, None | Some(Combinator::PseudoElement));

        true
    }

    fn visit_selector_list(
        &mut self,
        list_kind: SelectorListKind,
        list: &[Selector<Self::Impl>],
    ) -> bool {
        let in_selector_list_of = self.in_selector_list_of | list_kind;
        for selector in list {
            let mut nested = StylistSelectorVisitor {
                passed_rightmost_selector: false,
                needs_revalidation: &mut *self.needs_revalidation,
                in_selector_list_of,
                mapped_ids: &mut *self.mapped_ids,
                nth_of_mapped_ids: &mut *self.nth_of_mapped_ids,
                attribute_dependencies: &mut *self.attribute_dependencies,
                nth_of_class_dependencies: &mut *self.nth_of_class_dependencies,
                nth_of_attribute_dependencies: &mut *self.nth_of_attribute_dependencies,
                state_dependencies: &mut *self.state_dependencies,
                nth_of_state_dependencies: &mut *self.nth_of_state_dependencies,
                document_state_dependencies: &mut *self.document_state_dependencies,
            };
            let _ret = selector.visit(&mut nested);
            debug_assert!(_ret, "We never return false");
        }
        true
    }

    fn visit_attribute_selector(
        &mut self,
        _ns: &NamespaceConstraint<&Namespace>,
        name: &LocalName,
        lower_name: &LocalName,
    ) -> bool {
        if self.in_selector_list_of.in_nth_of() {
            self.nth_of_attribute_dependencies.insert(name.clone());
            if name != lower_name {
                self.nth_of_attribute_dependencies
                    .insert(lower_name.clone());
            }
        }

        self.attribute_dependencies.insert(name.clone());
        if name != lower_name {
            self.attribute_dependencies.insert(lower_name.clone());
        }

        true
    }

    fn visit_simple_selector(&mut self, s: &Component<SelectorImpl>) -> bool {
        *self.needs_revalidation = *self.needs_revalidation ||
            component_needs_revalidation(s, self.passed_rightmost_selector);

        match *s {
            Component::NonTSPseudoClass(ref p) => {
                self.state_dependencies.insert(p.state_flag());
                self.document_state_dependencies
                    .insert(p.document_state_flag());

                if self.in_selector_list_of.in_nth_of() {
                    self.nth_of_state_dependencies.insert(p.state_flag());
                }
            },
            Component::ID(ref id) => {
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
                if !self.passed_rightmost_selector {
                    self.mapped_ids.insert(id.0.clone());
                }

                if self.in_selector_list_of.in_nth_of() {
                    self.nth_of_mapped_ids.insert(id.0.clone());
                }
            },
            Component::Class(ref class) if self.in_selector_list_of.in_nth_of() => {
                self.nth_of_class_dependencies.insert(class.0.clone());
            },
            _ => {},
        }

        true
    }
}

/// A set of rules for element and pseudo-elements.
#[derive(Clone, Debug, Default, MallocSizeOf)]
struct GenericElementAndPseudoRules<Map> {
    /// Rules from stylesheets at this `CascadeData`'s origin.
    element_map: Map,

    /// Rules from stylesheets at this `CascadeData`'s origin that correspond
    /// to a given pseudo-element.
    ///
    /// FIXME(emilio): There are a bunch of wasted entries here in practice.
    /// Figure out a good way to do a `PerNonAnonBox` and `PerAnonBox` (for
    /// `precomputed_values_for_pseudo`) without duplicating a lot of code.
    pseudos_map: PerPseudoElementMap<Box<Map>>,
}

impl<Map: Default + MallocSizeOf> GenericElementAndPseudoRules<Map> {
    #[inline(always)]
    fn for_insertion(&mut self, pseudo_element: Option<&PseudoElement>) -> &mut Map {
        debug_assert!(
            pseudo_element.map_or(true, |pseudo| {
                !pseudo.is_precomputed() && !pseudo.is_unknown_webkit_pseudo_element()
            }),
            "Precomputed pseudos should end up in precomputed_pseudo_element_decls, \
             and unknown webkit pseudos should be discarded before getting here"
        );

        match pseudo_element {
            None => &mut self.element_map,
            Some(pseudo) => self
                .pseudos_map
                .get_or_insert_with(pseudo, || Box::new(Default::default())),
        }
    }

    #[inline]
    fn rules(&self, pseudo: Option<&PseudoElement>) -> Option<&Map> {
        match pseudo {
            Some(pseudo) => self.pseudos_map.get(pseudo).map(|p| &**p),
            None => Some(&self.element_map),
        }
    }

    /// Measures heap usage.
    #[cfg(feature = "gecko")]
    fn add_size_of(&self, ops: &mut MallocSizeOfOps, sizes: &mut ServoStyleSetSizes) {
        sizes.mElementAndPseudosMaps += self.element_map.size_of(ops);

        for elem in self.pseudos_map.iter() {
            if let Some(ref elem) = *elem {
                sizes.mElementAndPseudosMaps += <Box<_> as MallocSizeOf>::size_of(elem, ops);
            }
        }
    }
}

type ElementAndPseudoRules = GenericElementAndPseudoRules<SelectorMap<Rule>>;
type PartMap = PrecomputedHashMap<Atom, SmallVec<[Rule; 1]>>;
type PartElementAndPseudoRules = GenericElementAndPseudoRules<PartMap>;

impl ElementAndPseudoRules {
    // TODO(emilio): Should we retain storage of these?
    fn clear(&mut self) {
        self.element_map.clear();
        self.pseudos_map.clear();
    }

    fn shrink_if_needed(&mut self) {
        self.element_map.shrink_if_needed();
        for pseudo in self.pseudos_map.iter_mut() {
            if let Some(ref mut pseudo) = pseudo {
                pseudo.shrink_if_needed();
            }
        }
    }
}

impl PartElementAndPseudoRules {
    // TODO(emilio): Should we retain storage of these?
    fn clear(&mut self) {
        self.element_map.clear();
        self.pseudos_map.clear();
    }
}

/// The id of a given layer, a sequentially-increasing identifier.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, PartialOrd, Ord)]
pub struct LayerId(u16);

impl LayerId {
    /// The id of the root layer.
    pub const fn root() -> Self {
        Self(0)
    }
}

#[derive(Clone, Debug, MallocSizeOf)]
struct CascadeLayer {
    id: LayerId,
    order: LayerOrder,
    children: Vec<LayerId>,
}

impl CascadeLayer {
    const fn root() -> Self {
        Self {
            id: LayerId::root(),
            order: LayerOrder::root(),
            children: vec![],
        }
    }
}

/// The id of a given container condition, a sequentially-increasing identifier
/// for a given style set.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, PartialOrd, Ord)]
pub struct ContainerConditionId(u16);

impl ContainerConditionId {
    /// A special id that represents no container rule all.
    pub const fn none() -> Self {
        Self(0)
    }
}

#[derive(Clone, Debug, MallocSizeOf)]
struct ContainerConditionReference {
    parent: ContainerConditionId,
    #[ignore_malloc_size_of = "Arc"]
    condition: Option<Arc<ContainerCondition>>,
}

impl ContainerConditionReference {
    const fn none() -> Self {
        Self {
            parent: ContainerConditionId::none(),
            condition: None,
        }
    }
}

/// Data resulting from performing the CSS cascade that is specific to a given
/// origin.
///
/// FIXME(emilio): Consider renaming and splitting in `CascadeData` and
/// `InvalidationData`? That'd make `clear_cascade_data()` clearer.
#[derive(Debug, Clone, MallocSizeOf)]
pub struct CascadeData {
    /// The data coming from normal style rules that apply to elements at this
    /// cascade level.
    normal_rules: ElementAndPseudoRules,

    /// The `:host` pseudo rules that are the rightmost selector (without
    /// accounting for pseudo-elements).
    host_rules: Option<Box<ElementAndPseudoRules>>,

    /// The data coming from ::slotted() pseudo-element rules.
    ///
    /// We need to store them separately because an element needs to match
    /// ::slotted() pseudo-element rules in different shadow roots.
    ///
    /// In particular, we need to go through all the style data in all the
    /// containing style scopes starting from the closest assigned slot.
    slotted_rules: Option<Box<ElementAndPseudoRules>>,

    /// The data coming from ::part() pseudo-element rules.
    ///
    /// We need to store them separately because an element needs to match
    /// ::part() pseudo-element rules in different shadow roots.
    part_rules: Option<Box<PartElementAndPseudoRules>>,

    /// The invalidation map for these rules.
    invalidation_map: InvalidationMap,

    /// The attribute local names that appear in attribute selectors.  Used
    /// to avoid taking element snapshots when an irrelevant attribute changes.
    /// (We don't bother storing the namespace, since namespaced attributes are
    /// rare.)
    attribute_dependencies: PrecomputedHashSet<LocalName>,

    /// The classes that appear in the selector list of
    /// :nth-child(... of <selector list>). Used to avoid restyling siblings of
    /// an element when an irrelevant class changes.
    nth_of_class_dependencies: PrecomputedHashSet<Atom>,

    /// The attributes that appear in the selector list of
    /// :nth-child(... of <selector list>). Used to avoid restyling siblings of
    /// an element when an irrelevant attribute changes.
    nth_of_attribute_dependencies: PrecomputedHashSet<LocalName>,

    /// The element state bits that are relied on by selectors.  Like
    /// `attribute_dependencies`, this is used to avoid taking element snapshots
    /// when an irrelevant element state bit changes.
    state_dependencies: ElementState,

    /// The element state bits that are relied on by selectors that appear in
    /// the selector list of :nth-child(... of <selector list>).
    nth_of_state_dependencies: ElementState,

    /// The document state bits that are relied on by selectors.  This is used
    /// to tell whether we need to restyle the entire document when a document
    /// state bit changes.
    document_state_dependencies: DocumentState,

    /// The ids that appear in the rightmost complex selector of selectors (and
    /// hence in our selector maps).  Used to determine when sharing styles is
    /// safe: we disallow style sharing for elements whose id matches this
    /// filter, and hence might be in one of our selector maps.
    mapped_ids: PrecomputedHashSet<Atom>,

    /// The IDs that appear in the selector list of
    /// :nth-child(... of <selector list>). Used to avoid restyling siblings
    /// of an element when an irrelevant ID changes.
    nth_of_mapped_ids: PrecomputedHashSet<Atom>,

    /// Selectors that require explicit cache revalidation (i.e. which depend
    /// on state that is not otherwise visible to the cache, like attributes or
    /// tree-structural state like child index and pseudos).
    #[ignore_malloc_size_of = "Arc"]
    selectors_for_cache_revalidation: SelectorMap<RevalidationSelectorAndHashes>,

    /// A map with all the animations at this `CascadeData`'s origin, indexed
    /// by name.
    animations: LayerOrderedMap<KeyframesAnimation>,

    /// A map from cascade layer name to layer order.
    layer_id: FxHashMap<LayerName, LayerId>,

    /// The list of cascade layers, indexed by their layer id.
    layers: SmallVec<[CascadeLayer; 1]>,

    /// The list of container conditions, indexed by their id.
    container_conditions: SmallVec<[ContainerConditionReference; 1]>,

    /// Effective media query results cached from the last rebuild.
    effective_media_query_results: EffectiveMediaQueryResults,

    /// Extra data, like different kinds of rules, etc.
    extra_data: ExtraStyleData,

    /// A monotonically increasing counter to represent the order on which a
    /// style rule appears in a stylesheet, needed to sort them by source order.
    rules_source_order: u32,

    /// The total number of selectors.
    num_selectors: usize,

    /// The total number of declarations.
    num_declarations: usize,
}

impl CascadeData {
    /// Creates an empty `CascadeData`.
    pub fn new() -> Self {
        Self {
            normal_rules: ElementAndPseudoRules::default(),
            host_rules: None,
            slotted_rules: None,
            part_rules: None,
            invalidation_map: InvalidationMap::new(),
            nth_of_mapped_ids: PrecomputedHashSet::default(),
            nth_of_class_dependencies: PrecomputedHashSet::default(),
            nth_of_attribute_dependencies: PrecomputedHashSet::default(),
            nth_of_state_dependencies: ElementState::empty(),
            attribute_dependencies: PrecomputedHashSet::default(),
            state_dependencies: ElementState::empty(),
            document_state_dependencies: DocumentState::empty(),
            mapped_ids: PrecomputedHashSet::default(),
            // NOTE: We disable attribute bucketing for revalidation because we
            // rely on the buckets to match, but we don't want to just not share
            // style across elements with different attributes.
            //
            // An alternative to this would be to perform a style sharing check
            // like may_match_different_id_rules which would check that the
            // attribute buckets match on all scopes. But that seems
            // somewhat gnarly.
            selectors_for_cache_revalidation: SelectorMap::new_without_attribute_bucketing(),
            animations: Default::default(),
            layer_id: Default::default(),
            layers: smallvec::smallvec![CascadeLayer::root()],
            container_conditions: smallvec::smallvec![ContainerConditionReference::none()],
            extra_data: ExtraStyleData::default(),
            effective_media_query_results: EffectiveMediaQueryResults::new(),
            rules_source_order: 0,
            num_selectors: 0,
            num_declarations: 0,
        }
    }

    /// Rebuild the cascade data from a given SheetCollection, incrementally if
    /// possible.
    pub fn rebuild<'a, S>(
        &mut self,
        device: &Device,
        quirks_mode: QuirksMode,
        collection: SheetCollectionFlusher<S>,
        guard: &SharedRwLockReadGuard,
    ) -> Result<(), AllocErr>
    where
        S: StylesheetInDocument + PartialEq + 'static,
    {
        if !collection.dirty() {
            return Ok(());
        }

        let validity = collection.data_validity();

        match validity {
            DataValidity::Valid => {},
            DataValidity::CascadeInvalid => self.clear_cascade_data(),
            DataValidity::FullyInvalid => self.clear(),
        }

        let mut result = Ok(());

        collection.each(|stylesheet, rebuild_kind| {
            result = self.add_stylesheet(
                device,
                quirks_mode,
                stylesheet,
                guard,
                rebuild_kind,
                /* precomputed_pseudo_element_decls = */ None,
            );
            result.is_ok()
        });

        self.did_finish_rebuild();

        result
    }

    /// Returns the invalidation map.
    pub fn invalidation_map(&self) -> &InvalidationMap {
        &self.invalidation_map
    }

    /// Returns whether the given ElementState bit is relied upon by a selector
    /// of some rule.
    #[inline]
    pub fn has_state_dependency(&self, state: ElementState) -> bool {
        self.state_dependencies.intersects(state)
    }

    /// Returns whether the given ElementState bit is relied upon by a selector
    /// of some rule in the selector list of :nth-child(... of <selector list>).
    #[inline]
    pub fn has_nth_of_state_dependency(&self, state: ElementState) -> bool {
        self.nth_of_state_dependencies.intersects(state)
    }

    /// Returns whether the given attribute might appear in an attribute
    /// selector of some rule.
    #[inline]
    pub fn might_have_attribute_dependency(&self, local_name: &LocalName) -> bool {
        self.attribute_dependencies.contains(local_name)
    }

    /// Returns whether the given ID might appear in an ID selector in the
    /// selector list of :nth-child(... of <selector list>).
    #[inline]
    pub fn might_have_nth_of_id_dependency(&self, id: &Atom) -> bool {
        self.nth_of_mapped_ids.contains(id)
    }

    /// Returns whether the given class might appear in a class selector in the
    /// selector list of :nth-child(... of <selector list>).
    #[inline]
    pub fn might_have_nth_of_class_dependency(&self, class: &Atom) -> bool {
        self.nth_of_class_dependencies.contains(class)
    }

    /// Returns whether the given attribute might appear in an attribute
    /// selector in the selector list of :nth-child(... of <selector list>).
    #[inline]
    pub fn might_have_nth_of_attribute_dependency(&self, local_name: &LocalName) -> bool {
        self.nth_of_attribute_dependencies.contains(local_name)
    }

    /// Returns the normal rule map for a given pseudo-element.
    #[inline]
    pub fn normal_rules(&self, pseudo: Option<&PseudoElement>) -> Option<&SelectorMap<Rule>> {
        self.normal_rules.rules(pseudo)
    }

    /// Returns the host pseudo rule map for a given pseudo-element.
    #[inline]
    pub fn host_rules(&self, pseudo: Option<&PseudoElement>) -> Option<&SelectorMap<Rule>> {
        self.host_rules.as_ref().and_then(|d| d.rules(pseudo))
    }

    /// Whether there's any host rule that could match in this scope.
    pub fn any_host_rules(&self) -> bool {
        self.host_rules.is_some()
    }

    /// Returns the slotted rule map for a given pseudo-element.
    #[inline]
    pub fn slotted_rules(&self, pseudo: Option<&PseudoElement>) -> Option<&SelectorMap<Rule>> {
        self.slotted_rules.as_ref().and_then(|d| d.rules(pseudo))
    }

    /// Whether there's any ::slotted rule that could match in this scope.
    pub fn any_slotted_rule(&self) -> bool {
        self.slotted_rules.is_some()
    }

    /// Returns the parts rule map for a given pseudo-element.
    #[inline]
    pub fn part_rules(&self, pseudo: Option<&PseudoElement>) -> Option<&PartMap> {
        self.part_rules.as_ref().and_then(|d| d.rules(pseudo))
    }

    /// Whether there's any ::part rule that could match in this scope.
    pub fn any_part_rule(&self) -> bool {
        self.part_rules.is_some()
    }

    #[inline]
    fn layer_order_for(&self, id: LayerId) -> LayerOrder {
        self.layers[id.0 as usize].order
    }

    pub(crate) fn container_condition_matches<E>(
        &self,
        mut id: ContainerConditionId,
        stylist: &Stylist,
        element: E,
        context: &mut MatchingContext<E::Impl>,
    ) -> bool
    where
        E: TElement,
    {
        loop {
            let condition_ref = &self.container_conditions[id.0 as usize];
            let condition = match condition_ref.condition {
                None => return true,
                Some(ref c) => c,
            };
            let matches = condition
                .matches(
                    stylist.device(),
                    element,
                    context.extra_data.originating_element_style,
                    &mut context.extra_data.cascade_input_flags,
                )
                .to_bool(/* unknown = */ false);
            if !matches {
                return false;
            }
            id = condition_ref.parent;
        }
    }

    fn did_finish_rebuild(&mut self) {
        self.shrink_maps_if_needed();
        self.compute_layer_order();
    }

    fn shrink_maps_if_needed(&mut self) {
        self.normal_rules.shrink_if_needed();
        if let Some(ref mut host_rules) = self.host_rules {
            host_rules.shrink_if_needed();
        }
        if let Some(ref mut slotted_rules) = self.slotted_rules {
            slotted_rules.shrink_if_needed();
        }
        self.invalidation_map.shrink_if_needed();
        self.attribute_dependencies.shrink_if_needed();
        self.nth_of_attribute_dependencies.shrink_if_needed();
        self.nth_of_class_dependencies.shrink_if_needed();
        self.nth_of_mapped_ids.shrink_if_needed();
        self.mapped_ids.shrink_if_needed();
        self.layer_id.shrink_if_needed();
        self.selectors_for_cache_revalidation.shrink_if_needed();
    }

    fn compute_layer_order(&mut self) {
        debug_assert_ne!(
            self.layers.len(),
            0,
            "There should be at least the root layer!"
        );
        if self.layers.len() == 1 {
            return; // Nothing to do
        }
        let (first, remaining) = self.layers.split_at_mut(1);
        let root = &mut first[0];
        let mut order = LayerOrder::first();
        compute_layer_order_for_subtree(root, remaining, &mut order);

        // NOTE(emilio): This is a bit trickier than it should to avoid having
        // to clone() around layer indices.
        fn compute_layer_order_for_subtree(
            parent: &mut CascadeLayer,
            remaining_layers: &mut [CascadeLayer],
            order: &mut LayerOrder,
        ) {
            for child in parent.children.iter() {
                debug_assert!(
                    parent.id < *child,
                    "Children are always registered after parents"
                );
                let child_index = (child.0 - parent.id.0 - 1) as usize;
                let (first, remaining) = remaining_layers.split_at_mut(child_index + 1);
                let child = &mut first[child_index];
                compute_layer_order_for_subtree(child, remaining, order);
            }

            if parent.id != LayerId::root() {
                parent.order = *order;
                order.inc();
            }
        }
        #[cfg(feature = "gecko")]
        {
            self.extra_data.sort_by_layer(&self.layers);
        }
        self.animations
            .sort_with(&self.layers, compare_keyframes_in_same_layer);
    }

    /// Collects all the applicable media query results into `results`.
    ///
    /// This duplicates part of the logic in `add_stylesheet`, which is
    /// a bit unfortunate.
    ///
    /// FIXME(emilio): With a bit of smartness in
    /// `media_feature_affected_matches`, we could convert
    /// `EffectiveMediaQueryResults` into a vector without too much effort.
    fn collect_applicable_media_query_results_into<S>(
        device: &Device,
        stylesheet: &S,
        guard: &SharedRwLockReadGuard,
        results: &mut Vec<MediaListKey>,
        contents_list: &mut StyleSheetContentList,
    ) where
        S: StylesheetInDocument + 'static,
    {
        if !stylesheet.enabled() || !stylesheet.is_effective_for_device(device, guard) {
            return;
        }

        debug!(" + {:?}", stylesheet);
        let contents = stylesheet.contents();
        results.push(contents.to_media_list_key());

        // Safety: StyleSheetContents are reference-counted with Arc.
        contents_list.push(StylesheetContentsPtr(unsafe {
            Arc::from_raw_addrefed(contents)
        }));

        for rule in stylesheet.effective_rules(device, guard) {
            match *rule {
                CssRule::Import(ref lock) => {
                    let import_rule = lock.read_with(guard);
                    debug!(" + {:?}", import_rule.stylesheet.media(guard));
                    results.push(import_rule.to_media_list_key());
                },
                CssRule::Media(ref media_rule) => {
                    debug!(" + {:?}", media_rule.media_queries.read_with(guard));
                    results.push(media_rule.to_media_list_key());
                },
                _ => {},
            }
        }
    }

    fn add_rule_list<'a, S>(
        &mut self,
        rules: std::slice::Iter<'a, CssRule>,
        device: &'a Device,
        quirks_mode: QuirksMode,
        stylesheet: &S,
        guard: &'a SharedRwLockReadGuard,
        rebuild_kind: SheetRebuildKind,
        containing_rule_state: &mut ContainingRuleState<'a>,
        mut precomputed_pseudo_element_decls: Option<&mut PrecomputedPseudoElementDeclarations>,
    ) -> Result<(), AllocErr>
    where
        S: StylesheetInDocument + 'static,
    {
        for rule in rules {
            // Handle leaf rules first, as those are by far the most common
            // ones, and are always effective, so we can skip some checks.
            let mut handled = true;
            let mut selectors_for_nested_rules = None;
            match *rule {
                CssRule::Style(ref locked) => {
                    let style_rule = locked.read_with(guard);
                    self.num_declarations += style_rule.block.read_with(&guard).len();

                    let has_nested_rules = style_rule.rules.is_some();
                    let ancestor_selectors = containing_rule_state.ancestor_selector_lists.last();
                    if has_nested_rules {
                        selectors_for_nested_rules = Some(if ancestor_selectors.is_some() {
                            Cow::Owned(SelectorList(Default::default()))
                        } else {
                            Cow::Borrowed(&style_rule.selectors)
                        });
                    }

                    for selector in &style_rule.selectors.0 {
                        self.num_selectors += 1;

                        let pseudo_element = selector.pseudo_element();
                        if let Some(pseudo) = pseudo_element {
                            if pseudo.is_precomputed() {
                                debug_assert!(selector.is_universal());
                                debug_assert!(ancestor_selectors.is_none());
                                debug_assert!(!has_nested_rules);
                                debug_assert_eq!(stylesheet.contents().origin, Origin::UserAgent);
                                debug_assert_eq!(containing_rule_state.layer_id, LayerId::root());

                                precomputed_pseudo_element_decls
                                    .as_mut()
                                    .expect("Expected precomputed declarations for the UA level")
                                    .get_or_insert_with(pseudo, Vec::new)
                                    .push(ApplicableDeclarationBlock::new(
                                        StyleSource::from_rule(locked.clone()),
                                        self.rules_source_order,
                                        CascadeLevel::UANormal,
                                        selector.specificity(),
                                        LayerOrder::root(),
                                    ));
                                continue;
                            }
                            if pseudo.is_unknown_webkit_pseudo_element() {
                                continue;
                            }
                        }

                        let selector = match ancestor_selectors {
                            Some(s) => selector.replace_parent_selector(&s.0),
                            None => selector.clone(),
                        };

                        let hashes = AncestorHashes::new(&selector, quirks_mode);

                        let rule = Rule::new(
                            selector,
                            hashes,
                            locked.clone(),
                            self.rules_source_order,
                            containing_rule_state.layer_id,
                            containing_rule_state.container_condition_id,
                        );

                        if let Some(Cow::Owned(ref mut nested_selectors)) =
                            selectors_for_nested_rules
                        {
                            nested_selectors.0.push(rule.selector.clone())
                        }

                        if rebuild_kind.should_rebuild_invalidation() {
                            self.invalidation_map
                                .note_selector(&rule.selector, quirks_mode)?;
                            let mut needs_revalidation = false;
                            let mut visitor = StylistSelectorVisitor {
                                needs_revalidation: &mut needs_revalidation,
                                passed_rightmost_selector: false,
                                in_selector_list_of: SelectorListKind::default(),
                                mapped_ids: &mut self.mapped_ids,
                                nth_of_mapped_ids: &mut self.nth_of_mapped_ids,
                                attribute_dependencies: &mut self.attribute_dependencies,
                                nth_of_class_dependencies: &mut self.nth_of_class_dependencies,
                                nth_of_attribute_dependencies: &mut self
                                    .nth_of_attribute_dependencies,
                                state_dependencies: &mut self.state_dependencies,
                                nth_of_state_dependencies: &mut self.nth_of_state_dependencies,
                                document_state_dependencies: &mut self.document_state_dependencies,
                            };
                            rule.selector.visit(&mut visitor);

                            if needs_revalidation {
                                self.selectors_for_cache_revalidation.insert(
                                    RevalidationSelectorAndHashes::new(
                                        rule.selector.clone(),
                                        rule.hashes.clone(),
                                    ),
                                    quirks_mode,
                                )?;
                            }
                        }

                        // Part is special, since given it doesn't have any
                        // selectors inside, it's not worth using a whole
                        // SelectorMap for it.
                        if let Some(parts) = rule.selector.parts() {
                            // ::part() has all semantics, so we just need to
                            // put any of them in the selector map.
                            //
                            // We choose the last one quite arbitrarily,
                            // expecting it's slightly more likely to be more
                            // specific.
                            let map = self
                                .part_rules
                                .get_or_insert_with(|| Box::new(Default::default()))
                                .for_insertion(pseudo_element);
                            map.try_reserve(1)?;
                            let vec = map.entry(parts.last().unwrap().clone().0).or_default();
                            vec.try_reserve(1)?;
                            vec.push(rule);
                        } else {
                            // NOTE(emilio): It's fine to look at :host and then at
                            // ::slotted(..), since :host::slotted(..) could never
                            // possibly match, as <slot> is not a valid shadow host.
                            let rules = if rule
                                .selector
                                .is_featureless_host_selector_or_pseudo_element()
                            {
                                self.host_rules
                                    .get_or_insert_with(|| Box::new(Default::default()))
                            } else if rule.selector.is_slotted() {
                                self.slotted_rules
                                    .get_or_insert_with(|| Box::new(Default::default()))
                            } else {
                                &mut self.normal_rules
                            }
                            .for_insertion(pseudo_element);
                            rules.insert(rule, quirks_mode)?;
                        }
                    }
                    self.rules_source_order += 1;
                    handled = !has_nested_rules;
                },
                CssRule::Keyframes(ref keyframes_rule) => {
                    debug!("Found valid keyframes rule: {:?}", *keyframes_rule);
                    let keyframes_rule = keyframes_rule.read_with(guard);
                    let name = keyframes_rule.name.as_atom().clone();
                    let animation = KeyframesAnimation::from_keyframes(
                        &keyframes_rule.keyframes,
                        keyframes_rule.vendor_prefix.clone(),
                        guard,
                    );
                    self.animations.try_insert_with(
                        name,
                        animation,
                        containing_rule_state.layer_id,
                        compare_keyframes_in_same_layer,
                    )?;
                },
                #[cfg(feature = "gecko")]
                CssRule::FontFace(ref rule) => {
                    // NOTE(emilio): We don't care about container_condition_id
                    // because:
                    //
                    //     Global, name-defining at-rules such as @keyframes or
                    //     @font-face or @layer that are defined inside container
                    //     queries are not constrained by the container query
                    //     conditions.
                    //
                    // https://drafts.csswg.org/css-contain-3/#container-rule
                    // (Same elsewhere)
                    self.extra_data
                        .add_font_face(rule, containing_rule_state.layer_id);
                },
                #[cfg(feature = "gecko")]
                CssRule::FontFeatureValues(ref rule) => {
                    self.extra_data
                        .add_font_feature_values(rule, containing_rule_state.layer_id);
                },
                #[cfg(feature = "gecko")]
                CssRule::FontPaletteValues(ref rule) => {
                    self.extra_data
                        .add_font_palette_values(rule, containing_rule_state.layer_id);
                },
                #[cfg(feature = "gecko")]
                CssRule::CounterStyle(ref rule) => {
                    self.extra_data.add_counter_style(
                        guard,
                        rule,
                        containing_rule_state.layer_id,
                    )?;
                },
                #[cfg(feature = "gecko")]
                CssRule::Page(ref rule) => {
                    self.extra_data
                        .add_page(guard, rule, containing_rule_state.layer_id)?;
                },
                _ => {
                    handled = false;
                },
            }

            if handled {
                // Assert that there are no children, and that the rule is
                // effective.
                if cfg!(debug_assertions) {
                    let mut effective = false;
                    let children = EffectiveRulesIterator::children(
                        rule,
                        device,
                        quirks_mode,
                        guard,
                        &mut effective,
                    );
                    debug_assert!(children.is_none());
                    debug_assert!(effective);
                }
                continue;
            }

            let mut effective = false;
            let children =
                EffectiveRulesIterator::children(rule, device, quirks_mode, guard, &mut effective);

            if !effective {
                continue;
            }

            fn maybe_register_layer(data: &mut CascadeData, layer: &LayerName) -> LayerId {
                // TODO: Measure what's more common / expensive, if
                // layer.clone() or the double hash lookup in the insert
                // case.
                if let Some(id) = data.layer_id.get(layer) {
                    return *id;
                }
                let id = LayerId(data.layers.len() as u16);

                let parent_layer_id = if layer.layer_names().len() > 1 {
                    let mut parent = layer.clone();
                    parent.0.pop();

                    *data
                        .layer_id
                        .get_mut(&parent)
                        .expect("Parent layers should be registered before child layers")
                } else {
                    LayerId::root()
                };

                data.layers[parent_layer_id.0 as usize].children.push(id);
                data.layers.push(CascadeLayer {
                    id,
                    // NOTE(emilio): Order is evaluated after rebuild in
                    // compute_layer_order.
                    order: LayerOrder::first(),
                    children: vec![],
                });

                data.layer_id.insert(layer.clone(), id);

                id
            }

            fn maybe_register_layers(
                data: &mut CascadeData,
                name: Option<&LayerName>,
                containing_rule_state: &mut ContainingRuleState,
            ) {
                let anon_name;
                let name = match name {
                    Some(name) => name,
                    None => {
                        anon_name = LayerName::new_anonymous();
                        &anon_name
                    },
                };
                for name in name.layer_names() {
                    containing_rule_state.layer_name.0.push(name.clone());
                    containing_rule_state.layer_id =
                        maybe_register_layer(data, &containing_rule_state.layer_name);
                }
                debug_assert_ne!(containing_rule_state.layer_id, LayerId::root());
            }

            let saved_containing_rule_state = containing_rule_state.save();
            match *rule {
                CssRule::Import(ref lock) => {
                    let import_rule = lock.read_with(guard);
                    if rebuild_kind.should_rebuild_invalidation() {
                        self.effective_media_query_results
                            .saw_effective(import_rule);
                    }
                    match import_rule.layer {
                        ImportLayer::Named(ref name) => {
                            maybe_register_layers(self, Some(name), containing_rule_state)
                        },
                        ImportLayer::Anonymous => {
                            maybe_register_layers(self, None, containing_rule_state)
                        },
                        ImportLayer::None => {},
                    }
                },
                CssRule::Media(ref media_rule) => {
                    if rebuild_kind.should_rebuild_invalidation() {
                        self.effective_media_query_results
                            .saw_effective(&**media_rule);
                    }
                },
                CssRule::LayerBlock(ref rule) => {
                    maybe_register_layers(self, rule.name.as_ref(), containing_rule_state);
                },
                CssRule::LayerStatement(ref rule) => {
                    for name in &*rule.names {
                        maybe_register_layers(self, Some(name), containing_rule_state);
                        // Register each layer individually.
                        containing_rule_state.restore(&saved_containing_rule_state);
                    }
                },
                CssRule::Style(..) => {
                    if let Some(s) = selectors_for_nested_rules {
                        containing_rule_state.ancestor_selector_lists.push(s);
                    }
                },
                CssRule::Container(ref rule) => {
                    let id = ContainerConditionId(self.container_conditions.len() as u16);
                    self.container_conditions.push(ContainerConditionReference {
                        parent: containing_rule_state.container_condition_id,
                        condition: Some(rule.condition.clone()),
                    });
                    containing_rule_state.container_condition_id = id;
                },
                // We don't care about any other rule.
                _ => {},
            }

            if let Some(children) = children {
                self.add_rule_list(
                    children,
                    device,
                    quirks_mode,
                    stylesheet,
                    guard,
                    rebuild_kind,
                    containing_rule_state,
                    precomputed_pseudo_element_decls.as_deref_mut(),
                )?;
            }

            containing_rule_state.restore(&saved_containing_rule_state);
        }

        Ok(())
    }

    // Returns Err(..) to signify OOM
    fn add_stylesheet<S>(
        &mut self,
        device: &Device,
        quirks_mode: QuirksMode,
        stylesheet: &S,
        guard: &SharedRwLockReadGuard,
        rebuild_kind: SheetRebuildKind,
        mut precomputed_pseudo_element_decls: Option<&mut PrecomputedPseudoElementDeclarations>,
    ) -> Result<(), AllocErr>
    where
        S: StylesheetInDocument + 'static,
    {
        if !stylesheet.enabled() || !stylesheet.is_effective_for_device(device, guard) {
            return Ok(());
        }

        let contents = stylesheet.contents();

        if rebuild_kind.should_rebuild_invalidation() {
            self.effective_media_query_results.saw_effective(contents);
        }

        let mut state = ContainingRuleState::default();
        self.add_rule_list(
            contents.rules(guard).iter(),
            device,
            quirks_mode,
            stylesheet,
            guard,
            rebuild_kind,
            &mut state,
            precomputed_pseudo_element_decls.as_deref_mut(),
        )?;

        Ok(())
    }

    /// Returns whether all the media-feature affected values matched before and
    /// match now in the given stylesheet.
    pub fn media_feature_affected_matches<S>(
        &self,
        stylesheet: &S,
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
    ) -> bool
    where
        S: StylesheetInDocument + 'static,
    {
        use crate::invalidation::media_queries::PotentiallyEffectiveMediaRules;

        let effective_now = stylesheet.is_effective_for_device(device, guard);

        let effective_then = self
            .effective_media_query_results
            .was_effective(stylesheet.contents());

        if effective_now != effective_then {
            debug!(
                " > Stylesheet {:?} changed -> {}, {}",
                stylesheet.media(guard),
                effective_then,
                effective_now
            );
            return false;
        }

        if !effective_now {
            return true;
        }

        let mut iter = stylesheet.iter_rules::<PotentiallyEffectiveMediaRules>(device, guard);

        while let Some(rule) = iter.next() {
            match *rule {
                CssRule::Style(..) |
                CssRule::Namespace(..) |
                CssRule::FontFace(..) |
                CssRule::Container(..) |
                CssRule::CounterStyle(..) |
                CssRule::Supports(..) |
                CssRule::Keyframes(..) |
                CssRule::Page(..) |
                CssRule::Property(..) |
                CssRule::Document(..) |
                CssRule::LayerBlock(..) |
                CssRule::LayerStatement(..) |
                CssRule::FontPaletteValues(..) |
                CssRule::FontFeatureValues(..) => {
                    // Not affected by device changes.
                    continue;
                },
                CssRule::Import(ref lock) => {
                    let import_rule = lock.read_with(guard);
                    let effective_now = match import_rule.stylesheet.media(guard) {
                        Some(m) => m.evaluate(device, quirks_mode),
                        None => true,
                    };
                    let effective_then = self
                        .effective_media_query_results
                        .was_effective(import_rule);
                    if effective_now != effective_then {
                        debug!(
                            " > @import rule {:?} changed {} -> {}",
                            import_rule.stylesheet.media(guard),
                            effective_then,
                            effective_now
                        );
                        return false;
                    }

                    if !effective_now {
                        iter.skip_children();
                    }
                },
                CssRule::Media(ref media_rule) => {
                    let mq = media_rule.media_queries.read_with(guard);
                    let effective_now = mq.evaluate(device, quirks_mode);
                    let effective_then = self
                        .effective_media_query_results
                        .was_effective(&**media_rule);

                    if effective_now != effective_then {
                        debug!(
                            " > @media rule {:?} changed {} -> {}",
                            mq, effective_then, effective_now
                        );
                        return false;
                    }

                    if !effective_now {
                        iter.skip_children();
                    }
                },
            }
        }

        true
    }

    /// Clears the cascade data, but not the invalidation data.
    fn clear_cascade_data(&mut self) {
        self.normal_rules.clear();
        if let Some(ref mut slotted_rules) = self.slotted_rules {
            slotted_rules.clear();
        }
        if let Some(ref mut part_rules) = self.part_rules {
            part_rules.clear();
        }
        if let Some(ref mut host_rules) = self.host_rules {
            host_rules.clear();
        }
        self.animations.clear();
        self.layer_id.clear();
        self.layers.clear();
        self.layers.push(CascadeLayer::root());
        self.container_conditions.clear();
        self.container_conditions
            .push(ContainerConditionReference::none());
        #[cfg(feature = "gecko")]
        {
            self.extra_data.clear();
        }
        self.rules_source_order = 0;
        self.num_selectors = 0;
        self.num_declarations = 0;
    }

    fn clear(&mut self) {
        self.clear_cascade_data();
        self.invalidation_map.clear();
        self.attribute_dependencies.clear();
        self.nth_of_attribute_dependencies.clear();
        self.nth_of_class_dependencies.clear();
        self.state_dependencies = ElementState::empty();
        self.nth_of_state_dependencies = ElementState::empty();
        self.document_state_dependencies = DocumentState::empty();
        self.mapped_ids.clear();
        self.nth_of_mapped_ids.clear();
        self.selectors_for_cache_revalidation.clear();
        self.effective_media_query_results.clear();
    }
}

impl CascadeDataCacheEntry for CascadeData {
    fn cascade_data(&self) -> &CascadeData {
        self
    }

    fn rebuild<S>(
        device: &Device,
        quirks_mode: QuirksMode,
        collection: SheetCollectionFlusher<S>,
        guard: &SharedRwLockReadGuard,
        old: &Self,
    ) -> Result<Arc<Self>, AllocErr>
    where
        S: StylesheetInDocument + PartialEq + 'static,
    {
        debug_assert!(collection.dirty(), "We surely need to do something?");
        // If we're doing a full rebuild anyways, don't bother cloning the data.
        let mut updatable_entry = match collection.data_validity() {
            DataValidity::Valid | DataValidity::CascadeInvalid => old.clone(),
            DataValidity::FullyInvalid => Self::new(),
        };
        updatable_entry.rebuild(device, quirks_mode, collection, guard)?;
        Ok(Arc::new(updatable_entry))
    }

    #[cfg(feature = "gecko")]
    fn add_size_of(&self, ops: &mut MallocSizeOfOps, sizes: &mut ServoStyleSetSizes) {
        self.normal_rules.add_size_of(ops, sizes);
        if let Some(ref slotted_rules) = self.slotted_rules {
            slotted_rules.add_size_of(ops, sizes);
        }
        if let Some(ref part_rules) = self.part_rules {
            part_rules.add_size_of(ops, sizes);
        }
        if let Some(ref host_rules) = self.host_rules {
            host_rules.add_size_of(ops, sizes);
        }
        sizes.mInvalidationMap += self.invalidation_map.size_of(ops);
        sizes.mRevalidationSelectors += self.selectors_for_cache_revalidation.size_of(ops);
        sizes.mOther += self.animations.size_of(ops);
        sizes.mOther += self.effective_media_query_results.size_of(ops);
        sizes.mOther += self.extra_data.size_of(ops);
    }
}

impl Default for CascadeData {
    fn default() -> Self {
        CascadeData::new()
    }
}

/// A rule, that wraps a style rule, but represents a single selector of the
/// rule.
#[derive(Clone, Debug, MallocSizeOf)]
pub struct Rule {
    /// The selector this struct represents. We store this and the
    /// any_{important,normal} booleans inline in the Rule to avoid
    /// pointer-chasing when gathering applicable declarations, which
    /// can ruin performance when there are a lot of rules.
    #[ignore_malloc_size_of = "CssRules have primary refs, we measure there"]
    pub selector: Selector<SelectorImpl>,

    /// The ancestor hashes associated with the selector.
    pub hashes: AncestorHashes,

    /// The source order this style rule appears in. Note that we only use
    /// three bytes to store this value in ApplicableDeclarationsBlock, so
    /// we could repurpose that storage here if we needed to.
    pub source_order: u32,

    /// The current layer id of this style rule.
    pub layer_id: LayerId,

    /// The current @container rule id.
    pub container_condition_id: ContainerConditionId,

    /// The actual style rule.
    #[cfg_attr(
        feature = "gecko",
        ignore_malloc_size_of = "Secondary ref. Primary ref is in StyleRule under Stylesheet."
    )]
    #[cfg_attr(feature = "servo", ignore_malloc_size_of = "Arc")]
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
        level: CascadeLevel,
        cascade_data: &CascadeData,
    ) -> ApplicableDeclarationBlock {
        let source = StyleSource::from_rule(self.style_rule.clone());
        ApplicableDeclarationBlock::new(
            source,
            self.source_order,
            level,
            self.specificity(),
            cascade_data.layer_order_for(self.layer_id),
        )
    }

    /// Creates a new Rule.
    pub fn new(
        selector: Selector<SelectorImpl>,
        hashes: AncestorHashes,
        style_rule: Arc<Locked<StyleRule>>,
        source_order: u32,
        layer_id: LayerId,
        container_condition_id: ContainerConditionId,
    ) -> Self {
        Rule {
            selector,
            hashes,
            style_rule,
            source_order,
            layer_id,
            container_condition_id,
        }
    }
}

// The size of this is critical to performance on the bloom-basic
// microbenchmark.
// When iterating over a large Rule array, we want to be able to fast-reject
// selectors (with the inline hashes) with as few cache misses as possible.
size_of_test!(Rule, 40);

/// A function to be able to test the revalidation stuff.
pub fn needs_revalidation_for_testing(s: &Selector<SelectorImpl>) -> bool {
    let mut needs_revalidation = false;
    let mut mapped_ids = Default::default();
    let mut nth_of_mapped_ids = Default::default();
    let mut attribute_dependencies = Default::default();
    let mut nth_of_class_dependencies = Default::default();
    let mut nth_of_attribute_dependencies = Default::default();
    let mut state_dependencies = ElementState::empty();
    let mut nth_of_state_dependencies = ElementState::empty();
    let mut document_state_dependencies = DocumentState::empty();
    let mut visitor = StylistSelectorVisitor {
        passed_rightmost_selector: false,
        needs_revalidation: &mut needs_revalidation,
        in_selector_list_of: SelectorListKind::default(),
        mapped_ids: &mut mapped_ids,
        nth_of_mapped_ids: &mut nth_of_mapped_ids,
        attribute_dependencies: &mut attribute_dependencies,
        nth_of_class_dependencies: &mut nth_of_class_dependencies,
        nth_of_attribute_dependencies: &mut nth_of_attribute_dependencies,
        state_dependencies: &mut state_dependencies,
        nth_of_state_dependencies: &mut nth_of_state_dependencies,
        document_state_dependencies: &mut document_state_dependencies,
    };
    s.visit(&mut visitor);
    needs_revalidation
}
