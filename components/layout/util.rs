/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_blocks)]

use construct::ConstructionResult;
use incremental::RestyleDamage;
use parallel::DomParallelInfo;
use wrapper::{LayoutNode, TLayoutNode, ThreadSafeLayoutNode};

use azure::azure_hl::{Color};
use gfx::display_list::OpaqueNode;
use gfx;
use libc::{c_void, uintptr_t};
use script::dom::bindings::js::LayoutJS;
use script::dom::node::{Node, SharedLayoutData};
use script::layout_interface::{LayoutChan, TrustedNodeAddress};
use script_traits::UntrustedNodeAddress;
use std::mem;
use std::cell::{Ref, RefMut};
use style::properties::ComputedValues;
use style;
use std::sync::Arc;

/// Data that layout associates with a node.
pub struct PrivateLayoutData {
    /// The results of CSS styling for this node's `before` pseudo-element, if any.
    pub before_style: Option<Arc<ComputedValues>>,

    /// The results of CSS styling for this node's `after` pseudo-element, if any.
    pub after_style: Option<Arc<ComputedValues>>,

    /// Description of how to account for recent style changes.
    pub restyle_damage: RestyleDamage,

    /// The current results of flow construction for this node. This is either a flow or a
    /// `ConstructionItem`. See comments in `construct.rs` for more details.
    pub flow_construction_result: ConstructionResult,

    pub before_flow_construction_result: ConstructionResult,

    pub after_flow_construction_result: ConstructionResult,

    /// Information needed during parallel traversals.
    pub parallel: DomParallelInfo,

    /// Various flags.
    pub flags: LayoutDataFlags,
}

impl PrivateLayoutData {
    /// Creates new layout data.
    pub fn new() -> PrivateLayoutData {
        PrivateLayoutData {
            before_style: None,
            after_style: None,
            restyle_damage: RestyleDamage::empty(),
            flow_construction_result: ConstructionResult::None,
            before_flow_construction_result: ConstructionResult::None,
            after_flow_construction_result: ConstructionResult::None,
            parallel: DomParallelInfo::new(),
            flags: LayoutDataFlags::empty(),
        }
    }
}

bitflags! {
    flags LayoutDataFlags: u8 {
        #[doc="Whether a flow has been newly constructed."]
        const HAS_NEWLY_CONSTRUCTED_FLOW = 0x01
    }
}

pub struct LayoutDataWrapper {
    pub chan: Option<LayoutChan>,
    pub shared_data: SharedLayoutData,
    pub data: Box<PrivateLayoutData>,
}

#[allow(dead_code)]
fn static_assertion(x: Option<LayoutDataWrapper>) {
    unsafe {
        let _: Option<::script::dom::node::LayoutData> =
            ::std::intrinsics::transmute(x);
    }
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
        mem::transmute(self.get().layout_data_unchecked())
    }

    #[inline(always)]
    fn borrow_layout_data<'a>(&'a self) -> Ref<'a,Option<LayoutDataWrapper>> {
        unsafe {
            mem::transmute(self.get().layout_data())
        }
    }

    #[inline(always)]
    fn mutate_layout_data<'a>(&'a self) -> RefMut<'a,Option<LayoutDataWrapper>> {
        unsafe {
            mem::transmute(self.get().layout_data_mut())
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
    fn from_jsmanaged(node: &LayoutJS<Node>) -> Self;

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
            OpaqueNodeMethods::from_jsmanaged(node.get_jsmanaged())
        }
    }

    fn from_script_node(node: TrustedNodeAddress) -> OpaqueNode {
        unsafe {
            OpaqueNodeMethods::from_jsmanaged(&LayoutJS::from_trusted_node_address(node))
        }
    }

    fn from_jsmanaged(node: &LayoutJS<Node>) -> OpaqueNode {
        unsafe {
            let ptr: uintptr_t = node.get_jsobject() as uintptr_t;
            OpaqueNode(ptr)
        }
    }

    fn to_untrusted_node_address(&self) -> UntrustedNodeAddress {
        let OpaqueNode(addr) = *self;
        UntrustedNodeAddress(addr as *const c_void)
    }
}

/// Allows a CSS color to be converted into a graphics color.
pub trait ToGfxColor {
    /// Converts a CSS color to a graphics color.
    fn to_gfx_color(&self) -> Color;
}

impl ToGfxColor for style::values::RGBA {
    fn to_gfx_color(&self) -> Color {
        gfx::color::rgba(self.red, self.green, self.blue, self.alpha)
    }
}
