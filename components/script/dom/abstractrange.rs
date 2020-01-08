/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::AbstractRangeBinding::{self, AbstractRangeMethods};
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeConstants;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{DomRoot, MutDom};
use crate::dom::document::Document;
use crate::dom::node::{Node, ShadowIncluding};
use dom_struct::dom_struct;
use std::cell::Cell;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};

#[dom_struct]
pub struct AbstractRange {
    reflector_: Reflector,
    start: BoundaryPoint,
    end: BoundaryPoint,
}

impl AbstractRange {
    pub fn new_inherited(
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

    pub fn new(
        document: &Document,
        start_container: &Node,
        start_offset: u32,
        end_container: &Node,
        end_offset: u32,
    ) -> DomRoot<AbstractRange> {
        let abstractrange = reflect_dom_object(
            Box::new(AbstractRange::new_inherited(
                start_container,
                start_offset,
                end_container,
                end_offset,
            )),
            document.window(),
            AbstractRangeBinding::Wrap,
        );
        abstractrange
    }

    pub fn start(&self) -> &BoundaryPoint {
        &self.start
    }

    pub fn end(&self) -> &BoundaryPoint {
        &self.end
    }
}

impl AbstractRangeMethods for AbstractRange {
    // https://dom.spec.whatwg.org/#dom-range-startcontainer
    fn StartContainer(&self) -> DomRoot<Node> {
        self.start.node.get()
    }

    // https://dom.spec.whatwg.org/#dom-range-startoffset
    fn StartOffset(&self) -> u32 {
        self.start.offset.get()
    }

    // https://dom.spec.whatwg.org/#dom-range-endcontainer
    fn EndContainer(&self) -> DomRoot<Node> {
        self.end.node.get()
    }

    // https://dom.spec.whatwg.org/#dom-range-endoffset
    fn EndOffset(&self) -> u32 {
        self.end.offset.get()
    }

    // https://dom.spec.whatwg.org/#dom-range-collapsed
    fn Collapsed(&self) -> bool {
        self.start == self.end
    }
}

#[derive(DenyPublicFields, JSTraceable, MallocSizeOf)]
#[unrooted_must_root_lint::must_root]
pub struct BoundaryPoint {
    node: MutDom<Node>,
    offset: Cell<u32>,
}

impl BoundaryPoint {
    fn new(node: &Node, offset: u32) -> BoundaryPoint {
        debug_assert!(!node.is_doctype());
        BoundaryPoint {
            node: MutDom::new(node),
            offset: Cell::new(offset),
        }
    }

    pub fn set(&self, node: &Node, offset: u32) {
        self.node.set(node);
        self.set_offset(offset);
    }

    pub fn set_offset(&self, offset: u32) {
        self.offset.set(offset);
    }

    pub fn node(&self) -> &MutDom<Node> {
        &self.node
    }
}

#[allow(unrooted_must_root)]
impl PartialOrd for BoundaryPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        bp_position(
            &self.node.get(),
            self.offset.get(),
            &other.node.get(),
            other.offset.get(),
        )
    }
}

#[allow(unrooted_must_root)]
impl PartialEq for BoundaryPoint {
    fn eq(&self, other: &Self) -> bool {
        self.node.get() == other.node.get() && self.offset.get() == other.offset.get()
    }
}

// https://dom.spec.whatwg.org/#concept-range-bp-position
pub fn bp_position(a_node: &Node, a_offset: u32, b_node: &Node, b_offset: u32) -> Option<Ordering> {
    if a_node as *const Node == b_node as *const Node {
        // Step 1.
        return Some(a_offset.cmp(&b_offset));
    }
    let position = b_node.CompareDocumentPosition(a_node);
    if position & NodeConstants::DOCUMENT_POSITION_DISCONNECTED != 0 {
        // No order is defined for nodes not in the same tree.
        None
    } else if position & NodeConstants::DOCUMENT_POSITION_FOLLOWING != 0 {
        // Step 2.
        match bp_position(b_node, b_offset, a_node, a_offset).unwrap() {
            Ordering::Less => Some(Ordering::Greater),
            Ordering::Greater => Some(Ordering::Less),
            Ordering::Equal => unreachable!(),
        }
    } else if position & NodeConstants::DOCUMENT_POSITION_CONTAINS != 0 {
        // Step 3-1, 3-2.
        let mut b_ancestors = b_node.inclusive_ancestors(ShadowIncluding::No);
        let child = b_ancestors
            .find(|child| &*child.GetParentNode().unwrap() == a_node)
            .unwrap();
        // Step 3-3.
        if child.index() < a_offset {
            Some(Ordering::Greater)
        } else {
            // Step 4.
            Some(Ordering::Less)
        }
    } else {
        // Step 4.
        Some(Ordering::Less)
    }
}
