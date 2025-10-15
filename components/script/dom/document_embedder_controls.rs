/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use base::Epoch;
use embedder_traits::{
    EmbedderControlId, EmbedderControlRequest, EmbedderControlResponse, EmbedderMsg,
};
use rustc_hash::FxHashMap;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::script_runtime::CanGc;
use webrender_api::units::DeviceIntRect;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::inheritance::Castable as _;
use crate::dom::bindings::trace::NoTrace;
use crate::dom::types::{Element, HTMLInputElement, HTMLSelectElement, Window};

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum ControlElement {
    Select(DomRoot<HTMLSelectElement>),
    ColorInput(DomRoot<HTMLInputElement>),
}

impl ControlElement {
    fn element(&self) -> &Element {
        match self {
            ControlElement::Select(element) => element.upcast::<Element>(),
            ControlElement::ColorInput(element) => element.upcast::<Element>(),
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub(crate) struct DocumentEmbedderControls {
    /// The [`Window`] element for this [`DocumentUserInterfaceElements`].
    window: Dom<Window>,
    /// The id of the next user interface element that the `Document` requests that the
    /// embedder show. This is used to track user interface elements in the API.
    #[no_trace]
    user_interface_element_index: Cell<Epoch>,
    /// A map of visible user interface elements.
    visible_elements: DomRefCell<FxHashMap<NoTrace<Epoch>, ControlElement>>,
}

impl DocumentEmbedderControls {
    pub fn new(window: &Window) -> Self {
        Self {
            window: Dom::from_ref(window),
            user_interface_element_index: Default::default(),
            visible_elements: Default::default(),
        }
    }

    /// Generate the next unused [`EmbedderControlId`]. This method is only needed for some older
    /// types of controls that are still being migrated, and it will eventually be removed.
    pub(crate) fn next_control_id(&self) -> EmbedderControlId {
        let index = self.user_interface_element_index.get();
        self.user_interface_element_index.set(index.next());
        EmbedderControlId {
            webview_id: self.window.webview_id(),
            pipeline_id: self.window.pipeline_id(),
            index,
        }
    }

    pub(crate) fn show_embedder_control(
        &self,
        element: ControlElement,
        rect: DeviceIntRect,
        embedder_control: EmbedderControlRequest,
    ) -> EmbedderControlId {
        let id = self.next_control_id();
        self.visible_elements
            .borrow_mut()
            .insert(id.index.into(), element);
        self.window
            .send_to_embedder(EmbedderMsg::ShowEmbedderControl(id, rect, embedder_control));

        id
    }

    pub(crate) fn hide_embedder_control(&self, element: &Element) {
        self.visible_elements
            .borrow_mut()
            .retain(|index, control_element| {
                if control_element.element() != element {
                    return true;
                }
                let id = EmbedderControlId {
                    webview_id: self.window.webview_id(),
                    pipeline_id: self.window.pipeline_id(),
                    index: index.0,
                };
                self.window
                    .send_to_embedder(EmbedderMsg::HideEmbedderControl(id));
                false
            });
    }

    pub(crate) fn handle_embedder_control_response(
        &self,
        id: EmbedderControlId,
        response: EmbedderControlResponse,
        can_gc: CanGc,
    ) {
        assert_eq!(self.window.pipeline_id(), id.pipeline_id);
        assert_eq!(self.window.webview_id(), id.webview_id);

        let Some(element) = self.visible_elements.borrow_mut().remove(&id.index.into()) else {
            return;
        };

        match (element, response) {
            (
                ControlElement::Select(select_element),
                EmbedderControlResponse::SelectElement(response),
            ) => {
                select_element.handle_menu_response(response, can_gc);
            },
            (
                ControlElement::ColorInput(input_element),
                EmbedderControlResponse::ColorPicker(response),
            ) => {
                input_element.handle_color_picker_response(response, can_gc);
            },
            (_, _) => unreachable!(
                "The response to a form control should always match it's originating type."
            ),
        }
    }
}
