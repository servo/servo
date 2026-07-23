/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::NoGC;

use super::FlatTreeParent;
use crate::dom::Node;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRoot_Binding::ShadowRootMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot, UnrootedDom};
use crate::dom::element::Element;
use crate::dom::shadowroot::ShadowRoot;

/// Whether a tree traversal should pass shadow tree boundaries.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ShadowIncluding {
    No,
    Yes,
}

pub(crate) struct FollowingNodeIterator {
    current: Option<DomRoot<Node>>,
    root: DomRoot<Node>,
    shadow_including: ShadowIncluding,
}

impl FollowingNodeIterator {
    pub(crate) fn new(
        current: Option<DomRoot<Node>>,
        root: DomRoot<Node>,
        shadow_including: ShadowIncluding,
    ) -> Self {
        FollowingNodeIterator {
            current,
            root,
            shadow_including,
        }
    }
}

impl FollowingNodeIterator {
    /// Skips iterating the children of the current node
    pub(crate) fn next_skipping_children(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;
        self.next_skipping_children_impl(current)
    }

    fn next_skipping_children_impl(&mut self, current: DomRoot<Node>) -> Option<DomRoot<Node>> {
        if self.root == current {
            self.current = None;
            return None;
        }

        if let Some(next_sibling) = current.GetNextSibling() {
            self.current = Some(next_sibling);
            return current.GetNextSibling();
        }

        for ancestor in current.inclusive_ancestors(self.shadow_including) {
            if self.root == ancestor {
                break;
            }
            if let Some(next_sibling) = ancestor.GetNextSibling() {
                self.current = Some(next_sibling);
                return ancestor.GetNextSibling();
            }
        }
        self.current = None;
        None
    }
}

impl Iterator for FollowingNodeIterator {
    type Item = DomRoot<Node>;

    /// <https://dom.spec.whatwg.org/#concept-tree-following>
    fn next(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;

        if let Some(first_child) = current.GetFirstChild() {
            self.current = Some(first_child);
            return current.GetFirstChild();
        }

        self.next_skipping_children_impl(current)
    }
}

pub(crate) struct UnrootedFollowingNodeIterator<'b> {
    current: Option<UnrootedDom<'b, Node>>,
    root: UnrootedDom<'b, Node>,
    shadow_including: ShadowIncluding,
    no_gc: &'b NoGC,
}

impl<'b> UnrootedFollowingNodeIterator<'b> {
    pub(crate) fn new(
        current: Option<UnrootedDom<'b, Node>>,
        root: UnrootedDom<'b, Node>,
        shadow_including: ShadowIncluding,
        no_gc: &'b NoGC,
    ) -> Self {
        UnrootedFollowingNodeIterator {
            current,
            root,
            shadow_including,
            no_gc,
        }
    }
}

impl<'b> UnrootedFollowingNodeIterator<'b> {
    fn next_skipping_children_impl(
        &mut self,
        current: UnrootedDom<'b, Node>,
    ) -> Option<UnrootedDom<'b, Node>> {
        if self.root == current {
            self.current = None;
            return None;
        }

        if let Some(next_sibling) = current.get_next_sibling_unrooted(self.no_gc) {
            self.current = Some(next_sibling);
            return current.get_next_sibling_unrooted(self.no_gc);
        }

        for ancestor in current.inclusive_ancestors(self.shadow_including) {
            if **self.root == *ancestor {
                break;
            }
            if let Some(next_sibling) = ancestor.get_next_sibling_unrooted(self.no_gc) {
                self.current = Some(next_sibling);
                return ancestor.get_next_sibling_unrooted(self.no_gc);
            }
        }
        self.current = None;
        None
    }
}

impl<'b> Iterator for UnrootedFollowingNodeIterator<'b> {
    type Item = UnrootedDom<'b, Node>;

    /// <https://dom.spec.whatwg.org/#concept-tree-following>
    fn next(&mut self) -> Option<UnrootedDom<'b, Node>> {
        let current = self.current.take()?;

        if let Some(first_child) = current.get_first_child_unrooted(self.no_gc) {
            self.current = Some(first_child);
            return current.get_first_child_unrooted(self.no_gc);
        }

        self.next_skipping_children_impl(current)
    }
}

pub(crate) struct PrecedingNodeIterator {
    current: Option<DomRoot<Node>>,
    root: DomRoot<Node>,
}

impl PrecedingNodeIterator {
    pub(crate) fn new(current: Option<DomRoot<Node>>, root: DomRoot<Node>) -> Self {
        PrecedingNodeIterator { current, root }
    }
}

impl Iterator for PrecedingNodeIterator {
    type Item = DomRoot<Node>;

