/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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

use arrayvec::ArrayVec;
use context::{StyleContext, ThreadLocalStyleContext, TraversalStatistics};
use dom::{OpaqueNode, SendNode, TElement, TNode};
use rayon;
use scoped_tls::ScopedTLS;
use smallvec::SmallVec;
use std::borrow::Borrow;
use time;
use traversal::{DomTraversal, PerLevelTraversalData, PreTraverseToken};

/// The maximum number of child nodes that we will process as a single unit.
///
/// Larger values will increase style sharing cache hits and general DOM locality
/// at the expense of decreased opportunities for parallelism. This value has not
/// been measured and could potentially be tuned.
pub const WORK_UNIT_MAX: usize = 16;

/// A set of nodes, sized to the work unit. This gets copied when sent to other
/// threads, so we keep it compact.
type WorkUnit<N> = ArrayVec<[SendNode<N>; WORK_UNIT_MAX]>;

/// Entry point for the parallel traversal.
#[allow(unsafe_code)]
pub fn traverse_dom<E, D>(traversal: &D,
                          root: E,
                          token: PreTraverseToken,
                          pool: &rayon::ThreadPool)
    where E: TElement,
          D: DomTraversal<E>,
{
    let dump_stats = traversal.shared_context().options.dump_style_statistics;
    let start_time = if dump_stats { Some(time::precise_time_s()) } else { None };

    // Set up the SmallVec. We need to move this, and in most cases this is just
    // one node, so keep it small.
    let mut nodes = SmallVec::<[SendNode<E::ConcreteNode>; 8]>::new();

    debug_assert!(traversal.is_parallel());
    // Handle Gecko's eager initial styling. We don't currently support it
    // in conjunction with bottom-up traversal. If we did, we'd need to put
    // it on the context to make it available to the bottom-up phase.
    let depth = if token.traverse_unstyled_children_only() {
        debug_assert!(!D::needs_postorder_traversal());
        for kid in root.as_node().traversal_children() {
            if kid.as_element().map_or(false, |el| el.get_data().is_none()) {
                nodes.push(unsafe { SendNode::new(kid) });
            }
        }
        root.depth() + 1
    } else {
        nodes.push(unsafe { SendNode::new(root.as_node()) });
        root.depth()
    };

    if nodes.is_empty() {
        return;
    }

    let traversal_data = PerLevelTraversalData {
        current_dom_depth: depth,
    };
    let tls = ScopedTLS::<ThreadLocalStyleContext<E>>::new(pool);
    let root = root.as_node().opaque();

    pool.install(|| {
        rayon::scope(|scope| {
            let nodes = nodes;
            traverse_nodes(&*nodes,
                           DispatchMode::TailCall,
                           0,
                           root,
                           traversal_data,
                           scope,
                           pool,
                           traversal,
                           &tls);
        });
    });

    // Dump statistics to stdout if requested.
    if dump_stats {
        let slots = unsafe { tls.unsafe_get() };
        let mut aggregate = slots.iter().fold(TraversalStatistics::default(), |acc, t| {
            match *t.borrow() {
                None => acc,
                Some(ref cx) => &cx.borrow().statistics + &acc,
            }
        });
        aggregate.finish(traversal, start_time.unwrap());
        if aggregate.is_large_traversal() {
            println!("{}", aggregate);
        }
    }
}

/// A callback to create our thread local context.  This needs to be
/// out of line so we don't allocate stack space for the entire struct
/// in the caller.
#[inline(never)]
fn create_thread_local_context<'scope, E, D>(
    traversal: &'scope D,
    slot: &mut Option<ThreadLocalStyleContext<E>>)
    where E: TElement + 'scope,
          D: DomTraversal<E>
{
    *slot = Some(ThreadLocalStyleContext::new(traversal.shared_context()));
}

