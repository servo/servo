/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements parallel traversals over the DOM and flow trees.
//!
//! This code is highly unsafe. Keep this file small and easy to audit.

use context::{LayoutContext, SharedLayoutContext};
use flow::{Flow, MutableFlowUtils, PreorderFlowTraversal, PostorderFlowTraversal};
use flow;
use flow_ref::FlowRef;
use traversal::{RecalcStyleForNode, ConstructFlows};
use traversal::{AssignBSizesAndStoreOverflow, AssignISizes, BubbleISizes};
use url::Url;
use util::{LayoutDataAccess, LayoutDataWrapper};
use wrapper::{layout_node_to_unsafe_layout_node, layout_node_from_unsafe_layout_node, LayoutNode};
use wrapper::{PostorderNodeMutTraversal, UnsafeLayoutNode};
use wrapper::{PreorderDOMTraversal, PostorderDOMTraversal};

use servo_util::time::{TimeProfilerChan, profile};
use servo_util::time;
use servo_util::workqueue::{WorkQueue, WorkUnit, WorkerProxy};
use std::mem;
use std::ptr;
use std::sync::atomics::{AtomicInt, Relaxed, SeqCst};

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
pub trait ParallelPreorderDOMTraversal : PreorderDOMTraversal {
    fn run_parallel(&mut self,
                    node: UnsafeLayoutNode,
                    proxy: &mut WorkerProxy<*const SharedLayoutContext,UnsafeLayoutNode>);

