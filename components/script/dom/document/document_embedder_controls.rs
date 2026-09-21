/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![cfg_attr(crown, allow(crown::jscontext_first_arg))]

use std::cell::Cell;

use embedder_traits::{
    ClipboardAction, ContextMenuAction, ContextMenuElementInformation,
    ContextMenuElementInformationFlags, ContextMenuItem, ContextMenuRequest, EmbedderControlId,
    EmbedderControlRequest, EmbedderControlResponse, EmbedderMsg,
};
use euclid::{Point2D, Rect, Size2D};
use js::context::{JSContext, NoGC};
use net_traits::CoreResourceMsg;
use net_traits::filemanager_thread::FileManagerThreadMsg;
use rustc_hash::FxHashMap;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::HTMLAnchorElementBinding::HTMLAnchorElementMethods;
use script_bindings::codegen::GenericBindings::HTMLImageElementBinding::HTMLImageElementMethods;
use script_bindings::codegen::GenericBindings::HistoryBinding::HistoryMethods;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::inheritance::Castable;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::str::DOMString;
use servo_base::Epoch;
use servo_base::generic_channel::GenericSend;
use servo_constellation_traits::{LoadData, NavigationHistoryBehavior};
use servo_url::ServoUrl;
use webrender_api::units::{DeviceIntRect, DevicePoint};

