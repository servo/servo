/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Different checks done during the style sharing process in order to determine
//! quickly whether it's worth to share style, and whether two different
//! elements can indeed share the same style.

use bloom::StyleBloom;
use context::{SelectorFlagsMap, SharedStyleContext};
use dom::TElement;
use element_state::*;
use selectors::matching::StyleRelations;
use sharing::{StyleSharingCandidate, StyleSharingTarget};
use stylearc::Arc;

/// Determines, based on the results of selector matching, whether it's worth to
/// try to share style with this element, that is, to try to insert the element
/// in the chache.
#[inline]
pub fn relations_are_shareable(relations: &StyleRelations) -> bool {
    use selectors::matching::*;
    // If we start sharing things that are AFFECTED_BY_PSEUDO_ELEMENTS, we need
    // to track revalidation selectors on a per-pseudo-element basis.
    !relations.intersects(AFFECTED_BY_ID_SELECTOR |
                          AFFECTED_BY_PSEUDO_ELEMENTS)
}

/// Whether, given two elements, they have pointer-equal computed values.
///
/// Both elements need to be styled already.
///
/// This is used to know whether we can share style across cousins (if the two
/// parents have the same style).
pub fn same_computed_values<E>(first: Option<E>, second: Option<E>) -> bool
    where E: TElement,
{
    let (a, b) = match (first, second) {
        (Some(f), Some(s)) => (f, s),
        _ => return false,
    };

    let eq = Arc::ptr_eq(a.borrow_data().unwrap().styles().primary.values(),
                         b.borrow_data().unwrap().styles().primary.values());
    eq
}

/// Whether two elements have the same same style attribute (by pointer identity).
pub fn have_same_style_attribute<E>(
    target: &mut StyleSharingTarget<E>,
    candidate: &mut StyleSharingCandidate<E>
) -> bool
    where E: TElement,
{
    match (target.style_attribute(), candidate.style_attribute()) {
        (None, None) => true,
        (Some(_), None) | (None, Some(_)) => false,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b)
    }
}

/// Whether two elements have the same same presentational attributes.
pub fn have_same_presentational_hints<E>(
    target: &mut StyleSharingTarget<E>,
    candidate: &mut StyleSharingCandidate<E>
) -> bool
    where E: TElement,
{
    target.pres_hints() == candidate.pres_hints()
}

/// Whether a given element has the same class attribute than a given candidate.
///
/// We don't try to share style across elements with different class attributes.
pub fn have_same_class<E>(target: &mut StyleSharingTarget<E>,
                          candidate: &mut StyleSharingCandidate<E>)
                          -> bool
    where E: TElement,
{
    target.class_list() == candidate.class_list()
}

/// Compare element and candidate state, but ignore visitedness.  Styles don't
/// actually changed based on visitedness (since both possibilities are computed
/// up front), so it's safe to share styles if visitedness differs.
pub fn have_same_state_ignoring_visitedness<E>(element: E,
                                               candidate: &StyleSharingCandidate<E>)
                                               -> bool
    where E: TElement,
{
    let state_mask = !IN_VISITED_OR_UNVISITED_STATE;
    let state = element.get_state() & state_mask;
    let candidate_state = candidate.element.get_state() & state_mask;
    state == candidate_state
}

/// Whether a given element and a candidate match the same set of "revalidation"
/// selectors.
///
/// Revalidation selectors are those that depend on the DOM structure, like
/// :first-child, etc, or on attributes that we don't check off-hand (pretty
/// much every attribute selector except `id` and `class`.
#[inline]
pub fn revalidate<E>(target: &mut StyleSharingTarget<E>,
                     candidate: &mut StyleSharingCandidate<E>,
                     shared_context: &SharedStyleContext,
                     bloom: &StyleBloom<E>,
                     selector_flags_map: &mut SelectorFlagsMap<E>)
                     -> bool
    where E: TElement,
{
    let stylist = &shared_context.stylist;

    let for_element =
        target.revalidation_match_results(stylist, bloom, selector_flags_map);

    let for_candidate = candidate.revalidation_match_results(stylist, bloom);

    // This assert "ensures", to some extent, that the two candidates have
    // matched the same rulehash buckets, and as such, that the bits we're
    // comparing represent the same set of selectors.
    debug_assert_eq!(for_element.len(), for_candidate.len());

    for_element == for_candidate
}
