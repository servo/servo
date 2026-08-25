/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;

use crate::dom::NodeTraits;
use crate::dom::inputevent::HitTestResult;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DocumentSelectionDragHandler;

impl DocumentSelectionDragHandler {
    pub(crate) fn still_connected(&self) -> bool {
        true
    }

    /// Process a mouse move event on this [`DocumentSelectionDragHandler`].
    ///
    /// Returns `true` if the drag should continue and `false` otherwise.
    pub(crate) fn moved(&self, cx: &mut JSContext, hit_test_result: &HitTestResult) -> bool {
        let Some((container, offset)) = hit_test_result.dom_position_for_selection.as_ref() else {
            return true;
        };
        let Some(selection) = container.owner_document().selection() else {
            return true;
        };
        selection.collapse_or_extend_to_dom_position(cx, container, *offset);
        true
    }
}
