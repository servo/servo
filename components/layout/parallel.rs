/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements parallel traversals over the DOM and flow trees.
//!
//! This code is highly unsafe. Keep this file small and easy to audit.

#![allow(unsafe_code)]

use context::{LayoutContext, SharedLayoutContext};
use flow::{self, Flow, MutableFlowUtils, PostorderFlowTraversal, PreorderFlowTraversal};
use flow_ref::{self, FlowRef};
use gfx::display_list::OpaqueNode;
use profile_traits::time::{self, TimerMetadata, profile};
use std::mem;
use std::sync::atomic::{AtomicIsize, Ordering};
use traversal::PostorderNodeMutTraversal;
use traversal::{AssignBSizesAndStoreOverflow, AssignISizes, BubbleISizes};
use traversal::{BuildDisplayList, ComputeAbsolutePositions};
use traversal::{ConstructFlows, RecalcStyleForNode};
use traversal::{PostorderDomTraversal, PreorderDomTraversal};
use util::opts;
use util::workqueue::{WorkQueue, WorkUnit, WorkerProxy};
use wrapper::UnsafeLayoutNode;
use wrapper::{LayoutNode, ServoLayoutNode, layout_node_from_unsafe_layout_node, layout_node_to_unsafe_layout_node};

const CHUNK_SIZE: usize = 64;

pub struct WorkQueueData(usize, usize);

