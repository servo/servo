/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversing the DOM tree; the bloom filter.

#![deny(missing_docs)]

use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use context::{SharedStyleContext, StyleContext, ThreadLocalStyleContext};
use data::{ElementData, ElementStyles, StoredRestyleHint};
use dom::{NodeInfo, TElement, TNode};
use matching::{MatchMethods, MatchResults};
use restyle_hints::{RESTYLE_DESCENDANTS, RESTYLE_SELF};
use selector_parser::RestyleDamage;
use servo_config::opts;
use std::borrow::BorrowMut;
use std::mem;
use stylist::Stylist;

/// A per-traversal-level chunk of data. This is sent down by the traversal, and
/// currently only holds the dom depth for the bloom filter.
///
/// NB: Keep this as small as possible, please!
#[derive(Clone, Debug)]
pub struct PerLevelTraversalData {
    /// The current dom depth, if known, or `None` otherwise.
    ///
    /// This is kept with cooperation from the traversal code and the bloom
    /// filter.
    pub current_dom_depth: Option<usize>,
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

/// A DOM Traversal trait, that is used to generically implement styling for
/// Gecko and Servo.
pub trait DomTraversal<E: TElement> : Sync {
    /// The thread-local context, used to store non-thread-safe stuff that needs
    /// to be used in the traversal, and of which we use one per worker, like
    /// the bloom filter, for example.
    type ThreadLocalContext: Send + BorrowMut<ThreadLocalStyleContext<E>>;