use crate::dom::activation::Activatable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::trace::NoTrace;
use crate::dom::inputevent::HitTestResult;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::types::{
    Element, HTMLAnchorElement, HTMLElement, HTMLImageElement, HTMLInputElement, HTMLSelectElement,
    Window,
};
use crate::messaging::MainThreadScriptMsg;
use crate::navigation::navigate;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum ControlElement {
    Select(Dom<HTMLSelectElement>),
    ColorInput(Dom<HTMLInputElement>),
    FileInput(Dom<HTMLInputElement>),
    Ime(Dom<HTMLElement>),
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

impl js::gc::Rootable for ControlElement {}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
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

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
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

        self.send_embedder_control_request(request, id, rect);
        id
    }

    fn send_embedder_control_request(
        &self,
        request: EmbedderControlRequest,
        id: EmbedderControlId,
        rect: DeviceIntRect,
    ) {
        match request {
            EmbedderControlRequest::SelectElement(..) |
            EmbedderControlRequest::ColorPicker(..) |
            EmbedderControlRequest::InputMethod(..) |
            EmbedderControlRequest::ContextMenu(..) => self
                .window
                .send_to_embedder(EmbedderMsg::ShowEmbedderControl(id, rect, request)),
            EmbedderControlRequest::FilePicker(file_picker_request) => {
                let main_thread_sender = self.window.main_thread_script_chan().clone();
                let callback =
                    profile_traits::generic_callback::GenericCallback::new(move |result| {
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
                    })
                    .expect("Could not create callback");
                self.window
                    .as_global_scope()
                    .resource_threads()
                    .sender()
                    .send(CoreResourceMsg::ToFileManager(
                        FileManagerThreadMsg::SelectFiles(id, file_picker_request, callback),
                    ))
                    .unwrap();
            },
        }
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
        cx: &mut JSContext,
        id: EmbedderControlId,
        response: EmbedderControlResponse,
    ) {
        assert_eq!(self.window.pipeline_id(), id.pipeline_id);
        assert_eq!(self.window.webview_id(), id.webview_id);

        rooted!(&in(cx) let visible_element = self.visible_elements.borrow_mut().remove(&id.index.into()));

        let Some(element) = &*visible_element else {
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
                select_element.handle_embedder_response(cx, response);
            },
            (
                ControlElement::ColorInput(input_element),
                EmbedderControlResponse::ColorPicker(response),
            ) => {
                input_element.handle_color_picker_response(cx, response);
            },
            (
                ControlElement::FileInput(input_element),
                EmbedderControlResponse::FilePicker(response),
            ) => {
                input_element.handle_file_picker_response(cx, response);
            },
            (
                ControlElement::ContextMenu(context_menu_nodes),
                EmbedderControlResponse::ContextMenu(action),
            ) => {
                context_menu_nodes.handle_context_menu_action(action, cx);
            },
            (_, _) => unreachable!(
                "The response to a form control should always match it's originating type."
            ),
        }
    }

    pub(crate) fn show_context_menu(&self, no_gc: &NoGC, hit_test_result: &HitTestResult) {
        {
            let mut visible_elements = self.visible_elements.borrow_mut();
            visible_elements.retain(|index, control_element| {
                if matches!(control_element, ControlElement::ContextMenu(..)) {
                    let id = EmbedderControlId {
                        webview_id: self.window.webview_id(),
                        pipeline_id: self.window.pipeline_id(),
                        index: index.0,
                    };
                    self.window
                        .send_to_embedder(EmbedderMsg::HideEmbedderControl(id));
                    false
                } else {
                    true
                }
            });
        }

        let mut anchor_element = None;
        let mut image_element = None;
        for node in hit_test_result
            .node
            .inclusive_ancestors(ShadowIncluding::Yes)
        {
            if anchor_element.is_none() &&
                let Some(candidate_anchor_element) = node.downcast::<HTMLAnchorElement>() &&
                candidate_anchor_element.is_instance_activatable()
            {
                anchor_element = Some(DomRoot::from_ref(candidate_anchor_element));
            }

            if image_element.is_none() &&
                let Some(candidate_image_element) = node.downcast::<HTMLImageElement>()
            {
                image_element = Some(DomRoot::from_ref(candidate_image_element))
            }
        }

        let mut info = ContextMenuElementInformation::default();
        let mut items = Vec::new();
        if let Some(anchor_element) = anchor_element.as_ref() {
            info.flags.insert(ContextMenuElementInformationFlags::Link);
            info.link_url = anchor_element
                .full_href_url_for_user_interface(no_gc)
                .map(ServoUrl::into_url);

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

        if let Some(image_element) = image_element.as_ref() {
            info.flags.insert(ContextMenuElementInformationFlags::Image);
            info.image_url = image_element
                .full_image_url_for_user_interface()
                .map(ServoUrl::into_url);

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

        let document = self.window.Document();
        let editing_context = document.editing_context(no_gc, &hit_test_result.node);

        let has_selection = editing_context.has_uncollapsed_selection();
        if has_selection {
            info.flags
                .insert(ContextMenuElementInformationFlags::Selection);
        }

        let cutting_and_pasting_enabled = editing_context.cutting_and_pasting_enabled();
        let can_cut = has_selection && cutting_and_pasting_enabled;
        if cutting_and_pasting_enabled {
            info.flags
                .insert(ContextMenuElementInformationFlags::EditableText);
        }
        items.extend(vec![
            ContextMenuItem::Item {
                label: "Cut".into(),
                action: ContextMenuAction::Cut,
                enabled: can_cut,
            },
            ContextMenuItem::Item {
                label: "Copy".into(),
                action: ContextMenuAction::Copy,
                enabled: has_selection,
            },
            ContextMenuItem::Item {
                label: "Paste".into(),
                action: ContextMenuAction::Paste,
                enabled: cutting_and_pasting_enabled,
            },
            ContextMenuItem::Item {
                label: "Select All".into(),
                action: ContextMenuAction::SelectAll,
                enabled: editing_context.has_selectable_text(),
            },
            ContextMenuItem::Separator,
        ]);

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

        self.show_embedder_control(
            ControlElement::ContextMenu(ContextMenuNodes {
                node: hit_test_result.node.as_traced(),
                anchor_element: anchor_element.map(|element| element.as_traced()),
                image_element: image_element.map(|element| element.as_traced()),
            }),
            EmbedderControlRequest::ContextMenu(ContextMenuRequest {
                element_info: info,
                items,
            }),
            Some(hit_test_result.point_in_frame.cast_unit()),
        );
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ContextMenuNodes {
    /// The node that this menu was triggered on.
    node: Dom<Node>,
    /// The first inclusive ancestor of this node that is an `<a>` if one exists.
    anchor_element: Option<Dom<HTMLAnchorElement>>,
    /// The first inclusive ancestor of this node that is an `<img>` if one exists.
    image_element: Option<Dom<HTMLImageElement>>,
}

impl ContextMenuNodes {
    fn handle_context_menu_action(&self, action: Option<ContextMenuAction>, cx: &mut JSContext) {
        let Some(action) = action else {
            return;
        };

        let window = self.node.owner_window();
        let document = window.Document();
        let set_clipboard_text = |string: String| {
            if string.is_empty() {
                return;
            }
            window.send_to_embedder(EmbedderMsg::SetClipboardText(window.webview_id(), string));
        };

        let open_url_in_new_webview = |cx: &mut JSContext, url: ServoUrl| {
            let Some(browsing_context) = document.browsing_context() else {
                return;
            };
            let (browsing_context, new) = browsing_context.choose_a_navigable(
                cx,
                DOMString::from_static("_blank"),
                true, /* noopener */
            );
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
            let task = task!(open_link_in_new_webview: move |cx| {
                navigate(cx, &target.root(), NavigationHistoryBehavior::Replace, false, load_data);
            });
            target_document
                .owner_global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task);
        };

        match action {
            ContextMenuAction::GoBack => {
                let _ = window.History(cx).Back();
            },
            ContextMenuAction::GoForward => {
                let _ = window.History(cx).Forward();
            },
            ContextMenuAction::Reload => {
                window.Location(cx).reload_without_origin_check(cx);
            },
            ContextMenuAction::CopyLink => {
                let Some(anchor_element) = &self.anchor_element else {
                    return;
                };

                let url_string = anchor_element
                    .full_href_url_for_user_interface(cx.no_gc())
                    .as_ref()
                    .map(ServoUrl::to_string)
                    .unwrap_or_else(|| String::from(anchor_element.Href(cx.no_gc())));
                set_clipboard_text(url_string);
            },
            ContextMenuAction::OpenLinkInNewWebView => {
                let Some(anchor_element) = &self.anchor_element else {
                    return;
                };
                if let Some(url) = anchor_element.full_href_url_for_user_interface(cx.no_gc()) {
                    open_url_in_new_webview(cx, url);
                };
            },
            ContextMenuAction::CopyImageLink => {
                let Some(image_element) = &self.image_element else {
                    return;
                };
                let url_string = image_element
                    .full_image_url_for_user_interface()
                    .as_ref()
                    .map(ServoUrl::to_string)
                    .unwrap_or_else(|| String::from(image_element.CurrentSrc()));
                set_clipboard_text(url_string);
            },
            ContextMenuAction::OpenImageInNewView => {
                let Some(image_element) = &self.image_element else {
                    return;
                };
                if let Some(url) = image_element.full_image_url_for_user_interface() {
                    open_url_in_new_webview(cx, url);
                }
            },
            ContextMenuAction::Cut => {
                let editing_context = document.editing_context(cx.no_gc(), &self.node);
                document.handle_clipboard_action(cx, &editing_context, ClipboardAction::Cut);
            },
            ContextMenuAction::Copy => {
                let editing_context = document.editing_context(cx.no_gc(), &self.node);
                document.handle_clipboard_action(cx, &editing_context, ClipboardAction::Copy);
            },
            ContextMenuAction::Paste => {
                let editing_context = document.editing_context(cx.no_gc(), &self.node);
                document.handle_clipboard_action(cx, &editing_context, ClipboardAction::Paste);
            },
            ContextMenuAction::SelectAll => {
                let editing_context = document.editing_context(cx.no_gc(), &self.node);
                editing_context.select_all(cx);
            },
        }
    }
}
