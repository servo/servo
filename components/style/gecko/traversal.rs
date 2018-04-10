/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko-specific bits for the styling DOM traversal.

use context::{SharedStyleContext, StyleContext};
use dom::{TElement, TNode};
use gecko::wrapper::{GeckoElement, GeckoNode};
use traversal::{recalc_style_at, DomTraversal, PerLevelTraversalData};

/// This is the simple struct that Gecko uses to encapsulate a DOM traversal for
/// styling.
pub struct RecalcStyleOnly<'a> {
    shared: SharedStyleContext<'a>,
}

impl<'a> RecalcStyleOnly<'a> {
    /// Create a `RecalcStyleOnly` traversal from a `SharedStyleContext`.
    pub fn new(shared: SharedStyleContext<'a>) -> Self {
        RecalcStyleOnly { shared: shared }
    }
}

impl<'recalc, 'le> DomTraversal<GeckoElement<'le>> for RecalcStyleOnly<'recalc> {
    fn process_preorder<F>(
        &self,
        traversal_data: &PerLevelTraversalData,
        context: &mut StyleContext<GeckoElement<'le>>,
        node: GeckoNode<'le>,
        note_child: F,
    ) where
        F: FnMut(GeckoNode<'le>),
    {
        if let Some(el) = node.as_element() {
            let mut data = unsafe { el.ensure_data() };
            recalc_style_at(self, traversal_data, context, el, &mut data, note_child);
        }
    }

    fn process_postorder(&self, _: &mut StyleContext<GeckoElement<'le>>, _: GeckoNode<'le>) {
        unreachable!();
    }

    /// We don't use the post-order traversal for anything.
    fn needs_postorder_traversal() -> bool {
        false
    }

    fn shared_context(&self) -> &SharedStyleContext {
        &self.shared
    }
}
