/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements parallel traversals over the DOM and flow trees.
//!
//! This code is highly unsafe. Keep this file small and easy to audit.

use css::matching::MatchMethods;
use layout::context::LayoutContext;
use layout::flow::{Flow, LeafSet, PostorderFlowTraversal};
use layout::flow;
use layout::layout_task::{AssignHeightsAndStoreOverflowTraversal, BubbleWidthsTraversal};
use layout::util::OpaqueNode;
use layout::wrapper::LayoutNode;

use extra::arc::MutexArc;
use servo_util::time::{ProfilerChan, profile};
use servo_util::time;
use servo_util::workqueue::{WorkQueue, WorkUnit, WorkerProxy};
use std::cast;
use std::ptr;
use std::sync::atomics::{AtomicInt, Relaxed, SeqCst};
use style::{Stylist, TNode};

pub enum TraversalKind {
    BubbleWidthsTraversalKind,
    AssignHeightsAndStoreOverflowTraversalKind,
}

pub type UnsafeFlow = (uint, uint);

fn null_unsafe_flow() -> UnsafeFlow {
    (0, 0)
}

pub fn owned_flow_to_unsafe_flow(flow: *~Flow) -> UnsafeFlow {
    unsafe {
        cast::transmute_copy(&*flow)
    }
}

pub fn mut_owned_flow_to_unsafe_flow(flow: *mut ~Flow) -> UnsafeFlow {
    unsafe {
        cast::transmute_copy(&*flow)
    }
}

pub type UnsafeLayoutNode = (uint, uint);

fn layout_node_to_unsafe_layout_node(node: &LayoutNode) -> UnsafeLayoutNode {
    unsafe {
        cast::transmute_copy(node)
    }
}

/// Information that we need stored in each flow.
pub struct FlowParallelInfo {
    /// The number of children that still need work done.
    children_count: AtomicInt,
    /// The address of the parent flow.
    parent: UnsafeFlow,
}

impl FlowParallelInfo {
    pub fn new() -> FlowParallelInfo {
        FlowParallelInfo {
            children_count: AtomicInt::new(0),
            parent: null_unsafe_flow(),
        }
    }
}

/// A parallel bottom-up flow traversal.
trait ParallelPostorderFlowTraversal : PostorderFlowTraversal {
    fn run_parallel(&mut self, mut unsafe_flow: UnsafeFlow) {
        loop {
            unsafe {
                // Get a real flow.
                let flow: &mut ~Flow = cast::transmute(&unsafe_flow);

                // Perform the appropriate traversal.
                if self.should_process(*flow) {
                    self.process(*flow);
                }

                let base = flow::mut_base(*flow);

                // Reset the count of children for the next layout traversal.
                base.parallel.children_count.store(base.children.len() as int, Relaxed);

                // Possibly enqueue the parent.
                let unsafe_parent = base.parallel.parent;
                if unsafe_parent == null_unsafe_flow() {
                    // We're done!
                    break
                }

                // No, we're not at the root yet. Then are we the last sibling of our parent? If
                // so, we can continue on with our parent; otherwise, we've gotta wait.
                let parent: &mut ~Flow = cast::transmute(&unsafe_parent);
                let parent_base = flow::mut_base(*parent);
                if parent_base.parallel.children_count.fetch_sub(1, SeqCst) == 1 {
                    // We were the last child of our parent. Reflow our parent.
                    unsafe_flow = unsafe_parent
                } else {
                    // Stop.
                    break
                }
            }
        }
    }
}

impl<'a> ParallelPostorderFlowTraversal for BubbleWidthsTraversal<'a> {}

impl<'a> ParallelPostorderFlowTraversal for AssignHeightsAndStoreOverflowTraversal<'a> {}

fn match_and_cascade_node(unsafe_layout_node: UnsafeLayoutNode,
                          proxy: &mut WorkerProxy<*mut LayoutContext,UnsafeLayoutNode>) {
    unsafe {
        let layout_context: &mut LayoutContext = cast::transmute(*proxy.user_data());

        // Get a real layout node.
        let node: LayoutNode = cast::transmute(unsafe_layout_node);

        // Perform the CSS selector matching.
        let stylist: &Stylist = cast::transmute(layout_context.stylist);
        node.match_node(stylist);

        // Perform the CSS cascade.
        let parent_opt = if OpaqueNode::from_layout_node(&node) == layout_context.reflow_root {
            None
        } else {
            node.parent_node()
        };
        node.cascade_node(parent_opt);

        // Enqueue kids.
        for kid in node.children() {
            if kid.is_element() {
                proxy.push(WorkUnit {
                    fun: match_and_cascade_node,
                    data: layout_node_to_unsafe_layout_node(&kid),
                });
            }
        }
    }
}

fn bubble_widths(unsafe_flow: UnsafeFlow, proxy: &mut WorkerProxy<*mut LayoutContext,UnsafeFlow>) {
    let layout_context: &mut LayoutContext = unsafe {
        cast::transmute(*proxy.user_data())
    };
    let mut bubble_widths_traversal = BubbleWidthsTraversal {
        layout_context: layout_context,
    };
    bubble_widths_traversal.run_parallel(unsafe_flow)
}

fn assign_heights_and_store_overflow(unsafe_flow: UnsafeFlow,
                                     proxy: &mut WorkerProxy<*mut LayoutContext,UnsafeFlow>) {
    let layout_context: &mut LayoutContext = unsafe {
        cast::transmute(*proxy.user_data())
    };
    let mut assign_heights_traversal = AssignHeightsAndStoreOverflowTraversal {
        layout_context: layout_context,
    };
    assign_heights_traversal.run_parallel(unsafe_flow)
}

pub fn match_and_cascade_subtree(root_node: &LayoutNode,
                                 layout_context: &mut LayoutContext,
                                 queue: &mut WorkQueue<*mut LayoutContext,UnsafeLayoutNode>) {
    unsafe {
        queue.data = cast::transmute(layout_context)
    }

    // Enqueue the root node.
    queue.push(WorkUnit {
        fun: match_and_cascade_node,
        data: layout_node_to_unsafe_layout_node(root_node),
    });

    queue.run();

    queue.data = ptr::mut_null()
}

pub fn traverse_flow_tree(kind: TraversalKind,
                          leaf_set: &MutexArc<LeafSet>,
                          profiler_chan: ProfilerChan,
                          layout_context: &mut LayoutContext,
                          queue: &mut WorkQueue<*mut LayoutContext,UnsafeFlow>) {
    unsafe {
        queue.data = cast::transmute(layout_context)
    }

    let fun = match kind {
        BubbleWidthsTraversalKind => bubble_widths,
        AssignHeightsAndStoreOverflowTraversalKind => assign_heights_and_store_overflow,
    };

    profile(time::LayoutParallelWarmupCategory, profiler_chan, || {
        leaf_set.access(|leaf_set| {
            for &flow in leaf_set.iter() {
                queue.push(WorkUnit {
                    fun: fun,
                    data: flow,
                })
            }
        })
    });

    queue.run();

    queue.data = ptr::mut_null()
}

