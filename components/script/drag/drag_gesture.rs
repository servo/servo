/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_traits::ConstellationInputEvent;

use crate::dom::inputevent::HitTestResult;
use crate::dom::text_input::TextInputSelectionDragHandler;
use crate::drag::document_selection_drag::DocumentSelectionDragHandler;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DragGesture {
    handler: DragHandler,
}

impl DragGesture {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(handler: DragHandler) -> Self {
        Self { handler }
    }

    pub(crate) fn handle_mouse_button_event(&self, event: &ConstellationInputEvent) -> bool {
        event.primary_button_is_pressed() &&
            match &self.handler {
                DragHandler::TextInputSelection(handler) => handler.still_connected(),
                DragHandler::DocumentSelection(handler) => handler.still_connected(),
            }
    }

    /// Handle the a mouse move event.
    ///
    /// Returns `true` if the `DragGesture` should continue and `false` otherwise.
    pub(crate) fn handle_mouse_move_event(
        &self,
        cx: &mut JSContext,
        event: &ConstellationInputEvent,
        hit_test_result: &HitTestResult,
    ) -> bool {
        if !event.primary_button_is_pressed() {
            return false;
        }
        match &self.handler {
            DragHandler::TextInputSelection(handler) => handler.moved(hit_test_result),
            DragHandler::DocumentSelection(handler) => handler.moved(cx, hit_test_result),
        }
    }

    pub(crate) fn need_dom_position_from_hit_test(&self) -> bool {
        matches!(self.handler, DragHandler::DocumentSelection(..))
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum DragHandler {
    TextInputSelection(TextInputSelectionDragHandler),
    DocumentSelection(DocumentSelectionDragHandler),
}
