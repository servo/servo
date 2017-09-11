/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversing the DOM tree; the bloom filter.

use context::{ElementCascadeInputs, StyleContext, SharedStyleContext};
use data::{ElementData, ElementStyles};
use dom::{NodeInfo, OpaqueNode, TElement, TNode};
use invalidation::element::restyle_hints::{RECASCADE_SELF, RECASCADE_DESCENDANTS, RestyleHint};
use matching::{ChildCascadeRequirement, MatchMethods};
use selector_parser::PseudoElement;
use sharing::StyleSharingTarget;
use smallvec::SmallVec;
use style_resolver::{PseudoElementResolution, StyleResolverForElement};
#[cfg(feature = "servo")] use style_traits::ToCss;
use stylist::RuleInclusion;
use traversal_flags::{TraversalFlags, self};
#[cfg(feature = "servo")] use values::Either;
#[cfg(feature = "servo")] use values::generics::image::Image;

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
pub struct PreTraverseToken(bool);
impl PreTraverseToken {
    /// Whether we should traverse children.
    pub fn should_traverse(&self) -> bool { self.0 }
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
pub trait DomTraversal<E: TElement> : Sync {
    /// Process `node` on the way down, before its children have been processed.
    ///
    /// The callback is invoked for each child node that should be processed by
    /// the traversal.
    fn process_preorder<F>(&self,
                           data: &PerLevelTraversalData,
                           context: &mut StyleContext<E>,
                           node: E::ConcreteNode,
                           note_child: F)
        where F: FnMut(E::ConcreteNode);

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

    /// Style invalidations happen when traversing from a parent to its children.
    /// However, this mechanism can't handle style invalidations on the root. As
    /// such, we have a pre-traversal step to handle that part and determine whether
    /// a full traversal is needed.
    fn pre_traverse(
        root: E,
        shared_context: &SharedStyleContext,
        traversal_flags: TraversalFlags,
    ) -> PreTraverseToken {
        // If this is an unstyled-only traversal, the caller has already verified
        // that there's something to traverse, and we don't need to do any
        // invalidation since we're not doing any restyling.
        if traversal_flags.contains(traversal_flags::UnstyledOnly) {
            return PreTraverseToken(true)
        }

        let flags = shared_context.traversal_flags;
        let mut data = root.mutate_data();
        let mut data = data.as_mut().map(|d| &mut **d);

        if let Some(ref mut data) = data {
            // Invalidate our style, and the one of our siblings and descendants
            // as needed.
            data.invalidate_style_if_needed(root, shared_context, None);

            // Make sure we don't have any stale RECONSTRUCTED_ANCESTOR bits from
            // the last traversal (at a potentially-higher root). From the
            // perspective of this traversal, the root cannot have reconstructed
            // ancestors.
            data.restyle.set_reconstructed_ancestor(false);
        };

        let parent = root.traversal_parent();
        let parent_data = parent.as_ref().and_then(|p| p.borrow_data());
        let should_traverse = Self::element_needs_traversal(
            root,
            flags,
            data.as_mut().map(|d| &**d),
            parent_data.as_ref().map(|d| &**d)
        );

        // If we're not going to traverse at all, we may need to clear some state
        // off the root (which would normally be done at the end of recalc_style_at).
        if !should_traverse && data.is_some() {
            clear_state_after_traversing(root, data.unwrap(), flags);
        }

        PreTraverseToken(should_traverse)
    }

    /// Returns true if traversal should visit a text node. The style system
    /// never processes text nodes, but Servo overrides this to visit them for
    /// flow construction when necessary.
    fn text_node_needs_traversal(node: E::ConcreteNode, _parent_data: &ElementData) -> bool {
        debug_assert!(node.is_text_node());
        false
    }

