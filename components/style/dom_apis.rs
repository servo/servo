/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic implementations of some DOM APIs so they can be shared between Servo
//! and Gecko.

use context::QuirksMode;
use selectors::{Element, SelectorList};
use selectors::matching::{self, MatchingContext, MatchingMode};

/// https://dom.spec.whatwg.org/#dom-element-matches
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