    /// <https://dom.spec.whatwg.org/#concept-tree-preceding>
    fn next(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;

        self.current = if self.root == current {
            None
        } else if let Some(previous_sibling) = current.GetPreviousSibling() {
            if self.root == previous_sibling {
                None
            } else if let Some(last_child) = previous_sibling.descending_last_children().last() {
                Some(last_child)
            } else {
                Some(previous_sibling)
            }
        } else {
            current.GetParentNode()
        };
        self.current.clone()
    }
}

pub(crate) struct UnrootedPrecedingNodeIterator<'b> {
    current: Option<UnrootedDom<'b, Node>>,
    no_gc: &'b NoGC,
    root: UnrootedDom<'b, Node>,
}

impl<'b> UnrootedPrecedingNodeIterator<'b> {
    pub(crate) fn new(
        current: Option<UnrootedDom<'b, Node>>,
        root: UnrootedDom<'b, Node>,
        no_gc: &'b NoGC,
    ) -> Self {
        UnrootedPrecedingNodeIterator {
            current,
            no_gc,
            root,
        }
    }
}

impl<'b> Iterator for UnrootedPrecedingNodeIterator<'b> {
    type Item = UnrootedDom<'b, Node>;

    /// <https://dom.spec.whatwg.org/#concept-tree-preceding>
    fn next(&mut self) -> Option<UnrootedDom<'b, Node>> {
        let current = self.current.take()?;

        self.current = if self.root == current {
            None
        } else if let Some(previous_sibling) = current.get_previous_sibling_unrooted(self.no_gc) {
            if self.root == previous_sibling {
                None
            } else if let Some(last_child) = previous_sibling
                .descending_last_children_unrooted(self.no_gc)
                .last()
            {
                Some(last_child)
            } else {
                Some(previous_sibling)
            }
        } else {
            current.get_parent_node_unrooted(self.no_gc)
        };

        self.current.clone()
    }
}

pub(crate) struct SimpleNodeIterator<I>
where
    I: Fn(&Node) -> Option<DomRoot<Node>>,
{
    current: Option<DomRoot<Node>>,
    next_node: I,
}

impl<I> SimpleNodeIterator<I>
where
    I: Fn(&Node) -> Option<DomRoot<Node>>,
{
    pub(crate) fn new(current: Option<DomRoot<Node>>, next_node: I) -> Self {
        SimpleNodeIterator { current, next_node }
    }
}

impl<I> Iterator for SimpleNodeIterator<I>
where
    I: Fn(&Node) -> Option<DomRoot<Node>>,
{
    type Item = DomRoot<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take();
        self.current = current.as_ref().and_then(|c| (self.next_node)(c));
        current
    }
}