#[allow(dead_code)]
fn static_assertion(node: UnsafeLayoutNode) {
    unsafe {
        let _: UnsafeFlow = ::std::intrinsics::transmute(node);
        let _: UnsafeLayoutNodeList = ::std::intrinsics::transmute(node);
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

/// Information that we need stored in each DOM node.
pub struct DomParallelInfo {
    /// The number of children that still need work done.
    pub children_count: AtomicIsize,
}

impl DomParallelInfo {
    pub fn new() -> DomParallelInfo {
        DomParallelInfo {
            children_count: AtomicIsize::new(0),
        }
    }
}

pub type UnsafeLayoutNodeList = (Box<Vec<UnsafeLayoutNode>>, OpaqueNode);

pub type UnsafeFlowList = (Box<Vec<UnsafeLayoutNode>>, usize);

pub type ChunkedDomTraversalFunction =
    extern "Rust" fn(UnsafeLayoutNodeList,
                     &mut WorkerProxy<SharedLayoutContext, UnsafeLayoutNodeList>);

pub type DomTraversalFunction =
    extern "Rust" fn(OpaqueNode, UnsafeLayoutNode,
                     &mut WorkerProxy<SharedLayoutContext, UnsafeLayoutNodeList>);

pub type ChunkedFlowTraversalFunction =
    extern "Rust" fn(UnsafeFlowList, &mut WorkerProxy<SharedLayoutContext, UnsafeFlowList>);

pub type FlowTraversalFunction = extern "Rust" fn(UnsafeFlow, &SharedLayoutContext);

/// A parallel top-down DOM traversal.
pub trait ParallelPreorderDomTraversal : PreorderDomTraversal {
    fn run_parallel(&self,
                    nodes: UnsafeLayoutNodeList,
                    proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeLayoutNodeList>);

    #[inline(always)]
    fn run_parallel_helper(
            &self,
            unsafe_nodes: UnsafeLayoutNodeList,
            proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeLayoutNodeList>,
            top_down_func: ChunkedDomTraversalFunction,
            bottom_up_func: DomTraversalFunction) {
        let mut discovered_child_nodes = Vec::new();
        for unsafe_node in *unsafe_nodes.0 {
            // Get a real layout node.
            let node: ServoLayoutNode = unsafe {
                layout_node_from_unsafe_layout_node(&unsafe_node)
            };

            // Perform the appropriate traversal.
            self.process(node);

            let child_count = node.children_count();

            // Reset the count of children.
            {
                let mut layout_data_ref = node.mutate_layout_data();
                let layout_data = layout_data_ref.as_mut().expect("no layout data");
                layout_data.data.parallel.children_count.store(child_count as isize,
                                                               Ordering::Relaxed);
            }

            // Possibly enqueue the children.
            if child_count != 0 {
                for kid in node.children() {
                    discovered_child_nodes.push(layout_node_to_unsafe_layout_node(&kid))
                }
            } else {
                // If there were no more children, start walking back up.
                bottom_up_func(unsafe_nodes.1, unsafe_node, proxy)
            }
        }

        for chunk in discovered_child_nodes.chunks(CHUNK_SIZE) {
            proxy.push(WorkUnit {
                fun:  top_down_func,
                data: (box chunk.iter().cloned().collect(), unsafe_nodes.1),
            });
        }
    }
}

/// A parallel bottom-up DOM traversal.
trait ParallelPostorderDomTraversal : PostorderDomTraversal {
    fn root(&self) -> OpaqueNode;

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
    fn run_parallel(&self, unsafe_node: UnsafeLayoutNode) {
        // Get a real layout node.
        let mut node: ServoLayoutNode = unsafe {
            layout_node_from_unsafe_layout_node(&unsafe_node)
        };
        loop {
            // Perform the appropriate operation.
            self.process(node);

            let parent = match node.layout_parent_node(self.root()) {
                None => break,
                Some(parent) => parent,
            };

            let parent_layout_data = unsafe {
                &*parent.borrow_layout_data_unchecked()
            };
            let parent_layout_data = parent_layout_data.as_ref().expect("no layout data");

            if parent_layout_data
                .data
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
                for kid in flow::child_iter(flow) {
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

impl<'a> ParallelPostorderFlowTraversal for AssignBSizesAndStoreOverflow<'a> {}

impl<'a> ParallelPreorderFlowTraversal for ComputeAbsolutePositions<'a> {
    fn run_parallel(&self,
                    unsafe_flows: UnsafeFlowList,
                    proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeFlowList>) {
        self.run_parallel_helper(unsafe_flows,
                                 proxy,
                                 compute_absolute_positions,
                                 build_display_list)
    }

    fn should_record_thread_ids(&self) -> bool {
        false
    }
}

impl<'a> ParallelPostorderFlowTraversal for BuildDisplayList<'a> {}

impl<'a> ParallelPostorderDomTraversal for ConstructFlows<'a> {
    fn root(&self) -> OpaqueNode {
        self.root
    }
}

impl <'a> ParallelPreorderDomTraversal for RecalcStyleForNode<'a> {
    fn run_parallel(&self,
                    unsafe_nodes: UnsafeLayoutNodeList,
                    proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeLayoutNodeList>) {
        self.run_parallel_helper(unsafe_nodes, proxy, recalc_style, construct_flows)
    }
}

fn recalc_style(unsafe_nodes: UnsafeLayoutNodeList,
                proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeLayoutNodeList>) {
    let shared_layout_context = proxy.user_data();
    let layout_context = LayoutContext::new(shared_layout_context);
    let recalc_style_for_node_traversal = RecalcStyleForNode {
        layout_context: &layout_context,
        root: unsafe_nodes.1,
    };
    recalc_style_for_node_traversal.run_parallel(unsafe_nodes, proxy)
}

fn construct_flows(root: OpaqueNode,
                   unsafe_node: UnsafeLayoutNode,
                   proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeLayoutNodeList>) {
    let shared_layout_context = proxy.user_data();
    let layout_context = LayoutContext::new(shared_layout_context);
    let construct_flows_traversal = ConstructFlows {
        layout_context: &layout_context,
        root: root,
    };
    construct_flows_traversal.run_parallel(unsafe_node)
}

fn assign_inline_sizes(unsafe_flows: UnsafeFlowList,
                       proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeFlowList>) {
    let shared_layout_context = proxy.user_data();
    let layout_context = LayoutContext::new(shared_layout_context);
    let assign_inline_sizes_traversal = AssignISizes {
        layout_context: &layout_context,
    };
    assign_inline_sizes_traversal.run_parallel(unsafe_flows, proxy)
}

fn assign_block_sizes_and_store_overflow(
        unsafe_flow: UnsafeFlow,
        shared_layout_context: &SharedLayoutContext) {
    let layout_context = LayoutContext::new(shared_layout_context);
    let assign_block_sizes_traversal = AssignBSizesAndStoreOverflow {
        layout_context: &layout_context,
    };
    assign_block_sizes_traversal.run_parallel(unsafe_flow)
}

fn compute_absolute_positions(
        unsafe_flows: UnsafeFlowList,
        proxy: &mut WorkerProxy<SharedLayoutContext, UnsafeFlowList>) {
    let shared_layout_context = proxy.user_data();
    let layout_context = LayoutContext::new(shared_layout_context);
    let compute_absolute_positions_traversal = ComputeAbsolutePositions {
        layout_context: &layout_context,
    };
    compute_absolute_positions_traversal.run_parallel(unsafe_flows, proxy);
}

fn build_display_list(unsafe_flow: UnsafeFlow,
                      shared_layout_context: &SharedLayoutContext) {
    let layout_context = LayoutContext::new(shared_layout_context);

    let build_display_list_traversal = BuildDisplayList {
        layout_context: &layout_context,
    };

    build_display_list_traversal.run_parallel(unsafe_flow);
}

fn run_queue_with_custom_work_data_type<To, F>(
        queue: &mut WorkQueue<SharedLayoutContext, WorkQueueData>,
        callback: F,
        shared_layout_context: &SharedLayoutContext)
        where To: 'static + Send, F: FnOnce(&mut WorkQueue<SharedLayoutContext, To>) {
    let queue: &mut WorkQueue<SharedLayoutContext, To> = unsafe {
        mem::transmute(queue)
    };
    callback(queue);
    queue.run(shared_layout_context);
}

pub fn traverse_dom_preorder(root: ServoLayoutNode,
                             shared_layout_context: &SharedLayoutContext,
                             queue: &mut WorkQueue<SharedLayoutContext, WorkQueueData>) {
    run_queue_with_custom_work_data_type(queue, |queue| {
        queue.push(WorkUnit {
            fun:  recalc_style,
            data: (box vec![layout_node_to_unsafe_layout_node(&root)], root.opaque()),
        });
    }, shared_layout_context);
}

pub fn traverse_flow_tree_preorder(
        root: &mut FlowRef,
        profiler_metadata: Option<TimerMetadata>,
        time_profiler_chan: time::ProfilerChan,
        shared_layout_context: &SharedLayoutContext,
        queue: &mut WorkQueue<SharedLayoutContext, WorkQueueData>) {
    if opts::get().bubble_inline_sizes_separately {
        let layout_context = LayoutContext::new(shared_layout_context);
        let bubble_inline_sizes = BubbleISizes { layout_context: &layout_context };
        flow_ref::deref_mut(root).traverse_postorder(&bubble_inline_sizes);
    }

    run_queue_with_custom_work_data_type(queue, |queue| {
        profile(time::ProfilerCategory::LayoutParallelWarmup, profiler_metadata,
                time_profiler_chan, || {
            queue.push(WorkUnit {
                fun: assign_inline_sizes,
                data: (box vec![mut_owned_flow_to_unsafe_flow(root)], 0),
            })
        });
    }, shared_layout_context);
}

pub fn build_display_list_for_subtree(
        root: &mut FlowRef,
        profiler_metadata: Option<TimerMetadata>,
        time_profiler_chan: time::ProfilerChan,
        shared_layout_context: &SharedLayoutContext,
        queue: &mut WorkQueue<SharedLayoutContext, WorkQueueData>) {
    run_queue_with_custom_work_data_type(queue, |queue| {
        profile(time::ProfilerCategory::LayoutParallelWarmup, profiler_metadata,
                time_profiler_chan, || {
            queue.push(WorkUnit {
                fun: compute_absolute_positions,
                data: (box vec![mut_owned_flow_to_unsafe_flow(root)], 0),
            })
        });
    }, shared_layout_context);
}