    /// Process `node` on the way down, before its children have been processed.
    fn process_preorder(&self, data: &mut PerLevelTraversalData,
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

    /// Must be invoked before traversing the root element to determine whether
    /// a traversal is needed. Returns a token that allows the caller to prove
    /// that the call happened.
    ///
    /// The unstyled_children_only parameter is used in Gecko to style newly-
    /// appended children without restyling the parent.
    fn pre_traverse(root: E, stylist: &Stylist, unstyled_children_only: bool)
                    -> PreTraverseToken
    {
        if unstyled_children_only {
            return PreTraverseToken {
                traverse: true,
                unstyled_children_only: true,
            };
        }

        // Expand the snapshot, if any. This is normally handled by the parent, so
        // we need a special case for the root.
        //
        // Expanding snapshots here may create a LATER_SIBLINGS restyle hint, which
        // we will drop on the floor. To prevent missed restyles, we assert against
        // restyling a root with later siblings.
        if let Some(mut data) = root.mutate_data() {
            if let Some(r) = data.get_restyle_mut() {
                debug_assert!(root.next_sibling_element().is_none());
                let _later_siblings = r.compute_final_hint(root, stylist);
            }
        }

        PreTraverseToken {
            traverse: Self::node_needs_traversal(root.as_node()),
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
    fn node_needs_traversal(node: E::ConcreteNode) -> bool {
        // Non-incremental layout visits every node.
        if cfg!(feature = "servo") && opts::get().nonincremental_layout {
            return true;
        }

        match node.as_element() {
            None => Self::text_node_needs_traversal(node),
            Some(el) => {
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
                    if !r.hint.is_empty() || r.recascade {
                        return true;
                    }
                }

                // Servo uses the post-order traversal for flow construction, so
                // we need to traverse any element with damage so that we can perform
                // fixup / reconstruction on our way back up the tree.
                if cfg!(feature = "servo") &&
                   data.get_restyle().map_or(false, |r| r.damage != RestyleDamage::empty())
                {
                    return true;
                }

                false
            },
        }
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
        debug_assert!(cfg!(feature = "gecko") || parent_data.has_current_styles());

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
            if Self::node_needs_traversal(kid) {
                let el = kid.as_element();
                if el.as_ref().and_then(|el| el.borrow_data())
                              .map_or(false, |d| d.has_styles())
                {
                    unsafe { parent.set_dirty_descendants(); }
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
fn resolve_style_internal<E, F>(context: &mut StyleContext<E>, element: E, ensure_data: &F)
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
        let match_results = element.match_element(context, &mut data);
        element.cascade_element(context, &mut data,
                                match_results.primary_is_shareable());

        // Conservatively mark us as having dirty descendants, since there might
        // be other unstyled siblings we miss when walking straight up the parent
        // chain.
        unsafe { element.set_dirty_descendants() };
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
            unsafe { curr.unset_dirty_descendants(); }
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

/// Calculates the style for a single node.
#[inline]
#[allow(unsafe_code)]
pub fn recalc_style_at<E, D>(traversal: &D,
                             traversal_data: &mut PerLevelTraversalData,
                             context: &mut StyleContext<E>,
                             element: E,
                             mut data: &mut AtomicRefMut<ElementData>)
    where E: TElement,
          D: DomTraversal<E>
{
    context.thread_local.begin_element(element, &data);
    context.thread_local.statistics.elements_traversed += 1;
    debug_assert!(data.get_restyle().map_or(true, |r| {
        r.snapshot.is_none() && !r.has_sibling_invalidations()
    }), "Should've computed the final hint and handled later_siblings already");

    let compute_self = !data.has_current_styles();
    let mut inherited_style_changed = false;

    debug!("recalc_style_at: {:?} (compute_self={:?}, dirty_descendants={:?}, data={:?})",
           element, compute_self, element.has_dirty_descendants(), data);

    // Compute style for this element if necessary.
    if compute_self {
        compute_style(traversal, traversal_data, context, element, &mut data);

        // If we're restyling this element to display:none, throw away all style
        // data in the subtree, notify the caller to early-return.
        let display_none = data.styles().is_display_none();
        if display_none {
            debug!("New element style is display:none - clearing data from descendants.");
            clear_descendant_data(element, &|e| unsafe { D::clear_element_data(&e) });
        }

        // FIXME(bholley): Compute this accurately from the call to CalcStyleDifference.
        inherited_style_changed = true;
    }

    // Now that matching and cascading is done, clear the bits corresponding to
    // those operations and compute the propagated restyle hint.
    let empty_hint = StoredRestyleHint::empty();
    let propagated_hint = match data.get_restyle_mut() {
        None => empty_hint,
        Some(r) => {
            r.recascade = false;
            mem::replace(&mut r.hint, empty_hint).propagate()
        },
    };
    debug_assert!(data.has_current_styles());
    trace!("propagated_hint={:?}, inherited_style_changed={:?}", propagated_hint, inherited_style_changed);

    // Preprocess children, propagating restyle hints and handling sibling relationships.
    if traversal.should_traverse_children(&mut context.thread_local, element, &data, DontLog) &&
       (element.has_dirty_descendants() || !propagated_hint.is_empty() || inherited_style_changed) {
        let damage_handled = data.get_restyle().map_or(RestyleDamage::empty(), |r| {
            r.damage_handled() | r.damage.handled_for_descendants()
        });
        preprocess_children(traversal, element, propagated_hint, damage_handled, inherited_style_changed);
    }

    // Make sure the dirty descendants bit is not set for the root of a
    // display:none subtree, even if the style didn't change (since, if
    // the style did change, we'd have already cleared it above).
    //
    // This keeps the tree in a valid state without requiring the DOM to
    // check display:none on the parent when inserting new children (which
    // can be moderately expensive). Instead, DOM implementations can
    // unconditionally set the dirty descendants bit on any styled parent,
    // and let the traversal sort it out.
    if data.styles().is_display_none() {
        unsafe { element.unset_dirty_descendants(); }
    }
}

fn compute_style<E, D>(_traversal: &D,
                       traversal_data: &mut PerLevelTraversalData,
                       context: &mut StyleContext<E>,
                       element: E,
                       mut data: &mut AtomicRefMut<ElementData>)
    where E: TElement,
          D: DomTraversal<E>,
{
    use data::RestyleKind::*;
    use matching::StyleSharingResult::*;

    context.thread_local.statistics.elements_styled += 1;
    let shared_context = context.shared;
    let kind = data.restyle_kind();

    // First, try the style sharing cache. If we get a match we can skip the rest
    // of the work.
    if let MatchAndCascade = kind {
        let sharing_result = unsafe {
            let cache = &mut context.thread_local.style_sharing_candidate_cache;
            element.share_style_if_possible(cache, shared_context, &mut data)
        };
        if let StyleWasShared(index) = sharing_result {
            context.thread_local.statistics.styles_shared += 1;
            context.thread_local.style_sharing_candidate_cache.touch(index);
            return;
        }
    }

    let match_results = match kind {
        MatchAndCascade => {
            // Ensure the bloom filter is up to date.
            let dom_depth =
                context.thread_local.bloom_filter
                       .insert_parents_recovering(element,
                                                  traversal_data.current_dom_depth);

            // Update the dom depth with the up-to-date dom depth.
            //
            // Note that this is always the same than the pre-existing depth,
            // but it can change from unknown to known at this step.
            traversal_data.current_dom_depth = Some(dom_depth);

            context.thread_local.bloom_filter.assert_complete(element);


            // Perform CSS selector matching.
            context.thread_local.statistics.elements_matched += 1;
            element.match_element(context, &mut data)
        }
        CascadeWithReplacements(hint) => {
            let rule_nodes_changed =
                element.cascade_with_replacements(hint, context, &mut data);
            MatchResults {
                primary_relations: None,
                rule_nodes_changed: rule_nodes_changed,
            }
        }
        CascadeOnly => {
            MatchResults {
                primary_relations: None,
                rule_nodes_changed: false,
            }
        }
    };

    // Cascade properties and compute values.
    let shareable = match_results.primary_is_shareable();
    unsafe {
        element.cascade_element(context, &mut data, shareable);
    }

    // If the style is shareable, add it to the LRU cache.
    if shareable {
        context.thread_local
               .style_sharing_candidate_cache
               .insert_if_possible(&element,
                                   data.styles().primary.values(),
                                   match_results.primary_relations.unwrap());
    }
}

fn preprocess_children<E, D>(traversal: &D,
                             element: E,
                             mut propagated_hint: StoredRestyleHint,
                             damage_handled: RestyleDamage,
                             parent_inherited_style_changed: bool)
    where E: TElement,
          D: DomTraversal<E>
{
    // Loop over all the children.
    for child in element.as_node().children() {
        // FIXME(bholley): Add TElement::element_children instead of this.
        let child = match child.as_element() {
            Some(el) => el,
            None => continue,
        };

        let mut child_data = unsafe { D::ensure_element_data(&child).borrow_mut() };

        // If the child is unstyled, we don't need to set up any restyling.
        if !child_data.has_styles() {
            continue;
        }

        // If the child doesn't have pre-existing RestyleData and we don't have
        // any reason to create one, avoid the useless allocation and move on to
        // the next child.
        if propagated_hint.is_empty() && !parent_inherited_style_changed &&
           damage_handled.is_empty() && !child_data.has_restyle() {
            continue;
        }
        let mut restyle_data = child_data.ensure_restyle();

        // Propagate the parent and sibling restyle hint.
        if !propagated_hint.is_empty() {
            restyle_data.hint.insert(&propagated_hint);
        }

        // Handle element snapshots and sibling restyle hints.
        let stylist = &traversal.shared_context().stylist;
        let later_siblings = restyle_data.compute_final_hint(child, stylist);
        if later_siblings {
            propagated_hint.insert(&(RESTYLE_SELF | RESTYLE_DESCENDANTS).into());
        }

        // Store the damage already handled by ancestors.
        restyle_data.set_damage_handled(damage_handled);

        // If properties that we inherited from the parent changed, we need to recascade.
        //
        // FIXME(bholley): Need to handle explicitly-inherited reset properties somewhere.
        if parent_inherited_style_changed {
            restyle_data.recascade = true;
        }
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

    unsafe { el.unset_dirty_descendants(); }
}
