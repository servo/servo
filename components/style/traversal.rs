/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversing the DOM tree; the bloom filter.

use atomic_refcell::AtomicRefCell;
use context::{SharedStyleContext, StyleContext, ThreadLocalStyleContext};
use data::{ElementData, ElementStyles, StoredRestyleHint};
use dom::{DirtyDescendants, NodeInfo, OpaqueNode, TElement, TNode};
use matching::{ChildCascadeRequirement, MatchMethods};
use restyle_hints::{CascadeHint, HintComputationContext, RECASCADE_SELF};
use restyle_hints::{RECASCADE_DESCENDANTS, RestyleHint};
use selector_parser::RestyleDamage;
use sharing::{StyleSharingBehavior, StyleSharingTarget};
#[cfg(feature = "servo")] use servo_config::opts;
use smallvec::SmallVec;
use std::borrow::BorrowMut;

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
        /// Traverse and update all elements with CSS animations since
        /// @keyframes rules may have changed
        const FOR_CSS_RULE_CHANGES = 0x08,
        /// Only include user agent style sheets when selector matching.
        const FOR_DEFAULT_STYLES = 0x10,
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

    /// Returns true if the traversal is to compute the default computed
    /// styles for an element.
    pub fn for_default_styles(&self) -> bool {
        self.contains(FOR_DEFAULT_STYLES)
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
    fn allow(&self) -> bool { matches!(*self, MayLog) }
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
    opts::get().nonincremental_layout
}

#[cfg(not(feature = "servo"))]
fn is_servo_nonincremental_layout() -> bool {
    false
}

/// A DOM Traversal trait, that is used to generically implement styling for
/// Gecko and Servo.
pub trait DomTraversal<E: TElement> : Sync {
    /// The thread-local context, used to store non-thread-safe stuff that needs
    /// to be used in the traversal, and of which we use one per worker, like
    /// the bloom filter, for example.
    type ThreadLocalContext: Send + BorrowMut<ThreadLocalStyleContext<E>>;

    /// Process `node` on the way down, before its children have been processed.
    fn process_preorder(&self,
                        data: &PerLevelTraversalData,
                        thread_local: &mut Self::ThreadLocalContext,
                        node: E::ConcreteNode);

