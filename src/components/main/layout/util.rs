/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use layout::box_::Box;
use layout::construct::{ConstructionResult, NoConstructionResult};
use layout::parallel::DomParallelInfo;
use layout::wrapper::{LayoutNode, TLayoutNode, ThreadSafeLayoutNode};

use script::dom::bindings::js::JS;
use script::dom::bindings::utils::Reflectable;
use script::dom::node::Node;
use script::layout_interface::{LayoutChan, UntrustedNodeAddress, TrustedNodeAddress};
use servo_util::range::Range;
use std::cast;
use std::cell::{Ref, RefMut};
use std::iter::Enumerate;
use std::libc::uintptr_t;
use std::vec::Items;
use style::ComputedValues;
use sync::Arc;

/// A range of nodes.
pub struct NodeRange {
    node: OpaqueNode,
    range: Range,
}

impl NodeRange {
    pub fn new(node: OpaqueNode, range: &Range) -> NodeRange {
        NodeRange {
            node: node,
            range: (*range).clone()
        }
    }
}

pub struct ElementMapping {
    priv entries: ~[NodeRange],
}

impl ElementMapping {
    pub fn new() -> ElementMapping {
        ElementMapping {
            entries: ~[],
        }
    }

    pub fn add_mapping(&mut self, node: OpaqueNode, range: &Range) {
        self.entries.push(NodeRange::new(node, range))
    }

    pub fn each(&self, callback: |nr: &NodeRange| -> bool) -> bool {
        for nr in self.entries.iter() {
            if !callback(nr) {
                break
            }
        }
        true
    }

    pub fn eachi<'a>(&'a self) -> Enumerate<Items<'a, NodeRange>> {
        self.entries.iter().enumerate()
    }

    pub fn repair_for_box_changes(&mut self, old_boxes: &[Box], new_boxes: &[Box]) {
        let entries = &mut self.entries;

        debug!("--- Old boxes: ---");
        for (i, box_) in old_boxes.iter().enumerate() {
            debug!("{:u} --> {:s}", i, box_.debug_str());
        }
        debug!("------------------");

        debug!("--- New boxes: ---");
        for (i, box_) in new_boxes.iter().enumerate() {
            debug!("{:u} --> {:s}", i, box_.debug_str());
        }
        debug!("------------------");

        debug!("--- Elem ranges before repair: ---");
        for (i, nr) in entries.iter().enumerate() {
            debug!("{:u}: {} --> {:?}", i, nr.range, nr.node.id());
        }
        debug!("----------------------------------");

        let mut old_i = 0;
        let mut new_j = 0;

        struct WorkItem {
            begin_idx: uint,
            entry_idx: uint,
        };
        let mut repair_stack : ~[WorkItem] = ~[];

            // index into entries
            let mut entries_k = 0;

            while old_i < old_boxes.len() {
                debug!("repair_for_box_changes: Considering old box {:u}", old_i);
                // possibly push several items
                while entries_k < entries.len() && old_i == entries[entries_k].range.begin() {
                    let item = WorkItem {begin_idx: new_j, entry_idx: entries_k};
                    debug!("repair_for_box_changes: Push work item for elem {:u}: {:?}", entries_k, item);
                    repair_stack.push(item);
                    entries_k += 1;
                }
                while new_j < new_boxes.len() && old_boxes[old_i].node != new_boxes[new_j].node {
                    debug!("repair_for_box_changes: Slide through new box {:u}", new_j);
                    new_j += 1;
                }

                old_i += 1;

                // possibly pop several items
                while repair_stack.len() > 0 && old_i == entries[repair_stack.last().get_ref().entry_idx].range.end() {
                    let item = repair_stack.pop().unwrap();
                    debug!("repair_for_box_changes: Set range for {:u} to {}",
                           item.entry_idx, Range::new(item.begin_idx, new_j - item.begin_idx));
                    entries[item.entry_idx].range = Range::new(item.begin_idx, new_j - item.begin_idx);
                }
            }
        debug!("--- Elem ranges after repair: ---");
        for (i, nr) in entries.iter().enumerate() {
            debug!("{:u}: {} --> {:?}", i, nr.range, nr.node.id());
        }
        debug!("----------------------------------");
    }
}

/// Data that layout associates with a node.
pub struct PrivateLayoutData {
    /// The results of CSS styling for this node.
    style: Option<Arc<ComputedValues>>,

    /// The results of CSS styling for this node's `before` pseudo-element, if any.
    before_style: Option<Arc<ComputedValues>>,

    /// The results of CSS styling for this node's `after` pseudo-element, if any.
    after_style: Option<Arc<ComputedValues>>,

