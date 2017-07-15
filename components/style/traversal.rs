/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversing the DOM tree; the bloom filter.

use context::{ElementCascadeInputs, StyleContext, SharedStyleContext};
use data::{ElementData, ElementStyles};
use dom::{NodeInfo, OpaqueNode, TElement, TNode};
use invalidation::element::restyle_hints::{RECASCADE_SELF, RECASCADE_DESCENDANTS, RestyleHint};
use matching::{ChildCascadeRequirement, MatchMethods};
use sharing::StyleSharingTarget;
use smallvec::SmallVec;
use style_resolver::StyleResolverForElement;
use stylist::RuleInclusion;

/// A per-traversal-level chunk of data. This is sent down by the traversal, and
/// currently only holds the dom depth for the bloom filter.
///
/// NB: Keep this as small as possible, please!
#[derive(Clone, Debug)]
pub struct PerLevelTraversalData {
    /// The current dom depth.
    ///
    /// This is kept with cooperation from the traversal code and the bloom
    /// filter.
    pub current_dom_depth: usize,
}

bitflags! {
    /// Flags that control the traversal process.
    pub flags TraversalFlags: u8 {
        /// Traverse only unstyled children.
        const UNSTYLED_CHILDREN_ONLY = 0x01,
        /// Traverse only elements for animation restyles.
        const ANIMATION_ONLY = 0x02,
        /// Traverse without generating any change hints.
        const FOR_RECONSTRUCT = 0x04,
        /// Traverse triggered by CSS rule changes.
        ///
        /// Traverse and update all elements with CSS animations since
        /// @keyframes rules may have changed
        const FOR_CSS_RULE_CHANGES = 0x08,
    }
}

impl TraversalFlags {
    /// Returns true if the traversal is for animation-only restyles.
    pub fn for_animation_only(&self) -> bool {
        self.contains(ANIMATION_ONLY)
    }

    /// Returns true if the traversal is for unstyled children.
    pub fn for_unstyled_children_only(&self) -> bool {
        self.contains(UNSTYLED_CHILDREN_ONLY)
    }

    /// Returns true if the traversal is for a frame reconstruction.
    pub fn for_reconstruct(&self) -> bool {
        self.contains(FOR_RECONSTRUCT)
    }

    /// Returns true if the traversal is triggered by CSS rule changes.
    pub fn for_css_rule_changes(&self) -> bool {
        self.contains(FOR_CSS_RULE_CHANGES)
    }
}

/// This structure exists to enforce that callers invoke pre_traverse, and also
/// to pass information from the pre-traversal into the primary traversal.
pub struct PreTraverseToken {
    traverse: bool,
    unstyled_children_only: bool,
}

impl PreTraverseToken {
    /// Whether we should traverse children.
    pub fn should_traverse(&self) -> bool {
        self.traverse
    }

    /// Whether we should traverse only unstyled children.
    pub fn traverse_unstyled_children_only(&self) -> bool {
        self.unstyled_children_only
    }
}

/// Enum to prevent duplicate logging.
pub enum LogBehavior {
    /// We should log.
    MayLog,
    /// We shouldn't log.
    DontLog,
}
use self::LogBehavior::*;
impl LogBehavior {
    fn allow(&self) -> bool {
        matches!(*self, MayLog)
    }
}

/// The kind of traversals we could perform.
#[derive(Debug, Copy, Clone)]
pub enum TraversalDriver {
    /// A potentially parallel traversal.
    Parallel,
    /// A sequential traversal.
    Sequential,
}

impl TraversalDriver {
    /// Returns whether this represents a parallel traversal or not.
    #[inline]
    pub fn is_parallel(&self) -> bool {
        matches!(*self, TraversalDriver::Parallel)
    }
}

#[cfg(feature = "servo")]
fn is_servo_nonincremental_layout() -> bool {
    use servo_config::opts;

    opts::get().nonincremental_layout
}

#[cfg(not(feature = "servo"))]
fn is_servo_nonincremental_layout() -> bool {
    false
}