    #[inline(always)]
    fn run_parallel_helper(&mut self,
                           unsafe_node: UnsafeLayoutNode,
                           proxy: &mut WorkerProxy<*const SharedLayoutContext,UnsafeLayoutNode>,
                           top_down_func: extern "Rust" fn(UnsafeFlow,
                                                           &mut WorkerProxy<*const SharedLayoutContext,
                                                                            UnsafeLayoutNode>),
                           bottom_up_func: extern "Rust" fn(UnsafeFlow,
                                                            &mut WorkerProxy<*const SharedLayoutContext,
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
            layout_data.data.parallel.children_count.store(child_count as int, Relaxed);
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
trait ParallelPostorderDOMTraversal : PostorderDOMTraversal {
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
    fn run_parallel(&mut self,
                    mut unsafe_node: UnsafeLayoutNode,
                    proxy: &mut WorkerProxy<*const SharedLayoutContext,UnsafeLayoutNode>) {
        loop {
            // Get a real layout node.
            let node: LayoutNode = unsafe {
                layout_node_from_unsafe_layout_node(&unsafe_node)
            };

            // Perform the appropriate traversal.
            self.process(node);

            let shared_layout_context = unsafe { &**proxy.user_data() };
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
                    .fetch_sub(1, SeqCst) == 1 {
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
    /// The number of children and absolute descendants that still need work done.
    pub children_and_absolute_descendant_count: AtomicInt,
    /// The address of the parent flow.
    pub parent: UnsafeFlow,
}

impl FlowParallelInfo {
    pub fn new() -> FlowParallelInfo {
        FlowParallelInfo {
            children_count: AtomicInt::new(0),
            children_and_absolute_descendant_count: AtomicInt::new(0),
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
    fn run_parallel(&mut self,
                    mut unsafe_flow: UnsafeFlow,
                    _: &mut WorkerProxy<*const SharedLayoutContext,UnsafeFlow>) {
        loop {
            unsafe {
                // Get a real flow.
                let flow: &mut FlowRef = mem::transmute(&unsafe_flow);

                // Perform the appropriate traversal.
                if self.should_process(flow.get_mut()) {
                    self.process(flow.get_mut());
                }


                let base = flow::mut_base(flow.get_mut());

                // Reset the count of children for the next layout traversal.
                base.parallel.children_count.store(base.children.len() as int, Relaxed);

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
                let parent_base = flow::mut_base(parent.get_mut());
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

/// A parallel top-down flow traversal.
trait ParallelPreorderFlowTraversal : PreorderFlowTraversal {
    fn run_parallel(&mut self,
                    unsafe_flow: UnsafeFlow,
                    proxy: &mut WorkerProxy<*const SharedLayoutContext,UnsafeFlow>);

    #[inline(always)]
    fn run_parallel_helper(&mut self,
                           unsafe_flow: UnsafeFlow,
                           proxy: &mut WorkerProxy<*const SharedLayoutContext,UnsafeFlow>,
                           top_down_func: extern "Rust" fn(UnsafeFlow,
                                                           &mut WorkerProxy<*const SharedLayoutContext,
                                                                            UnsafeFlow>),
                           bottom_up_func: extern "Rust" fn(UnsafeFlow,
                                                            &mut WorkerProxy<*const SharedLayoutContext,
                                                                             UnsafeFlow>)) {
        let mut had_children = false;
        unsafe {
            // Get a real flow.
            let flow: &mut FlowRef = mem::transmute(&unsafe_flow);

            if self.should_process(flow.get_mut()) {
                // Perform the appropriate traversal.
                self.process(flow.get_mut());
            }

            // Possibly enqueue the children.
            for kid in flow::child_iter(flow.get_mut()) {
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
    fn run_parallel(&mut self,
                    unsafe_flow: UnsafeFlow,
                    proxy: &mut WorkerProxy<*const SharedLayoutContext,UnsafeFlow>) {
        self.run_parallel_helper(unsafe_flow,
                                 proxy,
                                 assign_inline_sizes,
                                 assign_block_sizes_and_store_overflow)
    }
}

impl<'a> ParallelPostorderFlowTraversal for AssignBSizesAndStoreOverflow<'a> {}

impl<'a> ParallelPostorderDOMTraversal for ConstructFlows<'a> {}

impl <'a> ParallelPreorderDOMTraversal for RecalcStyleForNode<'a> {
    fn run_parallel(&mut self,
                    unsafe_node: UnsafeLayoutNode,
                    proxy: &mut WorkerProxy<*const SharedLayoutContext, UnsafeLayoutNode>) {
        self.run_parallel_helper(unsafe_node,
                                 proxy,
                                 recalc_style,
                                 construct_flows)
    }
}

fn recalc_style(unsafe_node: UnsafeLayoutNode,
                proxy: &mut WorkerProxy<*const SharedLayoutContext, UnsafeLayoutNode>) {
    let shared_layout_context = unsafe { &**proxy.user_data() };
    let layout_context = LayoutContext::new(shared_layout_context);
    let mut recalc_style_for_node_traversal = RecalcStyleForNode {
        layout_context: &layout_context,
    };
    recalc_style_for_node_traversal.run_parallel(unsafe_node, proxy)
}

fn construct_flows(unsafe_node: UnsafeLayoutNode,
                   proxy: &mut WorkerProxy<*const SharedLayoutContext, UnsafeLayoutNode>) {
    let shared_layout_context = unsafe { &**proxy.user_data() };
    let layout_context = LayoutContext::new(shared_layout_context);
    let mut construct_flows_traversal = ConstructFlows {
        layout_context: &layout_context,
    };
    construct_flows_traversal.run_parallel(unsafe_node, proxy)
}

fn assign_inline_sizes(unsafe_flow: UnsafeFlow,
                       proxy: &mut WorkerProxy<*const SharedLayoutContext,UnsafeFlow>) {
    let shared_layout_context = unsafe { &**proxy.user_data() };
    let layout_context = LayoutContext::new(shared_layout_context);
    let mut assign_inline_sizes_traversal = AssignISizes {
        layout_context: &layout_context,
    };
    assign_inline_sizes_traversal.run_parallel(unsafe_flow, proxy)
}

fn assign_block_sizes_and_store_overflow(unsafe_flow: UnsafeFlow,
                                     proxy: &mut WorkerProxy<*const SharedLayoutContext,UnsafeFlow>) {
    let shared_layout_context = unsafe { &**proxy.user_data() };
    let layout_context = LayoutContext::new(shared_layout_context);
    let mut assign_block_sizes_traversal = AssignBSizesAndStoreOverflow {
        layout_context: &layout_context,
    };
    assign_block_sizes_traversal.run_parallel(unsafe_flow, proxy)
}

fn compute_absolute_position(unsafe_flow: UnsafeFlow,
                             proxy: &mut WorkerProxy<*const SharedLayoutContext,UnsafeFlow>) {
    let mut had_descendants = false;
    unsafe {
        // Get a real flow.
        let flow: &mut FlowRef = mem::transmute(&unsafe_flow);

        // Compute the absolute position for the flow.
        flow.get_mut().compute_absolute_position();

        // If we are the containing block, count the number of absolutely-positioned children, so
        // that we don't double-count them in the `children_and_absolute_descendant_count`
        // reference count.
        let mut absolutely_positioned_child_count = 0u;
        for kid in flow::child_iter(flow.get_mut()) {
            if kid.is_absolutely_positioned() {
                absolutely_positioned_child_count += 1;
            }
        }

        drop(flow::mut_base(flow.get_mut()).parallel
                                           .children_and_absolute_descendant_count
                                           .fetch_sub(absolutely_positioned_child_count as int,
                                                      SeqCst));

        // Enqueue all non-absolutely-positioned children.
        for kid in flow::child_iter(flow.get_mut()) {
            if !kid.is_absolutely_positioned() {
                had_descendants = true;
                proxy.push(WorkUnit {
                    fun: compute_absolute_position,
                    data: borrowed_flow_to_unsafe_flow(kid),
                });
            }
        }

        // Possibly enqueue absolute descendants.
        for absolute_descendant_link in flow::mut_base(flow.get_mut()).abs_descendants.iter() {
            had_descendants = true;
            let descendant = absolute_descendant_link;
            proxy.push(WorkUnit {
                fun: compute_absolute_position,
                data: borrowed_flow_to_unsafe_flow(descendant),
            });
        }

        // If there were no more descendants, start building the display list.
        if !had_descendants {
            build_display_list(mut_owned_flow_to_unsafe_flow(flow), proxy)
        }
    }
}

fn build_display_list(mut unsafe_flow: UnsafeFlow,
                      proxy: &mut WorkerProxy<*const SharedLayoutContext,UnsafeFlow>) {
    let shared_layout_context = unsafe { &**proxy.user_data() };
    let layout_context = LayoutContext::new(shared_layout_context);

    loop {
        unsafe {
            // Get a real flow.
            let flow: &mut FlowRef = mem::transmute(&unsafe_flow);

            // Build display lists.
            flow.get_mut().build_display_list(&layout_context);

            {
                let base = flow::mut_base(flow.get_mut());

                // Reset the count of children and absolute descendants for the next layout
                // traversal.
                let children_and_absolute_descendant_count = base.children.len() +
                    base.abs_descendants.len();
                base.parallel
                    .children_and_absolute_descendant_count
                    .store(children_and_absolute_descendant_count as int, Relaxed);
            }

            // Possibly enqueue the parent.
            let unsafe_parent = if flow.get().is_absolutely_positioned() {
                match *flow::mut_base(flow.get_mut()).absolute_cb.get() {
                    None => fail!("no absolute containing block for absolutely positioned?!"),
                    Some(ref mut absolute_cb) => {
                        mut_borrowed_flow_to_unsafe_flow(absolute_cb.get_mut())
                    }
                }
            } else {
                flow::mut_base(flow.get_mut()).parallel.parent
            };
            if unsafe_parent == null_unsafe_flow() {
                // We're done!
                break
            }

            // No, we're not at the root yet. Then are we the last child
            // of our parent to finish processing? If so, we can continue
            // on with our parent; otherwise, we've gotta wait.
            let parent: &mut FlowRef = mem::transmute(&unsafe_parent);
            let parent_base = flow::mut_base(parent.get_mut());
            if parent_base.parallel
                          .children_and_absolute_descendant_count
                          .fetch_sub(1, SeqCst) == 1 {
                // We were the last child of our parent. Build display lists for our parent.
                unsafe_flow = unsafe_parent
            } else {
                // Stop.
                break
            }
        }
    }
}

pub fn traverse_dom_preorder(root: LayoutNode,
                             shared_layout_context: &SharedLayoutContext,
                             queue: &mut WorkQueue<*const SharedLayoutContext, UnsafeLayoutNode>) {
    queue.data = shared_layout_context as *const _;

    queue.push(WorkUnit {
        fun:  recalc_style,
        data: layout_node_to_unsafe_layout_node(&root),
    });

    queue.run();

    queue.data = ptr::null();
}

pub fn traverse_flow_tree_preorder(root: &mut FlowRef,
                                   url: &Url,
                                   iframe: bool,
                                   first_reflow: bool,
                                   time_profiler_chan: TimeProfilerChan,
                                   shared_layout_context: &SharedLayoutContext,
                                   queue: &mut WorkQueue<*const SharedLayoutContext,UnsafeFlow>) {
    queue.data = shared_layout_context as *const _;

    profile(time::LayoutParallelWarmupCategory, Some((url, iframe, first_reflow)), time_profiler_chan, || {
        queue.push(WorkUnit {
            fun: assign_inline_sizes,
            data: mut_owned_flow_to_unsafe_flow(root),
        })
    });

    queue.run();

    queue.data = ptr::null()
}

pub fn build_display_list_for_subtree(root: &mut FlowRef,
                                      url: &Url,
                                      iframe: bool,
                                      first_reflow: bool,
                                      time_profiler_chan: TimeProfilerChan,
                                      shared_layout_context: &SharedLayoutContext,
                                      queue: &mut WorkQueue<*const SharedLayoutContext,UnsafeFlow>) {
    queue.data = shared_layout_context as *const _;

    profile(time::LayoutParallelWarmupCategory, Some((url, iframe, first_reflow)), time_profiler_chan, || {
        queue.push(WorkUnit {
            fun: compute_absolute_position,
            data: mut_owned_flow_to_unsafe_flow(root),
        })
    });

    queue.run();

    queue.data = ptr::null()
}
