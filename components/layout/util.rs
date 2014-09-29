/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use construct::{ConstructionResult, NoConstructionResult};
use incremental::RestyleDamage;
use parallel::DomParallelInfo;
use wrapper::{LayoutNode, TLayoutNode, ThreadSafeLayoutNode};

use gfx::display_list::OpaqueNode;
use gfx;
use libc::uintptr_t;
use script::dom::bindings::js::JS;
use script::dom::bindings::utils::Reflectable;
use script::dom::node::{Node, SharedLayoutData};
use script::layout_interface::{LayoutChan, UntrustedNodeAddress, TrustedNodeAddress};
use std::mem;
use std::cell::{Ref, RefMut};
use style::ComputedValues;
use style;
use sync::Arc;

/// Data that layout associates with a node.
pub struct PrivateLayoutData {
    /// The results of CSS styling for this node's `before` pseudo-element, if any.
    pub before_style: Option<Arc<ComputedValues>>,

    /// The results of CSS styling for this node's `after` pseudo-element, if any.
    pub after_style: Option<Arc<ComputedValues>>,

    /// Description of how to account for recent style changes.
    pub restyle_damage: Option<RestyleDamage>,

    /// The current results of flow construction for this node. This is either a flow or a
    /// `ConstructionItem`. See comments in `construct.rs` for more details.
    pub flow_construction_result: ConstructionResult,

    pub before_flow_construction_result: ConstructionResult,

    pub after_flow_construction_result: ConstructionResult,

    /// Information needed during parallel traversals.
    pub parallel: DomParallelInfo,
}

impl PrivateLayoutData {
    /// Creates new layout data.
    pub fn new() -> PrivateLayoutData {
        PrivateLayoutData {
            before_style: None,
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
    pub chan: Option<LayoutChan>,
    pub shared_data: SharedLayoutData,
    pub data: Box<PrivateLayoutData>,
}

/// A trait that allows access to the layout data of a DOM node.
pub trait LayoutDataAccess {
    /// Borrows the layout data without checks.
    unsafe fn borrow_layout_data_unchecked(&self) -> *const Option<LayoutDataWrapper>;
    /// Borrows the layout data immutably. Fails on a conflicting borrow.
    fn borrow_layout_data<'a>(&'a self) -> Ref<'a,Option<LayoutDataWrapper>>;
    /// Borrows the layout data mutably. Fails on a conflicting borrow.
    fn mutate_layout_data<'a>(&'a self) -> RefMut<'a,Option<LayoutDataWrapper>>;
}

impl<'ln> LayoutDataAccess for LayoutNode<'ln> {
    #[inline(always)]
    unsafe fn borrow_layout_data_unchecked(&self) -> *const Option<LayoutDataWrapper> {
        mem::transmute(self.get().layout_data.borrow_unchecked())
    }

    #[inline(always)]
    fn borrow_layout_data<'a>(&'a self) -> Ref<'a,Option<LayoutDataWrapper>> {
        unsafe {
            mem::transmute(self.get().layout_data.borrow())
        }
    }

    #[inline(always)]
    fn mutate_layout_data<'a>(&'a self) -> RefMut<'a,Option<LayoutDataWrapper>> {
        unsafe {
            mem::transmute(self.get().layout_data.borrow_mut())
        }
    }
}

pub trait OpaqueNodeMethods {
    /// Converts a DOM node (layout view) to an `OpaqueNode`.
    fn from_layout_node(node: &LayoutNode) -> Self;

    /// Converts a thread-safe DOM node (layout view) to an `OpaqueNode`.
    fn from_thread_safe_layout_node(node: &ThreadSafeLayoutNode) -> Self;

    /// Converts a DOM node (script view) to an `OpaqueNode`.
    fn from_script_node(node: TrustedNodeAddress) -> Self;

    /// Converts a DOM node to an `OpaqueNode'.
    fn from_jsmanaged(node: &JS<Node>) -> Self;

    /// Converts this node to an `UntrustedNodeAddress`. An `UntrustedNodeAddress` is just the type
    /// of node that script expects to receive in a hit test.
    fn to_untrusted_node_address(&self) -> UntrustedNodeAddress;
}

impl OpaqueNodeMethods for OpaqueNode {
    fn from_layout_node(node: &LayoutNode) -> OpaqueNode {
        unsafe {
            OpaqueNodeMethods::from_jsmanaged(node.get_jsmanaged())
        }
    }

    fn from_thread_safe_layout_node(node: &ThreadSafeLayoutNode) -> OpaqueNode {
        unsafe {
            let abstract_node = node.get_jsmanaged();
            let ptr: uintptr_t = abstract_node.reflector().get_jsobject() as uintptr_t;
            OpaqueNode(ptr)
        }
    }

    fn from_script_node(node: TrustedNodeAddress) -> OpaqueNode {
        unsafe {
            OpaqueNodeMethods::from_jsmanaged(&JS::from_trusted_node_address(node))
        }
    }

    fn from_jsmanaged(node: &JS<Node>) -> OpaqueNode {
        unsafe {
            let ptr: uintptr_t = mem::transmute(node.reflector().get_jsobject());
            OpaqueNode(ptr)
        }
    }

    fn to_untrusted_node_address(&self) -> UntrustedNodeAddress {
        unsafe {
            let OpaqueNode(addr) = *self;
            let addr: UntrustedNodeAddress = mem::transmute(addr);
            addr
        }
    }
}

/// Allows a CSS color to be converted into a graphics color.
pub trait ToGfxColor {
    /// Converts a CSS color to a graphics color.
    fn to_gfx_color(&self) -> gfx::color::Color;
}

impl ToGfxColor for style::computed_values::RGBA {
    fn to_gfx_color(&self) -> gfx::color::Color {
        gfx::color::rgba(self.red, self.green, self.blue, self.alpha)
    }
}

