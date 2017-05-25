/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Code related to the style sharing cache, an optimization that allows similar
//! nodes to share style without having to run selector matching twice.

use Atom;
use bit_vec::BitVec;
use cache::{LRUCache, LRUCacheMutIterator};
use context::{CurrentElementInfo, SelectorFlagsMap, SharedStyleContext};
use data::{ComputedStyle, ElementData, ElementStyles};
use dom::{TElement, SendElement};
use matching::{ChildCascadeRequirement, MatchMethods};
use properties::ComputedValues;
use selectors::bloom::BloomFilter;
use selectors::matching::StyleRelations;
use sink::ForgetfulSink;

mod checks;

/// The amount of nodes that the style sharing candidate cache should hold at
/// most.
pub const STYLE_SHARING_CANDIDATE_CACHE_SIZE: usize = 8;

/// Controls whether the style sharing cache is used.
#[derive(Clone, Copy, PartialEq)]
pub enum StyleSharingBehavior {
    /// Style sharing allowed.
    Allow,
    /// Style sharing disallowed.
    Disallow,
}

/// Information regarding a style sharing candidate, that is, an entry in the
/// style sharing cache.
///
/// Note that this information is stored in TLS and cleared after the traversal,
/// and once here, the style information of the element is immutable, so it's
/// safe to access.
///
/// TODO: We can stick a lot more info here.
#[derive(Debug)]
pub struct StyleSharingCandidate<E: TElement> {
    /// The element. We use SendElement here so that the cache may live in
    /// ScopedTLS.
    element: SendElement<E>,
    /// The cached class names.
    class_attributes: Option<Vec<Atom>>,
    /// The cached result of matching this entry against the revalidation selectors.
    revalidation_match_results: Option<BitVec>,
}

impl<E: TElement> PartialEq<StyleSharingCandidate<E>> for StyleSharingCandidate<E> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}

/// A cache miss result.
#[derive(Clone, Debug)]
pub enum CacheMiss {
    /// The parents don't match.
    Parent,
    /// One element was NAC, while the other wasn't.
    NativeAnonymousContent,
    /// The local name of the element and the candidate don't match.
    LocalName,
    /// The namespace of the element and the candidate don't match.
    Namespace,
    /// One of the element or the candidate was a link, but the other one
    /// wasn't.
    Link,
    /// The element and the candidate match different kind of rules. This can
    /// only happen in Gecko.
    UserAndAuthorRules,
    /// The element and the candidate are in a different state.
    State,
    /// The element had an id attribute, which qualifies for a unique style.
    IdAttr,
    /// The element had a style attribute, which qualifies for a unique style.
    StyleAttr,
    /// The element and the candidate class names didn't match.
    Class,
    /// The presentation hints didn't match.
    PresHints,
    /// The element and the candidate didn't match the same set of revalidation
    /// selectors.
    Revalidation,
}

/// The results of attempting to share a style.
pub enum StyleSharingResult {
    /// We didn't find anybody to share the style with.
    CannotShare,
    /// The node's style can be shared. The integer specifies the index in the
    /// LRU cache that was hit and the damage that was done. The
    /// `ChildCascadeRequirement` indicates whether style changes due to using
    /// the shared style mean we need to recascade to children.
    StyleWasShared(usize, ChildCascadeRequirement),
}

/// An LRU cache of the last few nodes seen, so that we can aggressively try to
/// reuse their styles.
///
/// Note that this cache is flushed every time we steal work from the queue, so
/// storing nodes here temporarily is safe.
pub struct StyleSharingCandidateCache<E: TElement> {
    cache: LRUCache<StyleSharingCandidate<E>>,
}

impl<E: TElement> StyleSharingCandidateCache<E> {
    /// Create a new style sharing candidate cache.
    pub fn new() -> Self {
        StyleSharingCandidateCache {
            cache: LRUCache::new(STYLE_SHARING_CANDIDATE_CACHE_SIZE),
        }
    }

    /// Returns the number of entries in the cache.
    pub fn num_entries(&self) -> usize {
        self.cache.num_entries()
    }

    fn iter_mut(&mut self) -> LRUCacheMutIterator<StyleSharingCandidate<E>> {
        self.cache.iter_mut()
    }

