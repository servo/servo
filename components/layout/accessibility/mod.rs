/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use layout_api::AccessibilityDamage;
use rustc_hash::{FxHashMap, FxHashSet};
use script::layout_dom::ServoLayoutNode;
use style::dom::OpaqueNode;

use crate::display_list::StackingContextTree;
use crate::layout_impl::LayoutThread;

mod accessibility_node;
mod accessibility_tree;

pub use accessibility_tree::AccessibilityTree;

/// Everything the accessibility tree needs from layout in order to compute node bounds during an
/// update.
pub(crate) struct AccessibilityContext<'update> {
    pub(crate) layout_thread: &'update LayoutThread,
    pub(crate) stacking_context_tree: &'update StackingContextTree,
    pub(crate) focused_element: Option<OpaqueNode>,
    pub(crate) rooted_nodes_for_integrity_check: Option<FxHashSet<OpaqueNode>>,
}

/// All the [`AccessibilityDamage`] which comes from outside the accessibility tree itself.
pub(crate) type AccessibilityDamageMap<'a> =
    FxHashMap<OpaqueNode, (ServoLayoutNode<'a>, AccessibilityDamage)>;