/// A DOM Traversal trait, that is used to generically implement styling for
/// Gecko and Servo.
pub trait DomTraversal<E: TElement> : Sync {
    /// Process `node` on the way down, before its children have been processed.
    fn process_preorder(&self,
                        data: &PerLevelTraversalData,
                        context: &mut StyleContext<E>,
                        node: E::ConcreteNode);

    /// Process `node` on the way up, after its children have been processed.
    ///
    /// This is only executed if `needs_postorder_traversal` returns true.
    fn process_postorder(&self,
                         contect: &mut StyleContext<E>,
                         node: E::ConcreteNode);

    /// Boolean that specifies whether a bottom up traversal should be
    /// performed.
    ///
    /// If it's false, then process_postorder has no effect at all.
    fn needs_postorder_traversal() -> bool { true }

    /// Handles the postorder step of the traversal, if it exists, by bubbling
    /// up the parent chain.
    ///
    /// If we are the last child that finished processing, recursively process
    /// our parent. Else, stop. Also, stop at the root.
    ///
    /// Thus, if we start with all the leaves of a tree, we end up traversing
    /// the whole tree bottom-up because each parent will be processed exactly
    /// once (by the last child that finishes processing).
    ///
    /// The only communication between siblings is that they both
    /// fetch-and-subtract the parent's children count. This makes it safe to
    /// call durign the parallel traversal.
    fn handle_postorder_traversal(
        &self,
        context: &mut StyleContext<E>,
        root: OpaqueNode,
        mut node: E::ConcreteNode,
        children_to_process: isize
    ) {
        // If the postorder step is a no-op, don't bother.
        if !Self::needs_postorder_traversal() {
            return;
        }

        if children_to_process == 0 {
            // We are a leaf. Walk up the chain.
            loop {
                self.process_postorder(context, node);
                if node.opaque() == root {
                    break;
                }
                let parent = node.traversal_parent().unwrap();
                let remaining = parent.did_process_child();
                if remaining != 0 {
                    // The parent has other unprocessed descendants. We only
                    // perform postorder processing after the last descendant
                    // has been processed.
                    break
                }

                node = parent.as_node();
            }
        } else {
            // Otherwise record the number of children to process when the time
            // comes.
            node.as_element().unwrap()
                .store_children_to_process(children_to_process);
        }
    }

    /// Must be invoked before traversing the root element to determine whether
    /// a traversal is needed. Returns a token that allows the caller to prove
    /// that the call happened.
    ///
    /// The traversal_flags is used in Gecko.
    ///
    /// If traversal_flag::UNSTYLED_CHILDREN_ONLY is specified, style newly-
    /// appended children without restyling the parent.
    ///
    /// If traversal_flag::ANIMATION_ONLY is specified, style only elements for
    /// animations.
    fn pre_traverse(
        root: E,
        shared_context: &SharedStyleContext,
        traversal_flags: TraversalFlags
    ) -> PreTraverseToken {
        debug_assert!(!(traversal_flags.for_reconstruct() &&
                        traversal_flags.for_unstyled_children_only()),
                      "must not specify FOR_RECONSTRUCT in combination with \
                       UNSTYLED_CHILDREN_ONLY");

        if traversal_flags.for_unstyled_children_only() {
            if root.borrow_data().map_or(true, |d| d.has_styles() && d.styles.is_display_none()) {
                return PreTraverseToken {
                    traverse: false,
                    unstyled_children_only: false,
                };
            }
            return PreTraverseToken {
                traverse: true,
                unstyled_children_only: true,
            };
        }

        // Look at whether there has been any attribute or state change, and
        // invalidate our style, and the one of our siblings and descendants as
        // needed.
        if let Some(mut data) = root.mutate_data() {
            data.invalidate_style_if_needed(root, shared_context);
        }

        PreTraverseToken {
            traverse: Self::node_needs_traversal(root.as_node(), traversal_flags),
            unstyled_children_only: false,
        }
    }

