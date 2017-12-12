/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Different checks done during the style sharing process in order to determine
//! quickly whether it's worth to share style, and whether two different
//! elements can indeed share the same style.

use bloom::StyleBloom;
use context::{SelectorFlagsMap, SharedStyleContext};
use dom::TElement;
use selectors::NthIndexCache;
use sharing::{StyleSharingCandidate, StyleSharingTarget};

/// Determines whether a target and a candidate have compatible parents for
/// sharing.
pub fn parents_allow_sharing<E>(
    target: &mut StyleSharingTarget<E>,
    candidate: &mut StyleSharingCandidate<E>
) -> bool
where
    E: TElement,
{
    // If the identity of the parent style isn't equal, we can't share. We check
    // this first, because the result is cached.
    if target.parent_style_identity() != candidate.parent_style_identity() {
        return false;
    }

    // Siblings can always share.
    let parent = target.inheritance_parent().unwrap();
    let candidate_parent = candidate.element.inheritance_parent().unwrap();
    if parent == candidate_parent {
        return true;
    }

    // Cousins are a bit more complicated.
    //
    // If a parent element was already styled and we traversed past it without
    // restyling it, that may be because our clever invalidation logic was able
    // to prove that the styles of that element would remain unchanged despite
    // changes to the id or class attributes. However, style sharing relies in
    // the strong guarantee that all the classes and ids up the respective parent
    // chains are identical. As such, if we skipped styling for one (or both) of
    // the parents on this traversal, we can't share styles across cousins.
    //
    // This is a somewhat conservative check. We could tighten it by having the
    // invalidation logic explicitly flag elements for which it ellided styling.
    let parent_data = parent.borrow_data().unwrap();
    let candidate_parent_data = candidate_parent.borrow_data().unwrap();
    if !parent_data.safe_for_cousin_sharing() || !candidate_parent_data.safe_for_cousin_sharing() {
        return false;
    }

    true
}

/// Whether two elements have the same same style attribute (by pointer identity).
pub fn have_same_style_attribute<E>(
    target: &mut StyleSharingTarget<E>,
    candidate: &mut StyleSharingCandidate<E>
) -> bool
where
    E: TElement,
{
    match (target.style_attribute(), candidate.style_attribute()) {
        (None, None) => true,
        (Some(_), None) | (None, Some(_)) => false,
        (Some(a), Some(b)) => &*a as *const _ == &*b as *const _
    }
}

/// Whether two elements have the same same presentational attributes.
pub fn have_same_presentational_hints<E>(
    target: &mut StyleSharingTarget<E>,
    candidate: &mut StyleSharingCandidate<E>
) -> bool
where
    E: TElement,
{
    target.pres_hints() == candidate.pres_hints()
}

/// Whether a given element has the same class attribute than a given candidate.
///
/// We don't try to share style across elements with different class attributes.
pub fn have_same_class<E>(
    target: &mut StyleSharingTarget<E>,
    candidate: &mut StyleSharingCandidate<E>,
) -> bool
where
    E: TElement,
{
    target.class_list() == candidate.class_list()
}

/// Whether a given element and a candidate match the same set of "revalidation"
/// selectors.
///
/// Revalidation selectors are those that depend on the DOM structure, like
/// :first-child, etc, or on attributes that we don't check off-hand (pretty
/// much every attribute selector except `id` and `class`.
#[inline]
pub fn revalidate<E>(
    target: &mut StyleSharingTarget<E>,
    candidate: &mut StyleSharingCandidate<E>,
    shared_context: &SharedStyleContext,
    bloom: &StyleBloom<E>,
    nth_index_cache: &mut NthIndexCache,
    selector_flags_map: &mut SelectorFlagsMap<E>,
) -> bool
where
    E: TElement,
{
    let stylist = &shared_context.stylist;

    let for_element = target.revalidation_match_results(
        stylist,
        bloom,
        nth_index_cache,
        selector_flags_map,
    );

    let for_candidate = candidate.revalidation_match_results(
        stylist,
        bloom,
        nth_index_cache,
    );

    // This assert "ensures", to some extent, that the two candidates have
    // matched the same rulehash buckets, and as such, that the bits we're
    // comparing represent the same set of selectors.
    debug_assert_eq!(for_element.len(), for_candidate.len());

    for_element == for_candidate
}

/// Checks whether we might have rules for either of the two ids.
#[inline]
pub fn may_match_different_id_rules<E>(
    shared_context: &SharedStyleContext,
    element: E,
    candidate: E,
) -> bool
where
    E: TElement,
{
    let element_id = element.get_id();
    let candidate_id = candidate.get_id();

    if element_id == candidate_id {
        return false;
    }

    let stylist = &shared_context.stylist;

    let may_have_rules_for_element = match element_id {
        Some(ref id) => stylist.may_have_rules_for_id(id, element),
        None => false
    };

    if may_have_rules_for_element {
        return true;
    }

    match candidate_id {
        Some(ref id) => stylist.may_have_rules_for_id(id, candidate),
        None => false
    }
}
