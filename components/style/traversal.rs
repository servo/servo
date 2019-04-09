/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Traversing the DOM tree; the bloom filter.

use crate::context::{ElementCascadeInputs, SharedStyleContext, StyleContext};
use crate::data::{ElementData, ElementStyles};
use crate::dom::{NodeInfo, OpaqueNode, TElement, TNode};
use crate::invalidation::element::restyle_hints::RestyleHint;
use crate::matching::{ChildCascadeRequirement, MatchMethods};
use crate::selector_parser::PseudoElement;
use crate::sharing::StyleSharingTarget;
use crate::style_resolver::{PseudoElementResolution, StyleResolverForElement};
use crate::stylist::RuleInclusion;
use crate::traversal_flags::TraversalFlags;
use selectors::NthIndexCache;
use smallvec::SmallVec;

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

/// We use this structure, rather than just returning a boolean from pre_traverse,
/// to enfore that callers process root invalidations before starting the traversal.
pub struct PreTraverseToken<E: TElement>(Option<E>);
impl<E: TElement> PreTraverseToken<E> {
    /// Whether we should traverse children.
    pub fn should_traverse(&self) -> bool {
        self.0.is_some()
    }

    /// Returns the traversal root for the current traversal.
    pub(crate) fn traversal_root(self) -> Option<E> {
        self.0
    }
}

#[cfg(feature = "servo")]
#[inline]
fn is_servo_nonincremental_layout() -> bool {
    use servo_config::opts;

    opts::get().nonincremental_layout
}

#[cfg(not(feature = "servo"))]
#[inline]
fn is_servo_nonincremental_layout() -> bool {
    false
}

/// A DOM Traversal trait, that is used to generically implement styling for
/// Gecko and Servo.
pub trait DomTraversal<E: TElement>: Sync {
    /// Process `node` on the way down, before its children have been processed.
    ///
    /// The callback is invoked for each child node that should be processed by
    /// the traversal.
    fn process_preorder<F>(
        &self,
        data: &PerLevelTraversalData,
        context: &mut StyleContext<E>,
        node: E::ConcreteNode,
        note_child: F,
    ) where
        F: FnMut(E::ConcreteNode);

    /// Process `node` on the way up, after its children have been processed.
    ///
    /// This is only executed if `needs_postorder_traversal` returns true.
    fn process_postorder(&self, contect: &mut StyleContext<E>, node: E::ConcreteNode);

