/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Code related to the style sharing cache, an optimization that allows similar
//! nodes to share style without having to run selector matching twice.
//!
//! The basic setup is as follows.  We have an LRU cache of style sharing
//! candidates.  When we try to style a target element, we first check whether
//! we can quickly determine that styles match something in this cache, and if
//! so we just use the cached style information.  This check is done with a
//! StyleBloom filter set up for the target element, which may not be a correct
//! state for the cached candidate element if they're cousins instead of
//! siblings.
//!
//! The complicated part is determining that styles match.  This is subject to
//! the following constraints:
//!
//! 1) The target and candidate must be inheriting the same styles.
//! 2) The target and candidate must have exactly the same rules matching them.
//! 3) The target and candidate must have exactly the same non-selector-based
//!    style information (inline styles, presentation hints).
//! 4) The target and candidate must have exactly the same rules matching their
//!    pseudo-elements, because an element's style data points to the style
//!    data for its pseudo-elements.
//!
//! These constraints are satisfied in the following ways:
//!
//! * We check that the parents of the target and the candidate have the same
//!   computed style.  This addresses constraint 1.
//!
//! * We check that the target and candidate have the same inline style and
//!   presentation hint declarations.  This addresses constraint 3.
//!
//! * We ensure that a target matches a candidate only if they have the same
//!   matching result for all selectors that target either elements or the
//!   originating elements of pseudo-elements.  This addresses constraint 4
//!   (because it prevents a target that has pseudo-element styles from matching
//!   a candidate that has different pseudo-element styles) as well as
//!   constraint 2.
//!
//! The actual checks that ensure that elements match the same rules are
//! conceptually split up into two pieces.  First, we do various checks on
//! elements that make sure that the set of possible rules in all selector maps
//! in the stylist (for normal styling and for pseudo-elements) that might match
//! the two elements is the same.  For example, we enforce that the target and
//! candidate must have the same localname and namespace.  Second, we have a
//! selector map of "revalidation selectors" that the stylist maintains that we
//! actually match against the target and candidate and then check whether the
//! two sets of results were the same.  Due to the up-front selector map checks,
//! we know that the target and candidate will be matched against the same exact
//! set of revalidation selectors, so the match result arrays can be compared
//! directly.
//!
//! It's very important that a selector be added to the set of revalidation
//! selectors any time there are two elements that could pass all the up-front
//! checks but match differently against some ComplexSelector in the selector.
//! If that happens, then they can have descendants that might themselves pass
//! the up-front checks but would have different matching results for the
//! selector in question.  In this case, "descendants" includes pseudo-elements,
//! so there is a single selector map of revalidation selectors that includes
//! both selectors targeting elements and selectors targeting pseudo-element
//! originating elements.  We ensure that the pseudo-element parts of all these
//! selectors are effectively stripped off, so that matching them all against
//! elements makes sense.

use Atom;
use applicable_declarations::ApplicableDeclarationBlock;
use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use bloom::StyleBloom;
use cache::LRUCache;
use context::{SelectorFlagsMap, SharedStyleContext, StyleContext};
use data::ElementStyles;
use dom::{TElement, SendElement};
use matching::MatchMethods;
use owning_ref::OwningHandle;
use properties::ComputedValues;
use selectors::matching::{ElementSelectorFlags, VisitedHandlingMode};
use servo_arc::Arc;
use smallbitvec::SmallBitVec;
use smallvec::SmallVec;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use stylist::Stylist;

mod checks;

/// The amount of nodes that the style sharing candidate cache should hold at
/// most.  We'd somewhat like 32, but ArrayDeque only implements certain backing
/// store sizes.  A cache size of 32 would mean a backing store of 33, but
/// that's not an implemented size: we can do 32 or 40.
///
/// The cache size was chosen by measuring style sharing and resulting
/// performance on a few pages; sizes up to about 32 were giving good sharing
/// improvements (e.g. 3x fewer styles having to be resolved than at size 8) and
/// slight performance improvements.  Sizes larger than 32 haven't really been
/// tested.
pub const SHARING_CACHE_SIZE: usize = 31;
const SHARING_CACHE_BACKING_STORE_SIZE: usize = SHARING_CACHE_SIZE + 1;

/// Controls whether the style sharing cache is used.
#[derive(Clone, Copy, PartialEq)]
pub enum StyleSharingBehavior {
    /// Style sharing allowed.
    Allow,
    /// Style sharing disallowed.
    Disallow,
}

