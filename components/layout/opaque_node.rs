/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use gfx::display_list::OpaqueNode;
use libc::{c_void, uintptr_t};
use script::dom::bindings::js::LayoutJS;
use script::dom::node::Node;
use script::layout_interface::{TrustedNodeAddress};
use script_traits::UntrustedNodeAddress;

pub trait OpaqueNodeMethods {
    /// Converts a DOM node (script view) to an `OpaqueNode`.
    fn from_script_node(node: TrustedNodeAddress) -> Self;

    /// Converts a DOM node to an `OpaqueNode'.
    fn from_jsmanaged(node: &LayoutJS<Node>) -> Self;

    /// Converts this node to an `UntrustedNodeAddress`. An `UntrustedNodeAddress` is just the type
    /// of node that script expects to receive in a hit test.
    fn to_untrusted_node_address(&self) -> UntrustedNodeAddress;
}

impl OpaqueNodeMethods for OpaqueNode {
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
        UntrustedNodeAddress(self.0 as *const c_void)
    }
}
