/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The struct that takes care of encapsulating all the logic on where and how
//! element styles need to be invalidated.

use Atom;
use context::SharedStyleContext;
use data::ElementData;
use dom::{TElement, TNode};
use element_state::{ElementState, IN_VISITED_OR_UNVISITED_STATE};
use invalidation::element::element_wrapper::{ElementSnapshot, ElementWrapper};
use invalidation::element::invalidation_map::*;
use invalidation::element::restyle_hints::*;
use selector_map::SelectorMap;
use selector_parser::SelectorImpl;
use selectors::attr::CaseSensitivity;
use selectors::matching::{MatchingContext, MatchingMode, VisitedHandlingMode};
use selectors::matching::{matches_selector, matches_compound_selector};
use selectors::matching::CompoundSelectorMatchingResult;
use selectors::parser::{Combinator, Component, Selector};
use smallvec::SmallVec;
use std::fmt;

/// The struct that takes care of encapsulating all the logic on where and how
/// element styles need to be invalidated.
pub struct TreeStyleInvalidator<'a, 'b: 'a, E>
    where E: TElement,
{
    element: E,
    data: Option<&'a mut ElementData>,
    shared_context: &'a SharedStyleContext<'b>,
}

type InvalidationVector = SmallVec<[Invalidation; 10]>;

/// An `Invalidation` is a complex selector that describes which elements,
/// relative to a current element we are processing, must be restyled.
///
/// When `offset` points to the right-most compound selector in `selector`,
/// then the Invalidation `represents` the fact that the current element
/// must be restyled if the compound selector matches.  Otherwise, if
/// describes which descendants (or later siblings) must be restyled.
#[derive(Clone)]
struct Invalidation {
    selector: Selector<SelectorImpl>,
    offset: usize,
}

impl fmt::Debug for Invalidation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use cssparser::ToCss;

        f.write_str("Invalidation(")?;
        for component in self.selector.iter_raw_rev_from(self.offset - 1) {
            if matches!(*component, Component::Combinator(..)) {
                break;
            }
            component.to_css(f)?;
        }
        f.write_str(")")
    }
}

/// The result of processing a single invalidation for a given element.
struct InvalidationResult {
    /// Whether the element itself was invalidated.
    invalidated_self: bool,
    /// Whether the invalidation we've processed is effective for the next
    /// sibling or descendant after us.
    effective_for_next: bool,
}

