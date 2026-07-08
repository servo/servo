/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};

use deny_public_fields::DenyPublicFields;
use dom_struct::dom_struct;
use script_bindings::reflector::Reflector;

use crate::dom::bindings::codegen::Bindings::AbstractRangeBinding::AbstractRangeMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::{NodeConstants, NodeMethods};
use crate::dom::bindings::root::{DomRoot, MutDom};
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::Node;

#[dom_struct]
pub(crate) struct AbstractRange {
    reflector_: Reflector,
    start: BoundaryPoint,
    end: BoundaryPoint,
}

impl AbstractRange {
    pub(crate) fn new_inherited(
        start_container: &Node,
        start_offset: u32,
        end_container: &Node,
        end_offset: u32,
    ) -> AbstractRange {
        AbstractRange {
            reflector_: Reflector::new(),
            start: BoundaryPoint::new(start_container, start_offset),
            end: BoundaryPoint::new(end_container, end_offset),
        }
    }

    pub(crate) fn start(&self) -> &BoundaryPoint {
        &self.start
    }

    pub(crate) fn end(&self) -> &BoundaryPoint {
        &self.end
    }
}

impl AbstractRangeMethods<crate::DomTypeHolder> for AbstractRange {
    /// <https://dom.spec.whatwg.org/#dom-range-startcontainer>
    fn StartContainer(&self) -> DomRoot<Node> {
        self.start.node.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-range-startoffset>
    fn StartOffset(&self) -> u32 {
        self.start.offset.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-range-endcontainer>
    fn EndContainer(&self) -> DomRoot<Node> {
        self.end.node.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-range-endoffset>
    fn EndOffset(&self) -> u32 {
        self.end.offset.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-range-collapsed>
    fn Collapsed(&self) -> bool {
        // > The collapsed getter steps are to return true if this is collapsed; otherwise false.
        self.start == self.end
    }
}

/// <https://dom.spec.whatwg.org/#concept-range-bp>
#[derive(DenyPublicFields, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct BoundaryPoint {
    /// <https://dom.spec.whatwg.org/#boundary-point-node>
    node: MutDom<Node>,
    /// <https://dom.spec.whatwg.org/#concept-range-bp-offset>
    offset: Cell<u32>,
}

impl BoundaryPoint {
    pub(crate) fn new(node: &Node, offset: u32) -> BoundaryPoint {
        debug_assert!(!node.is_doctype());
        BoundaryPoint {
            node: MutDom::new(node),
            offset: Cell::new(offset),
        }
    }

    pub(crate) fn set(&self, node: &Node, offset: u32) {
        self.node.set(node);
        self.set_offset(offset);
    }

    pub(crate) fn get_offset(&self) -> u32 {
        self.offset.get()
    }

    pub(crate) fn set_offset(&self, offset: u32) {
        self.offset.set(offset);
    }

    pub(crate) fn node(&self) -> &MutDom<Node> {
        &self.node
    }
}

impl PartialOrd for BoundaryPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(bp_position(
            &self.node.get(),
            self.offset.get(),
            &other.node.get(),
            other.offset.get(),
        ))
    }
}

/// <https://dom.spec.whatwg.org/#range-collapsed>
impl PartialEq for BoundaryPoint {
    fn eq(&self, other: &Self) -> bool {
        // > A range is collapsed if its start node is its end node and its start offset is its end offset.
        self.node.get() == other.node.get() && self.offset.get() == other.offset.get()
    }
}

/// <https://dom.spec.whatwg.org/#concept-range-bp-position>
pub(crate) fn bp_position(a_node: &Node, a_offset: u32, b_node: &Node, b_offset: u32) -> Ordering {
    // Step 1: Assert: nodeA and nodeB have the same root.
    debug_assert!(
        a_node.GetRootNode(&Default::default()) == b_node.GetRootNode(&Default::default())
    );

    // Step 2: If nodeA is nodeB, then return equal if offsetA is offsetB, before if
    // offsetA is less than offsetB, and after if offsetA is greater than offsetB.
    if a_node == b_node {
        return a_offset.cmp(&b_offset);
    }

    let position = b_node.CompareDocumentPosition(a_node);
    assert!(
        position & NodeConstants::DOCUMENT_POSITION_DISCONNECTED == 0,
        "Nodes should be in the same tree"
    );
    if position & NodeConstants::DOCUMENT_POSITION_FOLLOWING != 0 {
        // Step 3: If nodeA is following nodeB, then if the position of (nodeB, offsetB)
        // relative to (nodeA, offsetA) is before, return after, and if it is after,
        // return before.
        return match bp_position(b_node, b_offset, a_node, a_offset) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => unreachable!("Should be impossible due to Step 2."),
        };
    } else if position & NodeConstants::DOCUMENT_POSITION_CONTAINS != 0 {
        // Step 4: If nodeA is an ancestor of nodeB:
        // Step 4.1: Let child be nodeB.
        // Step 4.2: While child is not a child of nodeA, set child to its parent.
        let mut b_ancestors = b_node.inclusive_ancestors(ShadowIncluding::No);
        let child = b_ancestors
            .find(|child| &*child.GetParentNode().unwrap() == a_node)
            .unwrap();

        // Step 4.3: If child’s index is less than offsetA, then return after.
        if child.index() < a_offset {
            return Ordering::Greater;
        }
    }

    // Step 5: Return before.
    Ordering::Less
}
