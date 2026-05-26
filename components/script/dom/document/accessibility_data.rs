/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use embedder_traits::UntrustedNodeAddress;
use js::context::NoGC;
use rustc_hash::FxHashSet;
use script_bindings::root::DomRoot;
use servo_config::pref;

use crate::dom::{Node, from_untrusted_node_address};

#[derive(Clone, Default, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct AccessibilityData {
    /// Nodes which have been unbound from the DOM but may not yet have been removed from the
    /// accessibility tree. This is cleared after each reflow.
    rooted_nodes: FxHashSet<DomRoot<Node>>,
}

impl AccessibilityData {
    /// Root a node which has been removed from the DOM but which may still have an associated
    /// accessibility tree node. It will be unrooted after the next reflow, as the accessibility
    /// tree is updated as part of the reflow process.
    pub(crate) fn root_removed_node_for_accessibility(
        &mut self,
        _no_gc: &NoGC,
        node_to_root: &Node,
    ) {
        assert!(pref!(accessibility_enabled));

        self.rooted_nodes.insert(DomRoot::from_ref(node_to_root));
    }

    /// Unroot a node which has been added to the DOM, if it was previously rooted due to
    /// `[Self::root_removed_node_for_accessibility()`].
    pub(crate) fn unroot_node_for_accessibility(&mut self, _no_gc: &NoGC, node_to_unroot: &Node) {
        assert!(pref!(accessibility_enabled));

        self.rooted_nodes.remove(&DomRoot::from_ref(node_to_unroot));
    }

    /// Clear all nodes which were rooted using [`Self::root_removed_node_for_accessibility()`].
    #[expect(unsafe_code)]
    pub(crate) fn unroot_all_nodes_for_accessibility(
        &mut self,
        removed_nodes_for_integrity_check: Option<Vec<UntrustedNodeAddress>>,
    ) {
        assert!(pref!(accessibility_enabled));

        if let Some(removed_nodes) = removed_nodes_for_integrity_check {
            assert!(pref!(expensive_accessibility_test_assertions_enabled));
            for address in removed_nodes {
                unsafe {
                    let removed_node = from_untrusted_node_address(address);
                    self.rooted_nodes.remove(&removed_node);
                }
            }
            assert!(self.rooted_nodes.is_empty());
        }

        self.rooted_nodes.clear();
    }
}