    /// Description of how to account for recent style changes.
    restyle_damage: Option<int>,

    /// The current results of flow construction for this node. This is either a flow or a
    /// `ConstructionItem`. See comments in `construct.rs` for more details.
    flow_construction_result: ConstructionResult,

    before_flow_construction_result: ConstructionResult,

    after_flow_construction_result: ConstructionResult, 

    /// Information needed during parallel traversals.
    parallel: DomParallelInfo,
}

impl PrivateLayoutData {
    /// Creates new layout data.
    pub fn new() -> PrivateLayoutData {
        PrivateLayoutData {
            before_style: None,
            style: None,
            after_style: None,
            restyle_damage: None,
            flow_construction_result: NoConstructionResult,
            before_flow_construction_result: NoConstructionResult,
            after_flow_construction_result: NoConstructionResult,
            parallel: DomParallelInfo::new(),
        }
    }
}

pub struct LayoutDataWrapper {
    chan: Option<LayoutChan>,
    data: ~PrivateLayoutData,
}

/// A trait that allows access to the layout data of a DOM node.
pub trait LayoutDataAccess {
    /// Borrows the layout data without checks.
    unsafe fn borrow_layout_data_unchecked(&self) -> *Option<LayoutDataWrapper>;
    /// Borrows the layout data immutably. Fails on a conflicting borrow.
    fn borrow_layout_data<'a>(&'a self) -> Ref<'a,Option<LayoutDataWrapper>>;
    /// Borrows the layout data mutably. Fails on a conflicting borrow.
    fn mutate_layout_data<'a>(&'a self) -> RefMut<'a,Option<LayoutDataWrapper>>;
}

impl<'ln> LayoutDataAccess for LayoutNode<'ln> {
    #[inline(always)]
    unsafe fn borrow_layout_data_unchecked(&self) -> *Option<LayoutDataWrapper> {
        cast::transmute(self.get().layout_data.borrow_unchecked())
    }

    #[inline(always)]
    fn borrow_layout_data<'a>(&'a self) -> Ref<'a,Option<LayoutDataWrapper>> {
        unsafe {
            cast::transmute(self.get().layout_data.borrow())
        }
    }

    #[inline(always)]
    fn mutate_layout_data<'a>(&'a self) -> RefMut<'a,Option<LayoutDataWrapper>> {
        unsafe {
            cast::transmute(self.get().layout_data.borrow_mut())
        }
    }
}

/// An opaque handle to a node. The only safe operation that can be performed on this node is to
/// compare it to another opaque handle or to another node.
///
/// Because the script task's GC does not trace layout, node data cannot be safely stored in layout
/// data structures. Also, layout code tends to be faster when the DOM is not being accessed, for
/// locality reasons. Using `OpaqueNode` enforces this invariant.
#[deriving(Clone, Eq)]
pub struct OpaqueNode(uintptr_t);

impl OpaqueNode {
    /// Converts a DOM node (layout view) to an `OpaqueNode`.
    pub fn from_layout_node(node: &LayoutNode) -> OpaqueNode {
        unsafe {
            OpaqueNode::from_jsmanaged(node.get_jsmanaged())
        }
    }

    /// Converts a thread-safe DOM node (layout view) to an `OpaqueNode`.
    pub fn from_thread_safe_layout_node(node: &ThreadSafeLayoutNode) -> OpaqueNode {
        unsafe {
            let abstract_node = node.get_jsmanaged();
            let ptr: uintptr_t = cast::transmute(abstract_node.reflector().get_jsobject());
            OpaqueNode(ptr)
        }
    }

    /// Converts a DOM node (script view) to an `OpaqueNode`.
    pub fn from_script_node(node: TrustedNodeAddress) -> OpaqueNode {
        unsafe {
            OpaqueNode::from_jsmanaged(&JS::from_trusted_node_address(node))
        }
    }

    /// Converts a DOM node to an `OpaqueNode'.
    fn from_jsmanaged(node: &JS<Node>) -> OpaqueNode {
        unsafe {
            let ptr: uintptr_t = cast::transmute(node.reflector().get_jsobject());
            OpaqueNode(ptr)
        }
    }

    /// Converts this node to an `UntrustedNodeAddress`. An `UntrustedNodeAddress` is just the type
    /// of node that script expects to receive in a hit test.
    pub fn to_untrusted_node_address(&self) -> UntrustedNodeAddress {
        unsafe {
            let OpaqueNode(addr) = *self;
            let addr: UntrustedNodeAddress = cast::transmute(addr);
            addr
        }
    }

    /// Returns the address of this node, for debugging purposes.
    pub fn id(&self) -> uintptr_t {
        unsafe {
            cast::transmute_copy(self)
        }
    }
}

