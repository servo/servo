/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use atomic_refcell::AtomicRefCell;
use context::{LocalStyleContext, SharedStyleContext, StyleContext};
use data::ElementData;
use dom::{NodeInfo, OpaqueNode, TNode};
use gecko::context::StandaloneStyleContext;
use gecko::wrapper::{GeckoElement, GeckoNode};
use std::mem;
use traversal::{DomTraversalContext, PerLevelTraversalData, recalc_style_at};

pub struct RecalcStyleOnly<'lc> {
    context: StandaloneStyleContext<'lc>,
}

impl<'lc, 'ln> DomTraversalContext<GeckoNode<'ln>> for RecalcStyleOnly<'lc> {
    type SharedContext = SharedStyleContext;
    #[allow(unsafe_code)]
    fn new<'a>(shared: &'a Self::SharedContext, _root: OpaqueNode) -> Self {
        // See the comment in RecalcStyleAndConstructFlows::new for an explanation of why this is
        // necessary.
        let shared_lc: &'lc Self::SharedContext = unsafe { mem::transmute(shared) };
        RecalcStyleOnly {
            context: StandaloneStyleContext::new(shared_lc),
        }
    }

    fn process_preorder(&self, node: GeckoNode<'ln>, traversal_data: &mut PerLevelTraversalData) {
        if node.is_element() {
            let el = node.as_element().unwrap();
            let mut data = unsafe { el.ensure_data() }.borrow_mut();
            recalc_style_at::<_, _, Self>(&self.context, traversal_data, el, &mut data);
        }
    }

    fn process_postorder(&self, _: GeckoNode<'ln>) {
        unreachable!();
    }

    /// We don't use the post-order traversal for anything.
    fn needs_postorder_traversal() -> bool { false }

    unsafe fn ensure_element_data<'a>(element: &'a GeckoElement<'ln>) -> &'a AtomicRefCell<ElementData> {
        element.ensure_data()
    }

    unsafe fn clear_element_data<'a>(element: &'a GeckoElement<'ln>) {
        element.clear_data()
    }

    fn local_context(&self) -> &LocalStyleContext {
        self.context.local_context()
    }
}
