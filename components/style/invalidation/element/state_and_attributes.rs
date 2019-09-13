/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An invalidation processor for style changes due to state and attribute
//! changes.

use crate::context::SharedStyleContext;
use crate::data::ElementData;
use crate::dom::TElement;
use crate::element_state::ElementState;
use crate::invalidation::element::element_wrapper::{ElementSnapshot, ElementWrapper};
use crate::invalidation::element::invalidation_map::*;
use crate::invalidation::element::invalidator::{DescendantInvalidationLists, InvalidationVector};
use crate::invalidation::element::invalidator::{Invalidation, InvalidationProcessor};
use crate::invalidation::element::restyle_hints::RestyleHint;
use crate::selector_map::SelectorMap;
use crate::selector_parser::Snapshot;
use crate::stylesheets::origin::OriginSet;
use crate::{Atom, WeakAtom};
use selectors::attr::CaseSensitivity;
use selectors::matching::matches_selector;
use selectors::matching::{MatchingContext, MatchingMode, VisitedHandlingMode};
use selectors::NthIndexCache;
use smallvec::SmallVec;

/// The collector implementation.
struct Collector<'a, 'b: 'a, 'selectors: 'a, E>
where
    E: TElement,
{
    element: E,
    wrapper: ElementWrapper<'b, E>,
    snapshot: &'a Snapshot,
    matching_context: &'a mut MatchingContext<'b, E::Impl>,
    lookup_element: E,
    removed_id: Option<&'a WeakAtom>,
    added_id: Option<&'a WeakAtom>,
    classes_removed: &'a SmallVec<[Atom; 8]>,
    classes_added: &'a SmallVec<[Atom; 8]>,
    state_changes: ElementState,
    descendant_invalidations: &'a mut DescendantInvalidationLists<'selectors>,
    sibling_invalidations: &'a mut InvalidationVector<'selectors>,
    invalidates_self: bool,
}

/// An invalidation processor for style changes due to state and attribute
/// changes.
pub struct StateAndAttrInvalidationProcessor<'a, 'b: 'a, E: TElement> {
    shared_context: &'a SharedStyleContext<'b>,
    element: E,
    data: &'a mut ElementData,
    matching_context: MatchingContext<'a, E::Impl>,
}

impl<'a, 'b: 'a, E: TElement + 'b> StateAndAttrInvalidationProcessor<'a, 'b, E> {
    /// Creates a new StateAndAttrInvalidationProcessor.
    pub fn new(
        shared_context: &'a SharedStyleContext<'b>,
        element: E,
        data: &'a mut ElementData,
        nth_index_cache: &'a mut NthIndexCache,
    ) -> Self {
        let matching_context = MatchingContext::new_for_visited(
            MatchingMode::Normal,
            None,
            Some(nth_index_cache),
            VisitedHandlingMode::AllLinksVisitedAndUnvisited,
            shared_context.quirks_mode(),
        );

        Self {
            shared_context,
            element,
            data,
            matching_context,
        }
    }
}

/// Whether we should process the descendants of a given element for style
/// invalidation.
pub fn should_process_descendants(data: &ElementData) -> bool {
    !data.styles.is_display_none() && !data.hint.contains(RestyleHint::RESTYLE_DESCENDANTS)
}

