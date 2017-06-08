/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements parallel traversals over the DOM and flow trees.
//!
//! This code is highly unsafe. Keep this file small and easy to audit.

#![allow(unsafe_code)]

use context::LayoutContext;
use flow::{self, Flow, MutableFlowUtils, PostorderFlowTraversal, PreorderFlowTraversal};
use flow_ref::FlowRef;
use profile_traits::time::{self, TimerMetadata, profile};
use rayon;
use servo_config::opts;
use smallvec::SmallVec;
use std::mem;
use std::sync::atomic::{AtomicIsize, Ordering};
use style::dom::UnsafeNode;
use traversal::{AssignISizes, BubbleISizes};
use traversal::AssignBSizes;

pub use style::parallel::traverse_dom;

/// Traversal chunk size.
const CHUNK_SIZE: usize = 16;

pub type FlowList = SmallVec<[UnsafeNode; CHUNK_SIZE]>;

#[allow(dead_code)]
fn static_assertion(node: UnsafeNode) {
    unsafe {
        let _: UnsafeFlow = ::std::intrinsics::transmute(node);
    }
}

/// Vtable + pointer representation of a Flow trait object.
pub type UnsafeFlow = (usize, usize);

fn null_unsafe_flow() -> UnsafeFlow {
    (0, 0)
}

pub fn mut_owned_flow_to_unsafe_flow(flow: *mut FlowRef) -> UnsafeFlow {
    unsafe {
        mem::transmute::<&Flow, UnsafeFlow>(&**flow)
    }
}

pub fn borrowed_flow_to_unsafe_flow(flow: &Flow) -> UnsafeFlow {
    unsafe {
        mem::transmute::<&Flow, UnsafeFlow>(flow)
    }
}

/// Information that we need stored in each flow.
pub struct FlowParallelInfo {
    /// The number of children that still need work done.
    pub children_count: AtomicIsize,
    /// The address of the parent flow.
    pub parent: UnsafeFlow,
}

impl FlowParallelInfo {
    pub fn new() -> FlowParallelInfo {
        FlowParallelInfo {
            children_count: AtomicIsize::new(0),
            parent: null_unsafe_flow(),
        }
    }
}

/// Process current flow and potentially traverse its ancestors.
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
fn buttom_up_flow(mut unsafe_flow: UnsafeFlow,
                  assign_bsize_traversal: &AssignBSizes) {
    loop {
        // Get a real flow.
        let flow: &mut Flow = unsafe {
            mem::transmute(unsafe_flow)
        };

        // Perform the appropriate traversal.
        if assign_bsize_traversal.should_process(flow) {
            assign_bsize_traversal.process(flow);
        }


        let base = flow::mut_base(flow);

        // Reset the count of children for the next layout traversal.
        base.parallel.children_count.store(base.children.len() as isize,
                                           Ordering::Relaxed);

        // Possibly enqueue the parent.
        let unsafe_parent = base.parallel.parent;
        if unsafe_parent == null_unsafe_flow() {
            // We're done!
            break
        }

        // No, we're not at the root yet. Then are we the last child
        // of our parent to finish processing? If so, we can continue
        // on with our parent; otherwise, we've gotta wait.
        let parent: &mut Flow = unsafe {
            mem::transmute(unsafe_parent)
        };
        let parent_base = flow::mut_base(parent);
        if parent_base.parallel.children_count.fetch_sub(1, Ordering::Relaxed) == 1 {
            // We were the last child of our parent. Reflow our parent.
            unsafe_flow = unsafe_parent
        } else {
            // Stop.
            break
        }
    }
}

fn top_down_flow<'scope>(unsafe_flows: &[UnsafeFlow],
                         scope: &rayon::Scope<'scope>,
                         assign_isize_traversal: &'scope AssignISizes,
                         assign_bsize_traversal: &'scope AssignBSizes)
{
    let mut discovered_child_flows = FlowList::new();

    for unsafe_flow in unsafe_flows {
        let mut had_children = false;
        unsafe {
            // Get a real flow.
            let flow: &mut Flow = mem::transmute(*unsafe_flow);

            // FIXME(emilio): With the switch to rayon we can no longer
            // access a thread id from here easily. Either instrument
            // rayon (the unstable feature) to get a worker thread
            // identifier, or remove all the layout tinting mode.
            //
            // flow::mut_base(flow).thread_id = proxy.worker_index();

            if assign_isize_traversal.should_process(flow) {
                // Perform the appropriate traversal.
                assign_isize_traversal.process(flow);
            }

            // Possibly enqueue the children.
            for kid in flow::child_iter_mut(flow) {
                had_children = true;
                discovered_child_flows.push(borrowed_flow_to_unsafe_flow(kid));
            }
        }

        // If there were no more children, start assigning block-sizes.
        if !had_children {
            buttom_up_flow(*unsafe_flow, &assign_bsize_traversal)
        }
    }

    if discovered_child_flows.is_empty() {
        return
    }

    if discovered_child_flows.len() <= CHUNK_SIZE {
        // We can handle all the children in this work unit.
        top_down_flow(&discovered_child_flows,
                      scope,
                      &assign_isize_traversal,
                      &assign_bsize_traversal);
    } else {
        // Spawn a new work unit for each chunk after the first.
        let mut chunks = discovered_child_flows.chunks(CHUNK_SIZE);
        let first_chunk = chunks.next();
        for chunk in chunks {
            let nodes = chunk.iter().cloned().collect::<FlowList>();
            scope.spawn(move |scope| {
                top_down_flow(&nodes, scope, &assign_isize_traversal, &assign_bsize_traversal);
            });
        }
        if let Some(chunk) = first_chunk {
            top_down_flow(chunk, scope, &assign_isize_traversal, &assign_bsize_traversal);
        }
    }
}

pub fn traverse_flow_tree_preorder(
        root: &mut Flow,
        profiler_metadata: Option<TimerMetadata>,
        time_profiler_chan: time::ProfilerChan,
        context: &LayoutContext,
        queue: &rayon::ThreadPool) {
    if opts::get().bubble_inline_sizes_separately {
        let bubble_inline_sizes = BubbleISizes { layout_context: &context };
        root.traverse_postorder(&bubble_inline_sizes);
    }

    let assign_isize_traversal = &AssignISizes { layout_context: &context };
    let assign_bsize_traversal = &AssignBSizes { layout_context: &context };
    let nodes = [borrowed_flow_to_unsafe_flow(root)];

    queue.install(move || {
        rayon::scope(move |scope| {
            profile(time::ProfilerCategory::LayoutParallelWarmup,
                    profiler_metadata, time_profiler_chan, move || {
                        top_down_flow(&nodes, scope, assign_isize_traversal, assign_bsize_traversal);
            });
        });
    });
}