/// Some data we want to avoid recomputing all the time while trying to share
/// style.
#[derive(Debug, Default)]
pub struct ValidationData {
    /// The class list of this element.
    ///
    /// TODO(emilio): See if it's worth to sort them, or doing something else in
    /// a similar fashion as what Boris is doing for the ID attribute.
    class_list: Option<SmallVec<[Atom; 5]>>,

    /// The list of presentational attributes of the element.
    pres_hints: Option<SmallVec<[ApplicableDeclarationBlock; 5]>>,

    /// The cached result of matching this entry against the revalidation
    /// selectors.
    revalidation_match_results: Option<SmallBitVec>,
}

impl ValidationData {
    /// Move the cached data to a new instance, and return it.
    pub fn take(&mut self) -> Self {
        mem::replace(self, Self::default())
    }

    /// Get or compute the list of presentational attributes associated with
    /// this element.
    pub fn pres_hints<E>(&mut self, element: E) -> &[ApplicableDeclarationBlock]
        where E: TElement,
    {
        if self.pres_hints.is_none() {
            let mut pres_hints = SmallVec::new();
            element.synthesize_presentational_hints_for_legacy_attributes(
                VisitedHandlingMode::AllLinksUnvisited,
                &mut pres_hints
            );
            self.pres_hints = Some(pres_hints);
        }
        &*self.pres_hints.as_ref().unwrap()
    }

    /// Get or compute the class-list associated with this element.
    pub fn class_list<E>(&mut self, element: E) -> &[Atom]
        where E: TElement,
    {
        if self.class_list.is_none() {
            let mut class_list = SmallVec::<[Atom; 5]>::new();
            element.each_class(|c| class_list.push(c.clone()));
            // Assuming there are a reasonable number of classes (we use the
            // inline capacity as "reasonable number"), sort them to so that
            // we don't mistakenly reject sharing candidates when one element
            // has "foo bar" and the other has "bar foo".
            if !class_list.spilled() {
                class_list.sort_by(|a, b| a.get_hash().cmp(&b.get_hash()));
            }
            self.class_list = Some(class_list);
        }
        &*self.class_list.as_ref().unwrap()
    }

    /// Computes the revalidation results if needed, and returns it.
    /// Inline so we know at compile time what bloom_known_valid is.
    #[inline]
    fn revalidation_match_results<E, F>(
        &mut self,
        element: E,
        stylist: &Stylist,
        bloom: &StyleBloom<E>,
        bloom_known_valid: bool,
        flags_setter: &mut F
    ) -> &SmallBitVec
        where E: TElement,
              F: FnMut(&E, ElementSelectorFlags),
    {
        if self.revalidation_match_results.is_none() {
            // The bloom filter may already be set up for our element.
            // If it is, use it.  If not, we must be in a candidate
            // (i.e. something in the cache), and the element is one
            // of our cousins, not a sibling.  In that case, we'll
            // just do revalidation selector matching without a bloom
            // filter, to avoid thrashing the filter.
            let bloom_to_use = if bloom_known_valid {
                debug_assert_eq!(bloom.current_parent(),
                                 element.traversal_parent());
                Some(bloom.filter())
            } else {
                if bloom.current_parent() == element.traversal_parent() {
                    Some(bloom.filter())
                } else {
                    None
                }
            };
            self.revalidation_match_results =
                Some(stylist.match_revalidation_selectors(&element,
                                                          bloom_to_use,
                                                          flags_setter));
        }

        self.revalidation_match_results.as_ref().unwrap()
    }
}

/// Information regarding a style sharing candidate, that is, an entry in the
/// style sharing cache.
///
/// Note that this information is stored in TLS and cleared after the traversal,
/// and once here, the style information of the element is immutable, so it's
/// safe to access.
///
/// Important: If you change the members/layout here, You need to do the same for
/// FakeCandidate below.
#[derive(Debug)]
pub struct StyleSharingCandidate<E: TElement> {
    /// The element.
    element: E,
    validation_data: ValidationData,
}

struct FakeCandidate {
    _element: usize,
    _validation_data: ValidationData,
}

impl<E: TElement> Deref for StyleSharingCandidate<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}


impl<E: TElement> StyleSharingCandidate<E> {
    /// Get the classlist of this candidate.
    fn class_list(&mut self) -> &[Atom] {
        self.validation_data.class_list(self.element)
    }

    /// Get the pres hints of this candidate.
    fn pres_hints(&mut self) -> &[ApplicableDeclarationBlock] {
        self.validation_data.pres_hints(self.element)
    }

