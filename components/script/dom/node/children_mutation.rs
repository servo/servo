/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::slice;

use js::context::NoGC;

use crate::dom::Node;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::element::Element;

pub(crate) enum ChildrenMutation<'a> {
    Append {
        prev: &'a Node,
        added: &'a [&'a Node],
    },
    Insert {
        prev: &'a Node,
        added: &'a [&'a Node],
        next: &'a Node,
    },
    Prepend {
        added: &'a [&'a Node],
        next: &'a Node,
    },
    Replace {
        prev: Option<&'a Node>,
        removed: &'a Node,
        added: &'a [&'a Node],
        next: Option<&'a Node>,
    },
    ReplaceAll {
        removed: &'a [&'a Node],
        added: &'a [&'a Node],
    },
    /// Mutation for when a Text node's data is modified.
    /// This doesn't change the structure of the list, which is what the other
    /// variants' fields are stored for at the moment, so this can just have no
    /// fields.
    ChangeText,
}

impl<'a> ChildrenMutation<'a> {
    pub(super) fn insert(
        prev: Option<&'a Node>,
        added: &'a [&'a Node],
        next: Option<&'a Node>,
    ) -> ChildrenMutation<'a> {
        match (prev, next) {
            (None, None) => ChildrenMutation::ReplaceAll {
                removed: &[],
                added,
            },
            (Some(prev), None) => ChildrenMutation::Append { prev, added },
            (None, Some(next)) => ChildrenMutation::Prepend { added, next },
            (Some(prev), Some(next)) => ChildrenMutation::Insert { prev, added, next },
        }
    }

    pub(super) fn replace(
        prev: Option<&'a Node>,
        removed: &'a Option<&'a Node>,
        added: &'a [&'a Node],
        next: Option<&'a Node>,
    ) -> ChildrenMutation<'a> {
        if let Some(ref removed) = *removed {
            if let (None, None) = (prev, next) {
                ChildrenMutation::ReplaceAll {
                    removed: slice::from_ref(removed),
                    added,
                }
            } else {
                ChildrenMutation::Replace {
                    prev,
                    removed,
                    added,
                    next,
                }
            }
        } else {
            ChildrenMutation::insert(prev, added, next)
        }
    }

    pub(super) fn replace_all(
        removed: &'a [&'a Node],
        added: &'a [&'a Node],
    ) -> ChildrenMutation<'a> {
        ChildrenMutation::ReplaceAll { removed, added }
    }

    /// Get the child that follows the added or removed children.
    /// Currently only used when this mutation might force us to
    /// restyle later children (see HAS_SLOW_SELECTOR_LATER_SIBLINGS and
    /// Element's implementation of VirtualMethods::children_changed).
    pub(crate) fn next_child(&self) -> Option<&Node> {
        match *self {
            ChildrenMutation::Append { .. } => None,
            ChildrenMutation::Insert { next, .. } => Some(next),
            ChildrenMutation::Prepend { next, .. } => Some(next),
            ChildrenMutation::Replace { next, .. } => next,
            ChildrenMutation::ReplaceAll { .. } => None,
            ChildrenMutation::ChangeText => None,
        }
    }

    /// If nodes were added or removed at the start or end of a container, return any
    /// previously-existing child whose ":first-child" or ":last-child" status *may* have changed.
    ///
    /// NOTE: This does not check whether the inserted/removed nodes were elements, so in some
    /// cases it will return a false positive.  This doesn't matter for correctness, because at
    /// worst the returned element will be restyled unnecessarily.
    pub(crate) fn modified_edge_element(&self, no_gc: &NoGC) -> Option<DomRoot<Node>> {
        match *self {
            // Add/remove at start of container: Return the first following element.
            ChildrenMutation::Prepend { next, .. } |
            ChildrenMutation::Replace {
                prev: None,
                next: Some(next),
                ..
            } => next
                .inclusively_following_siblings_unrooted(no_gc)
                .find(|node| node.is::<Element>())
                .map(|node| node.as_rooted()),
            // Add/remove at end of container: Return the last preceding element.
            ChildrenMutation::Append { prev, .. } |
            ChildrenMutation::Replace {
                prev: Some(prev),
                next: None,
                ..
            } => prev
                .inclusively_preceding_siblings_unrooted(no_gc)
                .find(|node| node.is::<Element>())
                .map(|node| node.as_rooted()),
            // Insert or replace in the middle:
            ChildrenMutation::Insert { prev, next, .. } |
            ChildrenMutation::Replace {
                prev: Some(prev),
                next: Some(next),
                ..
            } => {
                if prev
                    .inclusively_preceding_siblings_unrooted(no_gc)
                    .all(|node| !node.is::<Element>())
                {
                    // Before the first element: Return the first following element.
                    next.inclusively_following_siblings_unrooted(no_gc)
                        .find(|node| node.is::<Element>())
                        .map(|node| node.as_rooted())
                } else if next
                    .inclusively_following_siblings_unrooted(no_gc)
                    .all(|node| !node.is::<Element>())
                {
                    // After the last element: Return the last preceding element.
                    prev.inclusively_preceding_siblings_unrooted(no_gc)
                        .find(|node| node.is::<Element>())
                        .map(|node| node.as_rooted())
                } else {
                    None
                }
            },

            ChildrenMutation::Replace {
                prev: None,
                next: None,
                ..
            } => unreachable!(),
            ChildrenMutation::ReplaceAll { .. } => None,
            ChildrenMutation::ChangeText => None,
        }
    }
}
