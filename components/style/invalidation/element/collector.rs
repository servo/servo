/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An invalidation processor for style changes due to state and attribute
//! changes.

use Atom;
use atomic_refcell::AtomicRef;
use context::{QuirksMode, SharedStyleContext};
use data::ElementData;
use dom::TElement;
use element_state::ElementState;
use invalidation::element::element_wrapper::{ElementSnapshot, ElementWrapper};
use invalidation::element::invalidation_map::*;
use invalidation::element::invalidator::{InvalidationVector, Invalidation, InvalidationProcessor};
use invalidation::element::restyle_hints::RestyleHint;
use selector_map::SelectorMap;
use selector_parser::Snapshot;
use selectors::NthIndexCache;
use selectors::attr::CaseSensitivity;
use selectors::matching::{MatchingContext, MatchingMode, VisitedHandlingMode};
use selectors::matching::matches_selector;
use smallvec::SmallVec;
use stylesheets::origin::{Origin, OriginSet};
use stylist::Stylist;

#[derive(Debug, PartialEq)]
enum VisitedDependent {
    Yes,
    No,
}

/// The collector implementation.
struct Collector<'a, 'b: 'a, 'selectors: 'a, E>
where
    E: TElement,
{
    element: E,
    wrapper: ElementWrapper<'b, E>,
    nth_index_cache: Option<&'a mut NthIndexCache>,
    snapshot: &'a Snapshot,
    quirks_mode: QuirksMode,
    lookup_element: E,
    removed_id: Option<&'a Atom>,
    added_id: Option<&'a Atom>,
    classes_removed: &'a SmallVec<[Atom; 8]>,
    classes_added: &'a SmallVec<[Atom; 8]>,
    state_changes: ElementState,
    descendant_invalidations: &'a mut InvalidationVector<'selectors>,
    sibling_invalidations: &'a mut InvalidationVector<'selectors>,
    invalidates_self: bool,
}

/// An invalidation processor for style changes due to state and attribute
/// changes.
pub struct StateAndAttrInvalidationProcessor<'a, 'b: 'a, E: TElement> {
    shared_context: &'a SharedStyleContext<'b>,
    xbl_stylists: &'a [AtomicRef<'b, Stylist>],
    cut_off_inheritance: bool,
    element: E,
    data: &'a mut ElementData,
    matching_context: MatchingContext<'a, E::Impl>,
}

impl<'a, 'b: 'a, E: TElement> StateAndAttrInvalidationProcessor<'a, 'b, E> {
    /// Creates a new StateAndAttrInvalidationProcessor.
    pub fn new(
        shared_context: &'a SharedStyleContext<'b>,
        xbl_stylists: &'a [AtomicRef<'b, Stylist>],
        cut_off_inheritance: bool,
        element: E,
        data: &'a mut ElementData,
        nth_index_cache: Option<&'a mut NthIndexCache>,
    ) -> Self {
        let matching_context = MatchingContext::new_for_visited(
            MatchingMode::Normal,
            None,
            nth_index_cache,
            VisitedHandlingMode::AllLinksVisitedAndUnvisited,
            shared_context.quirks_mode(),
        );

        Self {
            shared_context,
            xbl_stylists,
            cut_off_inheritance,
            element,
            data,
            matching_context,
        }
    }
}