    /// Returns true if traversal should visit a text node. The style system
    /// never processes text nodes, but Servo overrides this to visit them for
    /// flow construction when necessary.
    fn text_node_needs_traversal(node: E::ConcreteNode) -> bool {
        debug_assert!(node.is_text_node());
        false
    }

    /// Returns true if traversal is needed for the given node and subtree.
    fn node_needs_traversal(
        node: E::ConcreteNode,
        traversal_flags: TraversalFlags
    ) -> bool {
        // Non-incremental layout visits every node.
        if is_servo_nonincremental_layout() {
            return true;
        }

        if traversal_flags.for_reconstruct() {
            return true;
        }

        let el = match node.as_element() {
            None => return Self::text_node_needs_traversal(node),
            Some(el) => el,
        };

        // If the element is native-anonymous and an ancestor frame will be
        // reconstructed, the child and all its descendants will be destroyed.
        // In that case, we wouldn't need to traverse the subtree...
        //
        // Except if there could be transitions of pseudo-elements, in which
        // case we still need to process them, unfortunately.
        //
        // We need to conservatively continue the traversal to style the
        // pseudo-element in order to properly process potentially-new
        // transitions that we won't see otherwise.
        //
        // But it may be that we no longer match, so detect that case and act
        // appropriately here.
        if el.is_native_anonymous() {
            if let Some(parent) = el.traversal_parent() {
                let parent_data = parent.borrow_data().unwrap();
                let going_to_reframe =
                    parent_data.restyle.reconstructed_self_or_ancestor();

                let mut is_before_or_after_pseudo = false;
                if let Some(pseudo) = el.implemented_pseudo_element() {
                    if pseudo.is_before_or_after() {
                        is_before_or_after_pseudo = true;
                        let still_match =
                            parent_data.styles.pseudos.get(&pseudo).is_some();

                        if !still_match {
                            debug_assert!(going_to_reframe,
                                          "We're removing a pseudo, so we \
                                           should reframe!");
                            return false;
                        }
                    }
                }

                if going_to_reframe && !is_before_or_after_pseudo {
                    debug!("Element {:?} is in doomed NAC subtree, \
                            culling traversal", el);
                    return false;
                }
            }
        }

        // In case of animation-only traversal we need to traverse the element
        // if the element has animation only dirty descendants bit,
        // animation-only restyle hint or recascade.
        if traversal_flags.for_animation_only() {
            // Skip elements that have no style data since animation-only
            // restyle is not necessary for the elements.
            let data = match el.borrow_data() {
                Some(d) => d,
                None => return false,
            };

            if !data.has_styles() {
                return false;
            }

            if el.has_animation_only_dirty_descendants() {
                return true;
            }

            return data.restyle.hint.has_animation_hint() ||
                   data.restyle.hint.has_recascade_self();
        }

        // If the dirty descendants bit is set, we need to traverse no matter
        // what. Skip examining the ElementData.
        if el.has_dirty_descendants() {
            return true;
        }

        // Check the element data. If it doesn't exist, we need to visit the
        // element.
        let data = match el.borrow_data() {
            Some(d) => d,
            None => return true,
        };

        // If we don't have any style data, we need to visit the element.
        if !data.has_styles() {
            return true;
        }

        // If we have a restyle hint or need to recascade, we need to visit the
        // element.
        //
        // Note that this is different than checking has_current_styles(),
        // since that can return true even if we have a restyle hint indicating
        // that the element's descendants (but not necessarily the element) need
        // restyling.
        if !data.restyle.hint.is_empty() {
            return true;
        }

        // Servo uses the post-order traversal for flow construction, so we need
        // to traverse any element with damage so that we can perform fixup /
        // reconstruction on our way back up the tree.
        //
        // We also need to traverse nodes with explicit damage and no other
        // restyle data, so that this damage can be cleared.
        if (cfg!(feature = "servo") || traversal_flags.for_reconstruct()) &&
           !data.restyle.damage.is_empty() {
            return true;
        }

        trace!("{:?} doesn't need traversal", el);
        false
    }

