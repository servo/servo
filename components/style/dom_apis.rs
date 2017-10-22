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

/// A selector query abstraction, in order to be generic over QuerySelector and
/// QuerySelectorAll.
pub trait SelectorQuery<E: TElement> {
    /// The output of the query.
    type Output;

    /// Whether the query should stop after the first element has been matched.
    fn should_stop_after_first_match() -> bool;

    /// Append an element matching after the first query.
    fn append_element(output: &mut Self::Output, element: E);

    /// Returns true if the output is empty.
    fn is_empty(output: &Self::Output) -> bool;
}

/// The result of a querySelectorAll call.
pub type QuerySelectorAllResult<E> = SmallVec<[E; 128]>;

/// A query for all the elements in a subtree.
pub struct QueryAll;

impl<E: TElement> SelectorQuery<E> for QueryAll {
    type Output = QuerySelectorAllResult<E>;

    fn should_stop_after_first_match() -> bool { false }

    fn append_element(output: &mut Self::Output, element: E) {
        output.push(element);
    }

    fn is_empty(output: &Self::Output) -> bool {
        output.is_empty()
    }
}

/// A query for the first in-tree match of all the elements in a subtree.
pub struct QueryFirst;

impl<E: TElement> SelectorQuery<E> for QueryFirst {
    type Output = Option<E>;

    fn should_stop_after_first_match() -> bool { true }

    fn append_element(output: &mut Self::Output, element: E) {
        if output.is_none() {
            *output = Some(element)
        }
    }

    fn is_empty(output: &Self::Output) -> bool {
        output.is_none()
    }
}

struct QuerySelectorProcessor<'a, E, Q>
where
    E: TElement + 'a,
    Q: SelectorQuery<E>,
    Q::Output: 'a,
{
    results: &'a mut Q::Output,
    matching_context: MatchingContext<'a, E::Impl>,
    selector_list: &'a SelectorList<E::Impl>,
}

impl<'a, E, Q> InvalidationProcessor<'a, E> for QuerySelectorProcessor<'a, E, Q>
where
    E: TElement + 'a,
    Q: SelectorQuery<E>,
    Q::Output: 'a,
{
    fn light_tree_only(&self) -> bool { true }

    fn collect_invalidations(
        &mut self,
        element: E,
        self_invalidations: &mut InvalidationVector<'a>,
        descendant_invalidations: &mut InvalidationVector<'a>,
        _sibling_invalidations: &mut InvalidationVector<'a>,
    ) -> bool {
        // TODO(emilio): If the element is not a root element, and
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
        // For now, assert it's a root element.
        debug_assert!(element.parent_element().is_none());

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
        if Q::should_stop_after_first_match() {
            return Q::is_empty(&self.results)
        }

        true
    }

    fn invalidated_self(&mut self, e: E) {
        Q::append_element(self.results, e);
    }

    fn recursion_limit_exceeded(&mut self, _e: E) {}
    fn invalidated_descendants(&mut self, _e: E, _child: E) {}
}

fn collect_all_elements<E, Q, F>(
    root: E::ConcreteNode,
    results: &mut Q::Output,
    mut filter: F,
)
where
    E: TElement,
    Q: SelectorQuery<E>,
    F: FnMut(E) -> bool,
{
    for node in root.dom_descendants() {
        let element = match node.as_element() {
            Some(e) => e,
            None => continue,
        };

        if !filter(element) {
            continue;
        }

        Q::append_element(results, element);
        if Q::should_stop_after_first_match() {
            return;
        }
    }
}

