/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An invalidation processor for style changes due to document state changes.

use crate::dom::TElement;
use crate::element_state::DocumentState;
use crate::invalidation::element::invalidation_map::Dependency;
use crate::invalidation::element::invalidator::{DescendantInvalidationLists, InvalidationVector};
use crate::invalidation::element::invalidator::{Invalidation, InvalidationProcessor};
use crate::invalidation::element::state_and_attributes;
use crate::stylist::CascadeData;
use selectors::matching::{MatchingContext, MatchingMode, QuirksMode, VisitedHandlingMode};

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
pub struct DocumentStateInvalidationProcessor<'a, E: TElement, I> {
    rules: I,
    matching_context: MatchingContext<'a, E::Impl>,
    document_states_changed: DocumentState,
}

impl<'a, E: TElement, I> DocumentStateInvalidationProcessor<'a, E, I> {
    /// Creates a new DocumentStateInvalidationProcessor.
    #[inline]
    pub fn new(rules: I, document_states_changed: DocumentState, quirks_mode: QuirksMode) -> Self {
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

        Self {
            rules,
            document_states_changed,
            matching_context,
        }
    }
}

impl<'a, E, I> InvalidationProcessor<'a, E> for DocumentStateInvalidationProcessor<'a, E, I>
where
    E: TElement,
    I: Iterator<Item = &'a CascadeData>,
{
    fn check_outer_dependency(&mut self, _: &Dependency, _: E) -> bool {
        debug_assert!(false, "how, we should only have parent-less dependencies here!");
        true
    }

    fn collect_invalidations(
        &mut self,
        _element: E,
        self_invalidations: &mut InvalidationVector<'a>,
        _descendant_invalidations: &mut DescendantInvalidationLists<'a>,
        _sibling_invalidations: &mut InvalidationVector<'a>,
    ) -> bool {
        for cascade_data in &mut self.rules {
            let map = cascade_data.invalidation_map();
            for dependency in &map.document_state_selectors {
                if !dependency.state.intersects(self.document_states_changed) {
                    continue;
                }

                // We pass `None` as a scope, as document state selectors aren't
                // affected by the current scope.
                //
                // FIXME(emilio): We should really pass the relevant host for
                // self.rules, so that we invalidate correctly if the selector
                // happens to have something like :host(:-moz-window-inactive)
                // for example.
                self_invalidations.push(Invalidation::new(
                    &dependency.dependency,
                    /* scope = */ None,
                ));
            }
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
