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

use dom::{OpaqueNode, SendNode, TElement, TNode};
use rayon;
use scoped_tls::ScopedTLS;
use servo_config::opts;
use std::sync::atomic::Ordering;
use traversal::{DomTraversal, PerLevelTraversalData, PreTraverseToken};
use traversal::{STYLE_SHARING_CACHE_HITS, STYLE_SHARING_CACHE_MISSES};

/// The chunk size used to split the parallel traversal nodes.
///
/// We send each `CHUNK_SIZE` nodes as a different work unit to the work queue.
pub const CHUNK_SIZE: usize = 64;

/// A parallel top down traversal, generic over `D`.
#[allow(unsafe_code)]
pub fn traverse_dom<E, D>(traversal: &D,
                          root: E,
                          known_root_dom_depth: Option<usize>,
                          token: PreTraverseToken,
                          queue: &rayon::ThreadPool)
    where E: TElement,
          D: DomTraversal<E>,
{
    if opts::get().style_sharing_stats {
        STYLE_SHARING_CACHE_HITS.store(0, Ordering::SeqCst);
        STYLE_SHARING_CACHE_MISSES.store(0, Ordering::SeqCst);
    }

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
        (children, known_root_dom_depth.map(|x| x + 1))
    } else {
        (vec![unsafe { SendNode::new(root.as_node()) }], known_root_dom_depth)
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

    if opts::get().style_sharing_stats {
        let hits = STYLE_SHARING_CACHE_HITS.load(Ordering::SeqCst);
        let misses = STYLE_SHARING_CACHE_MISSES.load(Ordering::SeqCst);

        println!("Style sharing stats:");
        println!(" * Hits: {}", hits);
        println!(" * Misses: {}", misses);
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

            // Reset the count of children if we need to do a bottom-up traversal
            // after the top up.
            if D::needs_postorder_traversal() {
                if children_to_process == 0 {
                    // If there were no more children, start walking back up.
                    bottom_up_dom(traversal, &mut *tlc, root, node)
                } else {
                    // Otherwise record the number of children to process when the
                    // time comes.
                    node.as_element().unwrap().store_children_to_process(children_to_process);
                }
            }
        }
    }

    if let Some(ref mut depth) = traversal_data.current_dom_depth {
        *depth += 1;
    }

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

/// Process current node and potentially traverse its ancestors.
///
/// If we are the last child that finished processing, recursively process
/// our parent. Else, stop. Also, stop at the root.
///
/// Thus, if we start with all the leaves of a tree, we end up traversing
/// the whole tree bottom-up because each parent will be processed exactly
/// once (by the last child that finishes processing).
///
/// The only communication between siblings is that they both
/// fetch-and-subtract the parent's children count.
fn bottom_up_dom<E, D>(traversal: &D,
                       thread_local: &mut D::ThreadLocalContext,
                       root: OpaqueNode,
                       mut node: E::ConcreteNode)
    where E: TElement,
          D: DomTraversal<E>,
{
    loop {
        // Perform the appropriate operation.
        traversal.process_postorder(thread_local, node);

        if node.opaque() == root {
            break;
        }

        let parent = match node.parent_element() {
            None => unreachable!("How can this happen after the break above?"),
            Some(parent) => parent,
        };

        let remaining = parent.did_process_child();
        if remaining != 0 {
            // Get out of here and find another node to work on.
            break
        }

        // We were the last child of our parent. Construct flows for our parent.
        node = parent.as_node();
    }
}
