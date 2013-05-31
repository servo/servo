/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

///
/// Implementation of the callbacks that the CSS selector engine uses to query the DOM.
///

use core::str::eq_slice;
use newcss::select::SelectHandler;
use script::dom::node::{AbstractNode, LayoutView};

pub struct NodeSelectHandler {
    node: AbstractNode<LayoutView>,
}

fn with_node_name<R>(node: AbstractNode<LayoutView>, f: &fn(&str) -> R) -> R {
    if !node.is_element() {
        fail!(~"attempting to style non-element node");
    }
    do node.with_imm_element |element_n| {
        f(element_n.tag_name)
    }
}

impl SelectHandler<AbstractNode<LayoutView>> for NodeSelectHandler {
    fn with_node_name<R>(&self, node: &AbstractNode<LayoutView>, f: &fn(&str) -> R) -> R {
        with_node_name(*node, f)
    }

    fn named_parent_node(&self, node: &AbstractNode<LayoutView>, name: &str)
                         -> Option<AbstractNode<LayoutView>> {
        match node.parent_node() {
            Some(parent) => {
                do with_node_name(parent) |node_name| {
                    if eq_slice(name, node_name) {
                        Some(parent)
                    } else {
                        None
                    }
                }
            }
            None => None
        }
    }

    fn parent_node(&self, node: &AbstractNode<LayoutView>) -> Option<AbstractNode<LayoutView>> {
        node.parent_node()
    }

    // TODO: Use a Bloom filter.
    fn named_ancestor_node(&self, node: &AbstractNode<LayoutView>, name: &str)
                           -> Option<AbstractNode<LayoutView>> {
        let mut node = *node;
        loop {
            let parent = node.parent_node();
            match parent {
                Some(parent) => {
                    let mut found = false;
                    do with_node_name(parent) |node_name| {
                        if eq_slice(name, node_name) {
                            found = true;
                        }
                    }
                    if found {
                        return Some(parent);
                    }
                    node = parent;
                }
                None => return None
            }
        }
    }

    fn node_is_root(&self, node: &AbstractNode<LayoutView>) -> bool {
        self.parent_node(node).is_none()
    }

    fn node_is_link(&self, node: &AbstractNode<LayoutView>) -> bool {
        if node.is_element() {
            do node.with_imm_element |element| {
                "a" == element.tag_name
            }
        } else {
            false
        }
    }

    fn with_node_classes<R>(&self, node: &AbstractNode<LayoutView>, f: &fn(Option<&str>) -> R) -> R {
        if !node.is_element() {
            fail!(~"attempting to style non-element node");
        }
        do node.with_imm_element() |element_n| {
            f(element_n.get_attr("class"))
        }
    }

    fn node_has_class(&self, node: &AbstractNode<LayoutView>, class: &str) -> bool {
        if !node.is_element() {
            fail!(~"attempting to style non-element node");
        }
        do node.with_imm_element |element_n| {
            match element_n.get_attr("class") {
                None => false,
                Some(existing_classes) => {
                    let mut ret = false;
                    for str::each_split_char(existing_classes, ' ') |s| {
                        if s == class {
                            ret = true;
                            break;
                        }
                    }
                    ret
                }
            }
        }
    }

    fn with_node_id<R>(&self, node: &AbstractNode<LayoutView>, f: &fn(Option<&str>) -> R) -> R {
        if !node.is_element() {
            fail!(~"attempting to style non-element node");
        }
        do node.with_imm_element() |element_n| {
            f(element_n.get_attr("id"))
        }
    }

    fn node_has_id(&self, node: &AbstractNode<LayoutView>, id: &str) -> bool {
        if !node.is_element() {
            fail!(~"attempting to style non-element node");
        }
        do node.with_imm_element |element_n| {
            match element_n.get_attr("id") {
                None => false,
                Some(existing_id) => id == existing_id
            }
        }
    }
}