/// Fast paths for a given selector query.
///
/// FIXME(emilio, nbp): This may very well be a good candidate for code to be
/// replaced by HolyJit :)
fn query_selector_fast<E, Q>(
    root: E::ConcreteNode,
    selector_list: &SelectorList<E::Impl>,
    results: &mut Q::Output,
    quirks_mode: QuirksMode,
) -> Result<(), ()>
where
    E: TElement,
    Q: SelectorQuery<E>,
{
    use selectors::parser::{Component, LocalName};
    use std::borrow::Borrow;

    // We need to return elements in document order, and reordering them
    // afterwards is kinda silly.
    if selector_list.0.len() > 1 {
        return Err(());
    }

    let selector = &selector_list.0[0];

    // Let's just care about the easy cases for now.
    //
    // FIXME(emilio): Blink has a fast path for classes in ancestor combinators
    // that may be worth stealing.
    if selector.len() > 1 {
        return Err(());
    }

    let component = selector.iter().next().unwrap();
    match *component {
        Component::ExplicitUniversalType => {
            collect_all_elements::<E, Q, _>(root, results, |_| true)
        }
        Component::ID(ref id) => {
            // TODO(emilio): We may want to reuse Gecko's document ID table.
            let case_sensitivity = quirks_mode.classes_and_ids_case_sensitivity();
            collect_all_elements::<E, Q, _>(root, results, |element| {
                element.has_id(id, case_sensitivity)
            })
        }
        Component::Class(ref class) => {
            let case_sensitivity = quirks_mode.classes_and_ids_case_sensitivity();
            collect_all_elements::<E, Q, _>(root, results, |element| {
                element.has_class(class, case_sensitivity)
            })
        }
        Component::LocalName(LocalName { ref name, ref lower_name }) => {
            collect_all_elements::<E, Q, _>(root, results, |element| {
                if element.is_html_element_in_html_document() {
                    element.get_local_name() == lower_name.borrow()
                } else {
                    element.get_local_name() == name.borrow()
                }
            })
        }
        // TODO(emilio): More fast paths?
        _ => {
            return Err(())
        }
    }

    Ok(())
}

// Slow path for a given selector query.
fn query_selector_slow<E, Q>(
    root: E::ConcreteNode,
    selector_list: &SelectorList<E::Impl>,
    results: &mut Q::Output,
    matching_context: &mut MatchingContext<E::Impl>,
)
where
    E: TElement,
    Q: SelectorQuery<E>,
{
    collect_all_elements::<E, Q, _>(root, results, |element| {
        matching::matches_selector_list(selector_list, &element, matching_context)
    });
}

/// <https://dom.spec.whatwg.org/#dom-parentnode-queryselector>
pub fn query_selector<E, Q>(
    root: E::ConcreteNode,
    selector_list: &SelectorList<E::Impl>,
    results: &mut Q::Output,
    quirks_mode: QuirksMode,
)
where
    E: TElement,
    Q: SelectorQuery<E>,
{
    use invalidation::element::invalidator::TreeStyleInvalidator;

    let fast_result = query_selector_fast::<E, Q>(
        root,
        selector_list,
        results,
        quirks_mode,
    );

    if fast_result.is_ok() {
        return;
    }

    // Slow path: Use the invalidation machinery if we're a root, and tree
    // traversal otherwise.
    //
    // See the comment in collect_invalidations to see why only if we're a root.
    let mut nth_index_cache = NthIndexCache::default();
    let mut matching_context = MatchingContext::new(
        MatchingMode::Normal,
        None,
        Some(&mut nth_index_cache),
        quirks_mode,
    );

    let root_element = root.as_element();
    matching_context.scope_element = root_element.map(|e| e.opaque());

    // The invalidation mechanism is only useful in presence of combinators.
    //
    // We could do that check properly here, though checking the length of the
    // selectors is a good heuristic.
    let invalidation_may_be_useful =
        selector_list.0.iter().any(|s| s.len() > 1);

    if root_element.is_some() || !invalidation_may_be_useful {
        query_selector_slow::<E, Q>(
            root,
            selector_list,
            results,
            &mut matching_context,
        );
    } else {
        let mut processor = QuerySelectorProcessor::<E, Q> {
            results,
            matching_context,
            selector_list,
        };

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