    /// Compute the bit vector of revalidation selector match results
    /// for this candidate.
    fn revalidation_match_results(
        &mut self,
        stylist: &Stylist,
        bloom: &StyleBloom<E>,
    ) -> &SmallBitVec {
        self.validation_data.revalidation_match_results(
            self.element,
            stylist,
            bloom,
            /* bloom_known_valid = */ false,
            &mut |_, _| {})
    }
}

impl<E: TElement> PartialEq<StyleSharingCandidate<E>> for StyleSharingCandidate<E> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}

/// An element we want to test against the style sharing cache.
pub struct StyleSharingTarget<E: TElement> {
    element: E,
    validation_data: ValidationData,
}

impl<E: TElement> Deref for StyleSharingTarget<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

impl<E: TElement> StyleSharingTarget<E> {
    /// Trivially construct a new StyleSharingTarget to test against the cache.
    pub fn new(element: E) -> Self {
        Self {
            element: element,
            validation_data: ValidationData::default(),
        }
    }

    fn class_list(&mut self) -> &[Atom] {
        self.validation_data.class_list(self.element)
    }

    /// Get the pres hints of this candidate.
    fn pres_hints(&mut self) -> &[ApplicableDeclarationBlock] {
        self.validation_data.pres_hints(self.element)
    }

    fn revalidation_match_results(
        &mut self,
        stylist: &Stylist,
        bloom: &StyleBloom<E>,
        selector_flags_map: &mut SelectorFlagsMap<E>
    ) -> &SmallBitVec {
        // It's important to set the selector flags. Otherwise, if we succeed in
        // sharing the style, we may not set the slow selector flags for the
        // right elements (which may not necessarily be |element|), causing
        // missed restyles after future DOM mutations.
        //
        // Gecko's test_bug534804.html exercises this. A minimal testcase is:
        // <style> #e:empty + span { ... } </style>
        // <span id="e">
        //   <span></span>
        // </span>
        // <span></span>
        //
        // The style sharing cache will get a hit for the second span. When the
        // child span is subsequently removed from the DOM, missing selector
        // flags would cause us to miss the restyle on the second span.
        let element = self.element;
        let mut set_selector_flags = |el: &E, flags: ElementSelectorFlags| {
            element.apply_selector_flags(selector_flags_map, el, flags);
        };

        self.validation_data.revalidation_match_results(
            self.element,
            stylist,
            bloom,
            /* bloom_known_valid = */ true,
            &mut set_selector_flags)
    }

    /// Attempts to share a style with another node.
    pub fn share_style_if_possible(
        &mut self,
        context: &mut StyleContext<E>,
    ) -> Option<ElementStyles> {
        let cache = &mut context.thread_local.sharing_cache;
        let shared_context = &context.shared;
        let selector_flags_map = &mut context.thread_local.selector_flags;
        let bloom_filter = &context.thread_local.bloom_filter;

        if cache.dom_depth != bloom_filter.matching_depth() {
            debug!("Can't share style, because DOM depth changed from {:?} to {:?}, element: {:?}",
                   cache.dom_depth, bloom_filter.matching_depth(), self.element);
            return None;
        }
        debug_assert_eq!(bloom_filter.current_parent(),
                         self.element.traversal_parent());

        cache.share_style_if_possible(
            shared_context,
            selector_flags_map,
            bloom_filter,
            self
        )
    }

    /// Gets the validation data used to match against this target, if any.
    pub fn take_validation_data(&mut self) -> ValidationData {
        self.validation_data.take()
    }
}

struct SharingCacheBase<Candidate> {
    entries: LRUCache<[Candidate; SHARING_CACHE_BACKING_STORE_SIZE]>,
}

impl<Candidate> Default for SharingCacheBase<Candidate> {
    fn default() -> Self {
        Self {
            entries: LRUCache::new(),
        }
    }
}

impl<Candidate> SharingCacheBase<Candidate> {
    fn clear(&mut self) {
        self.entries.evict_all();
    }

    fn is_empty(&self) -> bool {
        self.entries.num_entries() == 0
    }
}

impl<E: TElement> SharingCache<E> {
    fn insert(&mut self, el: E, validation_data_holder: &mut StyleSharingTarget<E>) {
        self.entries.insert(StyleSharingCandidate {
            element: el,
            validation_data: validation_data_holder.take_validation_data(),
        });
    }

    fn lookup<F>(&mut self, mut is_match: F) -> Option<ElementStyles>
    where
        F: FnMut(&mut StyleSharingCandidate<E>) -> bool
    {
        let mut index = None;
        for (i, candidate) in self.entries.iter_mut().enumerate() {
            if is_match(candidate) {
                index = Some(i);
                break;
            }
        };

        match index {
            None => None,
            Some(i) => {
                self.entries.touch(i);
                let front = self.entries.front_mut().unwrap();
                debug_assert!(is_match(front));
                Some(front.element.borrow_data().unwrap().styles.clone())
            }
        }
    }
}

