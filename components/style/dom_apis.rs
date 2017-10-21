/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic implementations of some DOM APIs so they can be shared between Servo
//! and Gecko.

use context::QuirksMode;
use dom::{TElement, TNode};
use invalidation::element::invalidator::{Invalidation, InvalidationProcessor, InvalidationVector};
use selectors::{Element, NthIndexCache, SelectorList};
use selectors::matching::{self, MatchingContext, MatchingMode};
use smallvec::SmallVec;

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

/// The result of a querySelector call.
pub type QuerySelectorResult<E> = SmallVec<[E; 128]>;

/// The query kind we're doing (either only the first descendant that matches or
/// all of them).
pub enum QuerySelectorKind {
    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall>
    All,
    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselector>
    First,
}

struct QuerySelectorProcessor<'a, E: TElement + 'a> {
    kind: QuerySelectorKind,
    results: &'a mut QuerySelectorResult<E>,
    matching_context: MatchingContext<'a, E::Impl>,
    selector_list: &'a SelectorList<E::Impl>,
}

impl<'a, E> InvalidationProcessor<'a, E> for QuerySelectorProcessor<'a, E>
where
    E: TElement + 'a,
{
    fn collect_invalidations(
        &mut self,
        _element: E,
        self_invalidations: &mut InvalidationVector<'a>,
        descendant_invalidations: &mut InvalidationVector<'a>,
        _sibling_invalidations: &mut InvalidationVector<'a>,
    ) -> bool {
        // FIXME(emilio): If the element is not a root element, and
        // selector_list has any descendant combinator, we need to do extra work
        // in order to handle properly things like:
        //
        //   <div id="a">
        //     <div id="b">
        //       <div id="c"></div>
        //     </div>
        //   </div>
        //
        // b.querySelector('#a div'); // Should return "c".
        //
        let target_vector =
            if self.matching_context.scope_element.is_some() {
                descendant_invalidations
            } else {
                self_invalidations
            };

        for selector in self.selector_list.0.iter() {
            target_vector.push(Invalidation::new(selector, 0))
        }

        false
    }

    fn matching_context(&mut self) -> &mut MatchingContext<'a, E::Impl> {
        &mut self.matching_context
    }

    fn should_process_descendants(&mut self, _: E) -> bool {
        match self.kind {
            QuerySelectorKind::All => true,
            QuerySelectorKind::First => self.results.is_empty(),
        }
    }

    fn invalidated_self(&mut self, e: E) {
        self.results.push(e);
    }

    fn recursion_limit_exceeded(&mut self, _e: E) {}
    fn invalidated_descendants(&mut self, _e: E, _child: E) {}
}

/// <https://dom.spec.whatwg.org/#dom-parentnode-queryselector>
pub fn query_selector<E: TElement>(
    root: E::ConcreteNode,
    selector_list: &SelectorList<E::Impl>,
    results: &mut QuerySelectorResult<E>,
    kind: QuerySelectorKind,
    quirks_mode: QuirksMode,
) {
    use invalidation::element::invalidator::TreeStyleInvalidator;

    let mut nth_index_cache = NthIndexCache::default();
    let mut matching_context = MatchingContext::new(
        MatchingMode::Normal,
        None,
        Some(&mut nth_index_cache),
        quirks_mode,
    );

    let root_element = root.as_element();
    matching_context.scope_element = root_element.map(|e| e.opaque());

    let mut processor = QuerySelectorProcessor {
        kind,
        results,
        matching_context,
        selector_list,
    };

    match root_element {
        Some(e) => {
            TreeStyleInvalidator::new(
                e,
                /* stack_limit_checker = */ None,
                &mut processor,
            ).invalidate();
        }
        None => {
            for node in root.dom_children() {
                if let Some(e) = node.as_element() {
                    TreeStyleInvalidator::new(
                        e,
                        /* stack_limit_checker = */ None,
                        &mut processor,
                    ).invalidate();
                }
            }
        }
    }
}
