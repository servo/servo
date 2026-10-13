/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::NoGC;
use script_bindings::dom::UnrootedDom;
use script_bindings::inheritance::Castable;

use crate::dom::types::{HTMLSlotElement, ShadowRoot};
use crate::dom::{Element, Node};

pub(crate) trait NoGcTraversal {
    fn parent<'a>(no_gc: &'a NoGC, node: &Node) -> Option<UnrootedDom<'a, Node>>;
    fn children<'a>(no_gc: &'a NoGC, node: &Node) -> impl Iterator<Item = UnrootedDom<'a, Node>>;
}

pub(crate) struct LightDomNoGcTraversal;

impl NoGcTraversal for LightDomNoGcTraversal {
    fn parent<'a>(no_gc: &'a NoGC, node: &Node) -> Option<UnrootedDom<'a, Node>> {
        node.get_parent_node_unrooted(no_gc)
    }
    fn children<'a>(no_gc: &'a NoGC, node: &Node) -> impl Iterator<Item = UnrootedDom<'a, Node>> {
        node.children_unrooted(no_gc)
    }
}

pub(crate) struct FlatTreeForSelectionNoGcTraversal;

impl NoGcTraversal for FlatTreeForSelectionNoGcTraversal {
    fn parent<'a>(no_gc: &'a NoGC, node: &Node) -> Option<UnrootedDom<'a, Node>> {
        if let Some(shadow_root) = node.downcast::<ShadowRoot>() {
            return Some(UnrootedDom::upcast(shadow_root.host_unrooted(no_gc)));
        }
        if let Some(assigned_slot) = node.assigned_slot_unrooted(no_gc) {
            return Some(UnrootedDom::upcast(assigned_slot));
        }
        node.get_parent_node_unrooted(no_gc)
    }

    fn children<'a>(no_gc: &'a NoGC, node: &Node) -> impl Iterator<Item = UnrootedDom<'a, Node>> {
        if let Some(shadow_root) = node
            .downcast::<Element>()
            .and_then(|element| element.shadow_root_unrooted(no_gc))
        {
            return FlatTreeChildIterator {
                no_gc,
                next_child: Some(UnrootedDom::from_ref(shadow_root.upcast(), no_gc)),
            };
        }

        if let Some(slot) = node.downcast::<HTMLSlotElement>() &&
            let Some(first_node) = slot.assigned_nodes().first()
        {
            return FlatTreeChildIterator {
                no_gc,
                next_child: Some(UnrootedDom::from_ref(first_node.node(), no_gc)),
            };
        }

        FlatTreeChildIterator {
            no_gc,
            next_child: node.first_child().get_unrooted(no_gc),
        }
    }
}

struct FlatTreeChildIterator<'no_gc> {
    no_gc: &'no_gc NoGC,
    next_child: Option<UnrootedDom<'no_gc, Node>>,
}

impl<'no_gc> Iterator for FlatTreeChildIterator<'no_gc> {
    type Item = UnrootedDom<'no_gc, Node>;

    fn next(&mut self) -> Option<Self::Item> {
        let child = self.next_child.take()?;
        self.next_child = child.next_flat_tree_sibling_unrooted(self.no_gc);
        Some(child)
    }
}