    /// Boolean that specifies whether a bottom up traversal should be
    /// performed.
    ///
    /// If it's false, then process_postorder has no effect at all.
    fn needs_postorder_traversal() -> bool {
        true
    }

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
        children_to_process: isize,
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
                    break;
                }

                node = parent.as_node();
            }
        } else {
            // Otherwise record the number of children to process when the time
            // comes.
            node.as_element()
                .unwrap()
                .store_children_to_process(children_to_process);
        }
    }

    /// Style invalidations happen when traversing from a parent to its children.
    /// However, this mechanism can't handle style invalidations on the root. As
    /// such, we have a pre-traversal step to handle that part and determine whether
    /// a full traversal is needed.
    fn pre_traverse(root: E, shared_context: &SharedStyleContext) -> PreTraverseToken<E> {
        let traversal_flags = shared_context.traversal_flags;

        let mut data = root.mutate_data();
        let mut data = data.as_mut().map(|d| &mut **d);

        if let Some(ref mut data) = data {
            if !traversal_flags.for_animation_only() {
                // Invalidate our style, and that of our siblings and
                // descendants as needed.
                let invalidation_result = data.invalidate_style_if_needed(
                    root,
                    shared_context,
                    None,
                    &mut NthIndexCache::default(),
                );

                if invalidation_result.has_invalidated_siblings() {
                    let actual_root = root.traversal_parent().expect(
                        "How in the world can you invalidate \
                         siblings without a parent?",
                    );
                    unsafe { actual_root.set_dirty_descendants() }
                    return PreTraverseToken(Some(actual_root));
                }
            }
        }

        let should_traverse =
            Self::element_needs_traversal(root, traversal_flags, data.as_mut().map(|d| &**d));

        // If we're not going to traverse at all, we may need to clear some state
        // off the root (which would normally be done at the end of recalc_style_at).
        if !should_traverse && data.is_some() {
            clear_state_after_traversing(root, data.unwrap(), traversal_flags);
        }

        PreTraverseToken(if should_traverse { Some(root) } else { None })
    }

    /// Returns true if traversal should visit a text node. The style system
    /// never processes text nodes, but Servo overrides this to visit them for
    /// flow construction when necessary.
    fn text_node_needs_traversal(node: E::ConcreteNode, _parent_data: &ElementData) -> bool {
        debug_assert!(node.is_text_node());
        false
    }

    /// Returns true if traversal is needed for the given element and subtree.
    fn element_needs_traversal(
        el: E,
        traversal_flags: TraversalFlags,
        data: Option<&ElementData>,
    ) -> bool {
        debug!(
            "element_needs_traversal({:?}, {:?}, {:?})",
            el, traversal_flags, data
        );

        // In case of animation-only traversal we need to traverse the element
        // if the element has animation only dirty descendants bit,
        // animation-only restyle hint or recascade.
        if traversal_flags.for_animation_only() {
            return data.map_or(false, |d| d.has_styles()) &&
                (el.has_animation_only_dirty_descendants() ||
                    data.as_ref()
                        .unwrap()
                        .hint
                        .has_animation_hint_or_recascade());
        }

        // Non-incremental layout visits every node.
        if is_servo_nonincremental_layout() {
            return true;
        }

        // Unwrap the data.
        let data = match data {
            Some(d) if d.has_styles() => d,
            _ => return true,
        };

        // If the dirty descendants bit is set, we need to traverse no matter
        // what. Skip examining the ElementData.
        if el.has_dirty_descendants() {
            return true;
        }

        // If we have a restyle hint or need to recascade, we need to visit the
        // element.
        //
        // Note that this is different than checking has_current_styles_for_traversal(),
        // since that can return true even if we have a restyle hint indicating
        // that the element's descendants (but not necessarily the element) need
        // restyling.
        if !data.hint.is_empty() {
            return true;
        }

        // Servo uses the post-order traversal for flow construction, so we need
        // to traverse any element with damage so that we can perform fixup /
        // reconstruction on our way back up the tree.
        if cfg!(feature = "servo") && !data.damage.is_empty() {
            return true;
        }

        trace!("{:?} doesn't need traversal", el);
        false
    }

    /// Returns true if we want to cull this subtree from the travesal.
    fn should_cull_subtree(
        &self,
        context: &mut StyleContext<E>,
        parent: E,
        parent_data: &ElementData,
        is_initial_style: bool,
    ) -> bool {
        debug_assert!(
            parent.has_current_styles_for_traversal(parent_data, context.shared.traversal_flags)
        );

        // If the parent computed display:none, we don't style the subtree.
        if parent_data.styles.is_display_none() {
            debug!("Parent {:?} is display:none, culling traversal", parent);
            return true;
        }

        // Gecko-only XBL handling.
        //
        // When we apply the XBL binding during frame construction, we restyle
        // the whole subtree again if the binding is valid, so assuming it's
        // likely to load valid bindings, we avoid wasted work here, which may
        // be a very big perf hit when elements with bindings are nested
        // heavily.
        if cfg!(feature = "gecko") &&
            is_initial_style &&
            parent_data.styles.primary().has_moz_binding()
        {
            debug!("Parent {:?} has XBL binding, deferring traversal", parent);
            return true;
        }

        return false;
    }

    /// Return the shared style context common to all worker threads.
    fn shared_context(&self) -> &SharedStyleContext;
}

