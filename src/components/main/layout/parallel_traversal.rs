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