    /// Returns true if traversal of this element's children is allowed. We use
    /// this to cull traversal of various subtrees.
    ///
    /// This may be called multiple times when processing an element, so we pass
    /// a parameter to keep the logs tidy.
    fn should_traverse_children(
        &self,
        context: &mut StyleContext<E>,
        parent: E,
        parent_data: &ElementData,
        log: LogBehavior
    ) -> bool {
        // See the comment on `cascade_node` for why we allow this on Gecko.
        debug_assert!(cfg!(feature = "gecko") ||
                      parent.has_current_styles(parent_data));

        // If the parent computed display:none, we don't style the subtree.
        if parent_data.styles.is_display_none() {
            if log.allow() {
                debug!("Parent {:?} is display:none, culling traversal",
                       parent);
            }
            return false;
        }

        // Gecko-only XBL handling.
        //
        // If we're computing initial styles and the parent has a Gecko XBL
        // binding, that binding may inject anonymous children and remap the
        // explicit children to an insertion point (or hide them entirely). It
        // may also specify a scoped stylesheet, which changes the rules that
        // apply within the subtree. These two effects can invalidate the result
        // of property inheritance and selector matching (respectively) within
        // the subtree.
        //
        // To avoid wasting work, we defer initial styling of XBL subtrees until
        // frame construction, which does an explicit traversal of the unstyled
        // children after shuffling the subtree. That explicit traversal may in
        // turn find other bound elements, which get handled in the same way.
        //
        // We explicitly avoid handling restyles here (explicitly removing or
        // changing bindings), since that adds complexity and is rarer. If it
        // happens, we may just end up doing wasted work, since Gecko
        // recursively drops Servo ElementData when the XBL insertion parent of
        // an Element is changed.
        if cfg!(feature = "gecko") && context.thread_local.is_initial_style() &&
           parent_data.styles.primary().has_moz_binding() {
            if log.allow() {
                debug!("Parent {:?} has XBL binding, deferring traversal",
                       parent);
            }
            return false;
        }

        return true;
    }

    /// Helper for the traversal implementations to select the children that
    /// should be enqueued for processing.
    fn traverse_children<F>(
        &self,
        context: &mut StyleContext<E>,
        parent: E,
        mut f: F
    )
    where
        F: FnMut(&mut StyleContext<E>, E::ConcreteNode)
    {
        // Check if we're allowed to traverse past this element.
        let should_traverse =
            self.should_traverse_children(
                context,
                parent,
                &parent.borrow_data().unwrap(),
                MayLog
            );

        context.thread_local.end_element(parent);
        if !should_traverse {
            return;
        }

        for kid in parent.as_node().traversal_children() {
            if Self::node_needs_traversal(kid, self.shared_context().traversal_flags) {
                // If we are in a restyle for reconstruction, there is no need to
                // perform a post-traversal, so we don't need to set the dirty
                // descendants bit on the parent.
                if !self.shared_context().traversal_flags.for_reconstruct() {
                    let el = kid.as_element();
                    if el.as_ref().and_then(|el| el.borrow_data())
                         .map_or(false, |d| d.has_styles()) {
                        if self.shared_context().traversal_flags.for_animation_only() {
                            unsafe { parent.set_animation_only_dirty_descendants(); }
                        } else {
                            unsafe { parent.set_dirty_descendants(); }
                        }
                    }
                }
                f(context, kid);
            }
        }
    }

    /// Return the shared style context common to all worker threads.
    fn shared_context(&self) -> &SharedStyleContext;

    /// Whether we're performing a parallel traversal.
    ///
    /// NB: We do this check on runtime. We could guarantee correctness in this
    /// regard via the type system via a `TraversalDriver` trait for this trait,
    /// that could be one of two concrete types. It's not clear whether the
    /// potential code size impact of that is worth it.
    fn is_parallel(&self) -> bool;
}