/// Propagates the bits after invalidating a descendant child.
pub fn invalidated_descendants<E>(element: E, child: E)
where
    E: TElement,
{
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

/// Sets the appropriate restyle hint after invalidating the style of a given
/// element.
pub fn invalidated_self<E>(element: E)
where
    E: TElement,
{
    if let Some(mut data) = element.mutate_data() {
        data.hint.insert(RestyleHint::RESTYLE_SELF);
    }
}

impl<'a, 'b: 'a, E: 'a> InvalidationProcessor<'a, E>
    for StateAndAttrInvalidationProcessor<'a, 'b, E>
where
    E: TElement,
{
    /// We need to invalidate style on an eager pseudo-element, in order to
    /// process changes that could otherwise end up in ::before or ::after
    /// content being generated.
    fn invalidates_on_eager_pseudo_element(&self) -> bool {
        true
    }

    fn matching_context(&mut self) -> &mut MatchingContext<'a, E::Impl> {
        &mut self.matching_context
    }

    fn collect_invalidations(
        &mut self,
        element: E,
        _self_invalidations: &mut InvalidationVector<'a>,
        descendant_invalidations: &mut DescendantInvalidationLists<'a>,
        sibling_invalidations: &mut InvalidationVector<'a>,
    ) -> bool {
        debug_assert_eq!(element, self.element);
        debug_assert!(element.has_snapshot(), "Why bothering?");

        let wrapper = ElementWrapper::new(element, &*self.shared_context.snapshot_map);

        let state_changes = wrapper.state_changes();
        let snapshot = wrapper.snapshot().expect("has_snapshot lied");

        if !snapshot.has_attrs() && state_changes.is_empty() {
            return false;
        }

        // If we the visited state changed, we force a restyle here. Matching
        // doesn't depend on the actual visited state at all, so we can't look
        // at matching results to decide what to do for this case.
        if state_changes.intersects(ElementState::IN_VISITED_OR_UNVISITED_STATE) {
            trace!(" > visitedness change, force subtree restyle");
            // We can't just return here because there may also be attribute
            // changes as well that imply additional hints for siblings.
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
            let current_id = element.id();

            if old_id != current_id {
                id_removed = old_id;
                id_added = current_id;
            }
        }

        if log_enabled!(::log::Level::Debug) {
            debug!("Collecting changes for: {:?}", element);
            if !state_changes.is_empty() {
                debug!(" > state: {:?}", state_changes);
            }
            if snapshot.id_changed() {
                debug!(" > id changed: +{:?} -{:?}", id_added, id_removed);
            }
            if snapshot.class_changed() {
                debug!(
                    " > class changed: +{:?} -{:?}",
                    classes_added, classes_removed
                );
            }
            if snapshot.other_attr_changed() {
                debug!(
                    " > attributes changed, old: {}",
                    snapshot.debug_list_attributes()
                )
            }
        }

        let lookup_element = if element.implemented_pseudo_element().is_some() {
            element.pseudo_element_originating_element().unwrap()
        } else {
            element
        };

        let mut shadow_rule_datas = SmallVec::<[_; 3]>::new();
        let matches_document_author_rules =
            element.each_applicable_non_document_style_rule_data(|data, host| {
                shadow_rule_datas.push((data, host.opaque()))
            });

        let invalidated_self = {
            let mut collector = Collector {
                wrapper,
                lookup_element,
                state_changes,
                element,
                snapshot: &snapshot,
                matching_context: &mut self.matching_context,
                removed_id: id_removed,
                added_id: id_added,
                classes_removed: &classes_removed,
                classes_added: &classes_added,
                descendant_invalidations,
                sibling_invalidations,
                invalidates_self: false,
            };

            let document_origins = if !matches_document_author_rules {
                OriginSet::ORIGIN_USER_AGENT | OriginSet::ORIGIN_USER
            } else {
                OriginSet::all()
            };

            for (cascade_data, origin) in self.shared_context.stylist.iter_origins() {
                if document_origins.contains(origin.into()) {
                    collector
                        .collect_dependencies_in_invalidation_map(cascade_data.invalidation_map());
                }
            }

            for &(ref data, ref host) in &shadow_rule_datas {
                collector.matching_context.current_host = Some(host.clone());
                collector.collect_dependencies_in_invalidation_map(data.invalidation_map());
            }

            collector.invalidates_self
        };

        // If we generated a ton of descendant invalidations, it's probably not
        // worth to go ahead and try to process them.
        //
        // Just restyle the descendants directly.
        //
        // This number is completely made-up, but the page that made us add this
        // code generated 1960+ invalidations (bug 1420741).
        //
        // We don't look at slotted_descendants because those don't propagate
        // down more than one level anyway.
        if descendant_invalidations.dom_descendants.len() > 150 {
            self.data.hint.insert(RestyleHint::RESTYLE_DESCENDANTS);
        }

        if invalidated_self {
            self.data.hint.insert(RestyleHint::RESTYLE_SELF);
        }

        invalidated_self
    }

    fn should_process_descendants(&mut self, element: E) -> bool {
        if element == self.element {
            return should_process_descendants(&self.data);
        }

        match element.borrow_data() {
            Some(d) => should_process_descendants(&d),
            None => return false,
        }
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
        invalidated_descendants(element, child)
    }

    fn invalidated_self(&mut self, element: E) {
        debug_assert_ne!(element, self.element);
        invalidated_self(element);
    }
}