/// An efficient SimpleNodeIterator because it skips rooting if there are no GC pauses.
///
/// Use this if you have a `&JSContext` or `NoGC`.
///
/// Normally we need to root every `Node` we come across as we do not know if we will have a GC pause.
/// This does not root the required children. Taking a `&NoGC` enforces that there is no `&mut JSContext`
/// while this iterator is alive.
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
pub(crate) struct UnrootedSimpleNodeIterator<'b, I>
where
    I: Fn(&Node, &'b NoGC) -> Option<UnrootedDom<'b, Node>>,
{
    current: Option<UnrootedDom<'b, Node>>,
    next_node: I,
    /// This is unused and only used for lifetime guarantee of NoGC
    no_gc: &'b NoGC,
}

impl<'b, I> UnrootedSimpleNodeIterator<'b, I>
where
    I: Fn(&Node, &'b NoGC) -> Option<UnrootedDom<'b, Node>>,
{
    pub(crate) fn new(
        current: Option<UnrootedDom<'b, Node>>,
        next_node: I,
        no_gc: &'b NoGC,
    ) -> Self {
        UnrootedSimpleNodeIterator {
            current,
            next_node,
            no_gc,
        }
    }
}

impl<'b, I> Iterator for UnrootedSimpleNodeIterator<'b, I>
where
    I: Fn(&Node, &'b NoGC) -> Option<UnrootedDom<'b, Node>>,
{
    type Item = UnrootedDom<'b, Node>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take();
        self.current = current
            .as_ref()
            .and_then(|c| (self.next_node)(c, self.no_gc));
        current
    }
}

pub(crate) struct TreeIterator {
    current: Option<DomRoot<Node>>,
    depth: usize,
    shadow_including: ShadowIncluding,
}

impl TreeIterator {
    pub(crate) fn new(root: &Node, shadow_including: ShadowIncluding) -> TreeIterator {
        TreeIterator {
            current: Some(DomRoot::from_ref(root)),
            depth: 0,
            shadow_including,
        }
    }

    pub(crate) fn next_skipping_children(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;

        self.next_skipping_children_impl(current)
    }

    fn next_skipping_children_impl(&mut self, current: DomRoot<Node>) -> Option<DomRoot<Node>> {
        let iter = current.inclusive_ancestors(self.shadow_including);

        for ancestor in iter {
            if self.depth == 0 {
                break;
            }
            if let Some(next_sibling) = ancestor.GetNextSibling() {
                self.current = Some(next_sibling);
                return Some(current);
            }
            if let Some(shadow_root) = ancestor.downcast::<ShadowRoot>() {
                // Shadow roots don't have sibling, so after we're done traversing
                // one we jump to the first child of the host
                if let Some(child) = shadow_root.Host().upcast::<Node>().GetFirstChild() {
                    self.current = Some(child);
                    return Some(current);
                }
            }
            self.depth -= 1;
        }
        debug_assert_eq!(self.depth, 0);
        self.current = None;
        Some(current)
    }

    pub(crate) fn peek(&self) -> Option<&DomRoot<Node>> {
        self.current.as_ref()
    }
}

impl Iterator for TreeIterator {
    type Item = DomRoot<Node>;

    /// <https://dom.spec.whatwg.org/#concept-tree-order>
    /// <https://dom.spec.whatwg.org/#concept-shadow-including-tree-order>
    fn next(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;

        // Handle a potential shadow root on the element
        if let Some(element) = current.downcast::<Element>() &&
            let Some(shadow_root) = element.shadow_root() &&
            self.shadow_including == ShadowIncluding::Yes
        {
            self.current = Some(DomRoot::from_ref(shadow_root.upcast::<Node>()));
            self.depth += 1;
            return Some(current);
        }

        if let Some(first_child) = current.GetFirstChild() {
            self.current = Some(first_child);
            self.depth += 1;
            return Some(current);
        };

        self.next_skipping_children_impl(current)
    }
}

/// An efficient TreeIterator because it skips rooting if there are no GC pauses.
///
/// Use this if you have a `&JSContext` or `NoGC`.
///
/// Normally we need to root every `Node` we come across as we do not know if we will have a GC pause.
/// This does not root the required children. Taking a `&NoGC` enforces that there is no `&mut JSContext`
/// while this iterator is alive.
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
pub(crate) struct UnrootedTreeIterator<'b> {
    current: Option<UnrootedDom<'b, Node>>,
    depth: usize,
    shadow_including: ShadowIncluding,
    /// This is unused and only used for lifetime guarantee of NoGC
    no_gc: &'b NoGC,
}

impl<'b> UnrootedTreeIterator<'b> {
    pub(crate) fn new(root: &Node, shadow_including: ShadowIncluding, no_gc: &'b NoGC) -> Self {
        Self {
            current: Some(UnrootedDom::from_dom(Dom::from_ref(root), no_gc)),
            depth: 0,
            shadow_including,
            no_gc,
        }
    }

    pub(crate) fn next_skipping_children(&mut self) -> Option<UnrootedDom<'b, Node>> {
        let current = self.current.take()?;

        let iter = current.inclusive_ancestors_unrooted(self.no_gc, self.shadow_including);

        for ancestor in iter {
            if self.depth == 0 {
                break;
            }

            let next_sibling_option = ancestor.get_next_sibling_unrooted(self.no_gc);

            if let Some(next_sibling) = next_sibling_option {
                self.current = Some(next_sibling);
                return Some(current);
            }

            if let Some(shadow_root) = ancestor.downcast::<ShadowRoot>() {
                // Shadow roots don't have sibling, so after we're done traversing
                // one we jump to the first child of the host
                let child_option = shadow_root
                    .host_unrooted(self.no_gc)
                    .upcast::<Node>()
                    .get_first_child_unrooted(self.no_gc);

                if let Some(child) = child_option {
                    self.current = Some(child);
                    return Some(current);
                }
            }
            self.depth -= 1;
        }
        debug_assert_eq!(self.depth, 0);
        self.current = None;
        Some(current)
    }
}

impl<'b> Iterator for UnrootedTreeIterator<'b> {
    type Item = UnrootedDom<'b, Node>;

    /// <https://dom.spec.whatwg.org/#concept-tree-order>
    /// <https://dom.spec.whatwg.org/#concept-shadow-including-tree-order>
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;

