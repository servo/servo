/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use base::{Epoch, IpcSend};
use embedder_traits::{
    ContextMenuAction, ContextMenuItem, ContextMenuRequest, EmbedderControlId,
    EmbedderControlRequest, EmbedderControlResponse, EmbedderMsg,
};
use euclid::{Point2D, Rect, Size2D};
use ipc_channel::router::ROUTER;
use net_traits::CoreResourceMsg;
use net_traits::filemanager_thread::FileManagerThreadMsg;
use rustc_hash::FxHashMap;
use script_bindings::codegen::GenericBindings::HistoryBinding::HistoryMethods;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::script_runtime::CanGc;
use webrender_api::units::{DeviceIntRect, DevicePoint};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::inheritance::Castable as _;
use crate::dom::bindings::trace::NoTrace;
use crate::dom::inputevent::HitTestResult;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::types::{Element, HTMLElement, HTMLInputElement, HTMLSelectElement, Window};
use crate::messaging::MainThreadScriptMsg;

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum ControlElement {
    Select(DomRoot<HTMLSelectElement>),
    ColorInput(DomRoot<HTMLInputElement>),
    FileInput(DomRoot<HTMLInputElement>),
    Ime(DomRoot<HTMLElement>),
    ContextMenu(DomRoot<Node>),
}

impl ControlElement {
    fn node(&self) -> &Node {
        match self {
            ControlElement::Select(element) => element.upcast::<Node>(),
            ControlElement::ColorInput(element) => element.upcast::<Node>(),
            ControlElement::FileInput(element) => element.upcast::<Node>(),
            ControlElement::Ime(element) => element.upcast::<Node>(),
            ControlElement::ContextMenu(element) => element,
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
        request: EmbedderControlRequest,
        point: Option<DevicePoint>,
    ) -> EmbedderControlId {
        let id = self.next_control_id();
        let rect = point
            .map(|point| DeviceIntRect::from_origin_and_size(point.to_i32(), Size2D::zero()))
            .unwrap_or_else(|| {
                let rect = element
                    .node()
                    .upcast::<Node>()
                    .border_box()
                    .unwrap_or_default();

                let rect = Rect::new(
                    Point2D::new(rect.origin.x.to_px(), rect.origin.y.to_px()),
                    Size2D::new(rect.size.width.to_px(), rect.size.height.to_px()),
                );

                // FIXME: This is a CSS pixel rect relative to this frame, we need a DevicePixel rectangle
                // relative to the entire WebView!
                DeviceIntRect::from_untyped(&rect.to_box2d())
            });

        self.visible_elements
            .borrow_mut()
            .insert(id.index.into(), element);

        match request {
            EmbedderControlRequest::SelectElement(..) |
            EmbedderControlRequest::ColorPicker(..) |
            EmbedderControlRequest::InputMethod(..) |
            EmbedderControlRequest::ContextMenu(..) => self
                .window
                .send_to_embedder(EmbedderMsg::ShowEmbedderControl(id, rect, request)),
            EmbedderControlRequest::FilePicker(file_picker_request) => {
                let (sender, receiver) = profile_traits::ipc::channel(
                    self.window.as_global_scope().time_profiler_chan().clone(),
                )
                .expect("Error initializing channel");
                let main_thread_sender = self.window.main_thread_script_chan().clone();
                ROUTER.add_typed_route(
                    receiver.to_ipc_receiver(),
                    Box::new(move |result| {
                        let Ok(embedder_control_response) = result else {
                            return;
                        };
                        if let Err(error) = main_thread_sender.send(
                            MainThreadScriptMsg::ForwardEmbedderControlResponseFromFileManager(
                                id,
                                embedder_control_response,
                            ),
                        ) {
                            warn!("Could not send FileManager response to main thread: {error}")
                        }
                    }),
                );
                self.window
                    .as_global_scope()
                    .resource_threads()
                    .sender()
                    .send(CoreResourceMsg::ToFileManager(
                        FileManagerThreadMsg::SelectFiles(id, file_picker_request, sender),
                    ))
                    .unwrap();
            },
        }

        id
    }

    pub(crate) fn hide_embedder_control(&self, element: &Element) {
        self.visible_elements
            .borrow_mut()
            .retain(|index, control_element| {
                if control_element.node() != element.upcast() {
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
            (
                ControlElement::FileInput(input_element),
                EmbedderControlResponse::FilePicker(response),
            ) => {
                input_element.handle_file_picker_response(response, can_gc);
            },
            (
                ControlElement::ContextMenu(element),
                EmbedderControlResponse::ContextMenu(action),
            ) => {
                self.handle_context_menu_action(&element, action, can_gc);
            },
            (_, _) => unreachable!(
                "The response to a form control should always match it's originating type."
            ),
        }
    }

    pub(crate) fn show_context_menu(&self, hit_test_result: &HitTestResult) {
        let items = vec![
            ContextMenuItem::Item {
                label: "Back".into(),
                action: ContextMenuAction::GoBack,
            },
            ContextMenuItem::Item {
                label: "Forward".into(),
                action: ContextMenuAction::GoForward,
            },
            ContextMenuItem::Item {
                label: "Reload".into(),
                action: ContextMenuAction::Reload,
            },
        ];

        self.show_embedder_control(
            ControlElement::ContextMenu(hit_test_result.node.clone()),
            EmbedderControlRequest::ContextMenu(ContextMenuRequest { items }),
            Some(hit_test_result.point_in_frame.cast_unit()),
        );
    }

    fn handle_context_menu_action(
        &self,
        node: &Node,
        action: Option<ContextMenuAction>,
        can_gc: CanGc,
    ) {
        let Some(action) = action else {
            return;
        };

        let window = node.owner_window();
        match action {
            ContextMenuAction::GoBack => {
                let _ = window.History().Back();
            },
            ContextMenuAction::GoForward => {
                let _ = window.History().Forward();
            },
            ContextMenuAction::Reload => {
                self.window.Location().reload_without_origin_check(can_gc);
            },
        }
    }
}
