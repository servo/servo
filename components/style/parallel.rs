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

use context::TraversalStatistics;
use dom::{OpaqueNode, SendNode, TElement, TNode};
use rayon;
use scoped_tls::ScopedTLS;
use std::borrow::Borrow;
use time;
use traversal::{DomTraversal, PerLevelTraversalData, PreTraverseToken};

/// The chunk size used to split the parallel traversal nodes.
///
/// We send each `CHUNK_SIZE` nodes as a different work unit to the work queue.
pub const CHUNK_SIZE: usize = 64;

/// A parallel top down traversal, generic over `D`.
#[allow(unsafe_code)]
pub fn traverse_dom<E, D>(traversal: &D,
                          root: E,
                          token: PreTraverseToken,
                          queue: &rayon::ThreadPool)
    where E: TElement,
          D: DomTraversal<E>,
{
    let dump_stats = TraversalStatistics::should_dump();
    let start_time = if dump_stats { Some(time::precise_time_s()) } else { None };

    debug_assert!(traversal.is_parallel());
    // Handle Gecko's eager initial styling. We don't currently support it
    // in conjunction with bottom-up traversal. If we did, we'd need to put
    // it on the context to make it available to the bottom-up phase.
    let (nodes, depth) = if token.traverse_unstyled_children_only() {
        debug_assert!(!D::needs_postorder_traversal());
        let mut children = vec![];
        for kid in root.as_node().children() {
            if kid.as_element().map_or(false, |el| el.get_data().is_none()) {
                children.push(unsafe { SendNode::new(kid) });
            }
        }
        (children, root.depth() + 1)
    } else {
        (vec![unsafe { SendNode::new(root.as_node()) }], root.depth())
    };

    let traversal_data = PerLevelTraversalData {
        current_dom_depth: depth,
    };
    let tls = ScopedTLS::<D::ThreadLocalContext>::new(queue);
    let root = root.as_node().opaque();

    queue.install(|| {
        rayon::scope(|scope| {
            traverse_nodes(nodes, root, traversal_data, scope, traversal, &tls);
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
        println!("{}", aggregate);
    }
}

/// A parallel top-down DOM traversal.
#[inline(always)]
#[allow(unsafe_code)]
fn top_down_dom<'a, 'scope, E, D>(nodes: &'a [SendNode<E::ConcreteNode>],
                                  root: OpaqueNode,
                                  mut traversal_data: PerLevelTraversalData,
                                  scope: &'a rayon::Scope<'scope>,
                                  traversal: &'scope D,
                                  tls: &'scope ScopedTLS<'scope, D::ThreadLocalContext>)
    where E: TElement + 'scope,
          D: DomTraversal<E>,
{
    let mut discovered_child_nodes = vec![];
    {
        // Scope the borrow of the TLS so that the borrow is dropped before
        // potentially traversing a child on this thread.
        let mut tlc = tls.ensure(|| traversal.create_thread_local_context());

        for n in nodes {
            // Perform the appropriate traversal.
            let node = **n;
            let mut children_to_process = 0isize;
            traversal.process_preorder(&mut traversal_data, &mut *tlc, node);
            if let Some(el) = node.as_element() {
                traversal.traverse_children(&mut *tlc, el, |_tlc, kid| {
                    children_to_process += 1;
                    discovered_child_nodes.push(unsafe { SendNode::new(kid) })
                });
            }

            traversal.handle_postorder_traversal(&mut *tlc, root, node,
                                                 children_to_process);
        }
    }

    traversal_data.current_dom_depth += 1;
    traverse_nodes(discovered_child_nodes, root, traversal_data, scope, traversal, tls);
}

fn traverse_nodes<'a, 'scope, E, D>(nodes: Vec<SendNode<E::ConcreteNode>>, root: OpaqueNode,
                                    traversal_data: PerLevelTraversalData,
                                    scope: &'a rayon::Scope<'scope>,
                                    traversal: &'scope D,
                                    tls: &'scope ScopedTLS<'scope, D::ThreadLocalContext>)
    where E: TElement + 'scope,
          D: DomTraversal<E>,
{
    if nodes.is_empty() {
        return;
    }

    // Optimization: traverse directly and avoid a heap-allocating spawn() call if
    // we're only pushing one work unit.
    if nodes.len() <= CHUNK_SIZE {
        let nodes = nodes.into_boxed_slice();
        top_down_dom(&nodes, root, traversal_data, scope, traversal, tls);
        return;
    }

    // General case.
    for chunk in nodes.chunks(CHUNK_SIZE) {
        let nodes = chunk.iter().cloned().collect::<Vec<_>>().into_boxed_slice();
        let traversal_data = traversal_data.clone();
        scope.spawn(move |scope| {
            let nodes = nodes;
            top_down_dom(&nodes, root, traversal_data, scope, traversal, tls)
        })
    }
}
