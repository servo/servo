/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Different checks done during the style sharing process in order to determine
//! quickly whether it's worth to share style, and whether two different
//! elements can indeed share the same style.

use Atom;
use bloom::StyleBloom;
use context::{SelectorFlagsMap, SharedStyleContext};
use dom::TElement;
use servo_arc::Arc;
use sharing::{StyleSharingCandidate, StyleSharingTarget};

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

    let eq = Arc::ptr_eq(a.borrow_data().unwrap().styles.primary(),
                         b.borrow_data().unwrap().styles.primary());
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
        (Some(a), Some(b)) => &*a as *const _ == &*b as *const _
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

/// Checks whether we might have rules for either of the two ids.
#[inline]
pub fn may_have_rules_for_ids(shared_context: &SharedStyleContext,
                              element_id: Option<&Atom>,
                              candidate_id: Option<&Atom>) -> bool
{
    // We shouldn't be called unless the ids are different.
    debug_assert!(element_id.is_some() || candidate_id.is_some());
    let stylist = &shared_context.stylist;

    let may_have_rules_for_element = match element_id {
        Some(id) => stylist.may_have_rules_for_id(id),
        None => false
    };

    if may_have_rules_for_element {
        return true;
    }

    match candidate_id {
        Some(id) => stylist.may_have_rules_for_id(id),
        None => false
    }
}
