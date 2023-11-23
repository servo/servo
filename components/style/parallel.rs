/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implements parallel traversal over the DOM tree.
//!
//! This traversal is based on Rayon, and therefore its safety is largely
//! verified by the type system.
//!
//! The primary trickiness and fine print for the above relates to the
//! thread safety of the DOM nodes themselves. Accessing a DOM element
//! concurrently on multiple threads is actually mostly "safe", since all
//! the mutable state is protected by an AtomicRefCell, and so we'll
//! generally panic if something goes wrong. Still, we try to to enforce our
//! thread invariants at compile time whenever possible. As such, TNode and
//! TElement are not Send, so ordinary style system code cannot accidentally
//! share them with other threads. In the parallel traversal, we explicitly
//! invoke |unsafe { SendNode::new(n) }| to put nodes in containers that may
//! be sent to other threads. This occurs in only a handful of places and is
//! easy to grep for. At the time of this writing, there is no other unsafe
//! code in the parallel traversal.

#![deny(missing_docs)]

use crate::context::{StyleContext, ThreadLocalStyleContext};
use crate::dom::{OpaqueNode, SendNode, TElement};
use crate::scoped_tls::ScopedTLS;
use crate::traversal::{DomTraversal, PerLevelTraversalData};
use rayon;
use smallvec::SmallVec;

/// The minimum stack size for a thread in the styling pool, in kilobytes.
#[cfg(feature = "gecko")]
pub const STYLE_THREAD_STACK_SIZE_KB: usize = 256;

/// The minimum stack size for a thread in the styling pool, in kilobytes.
/// Servo requires a bigger stack in debug builds.
#[cfg(feature = "servo")]
pub const STYLE_THREAD_STACK_SIZE_KB: usize = 512;

/// The stack margin. If we get this deep in the stack, we will skip recursive
/// optimizations to ensure that there is sufficient room for non-recursive work.
///
/// We allocate large safety margins because certain OS calls can use very large
/// amounts of stack space [1]. Reserving a larger-than-necessary stack costs us
/// address space, but if we keep our safety margin big, we will generally avoid
/// committing those extra pages, and only use them in edge cases that would
/// otherwise cause crashes.
///
/// When measured with 128KB stacks and 40KB margin, we could support 53
/// levels of recursion before the limiter kicks in, on x86_64-Linux [2]. When
/// we doubled the stack size, we added it all to the safety margin, so we should
/// be able to get the same amount of recursion.
///
/// [1] https://bugzilla.mozilla.org/show_bug.cgi?id=1395708#c15
/// [2] See Gecko bug 1376883 for more discussion on the measurements.
///
pub const STACK_SAFETY_MARGIN_KB: usize = 168;

/// See documentation of the pref for performance characteristics.
pub fn work_unit_max() -> usize {
    static_prefs::pref!("layout.css.stylo-work-unit-size") as usize
}

/// A callback to create our thread local context.  This needs to be
/// out of line so we don't allocate stack space for the entire struct
/// in the caller.
#[inline(never)]
fn create_thread_local_context<'scope, E>(slot: &mut Option<ThreadLocalStyleContext<E>>)
where
    E: TElement + 'scope,
{
    *slot = Some(ThreadLocalStyleContext::new());
}

