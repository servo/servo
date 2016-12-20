/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use atomic_refcell::AtomicRefCell;
use context::{SharedStyleContext, StyleContext, ThreadLocalStyleContext};
use data::ElementData;
use dom::{NodeInfo, TNode};
use gecko::wrapper::{GeckoElement, GeckoNode};
use traversal::{DomTraversal, PerLevelTraversalData, recalc_style_at};

pub struct RecalcStyleOnly {
    shared: SharedStyleContext,
}

impl RecalcStyleOnly {
    pub fn new(shared: SharedStyleContext) -> Self {
        RecalcStyleOnly {
            shared: shared,
        }
    }
}

impl<'ln> DomTraversal<GeckoNode<'ln>> for RecalcStyleOnly {
    type ThreadLocalContext = ThreadLocalStyleContext;

    fn process_preorder(&self, traversal_data: &mut PerLevelTraversalData,
                        thread_local: &mut ThreadLocalStyleContext,
                        node: GeckoNode<'ln>)
    {
        if node.is_element() {
            let el = node.as_element().unwrap();
            let mut data = unsafe { el.ensure_data() }.borrow_mut();
            let mut context = StyleContext {
                shared: &self.shared,
                thread_local: thread_local,
            };
            recalc_style_at(self, traversal_data, &mut context, el, &mut data);
        }
    }

    fn process_postorder(&self, _: &mut ThreadLocalStyleContext, _: GeckoNode<'ln>) {
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

    fn shared_context(&self) -> &SharedStyleContext {
        &self.shared
    }

    fn create_thread_local_context(&self) -> ThreadLocalStyleContext {
        ThreadLocalStyleContext::new(&self.shared)
    }
}
