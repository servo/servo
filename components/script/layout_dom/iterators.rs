/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::iter::FusedIterator;

use layout_api::{DangerousStyleNode, LayoutElement, LayoutNode};
use style::dom::{DomChildren, TElement, TShadowRoot};

use crate::layout_dom::{ServoDangerousStyleElement, ServoDangerousStyleNode, ServoLayoutNode};

pub struct ReverseChildrenIterator<'dom> {
    current: Option<ServoLayoutNode<'dom>>,
}

impl<'dom> Iterator for ReverseChildrenIterator<'dom> {
    type Item = ServoLayoutNode<'dom>;

    #[expect(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current;
        self.current = node.and_then(|node| unsafe { node.dangerous_previous_sibling() });
        node
    }
}

pub enum ServoLayoutNodeChildrenIterator<'dom> {
    /// Iterating over the children of a node
    Node(Option<ServoLayoutNode<'dom>>),
    /// Iterating over the assigned nodes of a `HTMLSlotElement`
    Slottables(<Vec<ServoDangerousStyleNode<'dom>> as IntoIterator>::IntoIter),
}

impl<'dom> ServoLayoutNodeChildrenIterator<'dom> {
    #[expect(unsafe_code)]
    pub(super) fn new_for_flat_tree(parent: ServoLayoutNode<'dom>) -> Self {
        if let Some(element) = parent.as_element() {
            if let Some(shadow) = element.shadow_root() {
                return Self::new_for_flat_tree(shadow.as_node().layout_node());
            };

            let element = unsafe { element.dangerous_style_element() };
            let slotted_nodes = element.slotted_nodes();
            if !slotted_nodes.is_empty() {
                #[expect(clippy::unnecessary_to_owned)] // Clippy is wrong.
                return Self::Slottables(slotted_nodes.to_owned().into_iter());
            }
        }

        Self::Node(unsafe { parent.dangerous_first_child() })
    }

    #[expect(unsafe_code)]
    pub(super) fn new_for_dom_tree(parent: ServoLayoutNode<'dom>) -> Self {
        Self::Node(unsafe { parent.dangerous_first_child() })
    }
}

impl<'dom> Iterator for ServoLayoutNodeChildrenIterator<'dom> {
    type Item = ServoLayoutNode<'dom>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Node(node) => {
                #[expect(unsafe_code)]
                let next_sibling = unsafe { (*node)?.dangerous_next_sibling() };
                std::mem::replace(node, next_sibling)
            },
            Self::Slottables(slots) => slots.next().map(|node| node.layout_node()),
        }
    }
}

impl FusedIterator for ServoLayoutNodeChildrenIterator<'_> {}

pub enum DOMDescendantIterator<'dom> {
    /// Iterating over the children of a node, including children of a potential
    /// [ShadowRoot](crate::dom::shadow_root::ShadowRoot)
    Children(DomChildren<ServoDangerousStyleNode<'dom>>),
    /// Iterating over the content's of a [`<slot>`](HTMLSlotElement) element.
    Slottables {
        slot: ServoDangerousStyleElement<'dom>,
        index: usize,
    },
}

impl<'dom> Iterator for DOMDescendantIterator<'dom> {
    type Item = ServoDangerousStyleNode<'dom>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Children(children) => children.next(),
            Self::Slottables { slot, index } => {
                let slottables = slot.slotted_nodes();
                let slot = slottables.get(*index)?;
                *index += 1;
                Some(*slot)
            },
        }
    }
}
