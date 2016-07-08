/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements parallel traversal over the DOM tree.
//!
//! This code is highly unsafe. Keep this file small and easy to audit.

#![allow(unsafe_code)]

use dom::{OpaqueNode, TNode, UnsafeNode};
use std::mem;
use std::sync::atomic::Ordering;
use traversal::DomTraversalContext;
use workqueue::{WorkQueue, WorkUnit, WorkerProxy};

#[allow(dead_code)]
fn static_assertion(node: UnsafeNode) {
    unsafe {
        let _: UnsafeNodeList = mem::transmute(node);
    }
}

pub type UnsafeNodeList = (Box<Vec<UnsafeNode>>, OpaqueNode);

pub const CHUNK_SIZE: usize = 64;

pub struct WorkQueueData(usize, usize);

pub fn run_queue_with_custom_work_data_type<To, F, SharedContext: Sync>(
        queue: &mut WorkQueue<SharedContext, WorkQueueData>,
        callback: F,
        shared: &SharedContext)
        where To: 'static + Send, F: FnOnce(&mut WorkQueue<SharedContext, To>) {
    let queue: &mut WorkQueue<SharedContext, To> = unsafe {
        mem::transmute(queue)
    };
    callback(queue);
    queue.run(shared);
}

pub fn traverse_dom<N, C>(root: N,
                          queue_data: &C::SharedContext,
                          queue: &mut WorkQueue<C::SharedContext, WorkQueueData>)
                          where N: TNode, C: DomTraversalContext<N> {
    run_queue_with_custom_work_data_type(queue, |queue| {
        queue.push(WorkUnit {
            fun: top_down_dom::<N, C>,
            data: (Box::new(vec![root.to_unsafe()]), root.opaque()),
        });
    }, queue_data);
}

/// A parallel top-down DOM traversal.
#[inline(always)]
fn top_down_dom<N, C>(unsafe_nodes: UnsafeNodeList,
                      proxy: &mut WorkerProxy<C::SharedContext, UnsafeNodeList>)
                      where N: TNode, C: DomTraversalContext<N> {
    let context = C::new(proxy.user_data(), unsafe_nodes.1);

    let mut discovered_child_nodes = vec![];
    for unsafe_node in *unsafe_nodes.0 {
        // Get a real layout node.
        let node = unsafe { N::from_unsafe(&unsafe_node) };

        // Perform the appropriate traversal.
        context.process_preorder(node);

        let child_count = node.children_count();

        // Reset the count of children.
        {
            let data = node.mutate_data().unwrap();
            data.parallel.children_count.store(child_count as isize,
                                               Ordering::Relaxed);
        }

        // Possibly enqueue the children.
        if child_count != 0 {
            for kid in node.children() {
                discovered_child_nodes.push(kid.to_unsafe())
            }
        } else {
            // If there were no more children, start walking back up.
            bottom_up_dom::<N, C>(unsafe_nodes.1, unsafe_node, proxy)
        }
    }

    for chunk in discovered_child_nodes.chunks(CHUNK_SIZE) {
        proxy.push(WorkUnit {
            fun:  top_down_dom::<N, C>,
            data: (Box::new(chunk.iter().cloned().collect()), unsafe_nodes.1),
        });
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
fn bottom_up_dom<N, C>(root: OpaqueNode,
                       unsafe_node: UnsafeNode,
                       proxy: &mut WorkerProxy<C::SharedContext, UnsafeNodeList>)
                       where N: TNode, C: DomTraversalContext<N> {
    let context = C::new(proxy.user_data(), root);

    // Get a real layout node.
    let mut node = unsafe { N::from_unsafe(&unsafe_node) };
    loop {
        // Perform the appropriate operation.
        context.process_postorder(node);

        let parent = match node.layout_parent_node(root) {
            None => break,
            Some(parent) => parent,
        };

        let parent_data = unsafe {
            &*parent.borrow_data_unchecked().unwrap()
        };

        if parent_data
            .parallel
            .children_count
            .fetch_sub(1, Ordering::Relaxed) != 1 {
            // Get out of here and find another node to work on.
            break
        }

        // We were the last child of our parent. Construct flows for our parent.
        node = parent;
    }
}