impl<'a, 'b, 'selectors, E> Collector<'a, 'b, 'selectors, E>
where
    E: TElement,
    'selectors: 'a,
{
    fn collect_dependencies_in_invalidation_map(&mut self, map: &'selectors InvalidationMap) {
        let quirks_mode = self.matching_context.quirks_mode();
        let removed_id = self.removed_id;
        if let Some(ref id) = removed_id {
            if let Some(deps) = map.id_to_selector.get(id, quirks_mode) {
                for dep in deps {
                    self.scan_dependency(dep);
                }
            }
        }

        let added_id = self.added_id;
        if let Some(ref id) = added_id {
            if let Some(deps) = map.id_to_selector.get(id, quirks_mode) {
                for dep in deps {
                    self.scan_dependency(dep);
                }
            }
        }

        for class in self.classes_added.iter().chain(self.classes_removed.iter()) {
            if let Some(deps) = map.class_to_selector.get(class, quirks_mode) {
                for dep in deps {
                    self.scan_dependency(dep);
                }
            }
        }

        let should_examine_attribute_selector_map = self.snapshot.other_attr_changed() ||
            (self.snapshot.class_changed() && map.has_class_attribute_selectors) ||
            (self.snapshot.id_changed() && map.has_id_attribute_selectors);

        if should_examine_attribute_selector_map {
            self.collect_dependencies_in_map(&map.other_attribute_affecting_selectors)
        }

        let state_changes = self.state_changes;
        if !state_changes.is_empty() {
            self.collect_state_dependencies(&map.state_affecting_selectors, state_changes)
        }
    }

    fn collect_dependencies_in_map(&mut self, map: &'selectors SelectorMap<Dependency>) {
        map.lookup_with_additional(
            self.lookup_element,
            self.matching_context.quirks_mode(),
            self.removed_id,
            self.classes_removed,
            |dependency| {
                self.scan_dependency(dependency);
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
            self.matching_context.quirks_mode(),
            self.removed_id,
            self.classes_removed,
            |dependency| {
                if !dependency.state.intersects(state_changes) {
                    return true;
                }
                self.scan_dependency(&dependency.dep);
                true
            },
        );
    }

    /// Check whether a dependency should be taken into account.
    fn check_dependency(&mut self, dependency: &Dependency) -> bool {
        let element = &self.element;
        let wrapper = &self.wrapper;
        let matches_now = matches_selector(
            &dependency.selector,
            dependency.selector_offset,
            None,
            element,
            &mut self.matching_context,
            &mut |_, _| {},
        );

        let matched_then = matches_selector(
            &dependency.selector,
            dependency.selector_offset,
            None,
            wrapper,
            &mut self.matching_context,
            &mut |_, _| {},
        );

        matched_then != matches_now
    }

    fn scan_dependency(&mut self, dependency: &'selectors Dependency) {
        debug!(
            "TreeStyleInvalidator::scan_dependency({:?}, {:?})",
            self.element, dependency
        );

        if !self.dependency_may_be_relevant(dependency) {
            return;
        }

        if self.check_dependency(dependency) {
            return self.note_dependency(dependency);
        }
    }

    fn note_dependency(&mut self, dependency: &'selectors Dependency) {
        debug_assert!(self.dependency_may_be_relevant(dependency));

        let invalidation_kind = dependency.invalidation_kind();
        if matches!(invalidation_kind, DependencyInvalidationKind::Element) {
            self.invalidates_self = true;
            return;
        }

        debug_assert_ne!(dependency.selector_offset, 0);
        debug_assert_ne!(dependency.selector_offset, dependency.selector.len());

        let invalidation = Invalidation::new(
            &dependency.selector,
            dependency.selector.len() - dependency.selector_offset + 1,
        );

        match invalidation_kind {
            DependencyInvalidationKind::Element => unreachable!(),
            DependencyInvalidationKind::ElementAndDescendants => {
                self.invalidates_self = true;
                self.descendant_invalidations
                    .dom_descendants
                    .push(invalidation);
            },
            DependencyInvalidationKind::Descendants => {
                self.descendant_invalidations
                    .dom_descendants
                    .push(invalidation);
            },
            DependencyInvalidationKind::Siblings => {
                self.sibling_invalidations.push(invalidation);
            },
            DependencyInvalidationKind::Parts => {
                self.descendant_invalidations.parts.push(invalidation);
            },
            DependencyInvalidationKind::SlottedElements => {
                self.descendant_invalidations
                    .slotted_descendants
                    .push(invalidation);
            },
        }
    }

    /// Returns whether `dependency` may cause us to invalidate the style of
    /// more elements than what we've already invalidated.
    fn dependency_may_be_relevant(&self, dependency: &Dependency) -> bool {
        match dependency.invalidation_kind() {
            DependencyInvalidationKind::Element => !self.invalidates_self,
            DependencyInvalidationKind::SlottedElements => self.element.is_html_slot_element(),
            DependencyInvalidationKind::Parts => self.element.shadow_root().is_some(),
            DependencyInvalidationKind::ElementAndDescendants |
            DependencyInvalidationKind::Siblings |
            DependencyInvalidationKind::Descendants => true,
        }
    }
}