    /// Tries to insert an element in the style sharing cache.
    ///
    /// Fails if we know it should never be in the cache.
    pub fn insert_if_possible(&mut self,
                              element: &E,
                              style: &ComputedValues,
                              relations: StyleRelations,
                              revalidation_match_results: Option<BitVec>) {
        let parent = match element.parent_element() {
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

        // These are things we don't check in the candidate match because they
        // are either uncommon or expensive.
        if !checks::relations_are_shareable(&relations) {
            debug!("Failing to insert to the cache: {:?}", relations);
            return;
        }

        // Make sure we noted any presentational hints in the StyleRelations.
        if cfg!(debug_assertions) {
            let mut hints = ForgetfulSink::new();
            element.synthesize_presentational_hints_for_legacy_attributes(&mut hints);
            debug_assert!(hints.is_empty(),
                          "Style relations should not be shareable!");
        }

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

        self.cache.insert(StyleSharingCandidate {
            element: unsafe { SendElement::new(*element) },
            class_attributes: None,
            revalidation_match_results: revalidation_match_results,
        });
    }

    /// Touch a given index in the style sharing candidate cache.
    pub fn touch(&mut self, index: usize) {
        self.cache.touch(index);
    }

    /// Clear the style sharing candidate cache.
    pub fn clear(&mut self) {
        self.cache.evict_all()
    }

    /// Attempts to share a style with another node.
    ///
    /// This method is unsafe because it depends on the
    /// `style_sharing_candidate_cache` having only live nodes in it, and we
    /// have no way to guarantee that at the type system level yet.
    pub unsafe fn share_style_if_possible(
        &mut self,
        shared_context: &SharedStyleContext,
        current_element_info: &mut CurrentElementInfo,
        selector_flags_map: &mut SelectorFlagsMap<E>,
        bloom_filter: &BloomFilter,
        element: E,
        data: &mut ElementData
    ) -> StyleSharingResult {
        if shared_context.options.disable_style_sharing_cache {
            debug!("{:?} Cannot share style: style sharing cache disabled",
                   element);
            return StyleSharingResult::CannotShare
        }

        if element.parent_element().is_none() {
            debug!("{:?} Cannot share style: element has no parent", element);
            return StyleSharingResult::CannotShare
        }

        if element.is_native_anonymous() {
            debug!("{:?} Cannot share style: NAC", element);
            return StyleSharingResult::CannotShare;
        }

        if element.style_attribute().is_some() {
            debug!("{:?} Cannot share style: element has style attribute",
                   element);
            return StyleSharingResult::CannotShare
        }

        if element.get_id().is_some() {
            debug!("{:?} Cannot share style: element has id", element);
            return StyleSharingResult::CannotShare
        }

        let mut should_clear_cache = false;
        for (i, candidate) in self.iter_mut().enumerate() {
            let sharing_result =
                Self::test_candidate(
                    element,
                    candidate,
                    &shared_context,
                    bloom_filter,
                    current_element_info,
                    selector_flags_map
                );

            match sharing_result {
                Ok(shared_style) => {
                    // Yay, cache hit. Share the style.

                    // Accumulate restyle damage.
                    debug_assert_eq!(data.has_styles(), data.has_restyle());
                    let old_values = data.get_styles_mut()
                                         .and_then(|s| s.primary.values.take());
                    let child_cascade_requirement =
                        element.accumulate_damage(
                            &shared_context,
                            data.get_restyle_mut(),
                            old_values.as_ref().map(|v| &**v),
                            shared_style.values(),
                            None
                        );

                    // We never put elements with pseudo style into the style
                    // sharing cache, so we can just mint an ElementStyles
                    // directly here.
                    //
                    // See https://bugzilla.mozilla.org/show_bug.cgi?id=1329361
                    let styles = ElementStyles::new(shared_style);
                    data.set_styles(styles);

                    return StyleSharingResult::StyleWasShared(i, child_cascade_requirement)
                }
                Err(miss) => {
                    debug!("Cache miss: {:?}", miss);

                    // Cache miss, let's see what kind of failure to decide
                    // whether we keep trying or not.
                    match miss {
                        // Cache miss because of parent, clear the candidate cache.
                        CacheMiss::Parent => {
                            should_clear_cache = true;
                            break;
                        },
                        // Too expensive failure, give up, we don't want another
                        // one of these.
                        CacheMiss::PresHints |
                        CacheMiss::Revalidation => break,
                        _ => {}
                    }
                }
            }
        }

        debug!("{:?} Cannot share style: {} cache entries", element,
               self.cache.num_entries());

        if should_clear_cache {
            self.clear();
        }

        StyleSharingResult::CannotShare
    }

    fn test_candidate(element: E,
                      candidate: &mut StyleSharingCandidate<E>,
                      shared: &SharedStyleContext,
                      bloom: &BloomFilter,
                      info: &mut CurrentElementInfo,
                      selector_flags_map: &mut SelectorFlagsMap<E>)
                      -> Result<ComputedStyle, CacheMiss> {
        macro_rules! miss {
            ($miss: ident) => {
                return Err(CacheMiss::$miss);
            }
        }

        // Check that we have the same parent, or at least the same pointer
        // identity for parent computed style. The latter check allows us to
        // share style between cousins if the parents shared style.
        let parent = element.parent_element();
        let candidate_parent = candidate.element.parent_element();
        if parent != candidate_parent &&
           !checks::same_computed_values(parent, candidate_parent) {
            miss!(Parent)
        }

        if element.is_native_anonymous() {
            debug_assert!(!candidate.element.is_native_anonymous(),
                          "Why inserting NAC into the cache?");
            miss!(NativeAnonymousContent)
        }

        if *element.get_local_name() != *candidate.element.get_local_name() {
            miss!(LocalName)
        }

        if *element.get_namespace() != *candidate.element.get_namespace() {
            miss!(Namespace)
        }

        if element.is_link() != candidate.element.is_link() {
            miss!(Link)
        }

        if element.matches_user_and_author_rules() !=
            candidate.element.matches_user_and_author_rules() {
            miss!(UserAndAuthorRules)
        }

        if !checks::have_same_state_ignoring_visitedness(element, candidate) {
            miss!(State)
        }

        if element.get_id() != candidate.element.get_id() {
            miss!(IdAttr)
        }

        if element.style_attribute().is_some() {
            miss!(StyleAttr)
        }

        if !checks::have_same_class(element, candidate) {
            miss!(Class)
        }

        if checks::has_presentational_hints(element) {
            miss!(PresHints)
        }

        if !checks::revalidate(element, candidate, shared, bloom, info,
                               selector_flags_map) {
            miss!(Revalidation)
        }

        let data = candidate.element.borrow_data().unwrap();
        debug_assert!(element.has_current_styles(&data));

        debug!("Sharing style between {:?} and {:?}",
               element, candidate.element);
        Ok(data.styles().primary.clone())
    }
}
