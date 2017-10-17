/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic implementations of some DOM APIs so they can be shared between Servo
//! and Gecko.

use context::QuirksMode;
use selectors::{Element, NthIndexCache, SelectorList};
use selectors::matching::{self, MatchingContext, MatchingMode};

/// <https://dom.spec.whatwg.org/#dom-element-matches>
pub fn element_matches<E>(
    element: &E,
    selector_list: &SelectorList<E::Impl>,
    quirks_mode: QuirksMode,
) -> bool
where
    E: Element,
{
    let mut context = MatchingContext::new(
        MatchingMode::Normal,
        None,
        None,
        quirks_mode,
    );
    context.scope_element = Some(element.opaque());
    matching::matches_selector_list(selector_list, element, &mut context)
}

/// <https://dom.spec.whatwg.org/#dom-element-closest>
pub fn element_closest<E>(
    element: E,
    selector_list: &SelectorList<E::Impl>,
    quirks_mode: QuirksMode,
) -> Option<E>
where
    E: Element,
{
    let mut nth_index_cache = NthIndexCache::default();

    let mut context = MatchingContext::new(
        MatchingMode::Normal,
        None,
        Some(&mut nth_index_cache),
        quirks_mode,
    );
    context.scope_element = Some(element.opaque());

    let mut current = Some(element);
    while let Some(element) = current.take() {
        if matching::matches_selector_list(selector_list, &element, &mut context) {
            return Some(element);
        }
        current = element.parent_element();
    }

    return None;
}
