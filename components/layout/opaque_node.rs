/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gfx::display_list::OpaqueNode;
use libc::c_void;
use script_traits::UntrustedNodeAddress;

pub trait OpaqueNodeMethods {
    /// Converts this node to an `UntrustedNodeAddress`. An `UntrustedNodeAddress` is just the type
    /// of node that script expects to receive in a hit test.
    fn to_untrusted_node_address(&self) -> UntrustedNodeAddress;
}

impl OpaqueNodeMethods for OpaqueNode {
    fn to_untrusted_node_address(&self) -> UntrustedNodeAddress {
        UntrustedNodeAddress(self.0 as *const c_void)
    }
}
