/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::Point2D;
use script_traits::ConstellationInputEvent;
use style_traits::CSSPixel;

use crate::dom::text_input::TextInputDragHandler;

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
                DragHandler::TextInput(handler) => handler.still_connected(),
            }
    }

    pub(crate) fn handle_mouse_move_event(
        &self,
        event: &ConstellationInputEvent,
        point_in_viewport: Point2D<Au, CSSPixel>,
    ) -> bool {
        if !event.primary_button_is_pressed() {
            return false;
        }
        match &self.handler {
            DragHandler::TextInput(handler) => handler.moved(point_in_viewport),
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum DragHandler {
    TextInput(TextInputDragHandler),
}
