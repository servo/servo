/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements parallel traversal over the DOM tree.
//!
//! This code is highly unsafe. Keep this file small and easy to audit.

use dom::{OpaqueNode, TElement, TNode, UnsafeNode};
use rayon;
use scoped_tls::ScopedTLS;
use servo_config::opts;
use std::sync::atomic::Ordering;
use traversal::{DomTraversal, PerLevelTraversalData, PreTraverseToken};
use traversal::{STYLE_SHARING_CACHE_HITS, STYLE_SHARING_CACHE_MISSES};

pub const CHUNK_SIZE: usize = 64;

pub fn traverse_dom<N, D>(traversal: &D,
                          root: N::ConcreteElement,
                          known_root_dom_depth: Option<usize>,
                          token: PreTraverseToken,
                          queue: &rayon::ThreadPool)
    where N: TNode,
          D: DomTraversal<N>
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
                children.push(kid.to_unsafe());
            }
        }
        (children, known_root_dom_depth.map(|x| x + 1))
    } else {
        (vec![root.as_node().to_unsafe()], known_root_dom_depth)
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
fn top_down_dom<'a, 'scope, N, D>(unsafe_nodes: &'a [UnsafeNode],
                                  root: OpaqueNode,
                                  mut traversal_data: PerLevelTraversalData,
                                  scope: &'a rayon::Scope<'scope>,
                                  traversal: &'scope D,
                                  tls: &'scope ScopedTLS<'scope, D::ThreadLocalContext>)
    where N: TNode,
          D: DomTraversal<N>,
{
    let mut discovered_child_nodes = vec![];
    {
        // Scope the borrow of the TLS so that the borrow is dropped before
        // potentially traversing a child on this thread.
        let mut tlc = tls.ensure(|| traversal.create_thread_local_context());

        for unsafe_node in unsafe_nodes {
            // Get a real layout node.
            let node = unsafe { N::from_unsafe(&unsafe_node) };

            // Perform the appropriate traversal.
            let mut children_to_process = 0isize;
            traversal.process_preorder(&mut traversal_data, &mut *tlc, node);
            if let Some(el) = node.as_element() {
                D::traverse_children(el, |kid| {
                    children_to_process += 1;
                    discovered_child_nodes.push(kid.to_unsafe())
                });
            }

            // Reset the count of children if we need to do a bottom-up traversal
            // after the top up.
            if D::needs_postorder_traversal() {
                if children_to_process == 0 {
                    // If there were no more children, start walking back up.
                    bottom_up_dom(traversal, &mut *tlc, root, *unsafe_node)
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

fn traverse_nodes<'a, 'scope, N, D>(nodes: Vec<UnsafeNode>, root: OpaqueNode,
                                    traversal_data: PerLevelTraversalData,
                                    scope: &'a rayon::Scope<'scope>,
                                    traversal: &'scope D,
                                    tls: &'scope ScopedTLS<'scope, D::ThreadLocalContext>)
    where N: TNode,
          D: DomTraversal<N>,
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
#[allow(unsafe_code)]
fn bottom_up_dom<N, D>(traversal: &D,
                       thread_local: &mut D::ThreadLocalContext,
                       root: OpaqueNode,
                       unsafe_node: UnsafeNode)
    where N: TNode,
          D: DomTraversal<N>
{
    // Get a real layout node.
    let mut node = unsafe { N::from_unsafe(&unsafe_node) };
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