/// Manually resolve style by sequentially walking up the parent chain to the
/// first styled Element, ignoring pending restyles. The resolved style is made
/// available via a callback, and can be dropped by the time this function
/// returns in the display:none subtree case.
pub fn resolve_style<E>(
    context: &mut StyleContext<E>,
    element: E,
    rule_inclusion: RuleInclusion,
) -> ElementStyles
where
    E: TElement,
{
    use style_resolver::StyleResolverForElement;

    debug_assert!(rule_inclusion == RuleInclusion::DefaultOnly ||
                  element.borrow_data().map_or(true, |d| !d.has_styles()),
                  "Why are we here?");
    let mut ancestors_requiring_style_resolution = SmallVec::<[E; 16]>::new();

    // Clear the bloom filter, just in case the caller is reusing TLS.
    context.thread_local.bloom_filter.clear();

    let mut style = None;
    let mut ancestor = element.traversal_parent();
    while let Some(current) = ancestor {
        if rule_inclusion == RuleInclusion::All {
            if let Some(data) = element.borrow_data() {
                if let Some(ancestor_style) = data.styles.get_primary() {
                    style = Some(ancestor_style.clone());
                    break;
                }
            }
        }
        ancestors_requiring_style_resolution.push(current);
        ancestor = current.traversal_parent();
    }

    if let Some(ancestor) = ancestor {
        context.thread_local.bloom_filter.rebuild(ancestor);
        context.thread_local.bloom_filter.push(ancestor);
    }

    let mut layout_parent_style = style.clone();
    while let Some(style) = layout_parent_style.take() {
        if !style.is_display_contents() {
            layout_parent_style = Some(style);
            break;
        }

        ancestor = ancestor.unwrap().traversal_parent();
        layout_parent_style = ancestor.map(|a| {
            a.borrow_data().unwrap().styles.primary().clone()
        });
    }

    for ancestor in ancestors_requiring_style_resolution.iter().rev() {
        context.thread_local.bloom_filter.assert_complete(*ancestor);

        let primary_style =
            StyleResolverForElement::new(*ancestor, context, rule_inclusion)
                .resolve_primary_style(
                    style.as_ref().map(|s| &**s),
                    layout_parent_style.as_ref().map(|s| &**s)
                );

        let is_display_contents = primary_style.style.is_display_contents();

        style = Some(primary_style.style);
        if !is_display_contents {
            layout_parent_style = style.clone();
        }

        context.thread_local.bloom_filter.push(*ancestor);
    }

    context.thread_local.bloom_filter.assert_complete(element);
    StyleResolverForElement::new(element, context, rule_inclusion)
        .resolve_style(
            style.as_ref().map(|s| &**s),
            layout_parent_style.as_ref().map(|s| &**s)
        )
}

