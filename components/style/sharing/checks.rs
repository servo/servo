/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Different checks done during the style sharing process in order to determine
//! quickly whether it's worth to share style, and whether two different
//! elements can indeed share the same style.

use context::{CurrentElementInfo, SelectorFlagsMap, SharedStyleContext};
use dom::TElement;
use element_state::*;
use matching::MatchMethods;
use selectors::bloom::BloomFilter;
use selectors::matching::{ElementSelectorFlags, StyleRelations};
use sharing::StyleSharingCandidate;
use sink::ForgetfulSink;
use stylearc::Arc;

/// Determines, based on the results of selector matching, whether it's worth to
/// try to share style with this element, that is, to try to insert the element
/// in the chache.
#[inline]
pub fn relations_are_shareable(relations: &StyleRelations) -> bool {
    use selectors::matching::*;
    !relations.intersects(AFFECTED_BY_ID_SELECTOR |
                          AFFECTED_BY_PSEUDO_ELEMENTS |
                          AFFECTED_BY_STYLE_ATTRIBUTE |
                          AFFECTED_BY_PRESENTATIONAL_HINTS)
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

/// Whether a given element has presentational hints.
///
/// We consider not worth to share style with an element that has presentational
/// hints, both because implementing the code that compares that the hints are
/// equal is somewhat annoying, and also because it'd be expensive enough.
pub fn has_presentational_hints<E>(element: E) -> bool
    where E: TElement,
{
    let mut hints = ForgetfulSink::new();
    element.synthesize_presentational_hints_for_legacy_attributes(&mut hints);
    !hints.is_empty()
}

/// Whether a given element has the same class attribute than a given candidate.
///
/// We don't try to share style across elements with different class attributes.
pub fn have_same_class<E>(element: E,
                          candidate: &mut StyleSharingCandidate<E>)
                          -> bool
    where E: TElement,
{
    // XXX Efficiency here, I'm only validating ideas.
    let mut element_class_attributes = vec![];
    element.each_class(|c| element_class_attributes.push(c.clone()));

    if candidate.class_attributes.is_none() {
        let mut attrs = vec![];
        candidate.element.each_class(|c| attrs.push(c.clone()));
        candidate.class_attributes = Some(attrs)
    }

    element_class_attributes == *candidate.class_attributes.as_ref().unwrap()
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
pub fn revalidate<E>(element: E,
                     candidate: &mut StyleSharingCandidate<E>,
                     shared_context: &SharedStyleContext,
                     bloom: &BloomFilter,
                     info: &mut CurrentElementInfo,
                     selector_flags_map: &mut SelectorFlagsMap<E>)
                     -> bool
    where E: TElement,
{
    let stylist = &shared_context.stylist;

    if info.revalidation_match_results.is_none() {
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
        let mut set_selector_flags = |el: &E, flags: ElementSelectorFlags| {
            element.apply_selector_flags(selector_flags_map, el, flags);
        };
        info.revalidation_match_results =
            Some(stylist.match_revalidation_selectors(&element, bloom,
                                                      &mut set_selector_flags));
    }

    if candidate.revalidation_match_results.is_none() {
        let results =
            stylist.match_revalidation_selectors(&*candidate.element, bloom,
                                                 &mut |_, _| {});
        candidate.revalidation_match_results = Some(results);
    }

    let for_element = info.revalidation_match_results.as_ref().unwrap();
    let for_candidate = candidate.revalidation_match_results.as_ref().unwrap();

    // This assert "ensures", to some extent, that the two candidates have
    // matched the same rulehash buckets, and as such, that the bits we're
    // comparing represent the same set of selectors.
    debug_assert_eq!(for_element.len(), for_candidate.len());

    for_element == for_candidate
}
