/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The struct that takes care of encapsulating all the logic on where and how
//! element styles need to be invalidated.

use crate::context::StackLimitChecker;
use crate::dom::{TElement, TNode, TShadowRoot};
use crate::invalidation::element::invalidation_map::{Dependency, DependencyInvalidationKind};
use selectors::matching::matches_compound_selector_from;
use selectors::matching::{CompoundSelectorMatchingResult, MatchingContext};
use selectors::parser::{Combinator, Component};
use selectors::OpaqueElement;
use smallvec::SmallVec;
use std::fmt;

/// A trait to abstract the collection of invalidations for a given pass.
pub trait InvalidationProcessor<'a, E>
where
    E: TElement,
{
    /// Whether an invalidation that contains only an eager pseudo-element
    /// selector like ::before or ::after triggers invalidation of the element
    /// that would originate it.
    fn invalidates_on_eager_pseudo_element(&self) -> bool {
        false
    }

    /// Whether the invalidation processor only cares about light-tree
    /// descendants of a given element, that is, doesn't invalidate
    /// pseudo-elements, NAC, shadow dom...
    fn light_tree_only(&self) -> bool {
        false
    }

    /// When a dependency from a :where or :is selector matches, it may still be
    /// the case that we don't need to invalidate the full style. Consider the
    /// case of:
    ///
    ///   div .foo:where(.bar *, .baz) .qux
    ///
    /// We can get to the `*` part after a .bar class change, but you only need
    /// to restyle the element if it also matches .foo.
    ///
    /// Similarly, you only need to restyle .baz if the whole result of matching
    /// the selector changes.
    ///
    /// This function is called to check the result of matching the "outer"
    /// dependency that we generate for the parent of the `:where` selector,
    /// that is, in the case above it should match
    /// `div .foo:where(.bar *, .baz)`.
    ///
    /// Returning true unconditionally here is over-optimistic and may
    /// over-invalidate.
    fn check_outer_dependency(&mut self, dependency: &Dependency, element: E) -> bool;

    /// The matching context that should be used to process invalidations.
    fn matching_context(&mut self) -> &mut MatchingContext<'a, E::Impl>;

    /// Collect invalidations for a given element's descendants and siblings.
    ///
    /// Returns whether the element itself was invalidated.
    fn collect_invalidations(
        &mut self,
        element: E,
        self_invalidations: &mut InvalidationVector<'a>,
        descendant_invalidations: &mut DescendantInvalidationLists<'a>,
        sibling_invalidations: &mut InvalidationVector<'a>,
    ) -> bool;

    /// Returns whether the invalidation process should process the descendants
    /// of the given element.
    fn should_process_descendants(&mut self, element: E) -> bool;

    /// Executes an arbitrary action when the recursion limit is exceded (if
    /// any).
    fn recursion_limit_exceeded(&mut self, element: E);

    /// Executes an action when `Self` is invalidated.
    fn invalidated_self(&mut self, element: E);

    /// Executes an action when any descendant of `Self` is invalidated.
    fn invalidated_descendants(&mut self, element: E, child: E);
}

/// Different invalidation lists for descendants.
#[derive(Debug, Default)]
pub struct DescendantInvalidationLists<'a> {
    /// Invalidations for normal DOM children and pseudo-elements.
    ///
    /// TODO(emilio): Having a list of invalidations just for pseudo-elements
    /// may save some work here and there.
    pub dom_descendants: InvalidationVector<'a>,
    /// Invalidations for slotted children of an element.
    pub slotted_descendants: InvalidationVector<'a>,
    /// Invalidations for ::part()s of an element.
    pub parts: InvalidationVector<'a>,
}

impl<'a> DescendantInvalidationLists<'a> {
    fn is_empty(&self) -> bool {
        self.dom_descendants.is_empty() &&
            self.slotted_descendants.is_empty() &&
            self.parts.is_empty()
    }
}

/// The struct that takes care of encapsulating all the logic on where and how
/// element styles need to be invalidated.
pub struct TreeStyleInvalidator<'a, 'b, E, P: 'a>
where
    'b: 'a,
    E: TElement,
    P: InvalidationProcessor<'b, E>,
{
    element: E,
    stack_limit_checker: Option<&'a StackLimitChecker>,
    processor: &'a mut P,
    _marker: ::std::marker::PhantomData<&'b ()>,
}