impl<'a, 'b: 'a, E: 'a> InvalidationProcessor<'a, E> for StateAndAttrInvalidationProcessor<'a, 'b, E>
where
    E: TElement,
{
    /// We need to invalidate style on an eager pseudo-element, in order to
    /// process changes that could otherwise end up in ::before or ::after
    /// content being generated.
    fn invalidates_on_eager_pseudo_element(&self) -> bool { true }

    fn matching_context(&mut self) -> &mut MatchingContext<'a, E::Impl> {
        &mut self.matching_context
    }

    fn collect_invalidations(
        &mut self,
        element: E,
        _self_invalidations: &mut InvalidationVector<'a>,
        descendant_invalidations: &mut InvalidationVector<'a>,
        sibling_invalidations: &mut InvalidationVector<'a>,
    ) -> bool {
        debug_assert!(element.has_snapshot(), "Why bothering?");

        let wrapper =
            ElementWrapper::new(element, &*self.shared_context.snapshot_map);

        let state_changes = wrapper.state_changes();
        let snapshot = wrapper.snapshot().expect("has_snapshot lied");

        if !snapshot.has_attrs() && state_changes.is_empty() {
            return false;
        }

        // If we are sensitive to visitedness and the visited state changed, we
        // force a restyle here. Matching doesn't depend on the actual visited
        // state at all, so we can't look at matching results to decide what to
        // do for this case.
        if state_changes.intersects(ElementState::IN_VISITED_OR_UNVISITED_STATE) {
            trace!(" > visitedness change, force subtree restyle");
            // We can't just return here because there may also be attribute
            // changes as well that imply additional hints.
            self.data.hint.insert(RestyleHint::restyle_subtree());
        }

        let mut classes_removed = SmallVec::<[Atom; 8]>::new();
        let mut classes_added = SmallVec::<[Atom; 8]>::new();
        if snapshot.class_changed() {
            // TODO(emilio): Do this more efficiently!
            snapshot.each_class(|c| {
                if !element.has_class(c, CaseSensitivity::CaseSensitive) {
                    classes_removed.push(c.clone())
                }
            });

            element.each_class(|c| {
                if !snapshot.has_class(c, CaseSensitivity::CaseSensitive) {
                    classes_added.push(c.clone())
                }
            })
        }

        let mut id_removed = None;
        let mut id_added = None;
        if snapshot.id_changed() {
            let old_id = snapshot.id_attr();
            let current_id = element.get_id();

            if old_id != current_id {
                id_removed = old_id;
                id_added = current_id;
            }
        }

        debug!("Collecting changes for: {:?}", element);
        debug!(" > state: {:?}", state_changes);
        debug!(
            " > id changed: {:?} -> +{:?} -{:?}",
            snapshot.id_changed(),
            id_added,
            id_removed
        );
        debug!(
            " > class changed: {:?} -> +{:?} -{:?}",
            snapshot.class_changed(),
            classes_added,
            classes_removed
        );

        let lookup_element =
            if element.implemented_pseudo_element().is_some() {
                element.pseudo_element_originating_element().unwrap()
            } else {
                element
            };

        let invalidated_self = {
            let mut collector = Collector {
                wrapper,
                lookup_element,
                state_changes,
                element,
                snapshot: &snapshot,
                quirks_mode: self.shared_context.quirks_mode(),
                nth_index_cache: self.matching_context.nth_index_cache.as_mut().map(|c| &mut **c),
                removed_id: id_removed.as_ref(),
                added_id: id_added.as_ref(),
                classes_removed: &classes_removed,
                classes_added: &classes_added,
                descendant_invalidations,
                sibling_invalidations,
                invalidates_self: false,
            };

            let document_origins = if self.cut_off_inheritance {
                Origin::UserAgent.into()
            } else {
                OriginSet::all()
            };

            self.shared_context.stylist.each_invalidation_map(|invalidation_map, origin| {
                if document_origins.contains(origin.into()) {
                    collector.collect_dependencies_in_invalidation_map(invalidation_map);
                }
            });

            for stylist in self.xbl_stylists {
                // FIXME(emilio): Replace with assert / remove when we
                // figure out what to do with the quirks mode mismatches
                // (that is, when bug 1406875 is properly fixed).
                collector.quirks_mode = stylist.quirks_mode();
                stylist.each_invalidation_map(|invalidation_map, _| {
                    collector.collect_dependencies_in_invalidation_map(invalidation_map);
                })
            }

            collector.invalidates_self
        };

        if invalidated_self {
            self.data.hint.insert(RestyleHint::RESTYLE_SELF);
        }

        invalidated_self
    }

    fn should_process_descendants(&mut self, element: E) -> bool {
        if element == self.element {
            return !self.data.styles.is_display_none() &&
                !self.data.hint.contains(RestyleHint::RESTYLE_DESCENDANTS)
        }

        let data = match element.borrow_data() {
            None => return false,
            Some(data) => data,
        };

        !data.styles.is_display_none() &&
            !data.hint.contains(RestyleHint::RESTYLE_DESCENDANTS)
    }

    fn recursion_limit_exceeded(&mut self, element: E) {
        if element == self.element {
            self.data.hint.insert(RestyleHint::RESTYLE_DESCENDANTS);
            return;
        }

        if let Some(mut data) = element.mutate_data() {
            data.hint.insert(RestyleHint::RESTYLE_DESCENDANTS);
        }
    }

    fn invalidated_descendants(&mut self, element: E, child: E) {
        if child.get_data().is_none() {
            return;
        }

        // The child may not be a flattened tree child of the current element,
        // but may be arbitrarily deep.
        //
        // Since we keep the traversal flags in terms of the flattened tree,
        // we need to propagate it as appropriate.
        let mut current = child.traversal_parent();
        while let Some(parent) = current.take() {
            unsafe { parent.set_dirty_descendants() };
            current = parent.traversal_parent();

            if parent == element {
                break;
            }
        }
    }

    fn invalidated_self(&mut self, element: E) {
        debug_assert_ne!(element, self.element);
        if let Some(mut data) = element.mutate_data() {
            data.hint.insert(RestyleHint::RESTYLE_SELF);
        }
    }
}