/// A parallel top-down DOM traversal.
///
/// This algorithm traverses the DOM in a breadth-first, top-down manner. The
/// goals are:
/// * Never process a child before its parent (since child style depends on
///   parent style). If this were to happen, the styling algorithm would panic.
/// * Prioritize discovering nodes as quickly as possible to maximize
///   opportunities for parallelism.  But this needs to be weighed against
///   styling cousins on a single thread to improve sharing.
/// * Style all the children of a given node (i.e. all sibling nodes) on
///   a single thread (with an upper bound to handle nodes with an
///   abnormally large number of children). This is important because we use
///   a thread-local cache to share styles between siblings.
#[inline(always)]
#[allow(unsafe_code)]
fn top_down_dom<'a, 'scope, E, D>(
    nodes: &'a [SendNode<E::ConcreteNode>],
    root: OpaqueNode,
    mut traversal_data: PerLevelTraversalData,
    scope: &'a rayon::ScopeFifo<'scope>,
    pool: &'scope rayon::ThreadPool,
    traversal: &'scope D,
    tls: &'scope ScopedTLS<'scope, ThreadLocalStyleContext<E>>,
) where
    E: TElement + 'scope,
    D: DomTraversal<E>,
{
    let work_unit_max = work_unit_max();
    debug_assert!(nodes.len() <= work_unit_max);

    // We set this below, when we have a borrow of the thread-local-context
    // available.
    let recursion_ok;

    // Collect all the children of the elements in our work unit. This will
    // contain the combined children of up to work_unit_max nodes, which may
    // be numerous. As such, we store it in a large SmallVec to minimize heap-
    // spilling, and never move it.
    let mut discovered_child_nodes = SmallVec::<[SendNode<E::ConcreteNode>; 128]>::new();
    {
        // Scope the borrow of the TLS so that the borrow is dropped before
        // a potential recursive call when we pass TailCall.
        let mut tlc = tls.ensure(|slot: &mut Option<ThreadLocalStyleContext<E>>| {
            create_thread_local_context(slot)
        });

        // Check that we're not in danger of running out of stack.
        recursion_ok = !tlc.stack_limit_checker.limit_exceeded();

        let mut context = StyleContext {
            shared: traversal.shared_context(),
            thread_local: &mut *tlc,
        };

        for n in nodes {
            // If the last node we processed produced children, we may want to
            // spawn them off into a work item. We do this at the beginning of
            // the loop (rather than at the end) so that we can traverse our
            // last bits of work directly on this thread without a spawn call.
            //
            // This has the important effect of removing the allocation and
            // context-switching overhead of the parallel traversal for perfectly
            // linear regions of the DOM, i.e.:
            //
            // <russian><doll><tag><nesting></nesting></tag></doll></russian>
            //
            // which are not at all uncommon.
            //
            // There's a tension here between spawning off a work item as soon
            // as discovered_child_nodes is nonempty and waiting until we have a
            // full work item to do so.  The former optimizes for speed of
            // discovery (we'll start discovering the kids of the things in
            // "nodes" ASAP).  The latter gives us better sharing (e.g. we can
            // share between cousins much better, because we don't hand them off
            // as separate work items, which are likely to end up on separate
            // threads) and gives us a chance to just handle everything on this
            // thread for small DOM subtrees, as in the linear example above.
            //
            // There are performance and "number of ComputedValues"
            // measurements for various testcases in
            // https://bugzilla.mozilla.org/show_bug.cgi?id=1385982#c10 and
            // following.
            //
            // The worst case behavior for waiting until we have a full work
            // item is a deep tree which has work_unit_max "linear" branches,
            // hence work_unit_max elements at each level.  Such a tree would
            // end up getting processed entirely sequentially, because we would
            // process each level one at a time as a single work unit, whether
            // via our end-of-loop tail call or not.  If we kicked off a
            // traversal as soon as we discovered kids, we would instead
            // process such a tree more or less with a thread-per-branch,
            // multiplexed across our actual threadpool.
            if discovered_child_nodes.len() >= work_unit_max {
                let mut traversal_data_copy = traversal_data.clone();
                traversal_data_copy.current_dom_depth += 1;
                traverse_nodes(
                    &discovered_child_nodes,
                    DispatchMode::NotTailCall,
                    recursion_ok,
                    root,
                    traversal_data_copy,
                    scope,
                    pool,
                    traversal,
                    tls,
                );
                discovered_child_nodes.clear();
            }

            let node = **n;
            let mut children_to_process = 0isize;
            traversal.process_preorder(&traversal_data, &mut context, node, |n| {
                children_to_process += 1;
                let send_n = unsafe { SendNode::new(n) };
                discovered_child_nodes.push(send_n);
            });

            traversal.handle_postorder_traversal(&mut context, root, node, children_to_process);
        }
    }

    // Handle whatever elements we have queued up but not kicked off traversals
    // for yet.  If any exist, we can process them (or at least one work unit's
    // worth of them) directly on this thread by passing TailCall.
    if !discovered_child_nodes.is_empty() {
        traversal_data.current_dom_depth += 1;
        traverse_nodes(
            &discovered_child_nodes,
            DispatchMode::TailCall,
            recursion_ok,
            root,
            traversal_data,
            scope,
            pool,
            traversal,
            tls,
        );
    }
}

/// Controls whether traverse_nodes may make a recursive call to continue
/// doing work, or whether it should always dispatch work asynchronously.
#[derive(Clone, Copy, PartialEq)]
pub enum DispatchMode {
    /// This is the last operation by the caller.
    TailCall,
    /// This is not the last operation by the caller.
    NotTailCall,
}

impl DispatchMode {
    fn is_tail_call(&self) -> bool {
        matches!(*self, DispatchMode::TailCall)
    }
}

/// Enqueues |nodes| for processing, possibly on this thread if the tail call
/// conditions are met.
#[inline]
pub fn traverse_nodes<'a, 'scope, E, D>(
    nodes: &[SendNode<E::ConcreteNode>],
    mode: DispatchMode,
    recursion_ok: bool,
    root: OpaqueNode,
    traversal_data: PerLevelTraversalData,
    scope: &'a rayon::ScopeFifo<'scope>,
    pool: &'scope rayon::ThreadPool,
    traversal: &'scope D,
    tls: &'scope ScopedTLS<'scope, ThreadLocalStyleContext<E>>,
) where
    E: TElement + 'scope,
    D: DomTraversal<E>,
{
    debug_assert_ne!(nodes.len(), 0);

    // This is a tail call from the perspective of the caller. However, we only
    // want to actually dispatch the job as a tail call if there's nothing left
    // in our local queue. Otherwise we need to return to it to maintain proper
    // breadth-first ordering. We also need to take care to avoid stack
    // overflow due to excessive tail recursion. The stack overflow avoidance
    // isn't observable to content -- we're still completely correct, just not
    // using tail recursion any more. See Gecko bugs 1368302 and 1376883.
    let may_dispatch_tail =
        mode.is_tail_call() && recursion_ok && !pool.current_thread_has_pending_tasks().unwrap();

    let work_unit_max = work_unit_max();
    // In the common case, our children fit within a single work unit, in which case we can pass
    // the nodes directly and avoid extra allocation.
    if nodes.len() <= work_unit_max {
        if may_dispatch_tail {
            top_down_dom(&nodes, root, traversal_data, scope, pool, traversal, tls);
        } else {
            let work = nodes.to_vec();
            scope.spawn_fifo(move |scope| {
                #[cfg(feature = "gecko")]
                gecko_profiler_label!(Layout, StyleComputation);
                top_down_dom(&work, root, traversal_data, scope, pool, traversal, tls);
            });
        }
    } else {
        for chunk in nodes.chunks(work_unit_max) {
            let work = chunk.to_vec();
            let traversal_data_copy = traversal_data.clone();
            scope.spawn_fifo(move |scope| {
                #[cfg(feature = "gecko")]
                gecko_profiler_label!(Layout, StyleComputation);
                let work = work;
                top_down_dom(
                    &work,
                    root,
                    traversal_data_copy,
                    scope,
                    pool,
                    traversal,
                    tls,
                )
            });
        }
    }
}
