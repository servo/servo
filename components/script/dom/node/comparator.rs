/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp::Ordering;

use bitflags::bitflags;
use js::context::NoGC;
use script_bindings::dom::UnrootedDom;

use crate::dom::Node;
use crate::dom::traversal::NoGcTraversal;

#[derive(Clone, Copy)]
pub(crate) struct DomPositionContainment(u8);

bitflags! {
    impl DomPositionContainment: u8 {
        const AContainsB = 1 << 0;
        const BContainsA = 1 << 1;
    }
}

pub(crate) fn compare_dom_positions<Traversal: NoGcTraversal>(
    no_gc: &NoGC,
    container_a: &Node,
    offset_a: u32,
    container_b: &Node,
    offset_b: u32,
) -> (Option<Ordering>, DomPositionContainment) {
    if container_a == container_b {
        return (
            Some(offset_a.cmp(&offset_b)),
            DomPositionContainment::empty(),
        );
    }

    if let Some(child_of_a) = find_child_in_ancestors::<Traversal>(no_gc, container_b, container_a)
    {
        let ordering =
            match compare_offset_and_node_in_same_parent::<Traversal>(no_gc, offset_a, &child_of_a)
            {
                Ordering::Equal => Ordering::Less,
                ordering => ordering,
            };
        return (Some(ordering), DomPositionContainment::AContainsB);
    }

    if let Some(child_of_b) = find_child_in_ancestors::<Traversal>(no_gc, container_a, container_b)
    {
        let ordering =
            match compare_offset_and_node_in_same_parent::<Traversal>(no_gc, offset_b, &child_of_b)
            {
                Ordering::Equal => Ordering::Greater,
                ordering => ordering.reverse(),
            };
        return (Some(ordering), DomPositionContainment::BContainsA);
    }

    let Some((least_common_ancestor_child_of_a, least_common_ancestor_child_of_b)) =
        least_common_ancestor_children::<Traversal>(no_gc, container_a, container_b)
    else {
        return (None, DomPositionContainment::empty());
    };

    let ordering = compare_nodes_in_same_parent::<Traversal>(
        no_gc,
        &least_common_ancestor_child_of_a,
        &least_common_ancestor_child_of_b,
    );
    (Some(ordering), DomPositionContainment::empty())
}

/// If `possible_ancestor` is an ancestor of `possible_descendant` return the
/// child of `possible_ancestor` that is an ancestor of `possible_descendant` or
/// is `possible_descendant` itself.
fn find_child_in_ancestors<'a, Traversal: NoGcTraversal>(
    no_gc: &'a NoGC,
    possible_descendant: &Node,
    possible_ancestor: &Node,
) -> Option<UnrootedDom<'a, Node>> {
    let mut child = UnrootedDom::from_ref(possible_descendant, no_gc);
    let mut maybe_ancestor = Traversal::parent(no_gc, possible_descendant);
    while let Some(ancestor) = maybe_ancestor {
        if **ancestor == *possible_ancestor {
            return Some(child);
        }

        maybe_ancestor = Traversal::parent(no_gc, &ancestor);
        child = ancestor;
    }
    None
}

/// Compare an offset in a parent node with a child node in that same parent node.
fn compare_offset_and_node_in_same_parent<Traversal: NoGcTraversal>(
    no_gc: &NoGC,
    offset_a: u32,
    node_b: &Node,
) -> Ordering {
    let parent = Traversal::parent(no_gc, node_b).expect("Node should always have a parent");
    for (current_offset, child) in Traversal::children(no_gc, &parent).enumerate() {
        if current_offset == offset_a as usize && **child == *node_b {
            return Ordering::Equal;
        }
        if current_offset == offset_a as usize {
            return Ordering::Less;
        }
        if **child == *node_b {
            return Ordering::Greater;
        }
    }
    unreachable!("A node should always be a child of its parent.");
}

/// Compare two nodes that are both children of the same parent node.
fn compare_nodes_in_same_parent<Traversal: NoGcTraversal>(
    no_gc: &NoGC,
    node_a: &Node,
    node_b: &Node,
) -> Ordering {
    if node_a == node_b {
        return Ordering::Equal;
    }

    let parent = Traversal::parent(no_gc, node_a).expect("Node should always have a parent");
    for child in Traversal::children(no_gc, &parent) {
        if **child == *node_a {
            return Ordering::Less;
        }
        if **child == *node_b {
            return Ordering::Greater;
        }
    }
    unreachable!("A node should always be a child of its parent.");
}

/// When `node_a` and `node_b` share a least common ancestor, this function returns a
/// tuple containing the child of the least common ancestor that is an inclusive ancestor
/// of `node_a` and the child of the least common ancestor that is an inclusive ancestor
/// of `node_b`. If `node_a` and `node_b` do not have a least common ancestor, this
/// returns `None`.
///
/// Note: This function assumes that the least common inclusive ancestor is neither of the
/// nodes passed as arguments.
fn least_common_ancestor_children<'a, Traversal: NoGcTraversal>(
    no_gc: &'a NoGC,
    node_a: &Node,
    node_b: &Node,
) -> Option<(UnrootedDom<'a, Node>, UnrootedDom<'a, Node>)> {
    let mut depth_a = 0;
    let mut inclusive_ancestor = Some(UnrootedDom::from_ref(node_a, no_gc));
    while let Some(ancestor) = inclusive_ancestor {
        debug_assert!(**ancestor != *node_b);
        inclusive_ancestor = Traversal::parent(no_gc, &ancestor);
        depth_a += 1;
    }

    let mut depth_b = 0;
    let mut inclusive_ancestor = Some(UnrootedDom::from_ref(node_b, no_gc));
    while let Some(ancestor) = inclusive_ancestor {
        debug_assert!(**ancestor != *node_a);
        inclusive_ancestor = Traversal::parent(no_gc, &ancestor);
        depth_b += 1;
    }

    let mut inclusive_ancestor_of_a = Some(UnrootedDom::from_ref(node_a, no_gc));
    let mut inclusive_ancestor_of_b = Some(UnrootedDom::from_ref(node_b, no_gc));

    while depth_a > depth_b {
        let ancestor = inclusive_ancestor_of_a.expect("Guaranteed by depth");
        inclusive_ancestor_of_a = Traversal::parent(no_gc, &ancestor);
        depth_a -= 1;
    }

    while depth_b > depth_a {
        let ancestor = inclusive_ancestor_of_b.expect("Guaranteed by depth");
        inclusive_ancestor_of_b = Traversal::parent(no_gc, &ancestor);
        depth_b -= 1;
    }

    let mut candidate_child_a = inclusive_ancestor_of_a.expect("Should always have a candidate");
    let mut candidate_child_b = inclusive_ancestor_of_b.expect("Should always have a candidate");
    let mut inclusive_ancestor_of_a = Traversal::parent(no_gc, &candidate_child_a);
    let mut inclusive_ancestor_of_b = Traversal::parent(no_gc, &candidate_child_b);

    while let Some(ancestor_of_a) = inclusive_ancestor_of_a &&
        let Some(ancestor_of_b) = inclusive_ancestor_of_b
    {
        if ancestor_of_a == ancestor_of_b {
            return Some((candidate_child_a, candidate_child_b));
        }

        inclusive_ancestor_of_a = Traversal::parent(no_gc, &ancestor_of_a);
        inclusive_ancestor_of_b = Traversal::parent(no_gc, &ancestor_of_b);
        candidate_child_a = ancestor_of_a;
        candidate_child_b = ancestor_of_b;
    }

    None
}
