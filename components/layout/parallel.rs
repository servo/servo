/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements parallel traversals over the DOM and flow trees.
//!
//! This code is highly unsafe. Keep this file small and easy to audit.

#![allow(unsafe_blocks)]

use context::{LayoutContext, SharedLayoutContextWrapper, SharedLayoutContext};
use flow::{Flow, MutableFlowUtils, PreorderFlowTraversal, PostorderFlowTraversal};
use flow;
use flow_ref::FlowRef;
use traversal::{RecalcStyleForNode, ConstructFlows};
use traversal::{BubbleISizes, AssignISizes, AssignBSizesAndStoreOverflow};
use traversal::{ComputeAbsolutePositions, BuildDisplayList};
use util::{LayoutDataAccess, LayoutDataWrapper};
use wrapper::{layout_node_to_unsafe_layout_node, layout_node_from_unsafe_layout_node, LayoutNode};
use wrapper::{PostorderNodeMutTraversal, UnsafeLayoutNode};
use wrapper::{PreorderDomTraversal, PostorderDomTraversal};

use servo_util::opts;
use servo_util::time::{TimeProfilerCategory, ProfilerMetadata, TimeProfilerChan, profile};
use servo_util::workqueue::{WorkQueue, WorkUnit, WorkerProxy};
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicInt, Ordering};

#[allow(dead_code)]
fn static_assertion(node: UnsafeLayoutNode) {
    unsafe {
        let _: UnsafeFlow = ::std::intrinsics::transmute(node);
    }
}

/// Vtable + pointer representation of a Flow trait object.
pub type UnsafeFlow = (uint, uint);

fn null_unsafe_flow() -> UnsafeFlow {
    (0, 0)
}

pub fn owned_flow_to_unsafe_flow(flow: *const FlowRef) -> UnsafeFlow {
    unsafe {
        mem::transmute_copy(&*flow)
    }
}

pub fn mut_owned_flow_to_unsafe_flow(flow: *mut FlowRef) -> UnsafeFlow {
    unsafe {
        mem::transmute_copy(&*flow)
    }
}

pub fn borrowed_flow_to_unsafe_flow(flow: &Flow) -> UnsafeFlow {
    unsafe {
        mem::transmute_copy(&flow)
    }
}

pub fn mut_borrowed_flow_to_unsafe_flow(flow: &mut Flow) -> UnsafeFlow {
    unsafe {
        mem::transmute_copy(&flow)
    }
}

/// Information that we need stored in each DOM node.
pub struct DomParallelInfo {
    /// The number of children that still need work done.
    pub children_count: AtomicInt,
}

impl DomParallelInfo {
    pub fn new() -> DomParallelInfo {
        DomParallelInfo {
            children_count: AtomicInt::new(0),
        }
    }
}

/// A parallel top-down DOM traversal.
pub trait ParallelPreorderDomTraversal : PreorderDomTraversal {
    fn run_parallel(&self,
                    node: UnsafeLayoutNode,
                    proxy: &mut WorkerProxy<SharedLayoutContextWrapper,UnsafeLayoutNode>);

    #[inline(always)]
    fn run_parallel_helper(&self,
                           unsafe_node: UnsafeLayoutNode,
                           proxy: &mut WorkerProxy<SharedLayoutContextWrapper,UnsafeLayoutNode>,
                           top_down_func: extern "Rust" fn(UnsafeFlow,
                                                           &mut WorkerProxy<SharedLayoutContextWrapper,
                                                                            UnsafeLayoutNode>),
                           bottom_up_func: extern "Rust" fn(UnsafeFlow,
                                                            &mut WorkerProxy<SharedLayoutContextWrapper,
                                                                             UnsafeFlow>)) {
        // Get a real layout node.
        let node: LayoutNode = unsafe {
            layout_node_from_unsafe_layout_node(&unsafe_node)
        };

        // Perform the appropriate traversal.
        self.process(node);

        // NB: O(n).
        let child_count = node.children().count();

        // Reset the count of children.
        {
            let mut layout_data_ref = node.mutate_layout_data();
            let layout_data = layout_data_ref.as_mut().expect("no layout data");
            layout_data.data.parallel.children_count.store(child_count as int, Ordering::Relaxed);
        }

        // Possibly enqueue the children.
        if child_count != 0 {
            for kid in node.children() {
                proxy.push(WorkUnit {
                    fun:  top_down_func,
                    data: layout_node_to_unsafe_layout_node(&kid),
                });
            }
        } else {
            // If there were no more children, start walking back up.
            bottom_up_func(unsafe_node, proxy)
        }
    }
}