/// Style sharing caches are are large allocations, so we store them in thread-local
/// storage such that they can be reused across style traversals. Ideally, we'd just
/// stack-allocate these buffers with uninitialized memory, but right now rustc can't
/// avoid memmoving the entire cache during setup, which gets very expensive. See
/// issues like [1] and [2].
///
/// Given that the cache stores entries of type TElement, we transmute to usize
/// before storing in TLS. This is safe as long as we make sure to empty the cache
/// before we let it go.
///
/// [1] https://github.com/rust-lang/rust/issues/42763
/// [2] https://github.com/rust-lang/rust/issues/13707
type SharingCache<E> = SharingCacheBase<StyleSharingCandidate<E>>;
type TypelessSharingCache = SharingCacheBase<FakeCandidate>;
type StoredSharingCache = Arc<AtomicRefCell<TypelessSharingCache>>;

thread_local!(static SHARING_CACHE_KEY: StoredSharingCache =
              Arc::new(AtomicRefCell::new(TypelessSharingCache::default())));

/// An LRU cache of the last few nodes seen, so that we can aggressively try to
/// reuse their styles.
///
/// Note that this cache is flushed every time we steal work from the queue, so
/// storing nodes here temporarily is safe.
pub struct StyleSharingCache<E: TElement> {
    /// The LRU cache, with the type cast away to allow persisting the allocation.
    cache_typeless: OwningHandle<StoredSharingCache, AtomicRefMut<'static, TypelessSharingCache>>,
    /// Bind this structure to the lifetime of E, since that's what we effectively store.
    marker: PhantomData<SendElement<E>>,
    /// The DOM depth we're currently at.  This is used as an optimization to
    /// clear the cache when we change depths, since we know at that point
    /// nothing in the cache will match.
    dom_depth: usize,
}

impl<E: TElement> Drop for StyleSharingCache<E> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<E: TElement> StyleSharingCache<E> {
    #[allow(dead_code)]
    fn cache(&self) -> &SharingCache<E> {
        let base: &TypelessSharingCache = &*self.cache_typeless;
        unsafe { mem::transmute(base) }
    }

    fn cache_mut(&mut self) -> &mut SharingCache<E> {
        let base: &mut TypelessSharingCache = &mut *self.cache_typeless;
        unsafe { mem::transmute(base) }
    }

    /// Create a new style sharing candidate cache.
    pub fn new() -> Self {
        assert_eq!(mem::size_of::<SharingCache<E>>(), mem::size_of::<TypelessSharingCache>());
        assert_eq!(mem::align_of::<SharingCache<E>>(), mem::align_of::<TypelessSharingCache>());
        let cache_arc = SHARING_CACHE_KEY.with(|c| c.clone());
        let cache = OwningHandle::new_with_fn(cache_arc, |x| unsafe { x.as_ref() }.unwrap().borrow_mut());
        debug_assert!(cache.is_empty());

        StyleSharingCache {
            cache_typeless: cache,
            marker: PhantomData,
            dom_depth: 0,
        }
    }

    /// Tries to insert an element in the style sharing cache.
    ///
    /// Fails if we know it should never be in the cache.
    ///
    /// NB: We pass a source for the validation data, rather than the data itself,
    /// to avoid memmoving at each function call. See rust issue #42763.
    pub fn insert_if_possible(&mut self,
                              element: &E,
                              style: &ComputedValues,
                              validation_data_holder: &mut StyleSharingTarget<E>,
                              dom_depth: usize) {
        let parent = match element.traversal_parent() {
            Some(element) => element,
            None => {
                debug!("Failing to insert to the cache: no parent element");
                return;
            }
        };

        if element.is_native_anonymous() {
            debug!("Failing to insert into the cache: NAC");
            return;
        }

        // If the element has running animations, we can't share style.
        //
        // This is distinct from the specifies_{animations,transitions} check below,
        // because:
        //   * Animations can be triggered directly via the Web Animations API.
        //   * Our computed style can still be affected by animations after we no
        //     longer match any animation rules, since removing animations involves
        //     a sequential task and an additional traversal.
        if element.has_animations() {
            debug!("Failing to insert to the cache: running animations");
            return;
        }

        // In addition to the above running animations check, we also need to
        // check CSS animation and transition styles since it's possible that
        // we are about to create CSS animations/transitions.
        //
        // These are things we don't check in the candidate match because they
        // are either uncommon or expensive.
        let box_style = style.get_box();
        if box_style.specifies_transitions() {
            debug!("Failing to insert to the cache: transitions");
            return;
        }

        if box_style.specifies_animations() {
            debug!("Failing to insert to the cache: animations");
            return;
        }

        debug!("Inserting into cache: {:?} with parent {:?}", element, parent);

        if self.dom_depth != dom_depth {
            debug!("Clearing cache because depth changed from {:?} to {:?}, element: {:?}",
                   self.dom_depth, dom_depth, element);
            self.clear();
            self.dom_depth = dom_depth;
        }
        self.cache_mut().insert(*element, validation_data_holder);
    }