/// A vector of invalidations, optimized for small invalidation sets.
pub type InvalidationVector<'a> = SmallVec<[Invalidation<'a>; 10]>;

/// The kind of descendant invalidation we're processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DescendantInvalidationKind {
    /// A DOM descendant invalidation.
    Dom,
    /// A ::slotted() descendant invalidation.
    Slotted,
    /// A ::part() descendant invalidation.
    Part,
}

/// The kind of invalidation we're processing.
///
/// We can use this to avoid pushing invalidations of the same kind to our
/// descendants or siblings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InvalidationKind {
    Descendant(DescendantInvalidationKind),
    Sibling,
}

/// An `Invalidation` is a complex selector that describes which elements,
/// relative to a current element we are processing, must be restyled.
#[derive(Clone)]
pub struct Invalidation<'a> {
    /// The dependency that generated this invalidation.
    ///
    /// Note that the offset inside the dependency is not really useful after
    /// construction.
    dependency: &'a Dependency,
    /// The right shadow host from where the rule came from, if any.
    ///
    /// This is needed to ensure that we match the selector with the right
    /// state, as whether some selectors like :host and ::part() match depends
    /// on it.
    scope: Option<OpaqueElement>,
    /// The offset of the selector pointing to a compound selector.
    ///
    /// This order is a "parse order" offset, that is, zero is the leftmost part
    /// of the selector written as parsed / serialized.
    ///
    /// It is initialized from the offset from `dependency`.
    offset: usize,
    /// Whether the invalidation was already matched by any previous sibling or
    /// ancestor.
    ///
    /// If this is the case, we can avoid pushing invalidations generated by
    /// this one if the generated invalidation is effective for all the siblings
    /// or descendants after us.
    matched_by_any_previous: bool,
}

impl<'a> Invalidation<'a> {
    /// Create a new invalidation for matching a dependency.
    pub fn new(
        dependency: &'a Dependency,
        scope: Option<OpaqueElement>,
    ) -> Self {
        debug_assert!(
            dependency.selector_offset == dependency.selector.len() + 1 ||
            dependency.invalidation_kind() != DependencyInvalidationKind::Element,
            "No point to this, if the dependency matched the element we should just invalidate it"
        );
        Self {
            dependency,
            scope,
            // + 1 to go past the combinator.
            offset: dependency.selector.len() + 1 - dependency.selector_offset,
            matched_by_any_previous: false,
        }
    }

    /// Whether this invalidation is effective for the next sibling or
    /// descendant after us.
    fn effective_for_next(&self) -> bool {
        if self.offset == 0 {
            return true;
        }

        // TODO(emilio): For pseudo-elements this should be mostly false, except
        // for the weird pseudos in <input type="number">.
        //
        // We should be able to do better here!
        match self.dependency.selector.combinator_at_parse_order(self.offset - 1) {
            Combinator::Descendant | Combinator::LaterSibling | Combinator::PseudoElement => true,
            Combinator::Part |
            Combinator::SlotAssignment |
            Combinator::NextSibling |
            Combinator::Child => false,
        }
    }

    fn kind(&self) -> InvalidationKind {
        if self.offset == 0 {
            return InvalidationKind::Descendant(DescendantInvalidationKind::Dom);
        }

        match self.dependency.selector.combinator_at_parse_order(self.offset - 1) {
            Combinator::Child | Combinator::Descendant | Combinator::PseudoElement => {
                InvalidationKind::Descendant(DescendantInvalidationKind::Dom)
            },
            Combinator::Part => InvalidationKind::Descendant(DescendantInvalidationKind::Part),
            Combinator::SlotAssignment => {
                InvalidationKind::Descendant(DescendantInvalidationKind::Slotted)
            },
            Combinator::NextSibling | Combinator::LaterSibling => InvalidationKind::Sibling,
        }
    }
}

impl<'a> fmt::Debug for Invalidation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use cssparser::ToCss;

        f.write_str("Invalidation(")?;
        for component in self.dependency.selector.iter_raw_parse_order_from(self.offset) {
            if matches!(*component, Component::Combinator(..)) {
                break;
            }
            component.to_css(f)?;
        }
        f.write_str(")")
    }
}

/// The result of processing a single invalidation for a given element.
struct SingleInvalidationResult {
    /// Whether the element itself was invalidated.
    invalidated_self: bool,
    /// Whether the invalidation matched, either invalidating the element or
    /// generating another invalidation.
    matched: bool,
}

