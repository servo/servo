/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_util::tree::{TreeUtils, TreeNodeRef, TreeNode};
use layout::flow::{FlowContext,SequentialView,VisitView,VisitChildView};
use layout::clq::WorkstealingDeque;

use std::vec;
use std::rand::random;
use std::uint;
use std::unstable::atomics::{AtomicUint,SeqCst};
use std::unstable::sync::UnsafeAtomicRcBox;
use extra::arc::ARC;

/// Stub sequential traverser to test out memory safety.
pub struct SequentialTraverser;

impl SequentialTraverser {
    pub fn new() -> SequentialTraverser {
        return SequentialTraverser;
    }

    pub fn traverse_preorder(&self, 
                         tree: FlowContext<SequentialView,SequentialView>, 
                         callback: &fn(FlowContext<VisitView,VisitChildView>) -> bool) 
                         -> bool {
    
        unsafe {
            if !callback(tree.restrict_view()) {
                return false;
            }
        }

        for tree.each_child |kid| {
            // FIXME: Work around rust#2202. We should be able to pass the callback directly.
            if !self.traverse_preorder(kid, |a| callback(a)) {
                return false;
            }
        }

        true
    }

    pub fn traverse_postorder(&self, 
                         tree: FlowContext<SequentialView,SequentialView>, 
                         callback: &fn(FlowContext<VisitView,VisitChildView>) -> bool) 
                         -> bool {

        for tree.each_child |kid| {
            // FIXME: Work around rust#2202. We should be able to pass the callback directly.
            if !self.traverse_postorder(kid, |a| callback(a)) {
                return false;
            }
        }
        unsafe {
            callback(tree.restrict_view())
        }
    }

    /// Like traverse_preorder, but don't end the whole traversal if the callback
    /// returns false.
    pub fn partially_traverse_preorder(&self, 
                         tree: FlowContext<SequentialView,SequentialView>, 
                         callback: &fn(FlowContext<VisitView,VisitChildView>) -> bool) {

        unsafe {
            if !callback(tree.restrict_view()) {
                return;
            }
        }

        for tree.each_child |kid| {
            // FIXME: Work around rust#2202. We should be able to pass the callback directly.
            self.partially_traverse_preorder(kid, |a| callback(a));
        }
    }

    // Like traverse_postorder, but only visits nodes not marked as inorder
    pub fn traverse_bu_sub_inorder(&self, 
                         tree: FlowContext<SequentialView,SequentialView>, 
                         callback: &fn(FlowContext<VisitView,VisitChildView>) -> bool) 
                         -> bool {

        for tree.each_child |kid| {
            // FIXME: Work around rust#2202. We should be able to pass the callback directly.
            if !self.traverse_bu_sub_inorder(kid, |a| callback(a)) {
                return false;
            }
        }

        if !tree.is_inorder() {
            unsafe {
                callback(tree.restrict_view())
            }
        } else {
            true
        }
    }
}


/// Similar to a task pool, this allows farming out work to a number of tasks to
/// improve parallelism. However, tasks here are not actually swapped, so there
/// is no context switching overhead.
priv struct WorkPool<Work> {
    queues: ~[WorkstealingDeque<Work>]
}


impl<W> WorkPool<W> {
    fn new(num_tasks: uint, max_work: uint) -> WorkPool<W> {
        unsafe {
            WorkPool {
                queues: (vec::from_fn(num_tasks, |_| { WorkstealingDeque::new(max_work) }))
            }
        }
    }

    /// Add an initial item of work to the queue
    fn add_work(&mut self, work: *W) {
        self.queues[0].push(work);
    }

    /// Run the tasks on work items in the queue.
    /// The callee can obtain new work by calling the pop function,
    /// And can add work to the queue by calling the push function.
    /// Work stops when the callback returns false.
    fn execute(&mut self, 
               cb_factory: &fn() -> ~fn(push: &fn(work: *W), 
                                        pop: &fn() -> Option<*W>) 
                                        -> bool) {
        unsafe {
            let unsafe_queues: *mut ~[WorkstealingDeque<W>] = &mut self.queues;
            for uint::range(0,self.queues.len()) |i| {
                let callback = cb_factory();
                do spawn {
                    let mut this_q = (*unsafe_queues)[i];
                    loop {
                        // The push function simply adds work to this tasks queue.
                        // The pop function tries to pull work from the current
                        // queue, but steals from a random queue if this is not
                        // possible.
                        if !callback(|w| { this_q.push(w) }, || {
                            match this_q.pop() {
                                None => {
                                    (*unsafe_queues)[random::<uint>() % (*unsafe_queues).len()].steal()
                                }
                                Some(work) => Some(work)
                            }}) {

                            break;
                        }
                    }
                }
            }
        }
    }
}

priv trait TraversableTree {
    fn each_leaf (&self, callback: &fn(leaf: Self) -> bool) -> bool;
}

impl TraversableTree for FlowContext<SequentialView,SequentialView> {
    fn each_leaf (&self, 
                  callback: &fn(leaf: FlowContext<SequentialView,SequentialView>) -> bool) 
                  -> bool {
        for self.traverse_postorder |node| {
            if node.is_leaf(){
                if !callback(node){
                    break;
                }
            }
        }

        return true;
    }
}

pub enum TraversalType {
    TopDown,
    BottomUp(TraversalSubtype),
}

