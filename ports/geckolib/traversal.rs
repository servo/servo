/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use context::StandaloneStyleContext;
use std::mem;
use style::context::{LocalStyleContext, SharedStyleContext, StyleContext};
use style::dom::OpaqueNode;
use style::traversal::RestyleResult;
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

    fn process_preorder(&self, node: GeckoNode<'ln>) -> RestyleResult {
        // FIXME(pcwalton): Stop allocating here. Ideally this should just be done by the HTML
        // parser.
        node.initialize_data();

        recalc_style_at(&self.context, self.root, node)
    }

    fn process_postorder(&self, _: GeckoNode<'ln>) {
        unreachable!();
    }

    /// We don't use the post-order traversal for anything.
    fn needs_postorder_traversal(&self) -> bool { false }

    fn local_context(&self) -> &LocalStyleContext {
        self.context.local_context()
    }
}