    /// Clear the style sharing candidate cache.
    pub fn clear(&mut self) {
        self.cache_mut().clear();
    }

    /// Attempts to share a style with another node.
    fn share_style_if_possible(
        &mut self,
        shared_context: &SharedStyleContext,
        selector_flags_map: &mut SelectorFlagsMap<E>,
        bloom_filter: &StyleBloom<E>,
        target: &mut StyleSharingTarget<E>,
    ) -> Option<ElementStyles> {
        if shared_context.options.disable_style_sharing_cache {
            debug!("{:?} Cannot share style: style sharing cache disabled",
                   target.element);
            return None;
        }

        if target.traversal_parent().is_none() {
            debug!("{:?} Cannot share style: element has no parent",
                   target.element);
            return None;
        }

        if target.is_native_anonymous() {
            debug!("{:?} Cannot share style: NAC", target.element);
            return None;
        }

        self.cache_mut().lookup(|candidate| {
            Self::test_candidate(
                target,
                candidate,
                &shared_context,
                bloom_filter,
                selector_flags_map
            )
        })
    }

    fn test_candidate(
        target: &mut StyleSharingTarget<E>,
        candidate: &mut StyleSharingCandidate<E>,
        shared: &SharedStyleContext,
        bloom: &StyleBloom<E>,
        selector_flags_map: &mut SelectorFlagsMap<E>
    ) -> bool {
        // Check that we have the same parent, or at least that the parents
        // share styles and permit sharing across their children. The latter
        // check allows us to share style between cousins if the parents
        // shared style.
        let parent = target.traversal_parent();
        let candidate_parent = candidate.element.traversal_parent();
        if parent != candidate_parent &&
           !checks::can_share_style_across_parents(parent, candidate_parent) {
            trace!("Miss: Parent");
            return false;
        }

        if target.is_native_anonymous() {
            debug_assert!(!candidate.element.is_native_anonymous(),
                          "Why inserting NAC into the cache?");
            trace!("Miss: Native Anonymous Content");
            return false;
        }

        if *target.get_local_name() != *candidate.element.get_local_name() {
            trace!("Miss: Local Name");
            return false;
        }

        if *target.get_namespace() != *candidate.element.get_namespace() {
            trace!("Miss: Namespace");
            return false;
        }

        if target.is_link() != candidate.element.is_link() {
            trace!("Miss: Link");
            return false;
        }

        if target.matches_user_and_author_rules() !=
            candidate.element.matches_user_and_author_rules() {
            trace!("Miss: User and Author Rules");
            return false;
        }

        // We do not ignore visited state here, because Gecko
        // needs to store extra bits on visited style contexts,
        // so these contexts cannot be shared
        if target.element.get_state() != candidate.get_state() {
            trace!("Miss: User and Author State");
            return false;
        }

        let element_id = target.element.get_id();
        let candidate_id = candidate.element.get_id();
        if element_id != candidate_id {
            // It's possible that there are no styles for either id.
            if checks::may_have_rules_for_ids(shared, element_id.as_ref(),
                                              candidate_id.as_ref()) {
                trace!("Miss: ID Attr");
                return false;
            }
        }

        if !checks::have_same_style_attribute(target, candidate) {
            trace!("Miss: Style Attr");
            return false;
        }

        if !checks::have_same_class(target, candidate) {
            trace!("Miss: Class");
            return false;
        }

        if !checks::have_same_presentational_hints(target, candidate) {
            trace!("Miss: Pres Hints");
            return false;
        }

        if !checks::revalidate(target, candidate, shared, bloom,
                               selector_flags_map) {
            trace!("Miss: Revalidation");
            return false;
        }

        debug_assert!(target.has_current_styles_for_traversal(
            &candidate.element.borrow_data().unwrap(),
            shared.traversal_flags)
        );
        debug!("Sharing allowed between {:?} and {:?}", target.element, candidate.element);

        true
    }
}
