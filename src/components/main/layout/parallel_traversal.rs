/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_util::tree::TreeUtils;
use layout::flow::{FlowContext,SequentialView,VisitView,VisitChildView};

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
            if !callback(FlowContext::decode(tree.encode())) {
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
            callback(FlowContext::decode(tree.encode()))
        }
    }

    /// Like traverse_preorder, but don't end the whole traversal if the callback
    /// returns false.
    pub fn partially_traverse_preorder(&self, 
                         tree: FlowContext<SequentialView,SequentialView>, 
                         callback: &fn(FlowContext<VisitView,VisitChildView>) -> bool) {

        unsafe {
            if !callback(FlowContext::decode(tree.encode())) {
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
                callback(FlowContext::decode(tree.encode()))
            }
        } else {
            true
        }
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


impl [Option<(TraversalType, ~fn(FlowContext<VisitView,VisitChildView>) -> bool)>] {
    fn get_or_none(&self, index: uint) -> Option<TraversalType> {
        if index > self.len() {
            return None;
        }

        self[index]
    }
}

/// Parallel traversal handler. Traversals are run lazily so transitions can be optimized.
/// Currently, this only handles traversals that alternate in direction (e.g. 
/// td-bu-td-buSubInorder). This is enough for layout and makes the scheduler simple, but
/// we may have to support other traversal sequences in the future.
pub struct ParallelTraverser {
    traversals: ~[Option<(TraversalType, ~fn(FlowContext<VisitView,VisitChildView>) -> bool)>]
    traversals_added: uint,
}

struct SharedData {
    traversals: ~[Option<(TraversalType, ~fn(FlowContext<VisitView,VisitChildView>) -> bool)>]
    cur_traversal: uint,
}

impl ParallelTraverser {
    pub fn new(num_traversals: int) -> ParallelTraverser {
        ParallelTraverser {
            traversals: vec::from_elem(num_traversals, None),
            traversals_added: 0
        }
    }

    pub fn add_traversal(&mut self,
                         ty: TraversalType, 
                         cb: ~fn(FlowContext<VisitView,VisitChildView>) -> bool) {
        assert!(self.traversals_added < self.traversals.len(), "Tried to add too many traversals");

        self.traversals[traversals_added] = Some(ty,cb);
        traversals_added += 1;
    }

    pub fn run(~self, tree: FlowContext<SequentialView,SequentialView>, num_tasks: uint) {
        if self.traversals_added == 0 {
            return;
        }

        let last_nodes = AtomicUint::new(match self.traversals[self.traversals_added - 1] {
            Some((BottomUp(_), _)) => 1,
            _ => fail!("Last traversal must be bottom-up")
        });

        let q: [WorkstealingDequeue<uint>] = 
            vec::fom_fn(num_tasks, |_| { WorkstealingDequeue::new() });

        // all threads need access to the traversals, so wrap them in an ARC
        let shared_traversals = ARC(self.traversals);

        unsafe {
            let data = shared_traversals.get();
            // Initial conditions. Reset the traversal
            // counters on each starting node before
            // adding them to the work queue.
            match data[0] {
                // TD starts at the root.
                Some((TopDown, _)) => {
                    tree.set_traversal(0);
                    q[0].push(tree.encode()),
                }

                // Both types of BU start at the leaves
                Some((BottomUp(_), _)) => {
                    for tree.each_leaf |leaf| {
                        leaf.set_traversal(0);
                        q[0].push(leaf.encode());
                    }
                }
            }

            // Each task should only push and pop from its own queue in
            // this array. Pushing or popping from another queue will
            // be racy and possibly memory-unsafe. Any task can steal
            // from any queue safely.
            let q_ptr : *[WorkstealingDequeue<uint>] = &q;
            for uint::range(0, num_tasks) |i| {
                let this_data = shared.clone();
                do spawn {

                    let this_q = *q_ptr[i];
                    let traversals = this_data.get();

                    match this_q.pop() {
                        // If there's no more work to do, try to steal a random thread's
                        // work.
                        None => {    
                            match *q_ptr[random::<uint>() % q_ptr.len()].steal() {
                                None = { 
                                    if last_nodes <= 0 {
                                        break;
                                    }
                                }
                                Some(val) => { this_q.push(val); }
                            }
                        }

                        Some(node) => {

                            let node = FlowContext::decode(node);
                            match traversals[node.get_traversal()] {
                                Some(TopDown, callback) => {
                                    // simple case: visit the node then add its children
                                    // to the work queue
                                    callback(node.restrict_view());
                                    node.set_traversal(node.get_traversal() + 1);

                                    let mut has_children = false;
                                    for node.each_child |child| {
                                        has_children = true;
                                        child.set_traversal(node.get_traversal() - 1);
                                        this_q.push(child.encode());
                                    }

                                    // if the node is a leaf, try and start the next traversal
                                    // in the sequence
                                    if !has_children {
                                        match traversals.get_or_none(node.get_traversal()) {
                                            None => { last_nodes.fetch_sub(1); }
                                            Some(BottomUp)
                                            | Some (BottomUpInorder) ==> {
                                                this_q.push(node.encode());
                                            }
                                            _ => fail!("Unsupported traversal sequence")
                                        }
                                    }
                                }

                                Some(BottomUp(subtype), callback) => {

                                    match subtype {
                                        Normal => { callback(node.restrict_view()); }
                                        Inorder => {
                                            if !node.is_inorder {
                                                callback(node.restrict_view());
                                            }
                                        }
                                    }

                                    node.set_traversal(node.get_traversal() + 1);

                                    match node.parent_node() {
                                        // if the node is the root, try and start the next traversal
                                        // in the sequence
                                        None => {
                                            match traversals.get_or_none(node.get_traversal()) {
                                                None => { last_nodes.fetch_sub(1); }
                                                Some(TopDown) => {
                                                    this_q.push(node.encode());
                                                }
                                                _ => fail!("Unsupported traversal sequence")
                                            }
                                        }
                                        Some(parent) => {
                                            // update_child_counter synchronizes on the parent node,
                                            // so only one thread will add the parent to the queue
                                            if parent.update_child_counter() {
                                                parent.set_traversal(node.get_traversal() - 1);
                                                this_q.push(node.parent().encode());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Block until the worker threads are done.
            while last_nodes > 0 {}
        }
    }
}
