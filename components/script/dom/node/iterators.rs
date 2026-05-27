/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use js::context::NoGC;

use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRoot_Binding::ShadowRootMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot, UnrootedDom};
use crate::dom::element::Element;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::{Node, ShadowIncluding};

pub(crate) struct FollowingNodeIterator {
    pub(crate) current: Option<DomRoot<Node>>,
    pub(crate) root: DomRoot<Node>,
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

        for ancestor in current.inclusive_ancestors(ShadowIncluding::No) {
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

pub(crate) struct PrecedingNodeIterator {
    pub(crate) current: Option<DomRoot<Node>>,
    pub(crate) root: DomRoot<Node>,
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

pub(crate) struct SimpleNodeIterator<I>
where
    I: Fn(&Node) -> Option<DomRoot<Node>>,
{
    pub(crate) current: Option<DomRoot<Node>>,
    pub(crate) next_node: I,
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
pub(crate) struct UnrootedSimpleNodeIterator<'a, 'b, I>
where
    I: Fn(&Node, &'b NoGC) -> Option<UnrootedDom<'b, Node>>,
{
    pub(crate) current: Option<UnrootedDom<'b, Node>>,
    pub(crate) next_node: I,
    /// This is unused and only used for lifetime guarantee of NoGC
    pub(crate) no_gc: &'b NoGC,
    pub(crate) phantom: PhantomData<&'a Node>,
}

impl<'a, 'b, I> Iterator for UnrootedSimpleNodeIterator<'a, 'b, I>
where
    'b: 'a,
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
    pub(crate) current: Option<DomRoot<Node>>,
    pub(crate) depth: usize,
    pub(crate) shadow_including: ShadowIncluding,
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
pub(crate) struct UnrootedTreeIterator<'a, 'b> {
    current: Option<UnrootedDom<'b, Node>>,
    depth: usize,
    shadow_including: ShadowIncluding,
    /// This is unused and only used for lifetime guarantee of NoGC
    no_gc: &'b NoGC,
    phantom: PhantomData<&'a Node>,
}

impl<'a, 'b> UnrootedTreeIterator<'a, 'b>
where
    'b: 'a,
{
    pub(crate) fn new(
        root: &'a Node,
        shadow_including: ShadowIncluding,
        no_gc: &'b NoGC,
    ) -> UnrootedTreeIterator<'a, 'b> {
        UnrootedTreeIterator {
            current: Some(UnrootedDom::from_dom(Dom::from_ref(root), no_gc)),
            depth: 0,
            shadow_including,
            no_gc,
            phantom: PhantomData,
        }
    }

    pub(crate) fn next_skipping_children(&mut self) -> Option<UnrootedDom<'b, Node>> {
        let current = self.current.take()?;

        let iter = current.inclusive_ancestors(self.shadow_including);

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
                    .Host()
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

impl<'a, 'b> Iterator for UnrootedTreeIterator<'a, 'b>
where
    'b: 'a,
{
    type Item = UnrootedDom<'b, Node>;

    /// <https://dom.spec.whatwg.org/#concept-tree-order>
    /// <https://dom.spec.whatwg.org/#concept-shadow-including-tree-order>
    fn next(&mut self) -> Option<UnrootedDom<'b, Node>> {
        let current = self.current.take()?;

        // Handle a potential shadow root on the element
        if let Some(element) = current.downcast::<Element>() &&
            let Some(shadow_root) = element.shadow_root() &&
            self.shadow_including == ShadowIncluding::Yes
        {
            self.current = Some(UnrootedDom::from_dom(
                Dom::from_ref(shadow_root.upcast::<Node>()),
                self.no_gc,
            ));
            self.depth += 1;
            return Some(current);
        }

        let first_child_option = current.get_first_child_unrooted(self.no_gc);
        if let Some(first_child) = first_child_option {
            self.current = Some(first_child);
            self.depth += 1;
            return Some(current);
        };

        // current is empty.
        let _ = self.current.insert(current);
        self.next_skipping_children()
    }
}
