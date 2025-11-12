/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use base::{Epoch, IpcSend};
use constellation_traits::{LoadData, NavigationHistoryBehavior};
use embedder_traits::{
    ContextMenuAction, ContextMenuItem, ContextMenuRequest, EditingActionEvent, EmbedderControlId,
    EmbedderControlRequest, EmbedderControlResponse, EmbedderMsg,
};
use euclid::{Point2D, Rect, Size2D};
use ipc_channel::router::ROUTER;
use net_traits::CoreResourceMsg;
use net_traits::filemanager_thread::FileManagerThreadMsg;
use rustc_hash::FxHashMap;
use script_bindings::codegen::GenericBindings::HTMLAnchorElementBinding::HTMLAnchorElementMethods;
use script_bindings::codegen::GenericBindings::HTMLImageElementBinding::HTMLImageElementMethods;
use script_bindings::codegen::GenericBindings::HistoryBinding::HistoryMethods;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::inheritance::Castable;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::script_runtime::CanGc;
use script_bindings::str::USVString;
use servo_url::ServoUrl;
use webrender_api::units::{DeviceIntRect, DevicePoint};

use crate::dom::activation::Activatable;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::trace::NoTrace;
use crate::dom::inputevent::HitTestResult;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::textcontrol::TextControlElement;
use crate::dom::types::{
    Element, HTMLAnchorElement, HTMLElement, HTMLImageElement, HTMLInputElement, HTMLSelectElement,
    HTMLTextAreaElement, Window,
};
use crate::messaging::MainThreadScriptMsg;

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum ControlElement {
    Select(DomRoot<HTMLSelectElement>),
    ColorInput(DomRoot<HTMLInputElement>),
    FileInput(DomRoot<HTMLInputElement>),
    Ime(DomRoot<HTMLElement>),
    ContextMenu(ContextMenuNodes),
}