/// Manually resolve style by sequentially walking up the parent chain to the
/// first styled Element, ignoring pending restyles. The resolved style is made
/// available via a callback, and can be dropped by the time this function
/// returns in the display:none subtree case.
pub fn resolve_style<E>(
    context: &mut StyleContext<E>,
    element: E,
    rule_inclusion: RuleInclusion,
    pseudo: Option<&PseudoElement>,
) -> ElementStyles
where
    E: TElement,
{
    debug_assert!(
        rule_inclusion == RuleInclusion::DefaultOnly ||
            pseudo.map_or(false, |p| p.is_before_or_after()) ||
            element.borrow_data().map_or(true, |d| !d.has_styles()),
        "Why are we here?"
    );
    let mut ancestors_requiring_style_resolution = SmallVec::<[E; 16]>::new();

    // Clear the bloom filter, just in case the caller is reusing TLS.
    context.thread_local.bloom_filter.clear();

    let mut style = None;
    let mut ancestor = element.traversal_parent();
    while let Some(current) = ancestor {
        if rule_inclusion == RuleInclusion::All {
            if let Some(data) = current.borrow_data() {
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
        layout_parent_style = ancestor.map(|a| a.borrow_data().unwrap().styles.primary().clone());
    }

    for ancestor in ancestors_requiring_style_resolution.iter().rev() {
        context.thread_local.bloom_filter.assert_complete(*ancestor);

        // Actually `PseudoElementResolution` doesn't really matter here.
        // (but it does matter below!).
        let primary_style = StyleResolverForElement::new(
            *ancestor,
            context,
            rule_inclusion,
            PseudoElementResolution::IfApplicable,
        )
        .resolve_primary_style(
            style.as_ref().map(|s| &**s),
            layout_parent_style.as_ref().map(|s| &**s),
        );

        let is_display_contents = primary_style.style().is_display_contents();

        style = Some(primary_style.style.0);
        if !is_display_contents {
            layout_parent_style = style.clone();
        }

        context.thread_local.bloom_filter.push(*ancestor);
    }

    context.thread_local.bloom_filter.assert_complete(element);
    StyleResolverForElement::new(
        element,
        context,
        rule_inclusion,
        PseudoElementResolution::Force,
    )
    .resolve_style(
        style.as_ref().map(|s| &**s),
        layout_parent_style.as_ref().map(|s| &**s),
    )
    .into()
}

/// Calculates the style for a single node.
#[inline]
#[allow(unsafe_code)]
pub fn recalc_style_at<E, D, F>(
    traversal: &D,
    traversal_data: &PerLevelTraversalData,
    context: &mut StyleContext<E>,
    element: E,
    data: &mut ElementData,
    note_child: F,
) where
    E: TElement,
    D: DomTraversal<E>,
    F: FnMut(E::ConcreteNode),
{
    use std::cmp;

    let flags = context.shared.traversal_flags;
    let is_initial_style = !data.has_styles();

    context.thread_local.statistics.elements_traversed += 1;
    debug_assert!(
        flags.intersects(TraversalFlags::AnimationOnly) ||
            !element.has_snapshot() ||
            element.handled_snapshot(),
        "Should've handled snapshots here already"
    );

    let compute_self = !element.has_current_styles_for_traversal(data, flags);

    debug!(
        "recalc_style_at: {:?} (compute_self={:?}, \
         dirty_descendants={:?}, data={:?})",
        element,
        compute_self,
        element.has_dirty_descendants(),
        data
    );

    let mut child_cascade_requirement = ChildCascadeRequirement::CanSkipCascade;

    // Compute style for this element if necessary.
    if compute_self {
        child_cascade_requirement = compute_style(traversal_data, context, element, data);

        if element.is_in_native_anonymous_subtree() {
            // We must always cascade native anonymous subtrees, since they
            // may have pseudo-elements underneath that would inherit from the
            // closest non-NAC ancestor instead of us.
            child_cascade_requirement = cmp::max(
                child_cascade_requirement,
                ChildCascadeRequirement::MustCascadeChildren,
            );
        }

        // If we're restyling this element to display:none, throw away all style
        // data in the subtree, notify the caller to early-return.
        if data.styles.is_display_none() {
            debug!(
                "{:?} style is display:none - clearing data from descendants.",
                element
            );
            unsafe {
                clear_descendant_data(element);
            }
        }

        // Inform any paint worklets of changed style, to speculatively
        // evaluate the worklet code. In the case that the size hasn't changed,
        // this will result in increased concurrency between script and layout.
        notify_paint_worklet(context, data);
    } else {
        debug_assert!(data.has_styles());
        data.set_traversed_without_styling();
    }

    // Now that matching and cascading is done, clear the bits corresponding to
    // those operations and compute the propagated restyle hint (unless we're
    // not processing invalidations, in which case don't need to propagate it
    // and must avoid clearing it).
    debug_assert!(
        flags.for_animation_only() || !data.hint.has_animation_hint(),
        "animation restyle hint should be handled during \
         animation-only restyles"
    );
    let propagated_hint = data.hint.propagate(&flags);

    trace!(
        "propagated_hint={:?}, cascade_requirement={:?}, \
         is_display_none={:?}, implementing_pseudo={:?}",
        propagated_hint,
        child_cascade_requirement,
        data.styles.is_display_none(),
        element.implemented_pseudo_element()
    );
    debug_assert!(
        element.has_current_styles_for_traversal(data, flags),
        "Should have computed style or haven't yet valid computed \
         style in case of animation-only restyle"
    );

    let has_dirty_descendants_for_this_restyle = if flags.for_animation_only() {
        element.has_animation_only_dirty_descendants()
    } else {
        element.has_dirty_descendants()
    };

    // Before examining each child individually, try to prove that our children
    // don't need style processing. They need processing if any of the following
    // conditions hold:
    //
    //  * We have the dirty descendants bit.
    //  * We're propagating a restyle hint.
    //  * We can't skip the cascade.
    //  * This is a servo non-incremental traversal.
    //
    // Additionally, there are a few scenarios where we avoid traversing the
    // subtree even if descendant styles are out of date. These cases are
    // enumerated in should_cull_subtree().
    let mut traverse_children = has_dirty_descendants_for_this_restyle ||
        !propagated_hint.is_empty() ||
        !child_cascade_requirement.can_skip_cascade() ||
        is_servo_nonincremental_layout();

    traverse_children = traverse_children &&
        !traversal.should_cull_subtree(context, element, &data, is_initial_style);

    // Examine our children, and enqueue the appropriate ones for traversal.
    if traverse_children {
        note_children::<E, D, F>(
            context,
            element,
            data,
            propagated_hint,
            child_cascade_requirement,
            is_initial_style,
            note_child,
        );
    }

    // FIXME(bholley): Make these assertions pass for servo.
    if cfg!(feature = "gecko") && cfg!(debug_assertions) && data.styles.is_display_none() {
        debug_assert!(!element.has_dirty_descendants());
        debug_assert!(!element.has_animation_only_dirty_descendants());
    }

    clear_state_after_traversing(element, data, flags);
}

fn clear_state_after_traversing<E>(element: E, data: &mut ElementData, flags: TraversalFlags)
where
    E: TElement,
{
    if flags.intersects(TraversalFlags::FinalAnimationTraversal) {
        debug_assert!(flags.for_animation_only());
        data.clear_restyle_flags_and_damage();
        unsafe {
            element.unset_animation_only_dirty_descendants();
        }
    }
}

fn compute_style<E>(
    traversal_data: &PerLevelTraversalData,
    context: &mut StyleContext<E>,
    element: E,
    data: &mut ElementData,
) -> ChildCascadeRequirement
where
    E: TElement,
{
    use crate::data::RestyleKind::*;

    context.thread_local.statistics.elements_styled += 1;
    let kind = data.restyle_kind(context.shared);

    debug!("compute_style: {:?} (kind={:?})", element, kind);

    if data.has_styles() {
        data.set_restyled();
    }

    let mut important_rules_changed = false;
    let new_styles = match kind {
        MatchAndCascade => {
            debug_assert!(
                !context.shared.traversal_flags.for_animation_only(),
                "MatchAndCascade shouldn't be processed during \
                 animation-only traversal"
            );
            // Ensure the bloom filter is up to date.
            context
                .thread_local
                .bloom_filter
                .insert_parents_recovering(element, traversal_data.current_dom_depth);

            context.thread_local.bloom_filter.assert_complete(element);
            debug_assert_eq!(
                context.thread_local.bloom_filter.matching_depth(),
                traversal_data.current_dom_depth
            );

            // This is only relevant for animations as of right now.
            important_rules_changed = true;

            let mut target = StyleSharingTarget::new(element);

            // Now that our bloom filter is set up, try the style sharing
            // cache.
            match target.share_style_if_possible(context) {
                Some(shared_styles) => {
                    context.thread_local.statistics.styles_shared += 1;
                    shared_styles
                },
                None => {
                    context.thread_local.statistics.elements_matched += 1;
                    // Perform the matching and cascading.
                    let new_styles = {
                        let mut resolver = StyleResolverForElement::new(
                            element,
                            context,
                            RuleInclusion::All,
                            PseudoElementResolution::IfApplicable,
                        );

                        resolver.resolve_style_with_default_parents()
                    };

                    context.thread_local.sharing_cache.insert_if_possible(
                        &element,
                        &new_styles.primary,
                        Some(&mut target),
                        traversal_data.current_dom_depth,
                    );

                    new_styles
                },
            }
        },
        CascadeWithReplacements(flags) => {
            // Skipping full matching, load cascade inputs from previous values.
            let mut cascade_inputs = ElementCascadeInputs::new_from_element_data(data);
            important_rules_changed = element.replace_rules(flags, context, &mut cascade_inputs);

            let mut resolver = StyleResolverForElement::new(
                element,
                context,
                RuleInclusion::All,
                PseudoElementResolution::IfApplicable,
            );

            resolver.cascade_styles_with_default_parents(cascade_inputs)
        },
        CascadeOnly => {
            // Skipping full matching, load cascade inputs from previous values.
            let cascade_inputs = ElementCascadeInputs::new_from_element_data(data);

            let new_styles = {
                let mut resolver = StyleResolverForElement::new(
                    element,
                    context,
                    RuleInclusion::All,
                    PseudoElementResolution::IfApplicable,
                );

                resolver.cascade_styles_with_default_parents(cascade_inputs)
            };

            // Insert into the cache, but only if this style isn't reused from a
            // sibling or cousin. Otherwise, recascading a bunch of identical
            // elements would unnecessarily flood the cache with identical entries.
            //
            // This is analogous to the obvious "don't insert an element that just
            // got a hit in the style sharing cache" behavior in the MatchAndCascade
            // handling above.
            //
            // Note that, for the MatchAndCascade path, we still insert elements that
            // shared styles via the rule node, because we know that there's something
            // different about them that caused them to miss the sharing cache before
            // selector matching. If we didn't, we would still end up with the same
            // number of eventual styles, but would potentially miss out on various
            // opportunities for skipping selector matching, which could hurt
            // performance.
            if !new_styles.primary.reused_via_rule_node {
                context.thread_local.sharing_cache.insert_if_possible(
                    &element,
                    &new_styles.primary,
                    None,
                    traversal_data.current_dom_depth,
                );
            }

            new_styles
        },
    };

    element.finish_restyle(context, data, new_styles, important_rules_changed)
}

#[cfg(feature = "servo")]
fn notify_paint_worklet<E>(context: &StyleContext<E>, data: &ElementData)
where
    E: TElement,
{
    use crate::values::generics::image::Image;
    use crate::values::Either;
    use style_traits::ToCss;

    // We speculatively evaluate any paint worklets during styling.
    // This allows us to run paint worklets in parallel with style and layout.
    // Note that this is wasted effort if the size of the node has
    // changed, but in may cases it won't have.
    if let Some(ref values) = data.styles.primary {
        for image in &values.get_background().background_image.0 {
            let (name, arguments) = match *image {
                Either::Second(Image::PaintWorklet(ref worklet)) => {
                    (&worklet.name, &worklet.arguments)
                },
                _ => continue,
            };
            let painter = match context.shared.registered_speculative_painters.get(name) {
                Some(painter) => painter,
                None => continue,
            };
            let properties = painter
                .properties()
                .iter()
                .filter_map(|(name, id)| id.as_shorthand().err().map(|id| (name, id)))
                .map(|(name, id)| (name.clone(), values.computed_value_to_string(id)))
                .collect();
            let arguments = arguments
                .iter()
                .map(|argument| argument.to_css_string())
                .collect();
            debug!("Notifying paint worklet {}.", painter.name());
            painter.speculatively_draw_a_paint_image(properties, arguments);
        }
    }
}

#[cfg(feature = "gecko")]
fn notify_paint_worklet<E>(_context: &StyleContext<E>, _data: &ElementData)
where
    E: TElement,
{
    // The CSS paint API is Servo-only at the moment
}

fn note_children<E, D, F>(
    context: &mut StyleContext<E>,
    element: E,
    data: &ElementData,
    propagated_hint: RestyleHint,
    cascade_requirement: ChildCascadeRequirement,
    is_initial_style: bool,
    mut note_child: F,
) where
    E: TElement,
    D: DomTraversal<E>,
    F: FnMut(E::ConcreteNode),
{
    trace!("note_children: {:?}", element);
    let flags = context.shared.traversal_flags;

    // Loop over all the traversal children.
    for child_node in element.traversal_children() {
        let child = match child_node.as_element() {
            Some(el) => el,
            None => {
                if is_servo_nonincremental_layout() ||
                    D::text_node_needs_traversal(child_node, data)
                {
                    note_child(child_node);
                }
                continue;
            },
        };

        let mut child_data = child.mutate_data();
        let mut child_data = child_data.as_mut().map(|d| &mut **d);
        trace!(
            " > {:?} -> {:?} + {:?}, pseudo: {:?}",
            child,
            child_data.as_ref().map(|d| d.hint),
            propagated_hint,
            child.implemented_pseudo_element()
        );

        if let Some(ref mut child_data) = child_data {
            let mut child_hint = propagated_hint;
            match cascade_requirement {
                ChildCascadeRequirement::CanSkipCascade => {},
                ChildCascadeRequirement::MustCascadeDescendants => {
                    child_hint |= RestyleHint::RECASCADE_SELF | RestyleHint::RECASCADE_DESCENDANTS;
                },
                ChildCascadeRequirement::MustCascadeChildrenIfInheritResetStyle => {
                    use crate::properties::computed_value_flags::ComputedValueFlags;
                    if child_data
                        .styles
                        .primary()
                        .flags
                        .contains(ComputedValueFlags::INHERITS_RESET_STYLE)
                    {
                        child_hint |= RestyleHint::RECASCADE_SELF;
                    }
                },
                ChildCascadeRequirement::MustCascadeChildren => {
                    child_hint |= RestyleHint::RECASCADE_SELF;
                },
            }

            child_data.hint.insert(child_hint);

            // Handle element snapshots and invalidation of descendants and siblings
            // as needed.
            //
            // NB: This will be a no-op if there's no snapshot.
            child_data.invalidate_style_if_needed(
                child,
                &context.shared,
                Some(&context.thread_local.stack_limit_checker),
                &mut context.thread_local.nth_index_cache,
            );
        }

        if D::element_needs_traversal(child, flags, child_data.map(|d| &*d)) {
            note_child(child_node);

            // Set the dirty descendants bit on the parent as needed, so that we
            // can find elements during the post-traversal.
            //
            // Note that these bits may be cleared again at the bottom of
            // recalc_style_at if requested by the caller.
            if !is_initial_style {
                if flags.for_animation_only() {
                    unsafe {
                        element.set_animation_only_dirty_descendants();
                    }
                } else {
                    unsafe {
                        element.set_dirty_descendants();
                    }
                }
            }
        }
    }
}

/// Clear style data for all the subtree under `root` (but not for root itself).
///
/// We use a list to avoid unbounded recursion, which we need to avoid in the
/// parallel traversal because the rayon stacks are small.
pub unsafe fn clear_descendant_data<E>(root: E)
where
    E: TElement,
{
    let mut parents = SmallVec::<[E; 32]>::new();
    parents.push(root);
    while let Some(p) = parents.pop() {
        for kid in p.traversal_children() {
            if let Some(kid) = kid.as_element() {
                // We maintain an invariant that, if an element has data, all its
                // ancestors have data as well.
                //
                // By consequence, any element without data has no descendants with
                // data.
                if kid.get_data().is_some() {
                    kid.clear_data();
                    parents.push(kid);
                }
            }
        }
    }

    // Make sure not to clear NODE_NEEDS_FRAME on the root.
    root.clear_descendant_bits();
}