/// A parallel bottom-up DOM traversal.
trait ParallelPostorderDomTraversal : PostorderDomTraversal {
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
    fn run_parallel(&self,
                    mut unsafe_node: UnsafeLayoutNode,
                    proxy: &mut WorkerProxy<SharedLayoutContextWrapper,UnsafeLayoutNode>) {
        loop {
            // Get a real layout node.
            let node: LayoutNode = unsafe {
                layout_node_from_unsafe_layout_node(&unsafe_node)
            };

            // Perform the appropriate traversal.
            self.process(node);

            let shared_layout_context = unsafe { &*(proxy.user_data().0) };
            let layout_context = LayoutContext::new(shared_layout_context);

            let parent =
                match node.layout_parent_node(layout_context.shared) {
                    None         => break,
                    Some(parent) => parent,
                };

            unsafe {
                let parent_layout_data =
                    (*parent.borrow_layout_data_unchecked())
                    .as_ref()
                    .expect("no layout data");

                unsafe_node = layout_node_to_unsafe_layout_node(&parent);

                let parent_layout_data: &mut LayoutDataWrapper = mem::transmute(parent_layout_data);
                if parent_layout_data
                    .data
                    .parallel
                    .children_count
                    .fetch_sub(1, Ordering::SeqCst) == 1 {
                    // We were the last child of our parent. Construct flows for our parent.
                } else {
                    // Get out of here and find another node to work on.
                    break
                }
            }
        }
    }
}