impl ControlElement {
    fn node(&self) -> &Node {
        match self {
            ControlElement::Select(element) => element.upcast::<Node>(),
            ControlElement::ColorInput(element) => element.upcast::<Node>(),
            ControlElement::FileInput(element) => element.upcast::<Node>(),
            ControlElement::Ime(element) => element.upcast::<Node>(),
            ControlElement::ContextMenu(context_menu_nodes) => &context_menu_nodes.node,
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

        // Never process embedder responses on inactive `Document`s.
        if !element.node().owner_doc().is_active() {
            return;
        }

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
                ControlElement::ContextMenu(context_menu_nodes),
                EmbedderControlResponse::ContextMenu(action),
            ) => {
                context_menu_nodes.handle_context_menu_action(action, can_gc);
            },
            (_, _) => unreachable!(
                "The response to a form control should always match it's originating type."
            ),
        }
    }

    pub(crate) fn show_context_menu(&self, hit_test_result: &HitTestResult) {
        let mut anchor_element = None;
        let mut image_element = None;
        let mut text_input_element = None;
        for node in hit_test_result
            .node
            .inclusive_ancestors(ShadowIncluding::Yes)
        {
            if anchor_element.is_none() {
                if let Some(candidate_anchor_element) = node.downcast::<HTMLAnchorElement>() {
                    if candidate_anchor_element.is_instance_activatable() {
                        anchor_element = Some(DomRoot::from_ref(candidate_anchor_element))
                    }
                }
            }

            if image_element.is_none() {
                if let Some(candidate_image_element) = node.downcast::<HTMLImageElement>() {
                    image_element = Some(DomRoot::from_ref(candidate_image_element))
                }
            }

            if text_input_element.is_none() {
                if let Some(candidate_text_input_element) = node.as_text_input() {
                    text_input_element = Some(candidate_text_input_element);
                }
            }
        }

        let mut items = Vec::new();
        if anchor_element.is_some() {
            items.extend(vec![
                ContextMenuItem::Item {
                    label: "Open Link in New View".into(),
                    action: ContextMenuAction::OpenLinkInNewWebView,
                    enabled: true,
                },
                ContextMenuItem::Item {
                    label: "Copy Link".into(),
                    action: ContextMenuAction::CopyLink,
                    enabled: true,
                },
                ContextMenuItem::Separator,
            ]);
        }

        if image_element.is_some() {
            items.extend(vec![
                ContextMenuItem::Item {
                    label: "Open Image in New View".into(),
                    action: ContextMenuAction::OpenImageInNewView,
                    enabled: true,
                },
                ContextMenuItem::Item {
                    label: "Copy Image Link".into(),
                    action: ContextMenuAction::CopyImageLink,
                    enabled: true,
                },
                ContextMenuItem::Separator,
            ]);
        }

        if let Some(text_input_element) = &text_input_element {
            let has_selection = text_input_element.has_selection();
            items.extend(vec![
                ContextMenuItem::Item {
                    label: "Cut".into(),
                    action: ContextMenuAction::Cut,
                    enabled: has_selection,
                },
                ContextMenuItem::Item {
                    label: "Copy".into(),
                    action: ContextMenuAction::Copy,
                    enabled: has_selection,
                },
                ContextMenuItem::Item {
                    label: "Paste".into(),
                    action: ContextMenuAction::Paste,
                    enabled: true,
                },
                ContextMenuItem::Item {
                    label: "Select All".into(),
                    action: ContextMenuAction::SelectAll,
                    enabled: text_input_element.has_selectable_text(),
                },
                ContextMenuItem::Separator,
            ]);
        }

        items.extend(vec![
            ContextMenuItem::Item {
                label: "Back".into(),
                action: ContextMenuAction::GoBack,
                enabled: true,
            },
            ContextMenuItem::Item {
                label: "Forward".into(),
                action: ContextMenuAction::GoForward,
                enabled: true,
            },
            ContextMenuItem::Item {
                label: "Reload".into(),
                action: ContextMenuAction::Reload,
                enabled: true,
            },
        ]);

        let context_menu_nodes = ContextMenuNodes {
            node: hit_test_result.node.clone(),
            anchor_element,
            image_element,
            text_input_element,
        };

        self.show_embedder_control(
            ControlElement::ContextMenu(context_menu_nodes),
            EmbedderControlRequest::ContextMenu(ContextMenuRequest { items }),
            Some(hit_test_result.point_in_frame.cast_unit()),
        );
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct ContextMenuNodes {
    /// The node that this menu was triggered on.
    node: DomRoot<Node>,
    /// The first inclusive ancestor of this node that is an `<a>` if one exists.
    anchor_element: Option<DomRoot<HTMLAnchorElement>>,
    /// The first inclusive ancestor of this node that is an `<img>` if one exists.
    image_element: Option<DomRoot<HTMLImageElement>>,
    /// The first inclusive ancestor of this node which is a text entry field.
    text_input_element: Option<DomRoot<Element>>,
}

impl ContextMenuNodes {
    fn handle_context_menu_action(&self, action: Option<ContextMenuAction>, can_gc: CanGc) {
        let Some(action) = action else {
            return;
        };

        let window = self.node.owner_window();
        let set_clipboard_text = |string: USVString| {
            if string.is_empty() {
                return;
            }
            window.send_to_embedder(EmbedderMsg::SetClipboardText(
                window.webview_id(),
                string.to_string(),
            ));
        };

        let open_url_in_new_webview = |url_string: USVString| {
            let Ok(url) = ServoUrl::parse(&url_string) else {
                return;
            };
            let Some(browsing_context) = window.Document().browsing_context() else {
                return;
            };
            let (browsing_context, new) = browsing_context
                .choose_browsing_context("_blank".into(), true /* nooopener */);
            let Some(browsing_context) = browsing_context else {
                return;
            };
            assert!(new);
            let Some(target_document) = browsing_context.document() else {
                return;
            };

            let target_window = target_document.window();
            let target = Trusted::new(target_window);
            let load_data = LoadData::new_for_new_unrelated_webview(url);
            let task = task!(open_link_in_new_webview: move || {
                target.root().load_url(NavigationHistoryBehavior::Replace, false, load_data, CanGc::note());
            });
            target_document
                .owner_global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task);
        };

        match action {
            ContextMenuAction::GoBack => {
                let _ = window.History().Back();
            },
            ContextMenuAction::GoForward => {
                let _ = window.History().Forward();
            },
            ContextMenuAction::Reload => {
                window.Location().reload_without_origin_check(can_gc);
            },
            ContextMenuAction::CopyLink => {
                let Some(anchor_element) = &self.anchor_element else {
                    return;
                };
                set_clipboard_text(anchor_element.Href());
            },
            ContextMenuAction::OpenLinkInNewWebView => {
                let Some(anchor_element) = &self.anchor_element else {
                    return;
                };
                open_url_in_new_webview(anchor_element.Href());
            },
            ContextMenuAction::CopyImageLink => {
                let Some(image_element) = &self.image_element else {
                    return;
                };
                set_clipboard_text(image_element.Src());
            },
            ContextMenuAction::OpenImageInNewView => {
                let Some(image_element) = &self.image_element else {
                    return;
                };
                open_url_in_new_webview(image_element.Src());
            },
            ContextMenuAction::Cut => {
                window.Document().event_handler().handle_editing_action(
                    self.text_input_element.clone(),
                    EditingActionEvent::Cut,
                    can_gc,
                );
            },
            ContextMenuAction::Copy => {
                window.Document().event_handler().handle_editing_action(
                    self.text_input_element.clone(),
                    EditingActionEvent::Copy,
                    can_gc,
                );
            },
            ContextMenuAction::Paste => {
                window.Document().event_handler().handle_editing_action(
                    self.text_input_element.clone(),
                    EditingActionEvent::Paste,
                    can_gc,
                );
            },
            ContextMenuAction::SelectAll => {
                if let Some(text_input_element) = &self.text_input_element {
                    text_input_element.select_all();
                }
            },
        }
    }
}

impl Node {
    fn as_text_input(&self) -> Option<DomRoot<Element>> {
        if let Some(input_element) = self
            .downcast::<HTMLInputElement>()
            .filter(|input_element| input_element.is_textual_widget())
        {
            return Some(DomRoot::from_ref(input_element.upcast::<Element>()));
        }
        self.downcast::<HTMLTextAreaElement>()
            .map(Castable::upcast)
            .map(DomRoot::from_ref)
    }
}

impl Element {
    fn has_selection(&self) -> bool {
        self.downcast::<HTMLTextAreaElement>()
            .map(TextControlElement::has_selection)
            .or(self
                .downcast::<HTMLInputElement>()
                .map(TextControlElement::has_selection))
            .unwrap_or_default()
    }

    fn has_selectable_text(&self) -> bool {
        self.downcast::<HTMLTextAreaElement>()
            .map(TextControlElement::has_selectable_text)
            .or(self
                .downcast::<HTMLInputElement>()
                .map(TextControlElement::has_selectable_text))
            .unwrap_or_default()
    }

    fn select_all(&self) {
        self.downcast::<HTMLTextAreaElement>()
            .map(TextControlElement::select_all)
            .or(self
                .downcast::<HTMLInputElement>()
                .map(TextControlElement::select_all))
            .unwrap_or_default()
    }
}