impl<'a, 'b: 'a, E> TreeStyleInvalidator<'a, 'b, E>
    where E: TElement,
{
    /// Trivially constructs a new `TreeStyleInvalidator`.
    pub fn new(
        element: E,
        data: Option<&'a mut ElementData>,
        shared_context: &'a SharedStyleContext<'b>,
    ) -> Self {
        Self {
            element: element,
            data: data,
            shared_context: shared_context,
        }
    }

    /// Perform the invalidation pass.
    pub fn invalidate(mut self) {
        debug!("StyleTreeInvalidator::invalidate({:?})", self.element);
        debug_assert!(self.element.has_snapshot(), "Why bothering?");
        debug_assert!(self.data.is_some(), "How exactly?");

        let shared_context = self.shared_context;

        let wrapper =
            ElementWrapper::new(self.element, shared_context.snapshot_map);
        let state_changes = wrapper.state_changes();
        let snapshot = wrapper.snapshot().expect("has_snapshot lied");

        if !snapshot.has_attrs() && state_changes.is_empty() {
            return;
        }

        // If we are sensitive to visitedness and the visited state changed, we
        // force a restyle here. Matching doesn't depend on the actual visited
        // state at all, so we can't look at matching results to decide what to
        // do for this case.
        if state_changes.intersects(IN_VISITED_OR_UNVISITED_STATE) {
            trace!(" > visitedness change, force subtree restyle");
            // We can't just return here because there may also be attribute
            // changes as well that imply additional hints.
            let mut data = self.data.as_mut().unwrap();
            data.restyle.hint.insert(RestyleHint::restyle_subtree());
        }

        let mut classes_removed = SmallVec::<[Atom; 8]>::new();
        let mut classes_added = SmallVec::<[Atom; 8]>::new();
        if snapshot.class_changed() {
            // TODO(emilio): Do this more efficiently!
            snapshot.each_class(|c| {
                if !self.element.has_class(c, CaseSensitivity::CaseSensitive) {
                    classes_removed.push(c.clone())
                }
            });

            self.element.each_class(|c| {
                if !snapshot.has_class(c, CaseSensitivity::CaseSensitive) {
                    classes_added.push(c.clone())
                }
            })
        }

        let mut id_removed = None;
        let mut id_added = None;
        if snapshot.id_changed() {
            let old_id = snapshot.id_attr();
            let current_id = self.element.get_id();

            if old_id != current_id {
                id_removed = old_id;
                id_added = current_id;
            }
        }

        let lookup_element =
            if self.element.implemented_pseudo_element().is_some() {
                self.element.pseudo_element_originating_element().unwrap()
            } else {
                self.element
            };

        let mut descendant_invalidations = InvalidationVector::new();
        let mut sibling_invalidations = InvalidationVector::new();
        let invalidated_self = {
            let mut collector = InvalidationCollector {
                wrapper: wrapper,
                element: self.element,
                shared_context: self.shared_context,
                lookup_element: lookup_element,
                removed_id: id_removed.as_ref(),
                classes_removed: &classes_removed,
                descendant_invalidations: &mut descendant_invalidations,
                sibling_invalidations: &mut sibling_invalidations,
                invalidates_self: false,
            };

            let map = shared_context.stylist.invalidation_map();

            if let Some(ref id) = id_removed {
                if let Some(deps) = map.id_to_selector.get(id, shared_context.quirks_mode) {
                    collector.collect_dependencies_in_map(deps)
                }
            }

            if let Some(ref id) = id_added {
                if let Some(deps) = map.id_to_selector.get(id, shared_context.quirks_mode) {
                    collector.collect_dependencies_in_map(deps)
                }
            }

            for class in classes_added.iter().chain(classes_removed.iter()) {
                if let Some(deps) = map.class_to_selector.get(class, shared_context.quirks_mode) {
                    collector.collect_dependencies_in_map(deps)
                }
            }

            let should_examine_attribute_selector_map =
                snapshot.other_attr_changed() ||
                (snapshot.class_changed() && map.has_class_attribute_selectors) ||
                (snapshot.id_changed() && map.has_id_attribute_selectors);

            if should_examine_attribute_selector_map {
                collector.collect_dependencies_in_map(
                    &map.other_attribute_affecting_selectors
                )
            }

            if !state_changes.is_empty() {
                collector.collect_state_dependencies(
                    &map.state_affecting_selectors,
                    state_changes,
                )
            }

            collector.invalidates_self
        };

        if invalidated_self {
            if let Some(ref mut data) = self.data {
                data.restyle.hint.insert(RESTYLE_SELF);
            }
        }

        debug!("Collected invalidations (self: {}): ", invalidated_self);
        debug!(" > descendants: {:?}", descendant_invalidations);
        debug!(" > siblings: {:?}", sibling_invalidations);
        self.invalidate_descendants(&descendant_invalidations);
        self.invalidate_siblings(&mut sibling_invalidations);
    }

    /// Go through later DOM siblings, invalidating style as needed using the
    /// `sibling_invalidations` list.
    ///
    /// Returns whether any sibling's style or any sibling descendant's style
    /// was invalidated.
    fn invalidate_siblings(
        &mut self,
        sibling_invalidations: &mut InvalidationVector,
    ) -> bool {
        if sibling_invalidations.is_empty() {
            return false;
        }

        let mut current = self.element.next_sibling_element();
        let mut any_invalidated = false;

        while let Some(sibling) = current {
            let mut sibling_data = sibling.mutate_data();
            let sibling_data = sibling_data.as_mut().map(|d| &mut **d);

            let mut sibling_invalidator = TreeStyleInvalidator::new(
                sibling,
                sibling_data,
                self.shared_context
            );

            let mut invalidations_for_descendants = InvalidationVector::new();
            any_invalidated |=
                sibling_invalidator.process_sibling_invalidations(
                    &mut invalidations_for_descendants,
                    sibling_invalidations,
                );

            any_invalidated |=
                sibling_invalidator.invalidate_descendants(
                    &invalidations_for_descendants
                );

            if sibling_invalidations.is_empty() {
                break;
            }

            current = sibling.next_sibling_element();
        }

        any_invalidated
    }

    fn invalidate_pseudo_element_or_nac(
        &mut self,
        child: E,
        invalidations: &InvalidationVector
    ) -> bool {
        let mut sibling_invalidations = InvalidationVector::new();

        let result = self.invalidate_child(
            child,
            invalidations,
            &mut sibling_invalidations
        );

        // Roots of NAC subtrees can indeed generate sibling invalidations, but
        // they can be just ignored, since they have no siblings.
        debug_assert!(child.implemented_pseudo_element().is_none() ||
                      sibling_invalidations.is_empty(),
                      "pseudos can't generate sibling invalidations, since \
                      using them in other position that isn't the \
                      rightmost part of the selector is invalid \
                      (for now at least)");

        result
    }

    /// Invalidate a child and recurse down invalidating its descendants if
    /// needed.
    fn invalidate_child(
        &mut self,
        child: E,
        invalidations: &InvalidationVector,
        sibling_invalidations: &mut InvalidationVector,
    ) -> bool {
        let mut child_data = child.mutate_data();
        let child_data = child_data.as_mut().map(|d| &mut **d);

        let mut child_invalidator = TreeStyleInvalidator::new(
            child,
            child_data,
            self.shared_context
        );

        let mut invalidations_for_descendants = InvalidationVector::new();
        let mut invalidated_child = false;

        invalidated_child |=
            child_invalidator.process_sibling_invalidations(
                &mut invalidations_for_descendants,
                sibling_invalidations,
            );

        invalidated_child |=
            child_invalidator.process_descendant_invalidations(
                invalidations,
                &mut invalidations_for_descendants,
                sibling_invalidations,
            );

        // The child may not be a flattened tree child of the current element,
        // but may be arbitrarily deep.
        //
        // Since we keep the traversal flags in terms of the flattened tree,
        // we need to propagate it as appropriate.
        if invalidated_child {
            let mut current = child.traversal_parent();
            while let Some(parent) = current.take() {
                if parent == self.element {
                    break;
                }

                unsafe { parent.set_dirty_descendants() };
                current = parent.traversal_parent();
            }
        }

        let invalidated_descendants = child_invalidator.invalidate_descendants(
            &invalidations_for_descendants
        );

        invalidated_child || invalidated_descendants
    }

    fn invalidate_nac(
        &mut self,
        invalidations: &InvalidationVector,
    ) -> bool {
        let mut any_nac_root = false;

        let element = self.element;
        element.each_anonymous_content_child(|nac| {
            any_nac_root |=
                self.invalidate_pseudo_element_or_nac(nac, invalidations);
        });

        any_nac_root
    }

    // NB: It's important that this operates on DOM children, which is what
    // selector-matching operates on.
    fn invalidate_dom_descendants_of(
        &mut self,
        parent: E::ConcreteNode,
        invalidations: &InvalidationVector,
    ) -> bool {
        let mut any_descendant = false;

        let mut sibling_invalidations = InvalidationVector::new();
        for child in parent.children() {
            // TODO(emilio): We handle <xbl:children> fine, because they appear
            // in selector-matching (note bug 1374247, though).
            //
            // This probably needs a shadow root check on `child` here, and
            // recursing if that's the case.
            //
            // Also, what's the deal with HTML <content>? We don't need to
            // support that for now, though we probably need to recurse into the
            // distributed children too.
            let child = match child.as_element() {
                Some(e) => e,
                None => continue,
            };

            any_descendant |= self.invalidate_child(
                child,
                invalidations,
                &mut sibling_invalidations,
            );
        }

        any_descendant
    }

    /// Given a descendant invalidation list, go through the current element's
    /// descendants, and invalidate style on them.
    fn invalidate_descendants(
        &mut self,
        invalidations: &InvalidationVector,
    ) -> bool {
        if invalidations.is_empty() {
            return false;
        }

        debug!("StyleTreeInvalidator::invalidate_descendants({:?})",
               self.element);
        debug!(" > {:?}", invalidations);

        match self.data {
            None => return false,
            Some(ref data) => {
                if data.restyle.hint.contains_subtree() {
                    return false;
                }
            }
        }

        let mut any_descendant = false;

        if let Some(anon_content) = self.element.xbl_binding_anonymous_content() {
            any_descendant |=
                self.invalidate_dom_descendants_of(anon_content, invalidations);
        }

        // TODO(emilio): Having a list of invalidations just for pseudo-elements
        // may save some work here and there.
        if let Some(before) = self.element.before_pseudo_element() {
            any_descendant |=
                self.invalidate_pseudo_element_or_nac(before, invalidations);
        }

        let node = self.element.as_node();
        any_descendant |=
            self.invalidate_dom_descendants_of(node, invalidations);

        if let Some(after) = self.element.after_pseudo_element() {
            any_descendant |=
                self.invalidate_pseudo_element_or_nac(after, invalidations);
        }

        any_descendant |= self.invalidate_nac(invalidations);

        if any_descendant {
            unsafe { self.element.set_dirty_descendants() };
        }

        any_descendant
    }

    /// Process the given sibling invalidations coming from our previous
    /// sibling.
    ///
    /// The sibling invalidations are somewhat special because they can be
    /// modified on the fly. New invalidations may be added and removed.
    ///
    /// In particular, all descendants get the same set of invalidations from
    /// the parent, but the invalidations from a given sibling depend on the
    /// ones we got from the previous one.
    ///
    /// Returns whether invalidated the current element's style.
    fn process_sibling_invalidations(
        &mut self,
        descendant_invalidations: &mut InvalidationVector,
        sibling_invalidations: &mut InvalidationVector,
    ) -> bool {
        let mut i = 0;
        let mut new_sibling_invalidations = InvalidationVector::new();
        let mut invalidated_self = false;

        while i < sibling_invalidations.len() {
            let result = self.process_invalidation(
                &sibling_invalidations[i],
                descendant_invalidations,
                &mut new_sibling_invalidations
            );

            invalidated_self |= result.invalidated_self;
            if !result.effective_for_next {
                sibling_invalidations.remove(i);
            } else {
                i += 1;
            }
        }

        sibling_invalidations.extend(new_sibling_invalidations.into_iter());
        invalidated_self
    }

    /// Process a given invalidation list coming from our parent,
    /// adding to `descendant_invalidations` and `sibling_invalidations` as
    /// needed.
    ///
    /// Returns whether our style was invalidated as a result.
    fn process_descendant_invalidations(
        &mut self,
        invalidations: &InvalidationVector,
        descendant_invalidations: &mut InvalidationVector,
        sibling_invalidations: &mut InvalidationVector,
    ) -> bool {
        let mut invalidated = false;

        for invalidation in invalidations {
            let result = self.process_invalidation(
                invalidation,
                descendant_invalidations,
                sibling_invalidations,
            );

            invalidated |= result.invalidated_self;
            if result.effective_for_next {
                descendant_invalidations.push(invalidation.clone());
            }
        }

        invalidated
    }

    /// Processes a given invalidation, potentially invalidating the style of
    /// the current element.
    ///
    /// Returns whether invalidated the style of the element, and whether the
    /// invalidation should be effective to subsequent siblings or descendants
    /// down in the tree.
    fn process_invalidation(
        &mut self,
        invalidation: &Invalidation,
        descendant_invalidations: &mut InvalidationVector,
        sibling_invalidations: &mut InvalidationVector
    ) -> InvalidationResult {
        debug!("TreeStyleInvalidator::process_invalidation({:?}, {:?})",
               self.element, invalidation);

        let mut context =
            MatchingContext::new_for_visited(
                MatchingMode::Normal,
                None,
                VisitedHandlingMode::AllLinksVisitedAndUnvisited,
                self.shared_context.quirks_mode,
            );

        let matching_result = matches_compound_selector(
            &invalidation.selector,
            invalidation.offset,
            &mut context,
            &self.element
        );

        let mut invalidated_self = false;
        match matching_result {
            CompoundSelectorMatchingResult::Matched { next_combinator_offset: 0 } => {
                debug!(" > Invalidation matched completely");
                invalidated_self = true;
            }
            CompoundSelectorMatchingResult::Matched { next_combinator_offset } => {
                let next_combinator =
                    invalidation.selector.combinator_at(next_combinator_offset);

                if matches!(next_combinator, Combinator::PseudoElement) {
                    let pseudo_selector =
                        invalidation.selector
                            .iter_raw_rev_from(next_combinator_offset - 1)
                            .next()
                            .unwrap();
                    let pseudo = match *pseudo_selector {
                        Component::PseudoElement(ref pseudo) => pseudo,
                        _ => unreachable!("Someone seriously messed up selector parsing"),
                    };

                    // FIXME(emilio): This is not ideal, and could not be
                    // accurate if we ever have stateful element-backed eager
                    // pseudos.
                    //
                    // Ideally, we'd just remove element-backed eager pseudos
                    // altogether, given they work fine without it. Only gotcha
                    // is that we wouldn't style them in parallel, which may or
                    // may not be an issue.
                    //
                    // Also, this could be more fine grained now (perhaps a
                    // RESTYLE_PSEUDOS hint?).
                    //
                    // Note that we'll also restyle the pseudo-element because
                    // it would match this invalidation.
                    if pseudo.is_eager() {
                        invalidated_self = true;
                    }
                }


                let next_invalidation = Invalidation {
                    selector: invalidation.selector.clone(),
                    offset: next_combinator_offset,
                };

                debug!(" > Invalidation matched, next: {:?}, ({:?})",
                        next_invalidation, next_combinator);
                if next_combinator.is_ancestor() {
                    descendant_invalidations.push(next_invalidation);
                } else {
                    sibling_invalidations.push(next_invalidation);
                }
            }
            CompoundSelectorMatchingResult::NotMatched => {}
        }

        if invalidated_self {
            if let Some(ref mut data) = self.data {
                data.restyle.hint.insert(RESTYLE_SELF);
            }
        }

        // TODO(emilio): For pseudo-elements this should be mostly false, except
        // for the weird pseudos in <input type="number">.
        //
        // We should be able to do better here!
        let effective_for_next =
            match invalidation.selector.combinator_at(invalidation.offset) {
                Combinator::NextSibling |
                Combinator::Child => false,
                _ => true,
            };

        InvalidationResult {
            invalidated_self: invalidated_self,
            effective_for_next: effective_for_next,
        }
    }
}