    /// Returns true if traversal is needed for the given element and subtree.
    ///
    /// The caller passes |parent_data|, which is only null if there is no
    /// parent.
    fn element_needs_traversal(
        el: E,
        traversal_flags: TraversalFlags,
        data: Option<&ElementData>,
        parent_data: Option<&ElementData>,
    ) -> bool {
        debug!("element_needs_traversal({:?}, {:?}, {:?}, {:?})",
               el, traversal_flags, data, parent_data);

        if traversal_flags.contains(traversal_flags::UnstyledOnly) {
            return data.map_or(true, |d| !d.has_styles()) || el.has_dirty_descendants();
        }


        // In case of animation-only traversal we need to traverse the element
        // if the element has animation only dirty descendants bit,
        // animation-only restyle hint or recascade.
        if traversal_flags.for_animation_only() {
            return data.map_or(false, |d| d.has_styles()) &&
                   (el.has_animation_only_dirty_descendants() ||
                    data.as_ref().unwrap().restyle.hint.has_animation_hint_or_recascade());
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
            if let Some(parent_data) = parent_data {
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
        if !data.restyle.hint.is_empty() {
            return true;
        }

        // Servo uses the post-order traversal for flow construction, so we need
        // to traverse any element with damage so that we can perform fixup /
        // reconstruction on our way back up the tree.
        if cfg!(feature = "servo") && !data.restyle.damage.is_empty() {
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
    ) -> bool {
        debug_assert!(cfg!(feature = "gecko") ||
                      parent.has_current_styles_for_traversal(parent_data, context.shared.traversal_flags));

        // If the parent computed display:none, we don't style the subtree.
        if parent_data.styles.is_display_none() {
            debug!("Parent {:?} is display:none, culling traversal", parent);
            return true;
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
    ignore_existing_style: bool,
    pseudo: Option<&PseudoElement>,
) -> ElementStyles
where
    E: TElement,
{
    use style_resolver::StyleResolverForElement;

    debug_assert!(rule_inclusion == RuleInclusion::DefaultOnly ||
                  ignore_existing_style ||
                  pseudo.map_or(false, |p| p.is_before_or_after()) ||
                  element.borrow_data().map_or(true, |d| !d.has_styles()),
                  "Why are we here?");
    let mut ancestors_requiring_style_resolution = SmallVec::<[E; 16]>::new();

    // Clear the bloom filter, just in case the caller is reusing TLS.
    context.thread_local.bloom_filter.clear();

    let mut style = None;
    let mut ancestor = element.traversal_parent();
    while let Some(current) = ancestor {
        if rule_inclusion == RuleInclusion::All && !ignore_existing_style {
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
        layout_parent_style = ancestor.map(|a| {
            a.borrow_data().unwrap().styles.primary().clone()
        });
    }

    for ancestor in ancestors_requiring_style_resolution.iter().rev() {
        context.thread_local.bloom_filter.assert_complete(*ancestor);

        // Actually `PseudoElementResolution` doesn't really matter here.
        // (but it does matter below!).
        let primary_style =
            StyleResolverForElement::new(*ancestor, context, rule_inclusion, PseudoElementResolution::IfApplicable)
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
    StyleResolverForElement::new(element, context, rule_inclusion, PseudoElementResolution::Force)
        .resolve_style(
            style.as_ref().map(|s| &**s),
            layout_parent_style.as_ref().map(|s| &**s)
        )
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
)
where
    E: TElement,
    D: DomTraversal<E>,
    F: FnMut(E::ConcreteNode),
{
    use std::cmp;
    use traversal_flags::*;

    let flags = context.shared.traversal_flags;
    context.thread_local.begin_element(element, data);
    context.thread_local.statistics.elements_traversed += 1;
    debug_assert!(flags.intersects(AnimationOnly | UnstyledOnly) ||
                  !element.has_snapshot() || element.handled_snapshot(),
                  "Should've handled snapshots here already");

    let compute_self = !element.has_current_styles_for_traversal(data, flags);

    debug!("recalc_style_at: {:?} (compute_self={:?}, \
            dirty_descendants={:?}, data={:?})",
           element, compute_self, element.has_dirty_descendants(), data);

    let mut child_cascade_requirement = ChildCascadeRequirement::CanSkipCascade;

    // Compute style for this element if necessary.
    if compute_self {
        child_cascade_requirement =
            compute_style(traversal_data, context, element, data);

        if element.is_native_anonymous() {
            // We must always cascade native anonymous subtrees, since they inherit
            // styles from their first non-NAC ancestor.
            child_cascade_requirement = cmp::max(
                child_cascade_requirement,
                ChildCascadeRequirement::MustCascadeChildren,
            );
        }

        // If we're restyling this element to display:none, throw away all style
        // data in the subtree, notify the caller to early-return.
        if data.styles.is_display_none() {
            debug!("{:?} style is display:none - clearing data from descendants.",
                   element);
            unsafe { clear_descendant_data(element); }
        }

        // Inform any paint worklets of changed style, to speculatively
        // evaluate the worklet code. In the case that the size hasn't changed,
        // this will result in increased concurrency between script and layout.
        notify_paint_worklet(context, data);
    } else {
        debug_assert!(data.has_styles());
        data.restyle.set_traversed_without_styling();
    }

    // Now that matching and cascading is done, clear the bits corresponding to
    // those operations and compute the propagated restyle hint (unless we're
    // not processing invalidations, in which case don't need to propagate it
    // and must avoid clearing it).
    let propagated_hint = if flags.contains(UnstyledOnly) {
        RestyleHint::empty()
    } else {
        debug_assert!(flags.for_animation_only() ||
                      !data.restyle.hint.has_animation_hint(),
                      "animation restyle hint should be handled during \
                       animation-only restyles");
        data.restyle.hint.propagate(&flags)
    };

    trace!("propagated_hint={:?}, cascade_requirement={:?}, \
            is_display_none={:?}, implementing_pseudo={:?}",
           propagated_hint,
           child_cascade_requirement,
           data.styles.is_display_none(),
           element.implemented_pseudo_element());
    debug_assert!(element.has_current_styles_for_traversal(data, flags),
                  "Should have computed style or haven't yet valid computed \
                   style in case of animation-only restyle");

    let has_dirty_descendants_for_this_restyle =
        if flags.for_animation_only() {
            element.has_animation_only_dirty_descendants()
        } else {
            element.has_dirty_descendants()
        };

    // Before examining each child individually, try to prove that our children
    // don't need style processing. They need processing if any of the following
    // conditions hold:
    // * We have the dirty descendants bit.
    // * We're propagating a hint.
    // * This is the initial style.
    // * We generated a reconstruct hint on self (which could mean that we
    //   switched from display:none to something else, which means the children
    //   need initial styling).
    // * This is a servo non-incremental traversal.
    //
    // Additionally, there are a few scenarios where we avoid traversing the
    // subtree even if descendant styles are out of date. These cases are
    // enumerated in should_cull_subtree().
    let mut traverse_children = has_dirty_descendants_for_this_restyle ||
                                !propagated_hint.is_empty() ||
                                !child_cascade_requirement.can_skip_cascade() ||
                                context.thread_local.is_initial_style() ||
                                data.restyle.reconstructed_self() ||
                                is_servo_nonincremental_layout();

    traverse_children = traverse_children &&
                        !traversal.should_cull_subtree(context, element, &data);

    // Examine our children, and enqueue the appropriate ones for traversal.
    if traverse_children {
        note_children::<E, D, F>(
            context,
            element,
            data,
            propagated_hint,
            child_cascade_requirement,
            data.restyle.reconstructed_self_or_ancestor(),
            note_child
        );
    }

    // FIXME(bholley): Make these assertions pass for servo.
    if cfg!(feature = "gecko") && cfg!(debug_assertions) && data.styles.is_display_none() {
        debug_assert!(!element.has_dirty_descendants());
        debug_assert!(!element.has_animation_only_dirty_descendants());
    }

    debug_assert!(flags.for_animation_only() ||
                  !flags.contains(ClearDirtyBits) ||
                  !element.has_animation_only_dirty_descendants(),
                  "Should have cleared animation bits already");
    clear_state_after_traversing(element, data, flags);

    context.thread_local.end_element(element);
}

fn clear_state_after_traversing<E>(
    element: E,
    data: &mut ElementData,
    flags: TraversalFlags
)
where
    E: TElement,
{
    use traversal_flags::*;

    // If we are in a forgetful traversal, drop the existing restyle
    // data here, since we won't need to perform a post-traversal to pick up
    // any change hints.
    if flags.contains(Forgetful) {
        data.clear_restyle_flags_and_damage();
    }

    // Clear dirty bits as appropriate.
    if flags.for_animation_only() {
        if flags.intersects(ClearDirtyBits | ClearAnimationOnlyDirtyDescendants) {
            unsafe { element.unset_animation_only_dirty_descendants(); }
        }
    } else if flags.contains(ClearDirtyBits) {
        // The animation traversal happens first, so we don't need to guard against
        // clearing the animation bit on the regular traversal.
        unsafe { element.clear_dirty_bits(); }
    }
}

fn compute_style<E>(
    traversal_data: &PerLevelTraversalData,
    context: &mut StyleContext<E>,
    element: E,
    data: &mut ElementData
) -> ChildCascadeRequirement
where
    E: TElement,
{
    use data::RestyleKind::*;

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
                Some(styles) => {
                    context.thread_local.statistics.styles_shared += 1;
                    styles
                }
                None => {
                    context.thread_local.statistics.elements_matched += 1;
                    // Perform the matching and cascading.
                    let new_styles = {
                        let mut resolver =
                            StyleResolverForElement::new(
                                element,
                                context,
                                RuleInclusion::All,
                                PseudoElementResolution::IfApplicable
                            );

                        resolver.resolve_style_with_default_parents()
                    };

                    context.thread_local
                        .sharing_cache
                        .insert_if_possible(
                            &element,
                            new_styles.primary(),
                            &mut target,
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

            let mut resolver =
                StyleResolverForElement::new(
                    element,
                    context,
                    RuleInclusion::All,
                    PseudoElementResolution::IfApplicable
                );

            resolver.cascade_styles_with_default_parents(cascade_inputs)
        }
        CascadeOnly => {
            // Skipping full matching, load cascade inputs from previous values.
            let cascade_inputs =
                ElementCascadeInputs::new_from_element_data(data);

            let mut resolver =
                StyleResolverForElement::new(
                    element,
                    context,
                    RuleInclusion::All,
                    PseudoElementResolution::IfApplicable
                );

            resolver.cascade_styles_with_default_parents(cascade_inputs)
        }
    };

    element.finish_restyle(
        context,
        data,
        new_styles,
        important_rules_changed
    )
}

#[cfg(feature = "servo")]
fn notify_paint_worklet<E>(context: &StyleContext<E>, data: &ElementData)
where
    E: TElement,
{
    // We speculatively evaluate any paint worklets during styling.
    // This allows us to run paint worklets in parallel with style and layout.
    // Note that this is wasted effort if the size of the node has
    // changed, but in may cases it won't have.
    if let Some(ref values) = data.styles.primary {
        for image in &values.get_background().background_image.0 {
            let (name, arguments) = match *image {
                Either::Second(Image::PaintWorklet(ref worklet)) => (&worklet.name, &worklet.arguments),
                _ => continue,
            };
            let painter = match context.shared.registered_speculative_painters.get(name) {
                Some(painter) => painter,
                None => continue,
            };
            let properties = painter.properties().iter()
                .filter_map(|(name, id)| id.as_shorthand().err().map(|id| (name, id)))
                .map(|(name, id)| (name.clone(), values.computed_value_to_string(id)))
                .collect();
            let arguments = arguments.iter()
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
    reconstructed_ancestor: bool,
    mut note_child: F,
)
where
    E: TElement,
    D: DomTraversal<E>,
    F: FnMut(E::ConcreteNode),
{
    trace!("note_children: {:?}", element);
    let flags = context.shared.traversal_flags;
    let is_initial_style = context.thread_local.is_initial_style();

    // Loop over all the traversal children.
    for child_node in element.as_node().traversal_children() {
        let child = match child_node.as_element() {
            Some(el) => el,
            None => {
                if is_servo_nonincremental_layout() ||
                   D::text_node_needs_traversal(child_node, data) {
                    note_child(child_node);
                }
                continue;
            },
        };

        let mut child_data = child.mutate_data();
        let mut child_data = child_data.as_mut().map(|d| &mut **d);
        trace!(" > {:?} -> {:?} + {:?}, pseudo: {:?}",
               child,
               child_data.as_ref().map(|d| d.restyle.hint),
               propagated_hint,
               child.implemented_pseudo_element());

        if let Some(ref mut child_data) = child_data {
            // Propagate the parent restyle hint, that may make us restyle the whole
            // subtree.
            child_data.restyle.set_reconstructed_ancestor(reconstructed_ancestor);

            let mut child_hint = propagated_hint;
            match cascade_requirement {
                ChildCascadeRequirement::CanSkipCascade => {}
                ChildCascadeRequirement::MustCascadeDescendants => {
                    child_hint |= RECASCADE_SELF | RECASCADE_DESCENDANTS;
                }
                ChildCascadeRequirement::MustCascadeChildrenIfInheritResetStyle => {
                    use properties::computed_value_flags::INHERITS_RESET_STYLE;
                    if child_data.styles.primary().flags.contains(INHERITS_RESET_STYLE) {
                        child_hint |= RECASCADE_SELF;
                    }
                }
                ChildCascadeRequirement::MustCascadeChildren => {
                    child_hint |= RECASCADE_SELF;
                }
            }

            child_data.restyle.hint.insert(child_hint);

            // Handle element snapshots and invalidation of descendants and siblings
            // as needed.
            //
            // NB: This will be a no-op if there's no snapshot.
            child_data.invalidate_style_if_needed(
                child,
                &context.shared,
                Some(&context.thread_local.stack_limit_checker)
            );
        }

        if D::element_needs_traversal(child, flags, child_data.map(|d| &*d), Some(data)) {
            note_child(child_node);

            // Set the dirty descendants bit on the parent as needed, so that we
            // can find elements during the post-traversal.
            //
            // Note that these bits may be cleared again at the bottom of
            // recalc_style_at if requested by the caller.
            if !is_initial_style {
                if flags.for_animation_only() {
                    unsafe { element.set_animation_only_dirty_descendants(); }
                } else {
                    unsafe { element.set_dirty_descendants(); }
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
        for kid in p.as_node().traversal_children() {
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