        // Handle a potential shadow root on the element
        if let Some(element) = current.downcast::<Element>() &&
            let Some(shadow_root) = element.shadow_root_unrooted(self.no_gc) &&
            self.shadow_including == ShadowIncluding::Yes
        {
            self.current = Some(UnrootedDom::upcast(shadow_root));
            self.depth += 1;
            return Some(current);
        }

        let first_child_option = current.get_first_child_unrooted(self.no_gc);
        if let Some(first_child) = first_child_option {
            self.current = Some(first_child);
            self.depth += 1;
            return Some(current);
        };

        // Restore `self.current` emptied by `.take()`
        self.current = Some(current);
        self.next_skipping_children()
    }
}

/// An `Item` in an traversal that is both pre-order and post-order. Each iteration
/// of the traversal is either an record of entering a node or a leaving a node during
/// the course of traversal.
#[derive(Clone)]
pub(crate) enum PrePostIteration<T: Clone> {
    /// The traversal encountered this node for the first time. This happens before
    /// traversing the node's descendants.
    Enter(T),
    /// The traversal is leaving this node. This happens after traversing the node's
    /// descendants.
    Leave(T),
}

/// A traversal of a [`Document`]'s [flat tree]. This is both a pre-order and post-order
/// unrooted traversal. This means that an item is returned both when encountering a node
/// for the first time and when leaving a node. In addition, no garbage collection can
/// happen while iterating, allowing returning unrooted values for performance reasons.
///
/// [flat tree]: https://drafts.csswg.org/css-shadow-1/#flat-tree
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
pub(crate) struct UnrootedFollowingFlatTreeNodesTraversal<'no_gc> {
    start: UnrootedDom<'no_gc, Node>,
    previously_returned_item: Option<PrePostIteration<UnrootedDom<'no_gc, Node>>>,
    no_gc: &'no_gc NoGC,
}

impl<'no_gc> UnrootedFollowingFlatTreeNodesTraversal<'no_gc> {
    pub(crate) fn new(root: &Node, no_gc: &'no_gc NoGC) -> Self {
        Self {
            start: UnrootedDom::from_dom(Dom::from_ref(root), no_gc),
            previously_returned_item: None,
            no_gc,
        }
    }

    pub(crate) fn next_skipping_subtree(
        &mut self,
    ) -> Option<PrePostIteration<UnrootedDom<'no_gc, Node>>> {
        let next = self.find_next_skipping_subtree()?;
        self.previously_returned_item = Some(next);
        self.previously_returned_item.clone()
    }

    fn find_next_skipping_subtree(
        &mut self,
    ) -> Option<PrePostIteration<UnrootedDom<'no_gc, Node>>> {
        match &self.previously_returned_item {
            None => Some(PrePostIteration::Leave(self.start.clone())),
            Some(PrePostIteration::Enter(previous)) => {
                Some(PrePostIteration::Leave(previous.clone()))
            },
            Some(PrePostIteration::Leave(previous)) => {
                Self::find_next_after_post(self.no_gc, previous)
            },
        }
    }

    fn find_next_after_post(
        no_gc: &'no_gc NoGC,
        previous: &UnrootedDom<'no_gc, Node>,
    ) -> Option<PrePostIteration<UnrootedDom<'no_gc, Node>>> {
        if let Some(next_sibling) = previous.next_flat_tree_sibling_unrooted(no_gc) {
            return Some(PrePostIteration::Enter(next_sibling));
        }
        match previous.parent_in_flat_tree() {
            FlatTreeParent::Parent(parent_node) => {
                let parent_node = UnrootedDom::from_dom(parent_node.as_traced(), no_gc);
                Some(PrePostIteration::Leave(parent_node))
            },
            FlatTreeParent::NotInFlatTree => None,
            FlatTreeParent::RootNode => None,
        }
    }

    fn find_next(&mut self) -> Option<PrePostIteration<UnrootedDom<'no_gc, Node>>> {
        match &self.previously_returned_item {
            None => Some(PrePostIteration::Enter(self.start.clone())),
            Some(PrePostIteration::Enter(previous)) => {
                if let Some(first_child) = previous.first_flat_tree_child_unrooted(self.no_gc) {
                    return Some(PrePostIteration::Enter(first_child));
                }
                Some(PrePostIteration::Leave(previous.clone()))
            },
            Some(PrePostIteration::Leave(previous)) => {
                Self::find_next_after_post(self.no_gc, previous)
            },
        }
    }
}

impl<'no_gc> Iterator for UnrootedFollowingFlatTreeNodesTraversal<'no_gc> {
    type Item = PrePostIteration<UnrootedDom<'no_gc, Node>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.find_next()?;
        self.previously_returned_item = Some(next);
        self.previously_returned_item.clone()
    }
}