    /// Process `node` on the way up, after its children have been processed.
    ///
    /// This is only executed if `needs_postorder_traversal` returns true.
    fn process_postorder(&self,
                         thread_local: &mut Self::ThreadLocalContext,
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
    fn handle_postorder_traversal(&self,
                                  thread_local: &mut Self::ThreadLocalContext,
                                  root: OpaqueNode,
                                  mut node: E::ConcreteNode,
                                  children_to_process: isize)
    {
        // If the postorder step is a no-op, don't bother.
        if !Self::needs_postorder_traversal() {
            return;
        }

        if children_to_process == 0 {
            // We are a leaf. Walk up the chain.
            loop {
                self.process_postorder(thread_local, node);
                if node.opaque() == root {
                    break;
                }
                let parent = node.parent_element().unwrap();
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
            // Otherwise record the number of children to process when the
            // time comes.
            node.as_element().unwrap()
                .store_children_to_process(children_to_process);
        }
    }

    /// Must be invoked before traversing the root element to determine whether
    /// a traversal is needed. Returns a token that allows the caller to prove
    /// that the call happened.
    ///
    /// The traversal_flag is used in Gecko.
    /// If traversal_flag::UNSTYLED_CHILDREN_ONLY is specified, style newly-
    /// appended children without restyling the parent.
    /// If traversal_flag::ANIMATION_ONLY is specified, style only elements for
    /// animations.
    fn pre_traverse(root: E,
                    shared_context: &SharedStyleContext,
                    traversal_flags: TraversalFlags)
                    -> PreTraverseToken
    {
        debug_assert!(!(traversal_flags.for_reconstruct() &&
                        traversal_flags.for_unstyled_children_only()),
                      "must not specify FOR_RECONSTRUCT in combination with UNSTYLED_CHILDREN_ONLY");

        if traversal_flags.for_unstyled_children_only() {
            if root.borrow_data().map_or(true, |d| d.has_styles() && d.styles().is_display_none()) {
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

        // Expand the snapshot, if any. This is normally handled by the parent, so
        // we need a special case for the root.
        //
        // Expanding snapshots here may create a LATER_SIBLINGS restyle hint, which
        // we propagate to the next sibling element.
        if let Some(mut data) = root.mutate_data() {
            let later_siblings =
                data.compute_final_hint(root,
                                        shared_context,
                                        HintComputationContext::Root);
            if later_siblings {
                if let Some(next) = root.next_sibling_element() {
                    if let Some(mut next_data) = next.mutate_data() {
                        let hint = StoredRestyleHint::subtree_and_later_siblings();
                        next_data.ensure_restyle().hint.insert(hint);
                    }
                }
            }
        }

        PreTraverseToken {
            traverse: Self::node_needs_traversal(root.as_node(), traversal_flags),
            unstyled_children_only: false,
        }
    }

    /// Returns true if traversal should visit a text node. The style system never
    /// processes text nodes, but Servo overrides this to visit them for flow
    /// construction when necessary.
    fn text_node_needs_traversal(node: E::ConcreteNode) -> bool {
        debug_assert!(node.is_text_node());
        false
    }

    /// Returns true if traversal is needed for the given node and subtree.
    fn node_needs_traversal(node: E::ConcreteNode, traversal_flags: TraversalFlags) -> bool {
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

        // If the element is native-anonymous and an ancestor frame will
        // be reconstructed, the child and all its descendants will be
        // destroyed. In that case, we wouldn't need to traverse the
        // subtree...
        //
        // Except if there could be transitions of pseudo-elements, in
        // which
        // case we still need to process them, unfortunately.
        //
        // We need to conservatively continue the traversal to style the
        // pseudo-element in order to properly process potentially-new
        // transitions that we won't see otherwise.
        //
        // But it may be that we no longer match, so detect that case
        // and act appropriately here.
        if el.is_native_anonymous() {
            if let Some(parent) = el.parent_element() {
                let parent_data = parent.borrow_data().unwrap();
                let going_to_reframe = parent_data.get_restyle().map_or(false, |r| {
                    (r.damage | r.damage_handled())
                        .contains(RestyleDamage::reconstruct())
                });

                let mut is_before_or_after_pseudo = false;
                if let Some(pseudo) = el.implemented_pseudo_element() {
                    if pseudo.is_before_or_after() {
                        is_before_or_after_pseudo = true;
                        let still_match =
                            parent_data.styles().pseudos.get(&pseudo).is_some();

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

        // In case of animation-only traversal we need to traverse
        // the element if the element has animation only dirty
        // descendants bit, animation-only restyle hint or recascade.
        if traversal_flags.for_animation_only() {
            if el.has_animation_only_dirty_descendants() {
                return true;
            }

            let data = match el.borrow_data() {
                Some(d) => d,
                None => return false,
            };
            return data.get_restyle()
                       .map_or(false, |r| r.hint.has_animation_hint() ||
                                          r.hint.has_recascade_self());
        }

        // If the dirty descendants bit is set, we need to traverse no
        // matter what. Skip examining the ElementData.
        if el.has_dirty_descendants() {
            return true;
        }

        // Check the element data. If it doesn't exist, we need to visit
        // the element.
        let data = match el.borrow_data() {
            Some(d) => d,
            None => return true,
        };

        // If we don't have any style data, we need to visit the element.
        if !data.has_styles() {
            return true;
        }

        // Check the restyle data.
        if let Some(r) = data.get_restyle() {
            // If we have a restyle hint or need to recascade, we need to
            // visit the element.
            //
            // Note that this is different than checking has_current_styles(),
            // since that can return true even if we have a restyle hint
            // indicating that the element's descendants (but not necessarily
            // the element) need restyling.
            if !r.hint.is_empty() {
                return true;
            }
        }

        // Servo uses the post-order traversal for flow construction, so
        // we need to traverse any element with damage so that we can perform
        // fixup / reconstruction on our way back up the tree.
        //
        // We also need to traverse nodes with explicit damage and no other
        // restyle data, so that this damage can be cleared.
        if (cfg!(feature = "servo") || traversal_flags.for_reconstruct()) &&
           data.get_restyle().map_or(false, |r| !r.damage.is_empty()) {
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
    fn should_traverse_children(&self,
                                thread_local: &mut ThreadLocalStyleContext<E>,
                                parent: E,
                                parent_data: &ElementData,
                                log: LogBehavior) -> bool
    {
        // See the comment on `cascade_node` for why we allow this on Gecko.
        debug_assert!(cfg!(feature = "gecko") || parent.has_current_styles(parent_data));

        // If the parent computed display:none, we don't style the subtree.
        if parent_data.styles().is_display_none() {
            if log.allow() { debug!("Parent {:?} is display:none, culling traversal", parent); }
            return false;
        }

        // Gecko-only XBL handling.
        //
        // If we're computing initial styles and the parent has a Gecko XBL
        // binding, that binding may inject anonymous children and remap the
        // explicit children to an insertion point (or hide them entirely). It
        // may also specify a scoped stylesheet, which changes the rules that
        // apply within the subtree. These two effects can invalidate the result
        // of property inheritance and selector matching (respectively) within the
        // subtree.
        //
        // To avoid wasting work, we defer initial styling of XBL subtrees
        // until frame construction, which does an explicit traversal of the
        // unstyled children after shuffling the subtree. That explicit
        // traversal may in turn find other bound elements, which get handled
        // in the same way.
        //
        // We explicitly avoid handling restyles here (explicitly removing or
        // changing bindings), since that adds complexity and is rarer. If it
        // happens, we may just end up doing wasted work, since Gecko
        // recursively drops Servo ElementData when the XBL insertion parent of
        // an Element is changed.
        if cfg!(feature = "gecko") && thread_local.is_initial_style() &&
           parent_data.styles().primary.values().has_moz_binding() {
            if log.allow() { debug!("Parent {:?} has XBL binding, deferring traversal", parent); }
            return false;
        }

        return true;
    }

    /// Helper for the traversal implementations to select the children that
    /// should be enqueued for processing.
    fn traverse_children<F>(&self, thread_local: &mut Self::ThreadLocalContext, parent: E, mut f: F)
        where F: FnMut(&mut Self::ThreadLocalContext, E::ConcreteNode)
    {
        // Check if we're allowed to traverse past this element.
        let should_traverse =
            self.should_traverse_children(thread_local.borrow_mut(), parent,
                                          &parent.borrow_data().unwrap(), MayLog);
        thread_local.borrow_mut().end_element(parent);
        if !should_traverse {
            return;
        }

        for kid in parent.as_node().children() {
            if Self::node_needs_traversal(kid, self.shared_context().traversal_flags) {
                // If we are in a restyle for reconstruction, there is no need to
                // perform a post-traversal, so we don't need to set the dirty
                // descendants bit on the parent.
                if !self.shared_context().traversal_flags.for_reconstruct() {
                    let el = kid.as_element();
                    if el.as_ref().and_then(|el| el.borrow_data())
                                  .map_or(false, |d| d.has_styles()) {
                        unsafe { parent.set_dirty_descendants(); }
                    }
                }
                f(thread_local, kid);
            }
        }
    }

    /// Ensures the existence of the ElementData, and returns it. This can't live
    /// on TNode because of the trait-based separation between Servo's script
    /// and layout crates.
    ///
    /// This is only safe to call in top-down traversal before processing the
    /// children of |element|.
    unsafe fn ensure_element_data(element: &E) -> &AtomicRefCell<ElementData>;

    /// Clears the ElementData attached to this element, if any.
    ///
    /// This is only safe to call in top-down traversal before processing the
    /// children of |element|.
    unsafe fn clear_element_data(element: &E);

    /// Return the shared style context common to all worker threads.
    fn shared_context(&self) -> &SharedStyleContext;

    /// Creates a thread-local context.
    fn create_thread_local_context(&self) -> Self::ThreadLocalContext;

    /// Whether we're performing a parallel traversal.
    ///
    /// NB: We do this check on runtime. We could guarantee correctness in this
    /// regard via the type system via a `TraversalDriver` trait for this trait,
    /// that could be one of two concrete types. It's not clear whether the
    /// potential code size impact of that is worth it.
    fn is_parallel(&self) -> bool;
}

/// Helper for the function below.
fn resolve_style_internal<E, F>(context: &mut StyleContext<E>,
                                element: E, ensure_data: &F)
                                -> Option<E>
    where E: TElement,
          F: Fn(E),
{
    ensure_data(element);
    let mut data = element.mutate_data().unwrap();
    let mut display_none_root = None;

    // If the Element isn't styled, we need to compute its style.
    if data.get_styles().is_none() {
        // Compute the parent style if necessary.
        let parent = element.parent_element();
        if let Some(p) = parent {
            display_none_root = resolve_style_internal(context, p, ensure_data);
        }

        // Maintain the bloom filter. If it doesn't exist, we need to build it
        // from scratch. Otherwise we just need to push the parent.
        if context.thread_local.bloom_filter.is_empty() {
            context.thread_local.bloom_filter.rebuild(element);
        } else {
            context.thread_local.bloom_filter.push(parent.unwrap());
            context.thread_local.bloom_filter.assert_complete(element);
        }

        // Compute our style.
        context.thread_local.begin_element(element, &data);
        element.match_and_cascade(context,
                                  &mut data,
                                  StyleSharingBehavior::Disallow);
        context.thread_local.end_element(element);

        if !context.shared.traversal_flags.for_default_styles() {
            // Conservatively mark us as having dirty descendants, since there might
            // be other unstyled siblings we miss when walking straight up the parent
            // chain.  No need to do this if we're computing default styles, since
            // resolve_default_style will want the tree to be left as it is.
            unsafe { element.note_descendants::<DirtyDescendants>() };
        }
    }

    // If we're display:none and none of our ancestors are, we're the root
    // of a display:none subtree.
    if display_none_root.is_none() && data.styles().is_display_none() {
        display_none_root = Some(element);
    }

    return display_none_root
}

/// Manually resolve style by sequentially walking up the parent chain to the
/// first styled Element, ignoring pending restyles. The resolved style is
/// made available via a callback, and can be dropped by the time this function
/// returns in the display:none subtree case.
pub fn resolve_style<E, F, G, H>(context: &mut StyleContext<E>, element: E,
                                 ensure_data: &F, clear_data: &G, callback: H)
    where E: TElement,
          F: Fn(E),
          G: Fn(E),
          H: FnOnce(&ElementStyles)
{
    // Clear the bloom filter, just in case the caller is reusing TLS.
    context.thread_local.bloom_filter.clear();

    // Resolve styles up the tree.
    let display_none_root = resolve_style_internal(context, element, ensure_data);

    // Make them available for the scope of the callback. The callee may use the
    // argument, or perform any other processing that requires the styles to exist
    // on the Element.
    callback(element.borrow_data().unwrap().styles());

    // Clear any styles in display:none subtrees or subtrees not in the document,
    // to leave the tree in a valid state.  For display:none subtrees, we leave
    // the styles on the display:none root, but for subtrees not in the document,
    // we clear styles all the way up to the root of the disconnected subtree.
    let in_doc = element.as_node().is_in_doc();
    if !in_doc || display_none_root.is_some() {
        let mut curr = element;
        loop {
            unsafe {
                curr.unset_dirty_descendants();
                curr.unset_animation_only_dirty_descendants();
            }
            if in_doc && curr == display_none_root.unwrap() {
                break;
            }
            clear_data(curr);
            curr = match curr.parent_element() {
                Some(parent) => parent,
                None => break,
            };
        }
    }
}

/// Manually resolve default styles for the given Element, which are the styles
/// only taking into account user agent and user cascade levels.  The resolved
/// style is made available via a callback, and will be dropped by the time this
/// function returns.
pub fn resolve_default_style<E, F, G, H>(context: &mut StyleContext<E>,
                                         element: E,
                                         ensure_data: &F,
                                         set_data: &G,
                                         callback: H)
    where E: TElement,
          F: Fn(E),
          G: Fn(E, Option<ElementData>) -> Option<ElementData>,
          H: FnOnce(&ElementStyles)
{
    // Save and clear out element data from the element and its ancestors.
    let mut old_data: SmallVec<[(E, Option<ElementData>); 8]> = SmallVec::new();
    {
        let mut e = element;
        loop {
            old_data.push((e, set_data(e, None)));
            match e.parent_element() {
                Some(parent) => e = parent,
                None => break,
            }
        }
    }

    // Resolve styles up the tree.
    resolve_style_internal(context, element, ensure_data);

    // Make them available for the scope of the callback. The callee may use the
    // argument, or perform any other processing that requires the styles to exist
    // on the Element.
    callback(element.borrow_data().unwrap().styles());

    // Swap the old element data back into the element and its ancestors.
    for entry in old_data {
        set_data(entry.0, entry.1);
    }
}

/// Calculates the style for a single node.
#[inline]
#[allow(unsafe_code)]
pub fn recalc_style_at<E, D>(traversal: &D,
                             traversal_data: &PerLevelTraversalData,
                             context: &mut StyleContext<E>,
                             element: E,
                             data: &mut ElementData)
    where E: TElement,
          D: DomTraversal<E>
{
    context.thread_local.begin_element(element, data);
    context.thread_local.statistics.elements_traversed += 1;
    debug_assert!(!element.has_snapshot() || element.handled_snapshot(),
                  "Should've handled snapshots here already");
    debug_assert!(data.get_restyle().map_or(true, |r| {
        !r.has_sibling_invalidations()
    }), "Should've computed the final hint and handled later_siblings already");

    let compute_self = !element.has_current_styles(data);
    let mut cascade_hint = CascadeHint::empty();

    debug!("recalc_style_at: {:?} (compute_self={:?}, dirty_descendants={:?}, data={:?})",
           element, compute_self, element.has_dirty_descendants(), data);

    // Compute style for this element if necessary.
    if compute_self {
        match compute_style(traversal, traversal_data, context, element, data) {
            ChildCascadeRequirement::MustCascadeChildren => {
                cascade_hint |= RECASCADE_SELF;
            }
            ChildCascadeRequirement::MustCascadeDescendants => {
                cascade_hint |= RECASCADE_SELF | RECASCADE_DESCENDANTS;
            }
            ChildCascadeRequirement::CanSkipCascade => {}
        };

        // If we're restyling this element to display:none, throw away all style
        // data in the subtree, notify the caller to early-return.
        if data.styles().is_display_none() {
            debug!("{:?} style is display:none - clearing data from descendants.",
                   element);
            clear_descendant_data(element, &|e| unsafe { D::clear_element_data(&e) });
        }
    }

    // Now that matching and cascading is done, clear the bits corresponding to
    // those operations and compute the propagated restyle hint.
    let mut propagated_hint = match data.get_restyle_mut() {
        None => StoredRestyleHint::empty(),
        Some(r) => {
            debug_assert!(context.shared.traversal_flags.for_animation_only() ||
                          !r.hint.has_animation_hint(),
                          "animation restyle hint should be handled during \
                           animation-only restyles");
            r.hint.propagate(&context.shared.traversal_flags)
        },
    };

    // FIXME(bholley): Need to handle explicitly-inherited reset properties
    // somewhere.
    propagated_hint.insert_cascade_hint(cascade_hint);

    trace!("propagated_hint={:?}, cascade_hint={:?}, \
            is_display_none={:?}, implementing_pseudo={:?}",
           propagated_hint, cascade_hint,
           data.styles().is_display_none(),
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

    // Preprocess children, propagating restyle hints and handling sibling relationships.
    if traversal.should_traverse_children(&mut context.thread_local,
                                          element,
                                          &data,
                                          DontLog) &&
        (has_dirty_descendants_for_this_restyle ||
         !propagated_hint.is_empty()) {
        let damage_handled = data.get_restyle().map_or(RestyleDamage::empty(), |r| {
            r.damage_handled() | r.damage.handled_for_descendants()
        });

        preprocess_children::<E, D>(context,
                                    traversal_data,
                                    element,
                                    propagated_hint,
                                    damage_handled);
    }

    // If we are in a restyle for reconstruction, drop the existing restyle
    // data here, since we won't need to perform a post-traversal to pick up
    // any change hints.
    if context.shared.traversal_flags.for_reconstruct() {
        data.clear_restyle();
    }

    if context.shared.traversal_flags.for_animation_only() {
        unsafe { element.unset_animation_only_dirty_descendants(); }
    }

    // There are two cases when we want to clear the dity descendants bit
    // here after styling this element.
    //
    // The first case is when this element is the root of a display:none
    // subtree, even if the style didn't change (since, if the style did
    // change, we'd have already cleared it above).
    //
    // This keeps the tree in a valid state without requiring the DOM to
    // check display:none on the parent when inserting new children (which
    // can be moderately expensive). Instead, DOM implementations can
    // unconditionally set the dirty descendants bit on any styled parent,
    // and let the traversal sort it out.
    //
    // The second case is when we are in a restyle for reconstruction,
    // where we won't need to perform a post-traversal to pick up any
    // change hints.
    if data.styles().is_display_none() ||
       context.shared.traversal_flags.for_reconstruct() {
        unsafe { element.unset_dirty_descendants(); }
    }
}

fn compute_style<E, D>(_traversal: &D,
                       traversal_data: &PerLevelTraversalData,
                       context: &mut StyleContext<E>,
                       element: E,
                       data: &mut ElementData)
                       -> ChildCascadeRequirement
    where E: TElement,
          D: DomTraversal<E>,
{
    use data::RestyleKind::*;
    use sharing::StyleSharingResult::*;

    context.thread_local.statistics.elements_styled += 1;
    let kind = data.restyle_kind();

    // First, try the style sharing cache. If we get a match we can skip the rest
    // of the work.
    if let MatchAndCascade = kind {
        let target = StyleSharingTarget::new(element);
        let sharing_result = target.share_style_if_possible(context, data);

        if let StyleWasShared(index, had_damage) = sharing_result {
            context.thread_local.statistics.styles_shared += 1;
            context.thread_local.style_sharing_candidate_cache.touch(index);
            return had_damage;
        }
    }

    match kind {
        MatchAndCascade => {
            // Ensure the bloom filter is up to date.
            context.thread_local.bloom_filter
                   .insert_parents_recovering(element,
                                              traversal_data.current_dom_depth);

            context.thread_local.bloom_filter.assert_complete(element);
            context.thread_local.statistics.elements_matched += 1;

            // Perform the matching and cascading.
            element.match_and_cascade(
                context,
                data,
                StyleSharingBehavior::Allow
            )
        }
        CascadeWithReplacements(flags) => {
            let important_rules_changed = element.replace_rules(flags, context, data);
            element.cascade_primary_and_pseudos(
                context,
                data,
                important_rules_changed
            )
        }
        CascadeOnly => {
            element.cascade_primary_and_pseudos(
                context,
                data,
                /* important_rules_changed = */ false
            )
        }
    }
}

fn preprocess_children<E, D>(context: &mut StyleContext<E>,
                             parent_traversal_data: &PerLevelTraversalData,
                             element: E,
                             mut propagated_hint: StoredRestyleHint,
                             damage_handled: RestyleDamage)
    where E: TElement,
          D: DomTraversal<E>,
{
    trace!("preprocess_children: {:?}", element);

    // Loop over all the children.
    for child in element.as_node().children() {
        // FIXME(bholley): Add TElement::element_children instead of this.
        let child = match child.as_element() {
            Some(el) => el,
            None => continue,
        };

        let mut child_data =
            unsafe { D::ensure_element_data(&child).borrow_mut() };

        // If the child is unstyled, we don't need to set up any restyling.
        if !child_data.has_styles() {
            continue;
        }

        // Handle element snapshots and sibling restyle hints.
        //
        // NB: This will be a no-op if there's no restyle data and no snapshot.
        let later_siblings =
            child_data.compute_final_hint(child,
                                          &context.shared,
                                          HintComputationContext::Child {
                                              local_context: &mut context.thread_local,
                                              dom_depth: parent_traversal_data.current_dom_depth + 1,
                                          });

        trace!(" > {:?} -> {:?} + {:?}, pseudo: {:?}, later_siblings: {:?}",
               child,
               child_data.get_restyle().map(|r| &r.hint),
               propagated_hint,
               child.implemented_pseudo_element(),
               later_siblings);

        // If the child doesn't have pre-existing RestyleData and we don't have
        // any reason to create one, avoid the useless allocation and move on to
        // the next child.
        if propagated_hint.is_empty() && damage_handled.is_empty() && !child_data.has_restyle() {
            continue;
        }

        let mut restyle_data = child_data.ensure_restyle();

        // Propagate the parent and sibling restyle hint.
        restyle_data.hint.insert_from(&propagated_hint);

        if later_siblings {
            propagated_hint.insert(RestyleHint::subtree().into());
        }

        // Store the damage already handled by ancestors.
        restyle_data.set_damage_handled(damage_handled);
    }
}

/// Clear style data for all the subtree under `el`.
pub fn clear_descendant_data<E: TElement, F: Fn(E)>(el: E, clear_data: &F) {
    for kid in el.as_node().children() {
        if let Some(kid) = kid.as_element() {
            // We maintain an invariant that, if an element has data, all its ancestors
            // have data as well. By consequence, any element without data has no
            // descendants with data.
            if kid.get_data().is_some() {
                clear_data(kid);
                clear_descendant_data(kid, clear_data);
            }
        }
    }

    unsafe {
        el.unset_dirty_descendants();
        el.unset_animation_only_dirty_descendants();
    }
}