pub enum TraversalSubtype {
    Normal,
    Inorder
}

trait ParallelTraversalHelper{
    fn get_or_none(&self, uint) -> Option<TraversalType>;
}

impl ParallelTraversalHelper for 
~[(TraversalType, ~fn:Send+Freeze(FlowContext<VisitView,VisitChildView>) -> bool)] {
    fn get_or_none(&self, index: uint) -> Option<TraversalType> {
        if index > self.len() {
            return None;
        }

        match self[index] {
            (traversal,_) => Some(traversal)
        }
    }
}

/// Parallel traversal handler. Traversals are run lazily so transitions can be optimized.
/// Currently, this only handles traversals that alternate in direction (e.g. 
/// td-bu-td-buSubInorder). This is enough for layout and makes the scheduler simple, but
/// we may have to support other traversal sequences in the future.
pub struct ParallelTraverser {
    traversals: ~[(TraversalType, ~fn:Send+Freeze(FlowContext<VisitView,VisitChildView>) -> bool)],
}

impl ParallelTraverser {
    pub fn new(num_traversals: uint) -> ParallelTraverser {
        ParallelTraverser {
            traversals: vec::with_capacity(num_traversals),
        }
    }

    pub fn add_traversal(&mut self,
            ty: TraversalType, 
            cb: ~fn:Send+Freeze(FlowContext<VisitView,VisitChildView>) -> bool) {

        self.traversals.push((ty,cb));
    }

    pub fn run(~self, 
               tree: FlowContext<SequentialView,SequentialView>, 
               num_tasks: uint, 
               max_nodes: uint) {

        if self.traversals.len() == 0 {
            return;
        }

        let last_nodes = AtomicUint::new(match self.traversals[self.traversals.len() - 1] {
                (BottomUp(_), _) => 1,
                _ => fail!("Last traversal must be bottom-up")
        });

        let last_nodes = UnsafeAtomicRcBox::new(last_nodes);

        // all threads need access to the traversals, so wrap them in an ARC
        let shared_traversals = ARC(self.traversals);

        let mut work_pool = WorkPool::new(num_tasks, max_nodes);

        // Initial conditions. Reset the traversal
        // counters on each starting node before
        // adding them to the work queue.
        unsafe {
            let data = shared_traversals.get();
            match data[0] {
                // TD starts at the root.
                (TopDown, _) => {
                    tree.set_traversal(0);
                    work_pool.add_work(tree.encode());
                }

                // Both types of BU start at the leaves
                (BottomUp(_), _) => {
                    for tree.each_leaf |leaf| {
                        leaf.set_traversal(0);
                        work_pool.add_work(leaf.encode());
                    }
                }
            }
        }


        do work_pool.execute { 
            let this_last_nodes = last_nodes.clone();
            let this_traversals = shared_traversals.clone();
            |push, pop| {
                unsafe {
                    let mut last_nodes = *this_last_nodes.get();
                    let traversals = this_traversals.get();
                    let node = match pop() {
                        None => {
                            None
                        }
                        Some(work) => {
                            Some(FlowContext::decode(work))
                        }
                    };

                    match node {
                        None => {
                            if last_nodes.compare_and_swap(0, 0, SeqCst) == 0 {
                                false
                            } else {
                                true
                            }
                        }
                        Some(node) => {
                            match traversals[node.get_traversal()] {
                                (TopDown, ref callback) => {
                                    // simple case: visit the node then add its children
                                    // to the work queue
                                    (*callback)(node.restrict_view());
                                    node.set_traversal(node.get_traversal() + 1);

                                    let mut has_children = false;
                                    for node.each_child |child| {
                                        has_children = true;
                                        child.set_traversal(node.get_traversal() - 1);

                                        push(child.encode());
                                    }

                                    // if the node is a leaf, try and start the next traversal
                                    // in the sequence
                                    if !has_children {
                                        match traversals.get_or_none(node.get_traversal()) {
                                            None => { last_nodes.fetch_sub(1, SeqCst); }
                                            Some(BottomUp(_)) => {
                                                push(node.encode());
                                            }
                                            _ => fail!("Unsupported traversal sequence")
                                        }
                                    }
                                }

                                (BottomUp(subtype), ref callback) => {

                                    match subtype {
                                        Normal => { (*callback)(node.restrict_view()); }
                                        Inorder => {
                                            if !node.is_inorder() {
                                                (*callback)(node.restrict_view());
                                            }
                                        }
                                    }

                                    node.set_traversal(node.get_traversal() + 1);

                                    do node.with_base |base| {
                                        match base.parent_node() {
                                            // if the node is the root, try and start the next traversal
                                            // in the sequence
                                            None => {
                                                match traversals.get_or_none(node.get_traversal()) {
                                                    None => { last_nodes.fetch_sub(1, SeqCst); }
                                                    Some(TopDown) => {
                                                        push(node.encode());
                                                    }
                                                    _ => fail!("Unsupported traversal sequence")
                                                }
                                            }

                                            Some(ref mut parent) => {
                                                // update_child_counter synchronizes on the parent node,
                                                // so only one thread will add the parent to the queue
                                                if parent.update_child_counter() {
                                                    parent.set_traversal(node.get_traversal() - 1);

                                                    push(base.parent_node().get().encode());
                                                }
                                            }
                                        }
                                    }
                                }
                            };
                            true
                        }
                    }
                }
            }
        };
    }
}

