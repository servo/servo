/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom_traversal::NodeExt;
use style::context::SharedStyleContext;

#[derive(Debug)]
pub(super) enum ReplacedContent {
    // Not implemented yet
}

impl ReplacedContent {
    pub fn for_element<'dom, Node>(element: Node, _context: &SharedStyleContext) -> Option<Self>
    where
        Node: NodeExt<'dom>,
    {
        // FIXME: implement <img> etc.
        None
    }
}
