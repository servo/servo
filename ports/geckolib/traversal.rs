/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use context::StandaloneStyleContext;
use std::mem;
use style::context::SharedStyleContext;
use style::dom::OpaqueNode;
use style::dom::TNode;
use style::traversal::{DomTraversalContext, recalc_style_at};
use wrapper::GeckoNode;

pub struct RecalcStyleOnly<'lc> {
    context: StandaloneStyleContext<'lc>,
    root: OpaqueNode,
}

impl<'lc, 'ln> DomTraversalContext<GeckoNode<'ln>> for RecalcStyleOnly<'lc> {
    type SharedContext = SharedStyleContext;
    #[allow(unsafe_code)]
    fn new<'a>(shared: &'a Self::SharedContext, root: OpaqueNode) -> Self {
        // See the comment in RecalcStyleAndConstructFlows::new for an explanation of why this is
        // necessary.
        let shared_lc: &'lc Self::SharedContext = unsafe { mem::transmute(shared) };
        RecalcStyleOnly {
            context: StandaloneStyleContext::new(shared_lc),
            root: root,
        }
    }

    fn process_preorder(&self, node: GeckoNode<'ln>) {
        // FIXME(pcwalton): Stop allocating here. Ideally this should just be done by the HTML
        // parser.
        node.initialize_data();

        recalc_style_at(&self.context, self.root, node);
    }

    fn process_postorder(&self, _: GeckoNode<'ln>) {}

    /// In Gecko we use this traversal just for restyling, so we can stop once
    /// we know there aren't more dirty nodes under ourselves.
    fn should_process(&self, node: GeckoNode<'ln>) -> bool {
        node.is_dirty() || node.has_dirty_descendants()
    }

    fn pre_process_child_hook(&self, parent: GeckoNode<'ln>, kid: GeckoNode<'ln>) {
        // NOTE: At this point is completely safe to modify either the parent or
        // the child, since we have exclusive access to them.
        if parent.is_dirty() {
            unsafe {
                kid.set_dirty(true);
                parent.set_dirty_descendants(true);
            }
        }
    }
}