impl<'a, 'b, 'selectors, E> Collector<'a, 'b, 'selectors, E>
where
    E: TElement,
    'selectors: 'a,
{
    fn collect_dependencies_in_invalidation_map(
        &mut self,
        map: &'selectors InvalidationMap,
    ) {
        let quirks_mode = self.quirks_mode;
        let removed_id = self.removed_id;
        if let Some(ref id) = removed_id {
            if let Some(deps) = map.id_to_selector.get(id, quirks_mode) {
                for dep in deps {
                    self.scan_dependency(dep, VisitedDependent::No);
                }
            }
        }

        let added_id = self.added_id;
        if let Some(ref id) = added_id {
            if let Some(deps) = map.id_to_selector.get(id, quirks_mode) {
                for dep in deps {
                    self.scan_dependency(dep, VisitedDependent::No);
                }
            }
        }

        for class in self.classes_added.iter().chain(self.classes_removed.iter()) {
            if let Some(deps) = map.class_to_selector.get(class, quirks_mode) {
                for dep in deps {
                    self.scan_dependency(dep, VisitedDependent::No);
                }
            }
        }

        let should_examine_attribute_selector_map =
            self.snapshot.other_attr_changed() ||
            (self.snapshot.class_changed() && map.has_class_attribute_selectors) ||
            (self.snapshot.id_changed() && map.has_id_attribute_selectors);

        if should_examine_attribute_selector_map {
            self.collect_dependencies_in_map(
                &map.other_attribute_affecting_selectors
            )
        }

        let state_changes = self.state_changes;
        if !state_changes.is_empty() {
            self.collect_state_dependencies(
                &map.state_affecting_selectors,
                state_changes,
            )
        }
    }

    fn collect_dependencies_in_map(
        &mut self,
        map: &'selectors SelectorMap<Dependency>,
    ) {
        map.lookup_with_additional(
            self.lookup_element,
            self.quirks_mode,
            self.removed_id,
            self.classes_removed,
            |dependency| {
                self.scan_dependency(dependency, VisitedDependent::No);
                true
            },
        );
    }

    fn collect_state_dependencies(
        &mut self,
        map: &'selectors SelectorMap<StateDependency>,
        state_changes: ElementState,
    ) {
        map.lookup_with_additional(
            self.lookup_element,
            self.quirks_mode,
            self.removed_id,
            self.classes_removed,
            |dependency| {
                if !dependency.state.intersects(state_changes) {
                    return true;
                }
                let visited_dependent =
                    if dependency.state.intersects(ElementState::IN_VISITED_OR_UNVISITED_STATE) {
                        VisitedDependent::Yes
                    } else {
                        VisitedDependent::No
                    };
                self.scan_dependency(&dependency.dep, visited_dependent);
                true
            },
        );
    }

    /// Check whether a dependency should be taken into account, using a given
    /// visited handling mode.
    fn check_dependency(
        &mut self,
        visited_handling_mode: VisitedHandlingMode,
        dependency: &Dependency,
        relevant_link_found: &mut bool,
    ) -> bool {
        let (matches_now, relevant_link_found_now) = {
            let mut context = MatchingContext::new_for_visited(
                MatchingMode::Normal,
                None,
                self.nth_index_cache.as_mut().map(|c| &mut **c),
                visited_handling_mode,
                self.quirks_mode,
            );

            let matches_now = matches_selector(
                &dependency.selector,
                dependency.selector_offset,
                None,
                &self.element,
                &mut context,
                &mut |_, _| {},
            );

            (matches_now, context.relevant_link_found)
        };

        let (matched_then, relevant_link_found_then) = {
            let mut context = MatchingContext::new_for_visited(
                MatchingMode::Normal,
                None,
                self.nth_index_cache.as_mut().map(|c| &mut **c),
                visited_handling_mode,
                self.quirks_mode,
            );

            let matched_then = matches_selector(
                &dependency.selector,
                dependency.selector_offset,
                None,
                &self.wrapper,
                &mut context,
                &mut |_, _| {},
            );

            (matched_then, context.relevant_link_found)
        };

        *relevant_link_found = relevant_link_found_now;

        // Check for mismatches in both the match result and also the status
        // of whether a relevant link was found.
        matched_then != matches_now ||
            relevant_link_found_now != relevant_link_found_then
    }

    fn scan_dependency(
        &mut self,
        dependency: &'selectors Dependency,
        is_visited_dependent: VisitedDependent,
    ) {
        debug!("TreeStyleInvalidator::scan_dependency({:?}, {:?}, {:?})",
               self.element,
               dependency,
               is_visited_dependent);

        if !self.dependency_may_be_relevant(dependency) {
            return;
        }

        let mut relevant_link_found = false;

        let should_account_for_dependency = self.check_dependency(
            VisitedHandlingMode::AllLinksUnvisited,
            dependency,
            &mut relevant_link_found,
        );

        if should_account_for_dependency {
            return self.note_dependency(dependency);
        }

        // If there is a relevant link, then we also matched in visited
        // mode.
        //
        // Match again in this mode to ensure this also matches.
        //
        // Note that we never actually match directly against the element's true
        // visited state at all, since that would expose us to timing attacks.
        //
        // The matching process only considers the relevant link state and
        // visited handling mode when deciding if visited matches.  Instead, we
        // are rematching here in case there is some :visited selector whose
        // matching result changed for some other state or attribute change of
        // this element (for example, for things like [foo]:visited).
        //
        // NOTE: This thing is actually untested because testing it is flaky,
        // see the tests that were added and then backed out in bug 1328509.
        if is_visited_dependent == VisitedDependent::Yes && relevant_link_found {
            let should_account_for_dependency = self.check_dependency(
                VisitedHandlingMode::RelevantLinkVisited,
                dependency,
                &mut false,
            );

            if should_account_for_dependency {
                return self.note_dependency(dependency);
            }
        }
    }

    fn note_dependency(&mut self, dependency: &'selectors Dependency) {
        if dependency.affects_self() {
            self.invalidates_self = true;
        }

        if dependency.affects_descendants() {
            debug_assert_ne!(dependency.selector_offset, 0);
            debug_assert_ne!(dependency.selector_offset, dependency.selector.len());
            debug_assert!(!dependency.affects_later_siblings());
            self.descendant_invalidations.push(Invalidation::new(
                &dependency.selector,
                dependency.selector.len() - dependency.selector_offset + 1,
            ));
        } else if dependency.affects_later_siblings() {
            debug_assert_ne!(dependency.selector_offset, 0);
            debug_assert_ne!(dependency.selector_offset, dependency.selector.len());
            self.sibling_invalidations.push(Invalidation::new(
                &dependency.selector,
                dependency.selector.len() - dependency.selector_offset + 1,
            ));
        }
    }

    /// Returns whether `dependency` may cause us to invalidate the style of
    /// more elements than what we've already invalidated.
    fn dependency_may_be_relevant(&self, dependency: &Dependency) -> bool {
        if dependency.affects_descendants() || dependency.affects_later_siblings() {
            return true;
        }

        debug_assert!(dependency.affects_self());
        !self.invalidates_self
    }
}