/// A parallel top-down DOM traversal.
///
/// This algorithm traverses the DOM in a breadth-first, top-down manner. The
/// goals are:
/// * Never process a child before its parent (since child style depends on
///   parent style). If this were to happen, the styling algorithm would panic.
/// * Prioritize discovering nodes as quickly as possible to maximize
///   opportunities for parallelism.
/// * Style all the children of a given node (i.e. all sibling nodes) on
///   a single thread (with an upper bound to handle nodes with an
///   abnormally large number of children). This is important because we use
///   a thread-local cache to share styles between siblings.
#[inline(always)]
#[allow(unsafe_code)]
fn top_down_dom<'a, 'scope, E, D>(nodes: &'a [SendNode<E::ConcreteNode>],
                                  recursion_depth: usize,
                                  root: OpaqueNode,
                                  mut traversal_data: PerLevelTraversalData,
                                  scope: &'a rayon::Scope<'scope>,
                                  pool: &'scope rayon::ThreadPool,
                                  traversal: &'scope D,
                                  tls: &'scope ScopedTLS<'scope, ThreadLocalStyleContext<E>>)
    where E: TElement + 'scope,
          D: DomTraversal<E>,
{
    debug_assert!(nodes.len() <= WORK_UNIT_MAX);

    // Collect all the children of the elements in our work unit. This will
    // contain the combined children of up to WORK_UNIT_MAX nodes, which may
    // be numerous. As such, we store it in a large SmallVec to minimize heap-
    // spilling, and never move it.
    let mut discovered_child_nodes = SmallVec::<[SendNode<E::ConcreteNode>; 128]>::new();
    {
        // Scope the borrow of the TLS so that the borrow is dropped before
        // a potential recursive call when we pass TailCall.
        let mut tlc = tls.ensure(
            |slot: &mut Option<ThreadLocalStyleContext<E>>| create_thread_local_context(traversal, slot));
        let mut context = StyleContext {
            shared: traversal.shared_context(),
            thread_local: &mut *tlc,
        };

        for n in nodes {
            // If the last node we processed produced children, spawn them off
            // into a work item. We do this at the beginning of the loop (rather
            // than at the end) so that we can traverse the children of the last
            // sibling directly on this thread without a spawn call.
            //
            // This has the important effect of removing the allocation and
            // context-switching overhead of the parallel traversal for perfectly
            // linear regions of the DOM, i.e.:
            //
            // <russian><doll><tag><nesting></nesting></tag></doll></russian>
            //
            // Which are not at all uncommon.
            if !discovered_child_nodes.is_empty() {
                let mut traversal_data_copy = traversal_data.clone();
                traversal_data_copy.current_dom_depth += 1;
                traverse_nodes(&*discovered_child_nodes,
                               DispatchMode::NotTailCall,
                               recursion_depth,
                               root,
                               traversal_data_copy,
                               scope,
                               pool,
                               traversal,
                               tls);
                discovered_child_nodes.clear();
            }

            let node = **n;
            let mut children_to_process = 0isize;
            traversal.process_preorder(&traversal_data, &mut context, node, |n| {
                children_to_process += 1;
                let send_n = unsafe { SendNode::new(n) };
                discovered_child_nodes.push(send_n);
            });

            traversal.handle_postorder_traversal(&mut context, root, node,
                                                 children_to_process);
        }
    }

    // Handle the children of the last element in this work unit. If any exist,
    // we can process them (or at least one work unit's worth of them) directly
    // on this thread by passing TailCall.
    if !discovered_child_nodes.is_empty() {
        traversal_data.current_dom_depth += 1;
        traverse_nodes(&discovered_child_nodes,
                       DispatchMode::TailCall,
                       recursion_depth,
                       root,
                       traversal_data,
                       scope,
                       pool,
                       traversal,
                       tls);
    }
}

/// Controls whether traverse_nodes may make a recursive call to continue
/// doing work, or whether it should always dispatch work asynchronously.
#[derive(Clone, Copy, PartialEq)]
enum DispatchMode {
    TailCall,
    NotTailCall,
}

impl DispatchMode {
    fn is_tail_call(&self) -> bool { matches!(*self, DispatchMode::TailCall) }
}

// On x86_64-linux, a recursive cycle requires 3472 bytes of stack.  Limiting
// the depth to 150 therefore should keep the stack use by the recursion to
// 520800 bytes, which would give a generously conservative margin should we
// decide to reduce the thread stack size from its default of 2MB down to 1MB.
const RECURSION_DEPTH_LIMIT: usize = 150;

#[inline]
fn traverse_nodes<'a, 'scope, E, D>(nodes: &[SendNode<E::ConcreteNode>],
                                    mode: DispatchMode,
                                    recursion_depth: usize,
                                    root: OpaqueNode,
                                    traversal_data: PerLevelTraversalData,
                                    scope: &'a rayon::Scope<'scope>,
                                    pool: &'scope rayon::ThreadPool,
                                    traversal: &'scope D,
                                    tls: &'scope ScopedTLS<'scope, ThreadLocalStyleContext<E>>)
    where E: TElement + 'scope,
          D: DomTraversal<E>,
{
    debug_assert!(!nodes.is_empty());

    // This is a tail call from the perspective of the caller. However, we only
    // want to actually dispatch the job as a tail call if there's nothing left
    // in our local queue. Otherwise we need to return to it to maintain proper
    // breadth-first ordering. We also need to take care to avoid stack
    // overflow due to excessive tail recursion. The stack overflow isn't
    // observable to content -- we're still completely correct, just not
    // using tail recursion any more. See bug 1368302.
    debug_assert!(recursion_depth <= RECURSION_DEPTH_LIMIT);
    let may_dispatch_tail = mode.is_tail_call() &&
        recursion_depth != RECURSION_DEPTH_LIMIT &&
        !pool.current_thread_has_pending_tasks().unwrap();

    // In the common case, our children fit within a single work unit, in which
    // case we can pass the SmallVec directly and avoid extra allocation.
    if nodes.len() <= WORK_UNIT_MAX {
        let work = nodes.iter().cloned().collect::<WorkUnit<E::ConcreteNode>>();
        if may_dispatch_tail {
            top_down_dom(&work, recursion_depth + 1, root,
                         traversal_data, scope, pool, traversal, tls);
        } else {
            scope.spawn(move |scope| {
                let work = work;
                top_down_dom(&work, 0, root,
                             traversal_data, scope, pool, traversal, tls);
            });
        }
    } else {
        for chunk in nodes.chunks(WORK_UNIT_MAX) {
            let nodes = chunk.iter().cloned().collect::<WorkUnit<E::ConcreteNode>>();
            let traversal_data_copy = traversal_data.clone();
            scope.spawn(move |scope| {
                let n = nodes;
                top_down_dom(&*n, 0, root,
                             traversal_data_copy, scope, pool, traversal, tls)
            });
        }
    }
}