/// Information that we need stored in each flow.
pub struct FlowParallelInfo {
    /// The number of children that still need work done.
    pub children_count: AtomicInt,
    /// The address of the parent flow.
    pub parent: UnsafeFlow,
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
    fn run_parallel(&self,
                    mut unsafe_flow: UnsafeFlow,
                    _: &mut WorkerProxy<SharedLayoutContextWrapper,UnsafeFlow>) {
        loop {
            unsafe {
                // Get a real flow.
                let flow: &mut FlowRef = mem::transmute(&unsafe_flow);

                // Perform the appropriate traversal.
                if self.should_process(&mut **flow) {
                    self.process(&mut **flow);
                }


                let base = flow::mut_base(&mut **flow);

                // Reset the count of children for the next layout traversal.
                base.parallel.children_count.store(base.children.len() as int, Ordering::Relaxed);

                // Possibly enqueue the parent.
                let unsafe_parent = base.parallel.parent;
                if unsafe_parent == null_unsafe_flow() {
                    // We're done!
                    break
                }

                // No, we're not at the root yet. Then are we the last child
                // of our parent to finish processing? If so, we can continue
                // on with our parent; otherwise, we've gotta wait.
                let parent: &mut FlowRef = mem::transmute(&unsafe_parent);
                let parent_base = flow::mut_base(&mut **parent);
                if parent_base.parallel.children_count.fetch_sub(1, Ordering::SeqCst) == 1 {
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

/// A parallel top-down flow traversal.
trait ParallelPreorderFlowTraversal : PreorderFlowTraversal {
    fn run_parallel(&self,
                    unsafe_flow: UnsafeFlow,
                    proxy: &mut WorkerProxy<SharedLayoutContextWrapper,UnsafeFlow>);

    #[inline(always)]
    fn run_parallel_helper(&self,
                           unsafe_flow: UnsafeFlow,
                           proxy: &mut WorkerProxy<SharedLayoutContextWrapper,UnsafeFlow>,
                           top_down_func: extern "Rust" fn(UnsafeFlow,
                                                           &mut WorkerProxy<SharedLayoutContextWrapper,
                                                                            UnsafeFlow>),
                           bottom_up_func: extern "Rust" fn(UnsafeFlow,
                                                            &mut WorkerProxy<SharedLayoutContextWrapper,
                                                                             UnsafeFlow>)) {
        let mut had_children = false;
        unsafe {
            // Get a real flow.
            let flow: &mut FlowRef = mem::transmute(&unsafe_flow);

            if self.should_process(&mut **flow) {
                // Perform the appropriate traversal.
                self.process(&mut **flow);
            }

            // Possibly enqueue the children.
            for kid in flow::child_iter(&mut **flow) {
                had_children = true;
                proxy.push(WorkUnit {
                    fun: top_down_func,
                    data: borrowed_flow_to_unsafe_flow(kid),
                });
            }

        }

        // If there were no more children, start assigning block-sizes.
        if !had_children {
            bottom_up_func(unsafe_flow, proxy)
        }
    }
}

impl<'a> ParallelPostorderFlowTraversal for BubbleISizes<'a> {}

impl<'a> ParallelPreorderFlowTraversal for AssignISizes<'a> {
    fn run_parallel(&self,
                    unsafe_flow: UnsafeFlow,
                    proxy: &mut WorkerProxy<SharedLayoutContextWrapper,UnsafeFlow>) {
        self.run_parallel_helper(unsafe_flow,
                                 proxy,
                                 assign_inline_sizes,
                                 assign_block_sizes_and_store_overflow)
    }
}

impl<'a> ParallelPostorderFlowTraversal for AssignBSizesAndStoreOverflow<'a> {}

impl<'a> ParallelPreorderFlowTraversal for ComputeAbsolutePositions<'a> {
    fn run_parallel(&self,
                    unsafe_flow: UnsafeFlow,
                    proxy: &mut WorkerProxy<SharedLayoutContextWrapper, UnsafeFlow>) {
        self.run_parallel_helper(unsafe_flow,
                                 proxy,
                                 compute_absolute_positions,
                                 build_display_list)
    }
}

impl<'a> ParallelPostorderFlowTraversal for BuildDisplayList<'a> {}

impl<'a> ParallelPostorderDomTraversal for ConstructFlows<'a> {}

impl <'a> ParallelPreorderDomTraversal for RecalcStyleForNode<'a> {
    fn run_parallel(&self,
                    unsafe_node: UnsafeLayoutNode,
                    proxy: &mut WorkerProxy<SharedLayoutContextWrapper, UnsafeLayoutNode>) {
        self.run_parallel_helper(unsafe_node,
                                 proxy,
                                 recalc_style,
                                 construct_flows)
    }
}

fn recalc_style(unsafe_node: UnsafeLayoutNode,
                proxy: &mut WorkerProxy<SharedLayoutContextWrapper, UnsafeLayoutNode>) {
    let shared_layout_context = unsafe { &*(proxy.user_data().0) };
    let layout_context = LayoutContext::new(shared_layout_context);
    let recalc_style_for_node_traversal = RecalcStyleForNode {
        layout_context: &layout_context,
    };
    recalc_style_for_node_traversal.run_parallel(unsafe_node, proxy)
}

fn construct_flows(unsafe_node: UnsafeLayoutNode,
                   proxy: &mut WorkerProxy<SharedLayoutContextWrapper, UnsafeLayoutNode>) {
    let shared_layout_context = unsafe { &*(proxy.user_data().0) };
    let layout_context = LayoutContext::new(shared_layout_context);
    let construct_flows_traversal = ConstructFlows {
        layout_context: &layout_context,
    };
    construct_flows_traversal.run_parallel(unsafe_node, proxy)
}

fn assign_inline_sizes(unsafe_flow: UnsafeFlow,
                       proxy: &mut WorkerProxy<SharedLayoutContextWrapper,UnsafeFlow>) {
    let shared_layout_context = unsafe { &*(proxy.user_data().0) };
    let layout_context = LayoutContext::new(shared_layout_context);
    let assign_inline_sizes_traversal = AssignISizes {
        layout_context: &layout_context,
    };
    assign_inline_sizes_traversal.run_parallel(unsafe_flow, proxy)
}

fn assign_block_sizes_and_store_overflow(unsafe_flow: UnsafeFlow,
                                         proxy: &mut WorkerProxy<SharedLayoutContextWrapper,UnsafeFlow>) {
    let shared_layout_context = unsafe { &*(proxy.user_data().0) };
    let layout_context = LayoutContext::new(shared_layout_context);
    let assign_block_sizes_traversal = AssignBSizesAndStoreOverflow {
        layout_context: &layout_context,
    };
    assign_block_sizes_traversal.run_parallel(unsafe_flow, proxy)
}

fn compute_absolute_positions(unsafe_flow: UnsafeFlow,
                              proxy: &mut WorkerProxy<SharedLayoutContextWrapper, UnsafeFlow>) {
    let shared_layout_context = unsafe { &*(proxy.user_data().0) };
    let layout_context = LayoutContext::new(shared_layout_context);
    let compute_absolute_positions_traversal = ComputeAbsolutePositions {
        layout_context: &layout_context,
    };
    compute_absolute_positions_traversal.run_parallel(unsafe_flow, proxy);
}

fn build_display_list(unsafe_flow: UnsafeFlow,
                      proxy: &mut WorkerProxy<SharedLayoutContextWrapper, UnsafeFlow>) {
    let shared_layout_context = unsafe { &*(proxy.user_data().0) };
    let layout_context = LayoutContext::new(shared_layout_context);

    let build_display_list_traversal = BuildDisplayList {
        layout_context: &layout_context,
    };

    build_display_list_traversal.run_parallel(unsafe_flow, proxy);
}

pub fn traverse_dom_preorder(root: LayoutNode,
                             shared_layout_context: &SharedLayoutContext,
                             queue: &mut WorkQueue<SharedLayoutContextWrapper, UnsafeLayoutNode>) {
    queue.data = SharedLayoutContextWrapper(shared_layout_context as *const _);

    queue.push(WorkUnit {
        fun:  recalc_style,
        data: layout_node_to_unsafe_layout_node(&root),
    });

    queue.run();

    queue.data = SharedLayoutContextWrapper(ptr::null());
}

pub fn traverse_flow_tree_preorder(root: &mut FlowRef,
                                   profiler_metadata: ProfilerMetadata,
                                   time_profiler_chan: TimeProfilerChan,
                                   shared_layout_context: &SharedLayoutContext,
                                   queue: &mut WorkQueue<SharedLayoutContextWrapper,UnsafeFlow>) {
    if opts::get().bubble_inline_sizes_separately {
        let layout_context = LayoutContext::new(shared_layout_context);
        let bubble_inline_sizes = BubbleISizes { layout_context: &layout_context };
        root.traverse_postorder(&bubble_inline_sizes);
    }

    queue.data = SharedLayoutContextWrapper(shared_layout_context as *const _);

    profile(TimeProfilerCategory::LayoutParallelWarmup, profiler_metadata,
            time_profiler_chan, || {
        queue.push(WorkUnit {
            fun: assign_inline_sizes,
            data: mut_owned_flow_to_unsafe_flow(root),
        })
    });

    queue.run();

    queue.data = SharedLayoutContextWrapper(ptr::null())
}

pub fn build_display_list_for_subtree(root: &mut FlowRef,
                                      profiler_metadata: ProfilerMetadata,
                                      time_profiler_chan: TimeProfilerChan,
                                      shared_layout_context: &SharedLayoutContext,
                                      queue: &mut WorkQueue<SharedLayoutContextWrapper,UnsafeFlow>) {
    queue.data = SharedLayoutContextWrapper(shared_layout_context as *const _);

    profile(TimeProfilerCategory::LayoutParallelWarmup, profiler_metadata,
            time_profiler_chan, || {
        queue.push(WorkUnit {
            fun: compute_absolute_positions,
            data: mut_owned_flow_to_unsafe_flow(root),
        })
    });

    queue.run();

    queue.data = SharedLayoutContextWrapper(ptr::null())
}
