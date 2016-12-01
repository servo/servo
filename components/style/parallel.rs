/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements parallel traversal over the DOM tree.
//!
//! This code is highly unsafe. Keep this file small and easy to audit.

use dom::{OpaqueNode, TElement, TNode, UnsafeNode};
use rayon;
use std::sync::atomic::Ordering;
use traversal::{DomTraversalContext, PerLevelTraversalData, PreTraverseToken};
use traversal::{STYLE_SHARING_CACHE_HITS, STYLE_SHARING_CACHE_MISSES};
use util::opts;

pub const CHUNK_SIZE: usize = 64;

pub fn traverse_dom<N, C>(root: N::ConcreteElement,
                          known_root_dom_depth: Option<usize>,
                          shared_context: &C::SharedContext,
                          token: PreTraverseToken,
                          queue: &rayon::ThreadPool)
    where N: TNode,
          C: DomTraversalContext<N>
{
    if opts::get().style_sharing_stats {
        STYLE_SHARING_CACHE_HITS.store(0, Ordering::SeqCst);
        STYLE_SHARING_CACHE_MISSES.store(0, Ordering::SeqCst);
    }

    // Handle root skipping. We don't currently support it in conjunction with
    // bottom-up traversal. If we did, we'd need to put it on the context to make
    // it available to the bottom-up phase.
    debug_assert!(!token.should_skip_root() || !C::needs_postorder_traversal());
    let (nodes, depth) = if token.should_skip_root() {
        let mut children = vec![];
        C::traverse_children(root, |kid| children.push(kid.to_unsafe()));
        (children, known_root_dom_depth.map(|x| x + 1))
    } else {
        (vec![root.as_node().to_unsafe()], known_root_dom_depth)
    };

    let data = PerLevelTraversalData {
        current_dom_depth: depth,
    };

    let root = root.as_node().opaque();
    queue.install(|| {
        rayon::scope(|scope| {
            traverse_nodes::<_, C>(nodes, root, data, scope, shared_context);
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
fn top_down_dom<'a, 'scope, N, C>(unsafe_nodes: &'a [UnsafeNode],
                                  root: OpaqueNode,
                                  mut data: PerLevelTraversalData,
                                  scope: &'a rayon::Scope<'scope>,
                                  shared_context: &'scope C::SharedContext)
    where N: TNode,
          C: DomTraversalContext<N>,
{
    let context = C::new(shared_context, root);

    let mut discovered_child_nodes = vec![];
    for unsafe_node in unsafe_nodes {
        // Get a real layout node.
        let node = unsafe { N::from_unsafe(&unsafe_node) };

        // Perform the appropriate traversal.
        let mut children_to_process = 0isize;
        context.process_preorder(node, &mut data);
        if let Some(el) = node.as_element() {
            C::traverse_children(el, |kid| {
                children_to_process += 1;
                discovered_child_nodes.push(kid.to_unsafe())
            });
        }

        // Reset the count of children if we need to do a bottom-up traversal
        // after the top up.
        if C::needs_postorder_traversal() {
            if children_to_process == 0 {
                // If there were no more children, start walking back up.
                bottom_up_dom::<N, C>(root, *unsafe_node, shared_context)
            } else {
                // Otherwise record the number of children to process when the
                // time comes.
                node.as_element().unwrap().store_children_to_process(children_to_process);
            }
        }
    }

    // NB: In parallel traversal mode we have to purge the LRU cache in order to
    // be able to access it without races.
    context.local_context().style_sharing_candidate_cache.borrow_mut().clear();

    if let Some(ref mut depth) = data.current_dom_depth {
        *depth += 1;
    }

    traverse_nodes::<_, C>(discovered_child_nodes, root, data, scope, shared_context);
}

fn traverse_nodes<'a, 'scope, N, C>(nodes: Vec<UnsafeNode>, root: OpaqueNode,
                                    data: PerLevelTraversalData,
                                    scope: &'a rayon::Scope<'scope>,
                                    shared_context: &'scope C::SharedContext)
    where N: TNode,
          C: DomTraversalContext<N>,
{
    if nodes.is_empty() {
        return;
    }

    // Optimization: traverse directly and avoid a heap-allocating spawn() call if
    // we're only pushing one work unit.
    if nodes.len() <= CHUNK_SIZE {
        let nodes = nodes.into_boxed_slice();
        top_down_dom::<N, C>(&nodes, root, data, scope, shared_context);
        return;
    }

    // General case.
    for chunk in nodes.chunks(CHUNK_SIZE) {
        let nodes = chunk.iter().cloned().collect::<Vec<_>>().into_boxed_slice();
        let data = data.clone();
        scope.spawn(move |scope| {
            let nodes = nodes;
            top_down_dom::<N, C>(&nodes, root, data, scope, shared_context)
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
fn bottom_up_dom<N, C>(root: OpaqueNode,
                       unsafe_node: UnsafeNode,
                       shared_context: &C::SharedContext)
    where N: TNode,
          C: DomTraversalContext<N>
{
    let context = C::new(shared_context, root);

    // Get a real layout node.
    let mut node = unsafe { N::from_unsafe(&unsafe_node) };
    loop {
        // Perform the appropriate operation.
        context.process_postorder(node);

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
