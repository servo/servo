/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko-specific bits for the styling DOM traversal.

use atomic_refcell::AtomicRefCell;
use context::{SharedStyleContext, StyleContext, ThreadLocalStyleContext};
use data::ElementData;
use dom::{NodeInfo, TNode};
use gecko::wrapper::{GeckoElement, GeckoNode};
use traversal::{DomTraversal, PerLevelTraversalData, TraversalDriver, recalc_style_at};

/// This is the simple struct that Gecko uses to encapsulate a DOM traversal for
/// styling.
pub struct RecalcStyleOnly {
    shared: SharedStyleContext,
    driver: TraversalDriver,
}

impl RecalcStyleOnly {
    /// Create a `RecalcStyleOnly` traversal from a `SharedStyleContext`.
    pub fn new(shared: SharedStyleContext, driver: TraversalDriver) -> Self {
        RecalcStyleOnly {
            shared: shared,
            driver: driver,
        }
    }
}

impl<'le> DomTraversal<GeckoElement<'le>> for RecalcStyleOnly {
    type ThreadLocalContext = ThreadLocalStyleContext<GeckoElement<'le>>;

    fn process_preorder(&self, traversal_data: &mut PerLevelTraversalData,
                        thread_local: &mut Self::ThreadLocalContext,
                        node: GeckoNode<'le>)
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

    fn process_postorder(&self, _: &mut Self::ThreadLocalContext, _: GeckoNode<'le>) {
        unreachable!();
    }

    /// We don't use the post-order traversal for anything.
    fn needs_postorder_traversal() -> bool { false }

    unsafe fn ensure_element_data<'a>(element: &'a GeckoElement<'le>) -> &'a AtomicRefCell<ElementData> {
        element.ensure_data()
    }

    unsafe fn clear_element_data<'a>(element: &'a GeckoElement<'le>) {
        element.clear_data()
    }

    fn shared_context(&self) -> &SharedStyleContext {
        &self.shared
    }

    fn create_thread_local_context(&self) -> Self::ThreadLocalContext {
        ThreadLocalStyleContext::new(&self.shared)
    }

    fn is_parallel(&self) -> bool {
        self.driver.is_parallel()
    }
}