/// Calculates the style for a single node.
#[inline]
#[allow(unsafe_code)]
pub fn recalc_style_at<E, D>(
    traversal: &D,
    traversal_data: &PerLevelTraversalData,
    context: &mut StyleContext<E>,
    element: E,
    data: &mut ElementData
)
where
    E: TElement,
    D: DomTraversal<E>,
{
    context.thread_local.begin_element(element, data);
    context.thread_local.statistics.elements_traversed += 1;
    debug_assert!(!element.has_snapshot() || element.handled_snapshot(),
                  "Should've handled snapshots here already");

    let compute_self = !element.has_current_styles(data);
    let mut hint = RestyleHint::empty();

    debug!("recalc_style_at: {:?} (compute_self={:?}, \
            dirty_descendants={:?}, data={:?})",
           element, compute_self, element.has_dirty_descendants(), data);

    // Compute style for this element if necessary.
    if compute_self {
        match compute_style(traversal, traversal_data, context, element, data) {
            ChildCascadeRequirement::MustCascadeChildren => {
                hint |= RECASCADE_SELF;
            }
            ChildCascadeRequirement::MustCascadeDescendants => {
                hint |= RECASCADE_SELF | RECASCADE_DESCENDANTS;
            }
            ChildCascadeRequirement::CanSkipCascade => {}
        };

        // We must always cascade native anonymous subtrees, since they inherit
        // styles from their first non-NAC ancestor.
        if element.is_native_anonymous() {
            hint |= RECASCADE_SELF;
        }

        // If we're restyling this element to display:none, throw away all style
        // data in the subtree, notify the caller to early-return.
        if data.styles.is_display_none() {
            debug!("{:?} style is display:none - clearing data from descendants.",
                   element);
            clear_descendant_data(element)
        }
    }

    // Now that matching and cascading is done, clear the bits corresponding to
    // those operations and compute the propagated restyle hint.
    let mut propagated_hint = {
        debug_assert!(context.shared.traversal_flags.for_animation_only() ||
                      !data.restyle.hint.has_animation_hint(),
                      "animation restyle hint should be handled during \
                       animation-only restyles");
        data.restyle.hint.propagate(&context.shared.traversal_flags)
    };

    // FIXME(bholley): Need to handle explicitly-inherited reset properties
    // somewhere.
    propagated_hint.insert(hint);

    trace!("propagated_hint={:?} \
            is_display_none={:?}, implementing_pseudo={:?}",
           propagated_hint,
           data.styles.is_display_none(),
           element.implemented_pseudo_element());
    debug_assert!(element.has_current_styles(data) ||
                  context.shared.traversal_flags.for_animation_only(),
                  "Should have computed style or haven't yet valid computed \
                   style in case of animation-only restyle");

    let has_dirty_descendants_for_this_restyle =
        if context.shared.traversal_flags.for_animation_only() {
            element.has_animation_only_dirty_descendants()
        } else {
            element.has_dirty_descendants()
        };

    // Preprocess children, propagating restyle hints and handling sibling
    // relationships.
    let should_traverse_children = traversal.should_traverse_children(
        context,
        element,
        &data,
        DontLog
    );
    if should_traverse_children &&
        (has_dirty_descendants_for_this_restyle || !propagated_hint.is_empty()) {
        let reconstructed_ancestor =
            data.restyle.reconstructed_self_or_ancestor();

        preprocess_children::<E, D>(
            context,
            element,
            propagated_hint,
            reconstructed_ancestor,
        )
    }

    // If we are in a restyle for reconstruction, drop the existing restyle
    // data here, since we won't need to perform a post-traversal to pick up
    // any change hints.
    if context.shared.traversal_flags.for_reconstruct() {
        data.clear_restyle_state();
    }

    if context.shared.traversal_flags.for_animation_only() {
        unsafe { element.unset_animation_only_dirty_descendants(); }
    }

    // There are two cases when we want to clear the dity descendants bit here
    // after styling this element.
    //
    // The first case is when this element is the root of a display:none
    // subtree, even if the style didn't change (since, if the style did change,
    // we'd have already cleared it above).
    //
    // This keeps the tree in a valid state without requiring the DOM to check
    // display:none on the parent when inserting new children (which can be
    // moderately expensive). Instead, DOM implementations can unconditionally
    // set the dirty descendants bit on any styled parent, and let the traversal
    // sort it out.
    //
    // The second case is when we are in a restyle for reconstruction, where we
    // won't need to perform a post-traversal to pick up any change hints.
    if data.styles.is_display_none() ||
       context.shared.traversal_flags.for_reconstruct() {
        unsafe { element.unset_dirty_descendants(); }
    }
}

