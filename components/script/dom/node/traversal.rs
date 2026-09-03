/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::NoGC;
use script_bindings::dom::UnrootedDom;

use crate::dom::Node;

pub(crate) trait NoGcTraversal {
    fn parent<'a>(no_gc: &'a NoGC, node: &Node) -> Option<UnrootedDom<'a, Node>>;
    fn children<'a>(no_gc: &'a NoGC, node: &Node) -> impl Iterator<Item = UnrootedDom<'a, Node>>;
}

pub(crate) struct LightDomNoGcTraversal;

impl NoGcTraversal for LightDomNoGcTraversal {
    fn parent<'a>(no_gc: &'a NoGC, node: &Node) -> Option<UnrootedDom<'a, Node>> {
        node.get_parent_node_unrooted(no_gc)
    }
    fn children<'a>(no_gc: &'a NoGC, node: &Node) -> impl Iterator<Item = UnrootedDom<'a, Node>> {
        node.children_unrooted(no_gc)
    }
}