struct InvalidationCollector<'a, 'b: 'a, E>
    where E: TElement,
{
    element: E,
    wrapper: ElementWrapper<'b, E>,
    shared_context: &'a SharedStyleContext<'b>,
    lookup_element: E,
    removed_id: Option<&'a Atom>,
    classes_removed: &'a SmallVec<[Atom; 8]>,
    descendant_invalidations: &'a mut InvalidationVector,
    sibling_invalidations: &'a mut InvalidationVector,
    invalidates_self: bool,
}

impl<'a, 'b: 'a, E> InvalidationCollector<'a, 'b, E>
    where E: TElement,
{
    fn collect_dependencies_in_map(
        &mut self,
        map: &SelectorMap<Dependency>,
    ) {
        map.lookup_with_additional(
            self.lookup_element,
            self.shared_context.quirks_mode,
            self.removed_id,
            self.classes_removed,
            &mut |dependency| {
                self.scan_dependency(
                    dependency,
                    /* is_visited_dependent = */ false
                );
                true
            },
        );
    }
    fn collect_state_dependencies(
        &mut self,
        map: &SelectorMap<StateDependency>,
        state_changes: ElementState,
    ) {
        map.lookup_with_additional(
            self.lookup_element,
            self.shared_context.quirks_mode,
            self.removed_id,
            self.classes_removed,
            &mut |dependency| {
                if !dependency.state.intersects(state_changes) {
                    return true;
                }
                self.scan_dependency(
                    &dependency.dep,
                    dependency.state.intersects(IN_VISITED_OR_UNVISITED_STATE)
                );
                true
            },
        );
    }

    fn scan_dependency(
        &mut self,
        dependency: &Dependency,
        is_visited_dependent: bool
    ) {
        debug!("TreeStyleInvalidator::scan_dependency({:?}, {:?}, {})",
               self.element,
               dependency,
               is_visited_dependent);

        if !self.dependency_may_be_relevant(dependency) {
            return;
        }

        // TODO(emilio): Add a bloom filter here?
        //
        // If we decide to do so, we can't use the bloom filter for snapshots,
        // given that arbitrary elements in the parent chain may have mutated
        // their id's/classes, which means that they won't be in the filter, and
        // as such we may fast-reject selectors incorrectly.
        //
        // We may be able to improve this if we record as we go down the tree
        // whether any parent had a snapshot, and whether those snapshots were
        // taken due to an element class/id change, but it's not clear it'd be
        // worth it.
        let mut now_context =
            MatchingContext::new_for_visited(MatchingMode::Normal, None,
                                             VisitedHandlingMode::AllLinksUnvisited,
                                             self.shared_context.quirks_mode);
        let mut then_context =
            MatchingContext::new_for_visited(MatchingMode::Normal, None,
                                             VisitedHandlingMode::AllLinksUnvisited,
                                             self.shared_context.quirks_mode);

        let matched_then =
            matches_selector(&dependency.selector,
                             dependency.selector_offset,
                             &dependency.hashes,
                             &self.wrapper,
                             &mut then_context,
                             &mut |_, _| {});
        let matches_now =
            matches_selector(&dependency.selector,
                             dependency.selector_offset,
                             &dependency.hashes,
                             &self.element,
                             &mut now_context,
                             &mut |_, _| {});

        // Check for mismatches in both the match result and also the status
        // of whether a relevant link was found.
        if matched_then != matches_now ||
           then_context.relevant_link_found != now_context.relevant_link_found {
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
        if is_visited_dependent && now_context.relevant_link_found {
            then_context.visited_handling = VisitedHandlingMode::RelevantLinkVisited;
            let matched_then =
                matches_selector(&dependency.selector,
                                 dependency.selector_offset,
                                 &dependency.hashes,
                                 &self.wrapper,
                                 &mut then_context,
                                 &mut |_, _| {});

            now_context.visited_handling = VisitedHandlingMode::RelevantLinkVisited;
            let matches_now =
                matches_selector(&dependency.selector,
                                 dependency.selector_offset,
                                 &dependency.hashes,
                                 &self.element,
                                 &mut now_context,
                                 &mut |_, _| {});
            if matched_then != matches_now {
                return self.note_dependency(dependency);
            }
        }
    }

    fn note_dependency(&mut self, dependency: &Dependency) {
        if dependency.affects_self() {
            self.invalidates_self = true;
        }

        if dependency.affects_descendants() {
            debug_assert_ne!(dependency.selector_offset, 0);
            debug_assert!(!dependency.affects_later_siblings());
            self.descendant_invalidations.push(Invalidation {
                selector: dependency.selector.clone(),
                offset: dependency.selector_offset,
            });
        } else if dependency.affects_later_siblings() {
            debug_assert_ne!(dependency.selector_offset, 0);
            self.sibling_invalidations.push(Invalidation {
                selector: dependency.selector.clone(),
                offset: dependency.selector_offset,
            });
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
