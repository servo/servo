/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Invalidates style of all elements that depend on viewport units.

use crate::dom::{TElement, TNode};
use crate::invalidation::element::restyle_hints::RestyleHint;

/// Invalidates style of all elements that depend on viewport units.
///
/// Returns whether any element was invalidated.
pub fn invalidate<E>(root: E) -> bool
where
    E: TElement,
{
    debug!("invalidation::viewport_units::invalidate({:?})", root);
    invalidate_recursively(root)
}

fn invalidate_recursively<E>(element: E) -> bool
where
    E: TElement,
{
    let mut data = match element.mutate_data() {
        Some(data) => data,
        None => return false,
    };

    if data.hint.will_recascade_subtree() {
        debug!("invalidate_recursively: {:?} was already invalid", element);
        return false;
    }

    let uses_viewport_units = data.styles.uses_viewport_units();
    if uses_viewport_units {
        debug!("invalidate_recursively: {:?} uses viewport units", element);
        data.hint.insert(RestyleHint::RECASCADE_SELF);
    }

    let mut any_children_invalid = false;
    for child in element.traversal_children() {
        if let Some(child) = child.as_element() {
            any_children_invalid |= invalidate_recursively(child);
        }
    }

    if any_children_invalid {
        debug!(
            "invalidate_recursively: Children of {:?} changed, setting dirty descendants",
            element
        );
        unsafe { element.set_dirty_descendants() }
    }

    uses_viewport_units || any_children_invalid
}