fn compute_style<E, D>(
    _traversal: &D,
    traversal_data: &PerLevelTraversalData,
    context: &mut StyleContext<E>,
    element: E,
    data: &mut ElementData
) -> ChildCascadeRequirement
where
    E: TElement,
    D: DomTraversal<E>,
{
    use data::RestyleKind::*;
    use sharing::StyleSharingResult::*;

    context.thread_local.statistics.elements_styled += 1;
    let kind = data.restyle_kind(context.shared);

    debug!("compute_style: {:?} (kind={:?})", element, kind);

    if data.has_styles() {
        data.restyle.set_restyled();
    }

    let mut important_rules_changed = false;
    let new_styles = match kind {
        MatchAndCascade => {
            debug_assert!(!context.shared.traversal_flags.for_animation_only(),
                          "MatchAndCascade shouldn't be processed during \
                           animation-only traversal");
            // Ensure the bloom filter is up to date.
            context.thread_local.bloom_filter
                   .insert_parents_recovering(element,
                                              traversal_data.current_dom_depth);

            context.thread_local.bloom_filter.assert_complete(element);

            // This is only relevant for animations as of right now.
            important_rules_changed = true;

            let mut target = StyleSharingTarget::new(element);

            // Now that our bloom filter is set up, try the style sharing
            // cache.
            match target.share_style_if_possible(context) {
                StyleWasShared(index, styles) => {
                    context.thread_local.statistics.styles_shared += 1;
                    context.thread_local.style_sharing_candidate_cache.touch(index);
                    styles
                }
                CannotShare => {
                    context.thread_local.statistics.elements_matched += 1;
                    // Perform the matching and cascading.
                    let new_styles =
                        StyleResolverForElement::new(element, context, RuleInclusion::All)
                            .resolve_style_with_default_parents();

                    context.thread_local
                        .style_sharing_candidate_cache
                        .insert_if_possible(
                            &element,
                            new_styles.primary(),
                            target.take_validation_data(),
                            context.thread_local.bloom_filter.matching_depth(),
                        );

                    new_styles
                }
            }
        }
        CascadeWithReplacements(flags) => {
            // Skipping full matching, load cascade inputs from previous values.
            let mut cascade_inputs =
                ElementCascadeInputs::new_from_element_data(data);
            important_rules_changed =
                element.replace_rules(flags, context, &mut cascade_inputs);
            StyleResolverForElement::new(element, context, RuleInclusion::All)
                .cascade_styles_with_default_parents(cascade_inputs)
        }
        CascadeOnly => {
            // Skipping full matching, load cascade inputs from previous values.
            let cascade_inputs =
                ElementCascadeInputs::new_from_element_data(data);
            StyleResolverForElement::new(element, context, RuleInclusion::All)
                .cascade_styles_with_default_parents(cascade_inputs)
        }
    };

    element.finish_restyle(
        context,
        data,
        new_styles,
        important_rules_changed
    )
}

fn preprocess_children<E, D>(
    context: &mut StyleContext<E>,
    element: E,
    propagated_hint: RestyleHint,
    reconstructed_ancestor: bool,
)
where
    E: TElement,
    D: DomTraversal<E>,
{
    trace!("preprocess_children: {:?}", element);

    // Loop over all the traversal children.
    for child in element.as_node().traversal_children() {
        // FIXME(bholley): Add TElement::element_children instead of this.
        let child = match child.as_element() {
            Some(el) => el,
            None => continue,
        };

        // If the child is unstyled, we don't need to set up any restyling.
        if child.borrow_data().map_or(true, |d| !d.has_styles()) {
            continue;
        }

        let mut child_data = unsafe { child.ensure_data() };

        trace!(" > {:?} -> {:?} + {:?}, pseudo: {:?}",
               child,
               child_data.restyle.hint,
               propagated_hint,
               child.implemented_pseudo_element());

        // Propagate the parent restyle hint, that may make us restyle the whole
        // subtree.
        if reconstructed_ancestor {
            child_data.restyle.set_reconstructed_ancestor();
        }
        child_data.restyle.hint.insert(propagated_hint);

        // Handle element snapshots and invalidation of descendants and siblings
        // as needed.
        //
        // NB: This will be a no-op if there's no snapshot.
        child_data.invalidate_style_if_needed(child, &context.shared);
    }
}

/// Clear style data for all the subtree under `el`.
pub fn clear_descendant_data<E>(el: E)
where
    E: TElement,
{
    for kid in el.as_node().traversal_children() {
        if let Some(kid) = kid.as_element() {
            // We maintain an invariant that, if an element has data, all its
            // ancestors have data as well.
            //
            // By consequence, any element without data has no descendants with
            // data.
            if kid.get_data().is_some() {
                unsafe { kid.clear_data() };
                clear_descendant_data(kid);
            }
        }
    }

    unsafe {
        el.unset_dirty_descendants();
        el.unset_animation_only_dirty_descendants();
    }
}
