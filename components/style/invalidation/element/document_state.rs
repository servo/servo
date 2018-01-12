/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An invalidation processor for style changes due to document state changes.

use dom::TElement;
use element_state::DocumentState;
use invalidation::element::invalidator::{DescendantInvalidationLists, InvalidationVector};
use invalidation::element::invalidator::{Invalidation, InvalidationProcessor};
use invalidation::element::state_and_attributes;
use selectors::matching::{MatchingContext, MatchingMode, QuirksMode, VisitedHandlingMode};
use stylist::StyleRuleCascadeData;

/// A struct holding the members necessary to invalidate document state
/// selectors.
pub struct InvalidationMatchingData {
    /// The document state that has changed, which makes it always match.
    pub document_state: DocumentState,
}

impl Default for InvalidationMatchingData {
    #[inline(always)]
    fn default() -> Self {
        Self {
            document_state: DocumentState::empty(),
        }
    }
}

/// An invalidation processor for style changes due to state and attribute
/// changes.
pub struct DocumentStateInvalidationProcessor<'a, E: TElement> {
    // TODO(emilio): We might want to just run everything for every possible
    // binding along with the document data, or just apply the XBL stuff to the
    // bound subtrees.
    rules: &'a StyleRuleCascadeData,
    matching_context: MatchingContext<'a, E::Impl>,
    document_states_changed: DocumentState,
}

impl<'a, E: TElement> DocumentStateInvalidationProcessor<'a, E> {
    /// Creates a new DocumentStateInvalidationProcessor.
    #[inline]
    pub fn new(
        rules: &'a StyleRuleCascadeData,
        document_states_changed: DocumentState,
        quirks_mode: QuirksMode,
    ) -> Self {
        let mut matching_context = MatchingContext::new_for_visited(
            MatchingMode::Normal,
            None,
            None,
            VisitedHandlingMode::AllLinksVisitedAndUnvisited,
            quirks_mode,
        );

        matching_context.extra_data = InvalidationMatchingData {
            document_state: document_states_changed,
        };

        Self { rules, document_states_changed, matching_context }
    }
}

impl<'a, E: TElement> InvalidationProcessor<'a, E> for DocumentStateInvalidationProcessor<'a, E> {
    fn collect_invalidations(
        &mut self,
        _element: E,
        self_invalidations: &mut InvalidationVector<'a>,
        _descendant_invalidations: &mut DescendantInvalidationLists<'a>,
        _sibling_invalidations: &mut InvalidationVector<'a>,
    ) -> bool {
        let map = self.rules.invalidation_map();

        for dependency in &map.document_state_selectors {
            if !dependency.state.intersects(self.document_states_changed) {
                continue;
            }

            self_invalidations.push(Invalidation::new(&dependency.selector, 0));
        }

        false
    }

    fn matching_context(&mut self) -> &mut MatchingContext<'a, E::Impl> {
        &mut self.matching_context
    }

    fn recursion_limit_exceeded(&mut self, _: E) {
        unreachable!("We don't run document state invalidation with stack limits")
    }

    fn should_process_descendants(&mut self, element: E) -> bool {
        match element.borrow_data() {
            Some(d) => state_and_attributes::should_process_descendants(&d),
            None => false,
        }
    }

    fn invalidated_descendants(&mut self, element: E, child: E) {
        state_and_attributes::invalidated_descendants(element, child)
    }

    fn invalidated_self(&mut self, element: E) {
        state_and_attributes::invalidated_self(element);
    }
}
