/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements parallel traversals over the DOM and flow trees.
//!
//! This code is highly unsafe. Keep this file small and easy to audit.

#![allow(unsafe_code)]

use context::{LayoutContext, SharedLayoutContext};
use flow::{self, Flow, MutableFlowUtils, PostorderFlowTraversal, PreorderFlowTraversal};
use flow_ref::FlowRef;
use profile_traits::time::{self, TimerMetadata, profile};
use std::mem;
use std::sync::atomic::{AtomicIsize, Ordering};
use style::dom::UnsafeNode;
use style::parallel::run_queue_with_custom_work_data_type;
use style::parallel::{CHUNK_SIZE, WorkQueueData};
use style::workqueue::{WorkQueue, WorkUnit, WorkerProxy};
use traversal::AssignBSizes;
use traversal::{AssignISizes, BubbleISizes};
use util::opts;

pub use style::parallel::traverse_dom;

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

pub type UnsafeFlowList = (Box<Vec<UnsafeNode>>, usize);

pub type ChunkedFlowTraversalFunction =
    extern "Rust" fn(UnsafeFlowList, &mut WorkerProxy<SharedLayoutContext, UnsafeFlowList>);

pub type FlowTraversalFunction = extern "Rust" fn(UnsafeFlow, &SharedLayoutContext);

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

/// A parallel bottom-up flow traversal.
trait ParallelPostorderFlowTraversal : PostorderFlowTraversal {
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
    fn run_parallel(&self, mut unsafe_flow: UnsafeFlow) {
        loop {
            // Get a real flow.
            let flow: &mut Flow = unsafe {
                mem::transmute(unsafe_flow)
            };

            // Perform the appropriate traversal.
            if self.should_process(flow) {
                self.process(flow);
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
}

/// A parallel top-down flow traversal.
trait ParallelPreorderFlowTraversal : PreorderFlowTraversal {
    fn run_parallel(&self,
                    unsafe_flows: UnsafeFlowList,
                    proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeFlowList>);

    fn should_record_thread_ids(&self) -> bool;

    #[inline(always)]
    fn run_parallel_helper(&self,
                           unsafe_flows: UnsafeFlowList,
                           proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeFlowList>,
                           top_down_func: ChunkedFlowTraversalFunction,
                           bottom_up_func: FlowTraversalFunction) {
        let mut discovered_child_flows = Vec::new();
        for unsafe_flow in *unsafe_flows.0 {
            let mut had_children = false;
            unsafe {
                // Get a real flow.
                let flow: &mut Flow = mem::transmute(unsafe_flow);

                if self.should_record_thread_ids() {
                    flow::mut_base(flow).thread_id = proxy.worker_index();
                }

                if self.should_process(flow) {
                    // Perform the appropriate traversal.
                    self.process(flow);
                }

                // Possibly enqueue the children.
                for kid in flow::child_iter_mut(flow) {
                    had_children = true;
                    discovered_child_flows.push(borrowed_flow_to_unsafe_flow(kid));
                }
            }

            // If there were no more children, start assigning block-sizes.
            if !had_children {
                bottom_up_func(unsafe_flow, proxy.user_data())
            }
        }

        for chunk in discovered_child_flows.chunks(CHUNK_SIZE) {
            proxy.push(WorkUnit {
                fun: top_down_func,
                data: (box chunk.iter().cloned().collect(), 0),
            });
        }
    }
}

impl<'a> ParallelPreorderFlowTraversal for AssignISizes<'a> {
    fn run_parallel(&self,
                    unsafe_flows: UnsafeFlowList,
                    proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeFlowList>) {
        self.run_parallel_helper(unsafe_flows,
                                 proxy,
                                 assign_inline_sizes,
                                 assign_block_sizes_and_store_overflow)
    }

    fn should_record_thread_ids(&self) -> bool {
        true
    }
}

impl<'a> ParallelPostorderFlowTraversal for AssignBSizes<'a> {}

fn assign_inline_sizes(unsafe_flows: UnsafeFlowList,
                       proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeFlowList>) {
    let shared_layout_context = proxy.user_data();
    let assign_inline_sizes_traversal = AssignISizes {
        shared_context: &shared_layout_context.style_context,
    };
    assign_inline_sizes_traversal.run_parallel(unsafe_flows, proxy)
}

fn assign_block_sizes_and_store_overflow(
        unsafe_flow: UnsafeFlow,
        shared_layout_context: &SharedLayoutContext) {
    let layout_context = LayoutContext::new(shared_layout_context);
    let assign_block_sizes_traversal = AssignBSizes {
        layout_context: &layout_context,
    };
    assign_block_sizes_traversal.run_parallel(unsafe_flow)
}

pub fn traverse_flow_tree_preorder(
        root: &mut Flow,
        profiler_metadata: Option<TimerMetadata>,
        time_profiler_chan: time::ProfilerChan,
        shared_layout_context: &SharedLayoutContext,
        queue: &mut WorkQueue<SharedLayoutContext, WorkQueueData>) {
    if opts::get().bubble_inline_sizes_separately {
        let layout_context = LayoutContext::new(shared_layout_context);
        let bubble_inline_sizes = BubbleISizes { layout_context: &layout_context };
        root.traverse_postorder(&bubble_inline_sizes);
    }

    run_queue_with_custom_work_data_type(queue, |queue| {
        profile(time::ProfilerCategory::LayoutParallelWarmup, profiler_metadata,
                time_profiler_chan, || {
            queue.push(WorkUnit {
                fun: assign_inline_sizes,
                data: (box vec![borrowed_flow_to_unsafe_flow(root)], 0),
            })
        });
    }, shared_layout_context);
}