/// The result of a whole invalidation process for a given element.
pub struct InvalidationResult {
    /// Whether the element itself was invalidated.
    invalidated_self: bool,
    /// Whether the element's descendants were invalidated.
    invalidated_descendants: bool,
    /// Whether the element's siblings were invalidated.
    invalidated_siblings: bool,
}

impl InvalidationResult {
    /// Create an emtpy result.
    pub fn empty() -> Self {
        Self {
            invalidated_self: false,
            invalidated_descendants: false,
            invalidated_siblings: false,
        }
    }

    /// Whether the invalidation has invalidate the element itself.
    pub fn has_invalidated_self(&self) -> bool {
        self.invalidated_self
    }

    /// Whether the invalidation has invalidate desendants.
    pub fn has_invalidated_descendants(&self) -> bool {
        self.invalidated_descendants
    }

    /// Whether the invalidation has invalidate siblings.
    pub fn has_invalidated_siblings(&self) -> bool {
        self.invalidated_siblings
    }
}

impl<'a, 'b, E, P: 'a> TreeStyleInvalidator<'a, 'b, E, P>
where
    'b: 'a,
    E: TElement,
    P: InvalidationProcessor<'b, E>,
{
    /// Trivially constructs a new `TreeStyleInvalidator`.
    pub fn new(
        element: E,
        stack_limit_checker: Option<&'a StackLimitChecker>,
        processor: &'a mut P,
    ) -> Self {
        Self {
            element,
            stack_limit_checker,
            processor,
            _marker: ::std::marker::PhantomData,
        }
    }

    /// Perform the invalidation pass.
    pub fn invalidate(mut self) -> InvalidationResult {
        debug!("StyleTreeInvalidator::invalidate({:?})", self.element);

        let mut self_invalidations = InvalidationVector::new();
        let mut descendant_invalidations = DescendantInvalidationLists::default();
        let mut sibling_invalidations = InvalidationVector::new();

        let mut invalidated_self = self.processor.collect_invalidations(
            self.element,
            &mut self_invalidations,
            &mut descendant_invalidations,
            &mut sibling_invalidations,
        );

        debug!("Collected invalidations (self: {}): ", invalidated_self);
        debug!(
            " > self: {}, {:?}",
            self_invalidations.len(),
            self_invalidations
        );
        debug!(" > descendants: {:?}", descendant_invalidations);
        debug!(
            " > siblings: {}, {:?}",
            sibling_invalidations.len(),
            sibling_invalidations
        );

        let invalidated_self_from_collection = invalidated_self;

        invalidated_self |= self.process_descendant_invalidations(
            &self_invalidations,
            &mut descendant_invalidations,
            &mut sibling_invalidations,
            DescendantInvalidationKind::Dom,
        );

        if invalidated_self && !invalidated_self_from_collection {
            self.processor.invalidated_self(self.element);
        }

        let invalidated_descendants = self.invalidate_descendants(&descendant_invalidations);
        let invalidated_siblings = self.invalidate_siblings(&mut sibling_invalidations);

        InvalidationResult {
            invalidated_self,
            invalidated_descendants,
            invalidated_siblings,
        }
    }

    /// Go through later DOM siblings, invalidating style as needed using the
    /// `sibling_invalidations` list.
    ///
    /// Returns whether any sibling's style or any sibling descendant's style
    /// was invalidated.
    fn invalidate_siblings(&mut self, sibling_invalidations: &mut InvalidationVector<'b>) -> bool {
        if sibling_invalidations.is_empty() {
            return false;
        }

        let mut current = self.element.next_sibling_element();
        let mut any_invalidated = false;

        while let Some(sibling) = current {
            let mut sibling_invalidator =
                TreeStyleInvalidator::new(sibling, self.stack_limit_checker, self.processor);

            let mut invalidations_for_descendants = DescendantInvalidationLists::default();
            let invalidated_sibling = sibling_invalidator.process_sibling_invalidations(
                &mut invalidations_for_descendants,
                sibling_invalidations,
            );

            if invalidated_sibling {
                sibling_invalidator.processor.invalidated_self(sibling);
            }

            any_invalidated |= invalidated_sibling;

            any_invalidated |=
                sibling_invalidator.invalidate_descendants(&invalidations_for_descendants);

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
        invalidations: &[Invalidation<'b>],
    ) -> bool {
        let mut sibling_invalidations = InvalidationVector::new();

        let result = self.invalidate_child(
            child,
            invalidations,
            &mut sibling_invalidations,
            DescendantInvalidationKind::Dom,
        );

        // Roots of NAC subtrees can indeed generate sibling invalidations, but
        // they can be just ignored, since they have no siblings.
        //
        // Note that we can end up testing selectors that wouldn't end up
        // matching due to this being NAC, like those coming from document
        // rules, but we overinvalidate instead of checking this.

        result
    }

    /// Invalidate a child and recurse down invalidating its descendants if
    /// needed.
    fn invalidate_child(
        &mut self,
        child: E,
        invalidations: &[Invalidation<'b>],
        sibling_invalidations: &mut InvalidationVector<'b>,
        descendant_invalidation_kind: DescendantInvalidationKind,
    ) -> bool {
        let mut invalidations_for_descendants = DescendantInvalidationLists::default();

        let mut invalidated_child = false;
        let invalidated_descendants = {
            let mut child_invalidator =
                TreeStyleInvalidator::new(child, self.stack_limit_checker, self.processor);

            invalidated_child |= child_invalidator.process_sibling_invalidations(
                &mut invalidations_for_descendants,
                sibling_invalidations,
            );

            invalidated_child |= child_invalidator.process_descendant_invalidations(
                invalidations,
                &mut invalidations_for_descendants,
                sibling_invalidations,
                descendant_invalidation_kind,
            );

            if invalidated_child {
                child_invalidator.processor.invalidated_self(child);
            }

            child_invalidator.invalidate_descendants(&invalidations_for_descendants)
        };

        // The child may not be a flattened tree child of the current element,
        // but may be arbitrarily deep.
        //
        // Since we keep the traversal flags in terms of the flattened tree,
        // we need to propagate it as appropriate.
        if invalidated_child || invalidated_descendants {
            self.processor.invalidated_descendants(self.element, child);
        }

        invalidated_child || invalidated_descendants
    }

    fn invalidate_nac(&mut self, invalidations: &[Invalidation<'b>]) -> bool {
        let mut any_nac_root = false;

        let element = self.element;
        element.each_anonymous_content_child(|nac| {
            any_nac_root |= self.invalidate_pseudo_element_or_nac(nac, invalidations);
        });

        any_nac_root
    }

    // NB: It's important that this operates on DOM children, which is what
    // selector-matching operates on.
    fn invalidate_dom_descendants_of(
        &mut self,
        parent: E::ConcreteNode,
        invalidations: &[Invalidation<'b>],
    ) -> bool {
        let mut any_descendant = false;

        let mut sibling_invalidations = InvalidationVector::new();
        for child in parent.dom_children() {
            let child = match child.as_element() {
                Some(e) => e,
                None => continue,
            };

            any_descendant |= self.invalidate_child(
                child,
                invalidations,
                &mut sibling_invalidations,
                DescendantInvalidationKind::Dom,
            );
        }

        any_descendant
    }

    fn invalidate_parts_in_shadow_tree(
        &mut self,
        shadow: <E::ConcreteNode as TNode>::ConcreteShadowRoot,
        invalidations: &[Invalidation<'b>],
    ) -> bool {
        debug_assert!(!invalidations.is_empty());

        let mut any = false;
        let mut sibling_invalidations = InvalidationVector::new();

        for node in shadow.as_node().dom_descendants() {
            let element = match node.as_element() {
                Some(e) => e,
                None => continue,
            };

            if element.has_part_attr() {
                any |= self.invalidate_child(
                    element,
                    invalidations,
                    &mut sibling_invalidations,
                    DescendantInvalidationKind::Part,
                );
                debug_assert!(
                    sibling_invalidations.is_empty(),
                    "::part() shouldn't have sibling combinators to the right, \
                     this makes no sense! {:?}",
                    sibling_invalidations
                );
            }

            if let Some(shadow) = element.shadow_root() {
                if element.exports_any_part() {
                    any |= self.invalidate_parts_in_shadow_tree(shadow, invalidations)
                }
            }
        }

        any
    }

    fn invalidate_parts(&mut self, invalidations: &[Invalidation<'b>]) -> bool {
        if invalidations.is_empty() {
            return false;
        }

        let shadow = match self.element.shadow_root() {
            Some(s) => s,
            None => return false,
        };

        self.invalidate_parts_in_shadow_tree(shadow, invalidations)
    }

    fn invalidate_slotted_elements(&mut self, invalidations: &[Invalidation<'b>]) -> bool {
        if invalidations.is_empty() {
            return false;
        }

        let slot = self.element;
        self.invalidate_slotted_elements_in_slot(slot, invalidations)
    }

    fn invalidate_slotted_elements_in_slot(
        &mut self,
        slot: E,
        invalidations: &[Invalidation<'b>],
    ) -> bool {
        let mut any = false;

        let mut sibling_invalidations = InvalidationVector::new();
        for node in slot.slotted_nodes() {
            let element = match node.as_element() {
                Some(e) => e,
                None => continue,
            };

            if element.is_html_slot_element() {
                any |= self.invalidate_slotted_elements_in_slot(element, invalidations);
            } else {
                any |= self.invalidate_child(
                    element,
                    invalidations,
                    &mut sibling_invalidations,
                    DescendantInvalidationKind::Slotted,
                );
            }

            debug_assert!(
                sibling_invalidations.is_empty(),
                "::slotted() shouldn't have sibling combinators to the right, \
                 this makes no sense! {:?}",
                sibling_invalidations
            );
        }

        any
    }

    fn invalidate_non_slotted_descendants(&mut self, invalidations: &[Invalidation<'b>]) -> bool {
        if invalidations.is_empty() {
            return false;
        }

        if self.processor.light_tree_only() {
            let node = self.element.as_node();
            return self.invalidate_dom_descendants_of(node, invalidations);
        }

        let mut any_descendant = false;

        // NOTE(emilio): This is only needed for Shadow DOM to invalidate
        // correctly on :host(..) changes. Instead of doing this, we could add
        // a third kind of invalidation list that walks shadow root children,
        // but it's not clear it's worth it.
        //
        // Also, it's needed as of right now for document state invalidation,
        // where we rely on iterating every element that ends up in the composed
        // doc, but we could fix that invalidating per subtree.
        if let Some(root) = self.element.shadow_root() {
            any_descendant |= self.invalidate_dom_descendants_of(root.as_node(), invalidations);
        }

        if let Some(marker) = self.element.marker_pseudo_element() {
            any_descendant |= self.invalidate_pseudo_element_or_nac(marker, invalidations);
        }

        if let Some(before) = self.element.before_pseudo_element() {
            any_descendant |= self.invalidate_pseudo_element_or_nac(before, invalidations);
        }

        let node = self.element.as_node();
        any_descendant |= self.invalidate_dom_descendants_of(node, invalidations);

        if let Some(after) = self.element.after_pseudo_element() {
            any_descendant |= self.invalidate_pseudo_element_or_nac(after, invalidations);
        }

        any_descendant |= self.invalidate_nac(invalidations);

        any_descendant
    }

    /// Given the descendant invalidation lists, go through the current
    /// element's descendants, and invalidate style on them.
    fn invalidate_descendants(&mut self, invalidations: &DescendantInvalidationLists<'b>) -> bool {
        if invalidations.is_empty() {
            return false;
        }

        debug!(
            "StyleTreeInvalidator::invalidate_descendants({:?})",
            self.element
        );
        debug!(" > {:?}", invalidations);

        let should_process = self.processor.should_process_descendants(self.element);

        if !should_process {
            return false;
        }

        if let Some(checker) = self.stack_limit_checker {
            if checker.limit_exceeded() {
                self.processor.recursion_limit_exceeded(self.element);
                return true;
            }
        }

        let mut any_descendant = false;

        any_descendant |= self.invalidate_non_slotted_descendants(&invalidations.dom_descendants);
        any_descendant |= self.invalidate_slotted_elements(&invalidations.slotted_descendants);
        any_descendant |= self.invalidate_parts(&invalidations.parts);

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
        descendant_invalidations: &mut DescendantInvalidationLists<'b>,
        sibling_invalidations: &mut InvalidationVector<'b>,
    ) -> bool {
        let mut i = 0;
        let mut new_sibling_invalidations = InvalidationVector::new();
        let mut invalidated_self = false;

        while i < sibling_invalidations.len() {
            let result = self.process_invalidation(
                &sibling_invalidations[i],
                descendant_invalidations,
                &mut new_sibling_invalidations,
                InvalidationKind::Sibling,
            );

            invalidated_self |= result.invalidated_self;
            sibling_invalidations[i].matched_by_any_previous |= result.matched;
            if sibling_invalidations[i].effective_for_next() {
                i += 1;
            } else {
                sibling_invalidations.remove(i);
            }
        }

        sibling_invalidations.extend(new_sibling_invalidations.drain(..));
        invalidated_self
    }

    /// Process a given invalidation list coming from our parent,
    /// adding to `descendant_invalidations` and `sibling_invalidations` as
    /// needed.
    ///
    /// Returns whether our style was invalidated as a result.
    fn process_descendant_invalidations(
        &mut self,
        invalidations: &[Invalidation<'b>],
        descendant_invalidations: &mut DescendantInvalidationLists<'b>,
        sibling_invalidations: &mut InvalidationVector<'b>,
        descendant_invalidation_kind: DescendantInvalidationKind,
    ) -> bool {
        let mut invalidated = false;

        for invalidation in invalidations {
            let result = self.process_invalidation(
                invalidation,
                descendant_invalidations,
                sibling_invalidations,
                InvalidationKind::Descendant(descendant_invalidation_kind),
            );

            invalidated |= result.invalidated_self;
            if invalidation.effective_for_next() {
                let mut invalidation = invalidation.clone();
                invalidation.matched_by_any_previous |= result.matched;
                debug_assert_eq!(
                    descendant_invalidation_kind,
                    DescendantInvalidationKind::Dom,
                    "Slotted or part invalidations don't propagate."
                );
                descendant_invalidations.dom_descendants.push(invalidation);
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
        invalidation: &Invalidation<'b>,
        descendant_invalidations: &mut DescendantInvalidationLists<'b>,
        sibling_invalidations: &mut InvalidationVector<'b>,
        invalidation_kind: InvalidationKind,
    ) -> SingleInvalidationResult {
        debug!(
            "TreeStyleInvalidator::process_invalidation({:?}, {:?}, {:?})",
            self.element, invalidation, invalidation_kind
        );

        let matching_result = {
            let context = self.processor.matching_context();
            context.current_host = invalidation.scope;

            matches_compound_selector_from(
                &invalidation.dependency.selector,
                invalidation.offset,
                context,
                &self.element,
            )
        };

        let next_invalidation = match matching_result {
            CompoundSelectorMatchingResult::NotMatched => {
                return SingleInvalidationResult {
                    invalidated_self: false,
                    matched: false,
                }
            },
            CompoundSelectorMatchingResult::FullyMatched => {
                debug!(" > Invalidation matched completely");
                // We matched completely. If we're an inner selector now we need
                // to go outside our selector and carry on invalidating.
                let mut cur_dependency = invalidation.dependency;
                loop {
                    cur_dependency = match cur_dependency.parent {
                        None => return SingleInvalidationResult {
                            invalidated_self: true,
                            matched: true,
                        },
                        Some(ref p) => &**p,
                    };

                    debug!(" > Checking outer dependency {:?}", cur_dependency);

                    // The inner selector changed, now check if the full
                    // previous part of the selector did, before keeping
                    // checking for descendants.
                    if !self.processor.check_outer_dependency(cur_dependency, self.element) {
                        return SingleInvalidationResult {
                            invalidated_self: false,
                            matched: false,
                        }
                    }

                    if cur_dependency.invalidation_kind() == DependencyInvalidationKind::Element {
                        continue;
                    }

                    debug!(" > Generating invalidation");
                    break Invalidation::new(cur_dependency, invalidation.scope)
                }
            },
            CompoundSelectorMatchingResult::Matched {
                next_combinator_offset,
            } => {
                Invalidation {
                    dependency: invalidation.dependency,
                    scope: invalidation.scope,
                    offset: next_combinator_offset + 1,
                    matched_by_any_previous: false,
                }
            },
        };

        debug_assert_ne!(
            next_invalidation.offset,
            0,
            "Rightmost selectors shouldn't generate more invalidations",
        );

        let mut invalidated_self = false;
        let next_combinator = next_invalidation
            .dependency
            .selector
            .combinator_at_parse_order(next_invalidation.offset - 1);

        if matches!(next_combinator, Combinator::PseudoElement) {
            // This will usually be the very next component, except for
            // the fact that we store compound selectors the other way
            // around, so there could also be state pseudo-classes.
            let pseudo_selector = next_invalidation
                .dependency
                .selector
                .iter_raw_parse_order_from(next_invalidation.offset)
                .skip_while(|c| matches!(**c, Component::NonTSPseudoClass(..)))
                .next()
                .unwrap();

            let pseudo = match *pseudo_selector {
                Component::PseudoElement(ref pseudo) => pseudo,
                _ => unreachable!(
                    "Someone seriously messed up selector parsing: \
                     {:?} at offset {:?}: {:?}",
                    next_invalidation.dependency, next_invalidation.offset, pseudo_selector,
                ),
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
            if self.processor.invalidates_on_eager_pseudo_element() {
                if pseudo.is_eager() {
                    invalidated_self = true;
                }
                // If we start or stop matching some marker rules, and
                // don't have a marker, then we need to restyle the
                // element to potentially create one.
                //
                // Same caveats as for other eager pseudos apply, this
                // could be more fine-grained.
                if pseudo.is_marker() && self.element.marker_pseudo_element().is_none() {
                    invalidated_self = true;
                }

                // FIXME: ::selection doesn't generate elements, so the
                // regular invalidation doesn't work for it. We store
                // the cached selection style holding off the originating
                // element, so we need to restyle it in order to invalidate
                // it. This is still not quite correct, since nothing
                // triggers a repaint necessarily, but matches old Gecko
                // behavior, and the ::selection implementation needs to
                // change significantly anyway to implement
                // https://github.com/w3c/csswg-drafts/issues/2474.
                if pseudo.is_selection() {
                    invalidated_self = true;
                }
            }
        }

        debug!(
            " > Invalidation matched, next: {:?}, ({:?})",
            next_invalidation, next_combinator
        );

        let next_invalidation_kind = next_invalidation.kind();

        // We can skip pushing under some circumstances, and we should
        // because otherwise the invalidation list could grow
        // exponentially.
        //
        //  * First of all, both invalidations need to be of the same
        //    kind. This is because of how we propagate them going to
        //    the right of the tree for sibling invalidations and going
        //    down the tree for children invalidations. A sibling
        //    invalidation that ends up generating a children
        //    invalidation ends up (correctly) in five different lists,
        //    not in the same list five different times.
        //
        //  * Then, the invalidation needs to be matched by a previous
        //    ancestor/sibling, in order to know that this invalidation
        //    has been generated already.
        //
        //  * Finally, the new invalidation needs to be
        //    `effective_for_next()`, in order for us to know that it is
        //    still in the list, since we remove the dependencies that
        //    aren't from the lists for our children / siblings.
        //
        // To go through an example, let's imagine we are processing a
        // dom subtree like:
        //
        //   <div><address><div><div/></div></address></div>
        //
        // And an invalidation list with a single invalidation like:
        //
        //   [div div div]
        //
        // When we process the invalidation list for the outer div, we
        // match it, and generate a `div div` invalidation, so for the
        // <address> child we have:
        //
        //   [div div div, div div]
        //
        // With the first of them marked as `matched`.
        //
        // When we process the <address> child, we don't match any of
        // them, so both invalidations go untouched to our children.
        //
        // When we process the second <div>, we match _both_
        // invalidations.
        //
        // However, when matching the first, we can tell it's been
        // matched, and not push the corresponding `div div`
        // invalidation, since we know it's necessarily already on the
        // list.
        //
        // Thus, without skipping the push, we'll arrive to the
        // innermost <div> with:
        //
        //   [div div div, div div, div div, div]
        //
        // While skipping it, we won't arrive here with duplicating
        // dependencies:
        //
        //   [div div div, div div, div]
        //
        let can_skip_pushing = next_invalidation_kind == invalidation_kind &&
            invalidation.matched_by_any_previous &&
            next_invalidation.effective_for_next();

        if can_skip_pushing {
            debug!(
                " > Can avoid push, since the invalidation had \
                 already been matched before"
            );
        } else {
            match next_invalidation_kind {
                InvalidationKind::Descendant(DescendantInvalidationKind::Dom) => {
                    descendant_invalidations
                        .dom_descendants
                        .push(next_invalidation);
                },
                InvalidationKind::Descendant(DescendantInvalidationKind::Part) => {
                    descendant_invalidations.parts.push(next_invalidation);
                },
                InvalidationKind::Descendant(DescendantInvalidationKind::Slotted) => {
                    descendant_invalidations
                        .slotted_descendants
                        .push(next_invalidation);
                },
                InvalidationKind::Sibling => {
                    sibling_invalidations.push(next_invalidation);
                },
            }
        }

        SingleInvalidationResult {
            invalidated_self,
            matched: true,
        }
    }
}
