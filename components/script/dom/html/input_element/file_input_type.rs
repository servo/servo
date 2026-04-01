/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::Ref;
use std::path::PathBuf;
use std::str::FromStr;

use embedder_traits::{EmbedderControlRequest, FilePickerRequest, FilterPattern, SelectedFile};
use html5ever::{local_name, ns};
use js::context::JSContext;
use markup5ever::QualName;
use script_bindings::codegen::GenericBindings::FileListBinding::FileListMethods;
use script_bindings::codegen::GenericBindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use script_bindings::codegen::GenericBindings::HTMLElementBinding::HTMLElementMethods;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::domstring::DOMString;
use script_bindings::inheritance::Castable;
use script_bindings::root::Dom;
use script_bindings::script_runtime::CanGc;
use style::selector_parser::PseudoElement;
use style::str::split_commas;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::document_embedder_controls::ControlElement;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventComposed};
use crate::dom::eventtarget::EventTarget;
use crate::dom::file::File;
use crate::dom::filelist::FileList;
use crate::dom::htmlbuttonelement::HTMLButtonElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::input_element::HTMLInputElement;
use crate::dom::input_element::input_type::SpecificInputType;
use crate::dom::node::{Node, NodeTraits};

const DEFAULT_FILE_INPUT_VALUE: &str = "No file chosen";
const DEFAULT_FILE_INPUT_MULTIPLE_VALUE: &str = "No files chosen";
const SELECTOR_BUTTON_TEXT: &str = "Choose file";
const SELECTOR_BUTTON_MULTIPLE_TEXT: &str = "Choose files";

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct FileInputType {
    filelist: MutNullableDom<FileList>,
    shadow_tree: DomRefCell<Option<FileInputShadowTree>>,
}

impl FileInputType {
    /// Get the shadow tree for this [`HTMLInputElement`], if it is created and valid, otherwise
    /// recreate the shadow tree and return it.
    fn get_or_create_shadow_tree(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
    ) -> Ref<'_, FileInputShadowTree> {
        {
            if let Ok(shadow_tree) = Ref::filter_map(self.shadow_tree.borrow(), |shadow_tree| {
                shadow_tree.as_ref()
            }) {
                return shadow_tree;
            }
        }

        let element = input.upcast::<Element>();
        let shadow_root = element
            .shadow_root()
            .unwrap_or_else(|| element.attach_ua_shadow_root(cx, true));
        let shadow_root = shadow_root.upcast();
        *self.shadow_tree.borrow_mut() = Some(FileInputShadowTree::new(cx, shadow_root));
        self.get_or_create_shadow_tree(cx, input)
    }

    pub(crate) fn handle_file_picker_response(
        &self,
        input: &HTMLInputElement,
        response: Option<Vec<SelectedFile>>,
        can_gc: CanGc,
    ) {
        let mut files = Vec::new();

        if let Some(pending_webdriver_reponse) =
            input.pending_webdriver_response.borrow_mut().take()
        {
            // From: <https://w3c.github.io/webdriver/#dfn-dispatch-actions-for-a-string>
            // "Complete implementation specific steps equivalent to setting the selected
            // files on the input element. If multiple is true files are be appended to
            // element's selected files."
            //
            // Note: This is annoying.
            if input.Multiple() {
                if let Some(filelist) = self.get_files() {
                    files = filelist.iter_files().map(|file| file.as_rooted()).collect();
                }
            }

            let number_files_selected = response.as_ref().map(Vec::len).unwrap_or_default();
            pending_webdriver_reponse.finish(number_files_selected);
        }

        let Some(response_files) = response else {
            return;
        };

        let window = input.owner_window();
        files.extend(
            response_files
                .into_iter()
                .map(|file| File::new_from_selected(&window, file, can_gc)),
        );

        // Only use the last file if this isn't a multi-select file input. This could
        // happen if the attribute changed after the file dialog was initiated.
        if !input.Multiple() {
            files = files
                .pop()
                .map(|last_file| vec![last_file])
                .unwrap_or_default();
        }

        self.set_files(&FileList::new(&window, files, can_gc));

        let target = input.upcast::<EventTarget>();
        target.fire_event_with_params(
            atom!("input"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            EventComposed::Composed,
            can_gc,
        );
        target.fire_bubbling_event(atom!("change"), can_gc);
    }
}

impl SpecificInputType for FileInputType {
    /// <https://html.spec.whatwg.org/multipage/#file-upload-state-(type=file):suffering-from-being-missing>
    fn suffers_from_being_missing(&self, input: &HTMLInputElement, _value: &DOMString) -> bool {
        input.Required() && self.filelist.get().is_none_or(|files| files.Length() == 0)
    }

    fn value_for_shadow_dom(&self, input: &HTMLInputElement) -> DOMString {
        let Some(filelist) = self.filelist.get() else {
            if input.Multiple() {
                return DEFAULT_FILE_INPUT_MULTIPLE_VALUE.into();
            }
            return DEFAULT_FILE_INPUT_VALUE.into();
        };
        let length = filelist.Length();
        if length > 1 {
            return format!("{length} files").into();
        }

        let Some(first_item) = filelist.Item(0) else {
            if input.Multiple() {
                return DEFAULT_FILE_INPUT_MULTIPLE_VALUE.into();
            }
            return DEFAULT_FILE_INPUT_VALUE.into();
        };
        first_item.name().to_string().into()
    }

    /// <https://html.spec.whatwg.org/multipage/#file-upload-state-(type=file):input-activation-behavior>
    fn activation_behavior(
        &self,
        input: &HTMLInputElement,
        _event: &Event,
        _target: &EventTarget,
        _can_gc: CanGc,
    ) {
        input.show_the_picker_if_applicable();
    }

    fn show_the_picker_if_applicable(&self, input: &HTMLInputElement) {
        self.select_files(input, None)
    }

    /// Select files by invoking UI or by passed in argument.
    ///
    /// <https://html.spec.whatwg.org/multipage/#file-upload-state-(type=file)>
    fn select_files(&self, input: &HTMLInputElement, test_paths: Option<Vec<DOMString>>) {
        let current_paths = match &test_paths {
            Some(test_paths) => test_paths
                .iter()
                .filter_map(|path_str| PathBuf::from_str(&path_str.str()).ok())
                .collect(),
            // TODO: This should get the pathnames of the current files, but we currently don't have
            // that information in Script. It should be passed through here.
            None => Default::default(),
        };

        let accept_current_paths_for_testing = test_paths.is_some();
        input
            .owner_document()
            .embedder_controls()
            .show_embedder_control(
                ControlElement::FileInput(DomRoot::from_ref(input)),
                EmbedderControlRequest::FilePicker(FilePickerRequest {
                    origin: input.owner_window().origin().immutable().clone(),
                    current_paths,
                    filter_patterns: filter_from_accept(&input.Accept()),
                    allow_select_multiple: input.Multiple(),
                    accept_current_paths_for_testing,
                }),
                None,
            );
    }

    fn get_files(&self) -> Option<DomRoot<FileList>> {
        self.filelist.get()
    }

    fn set_files(&self, filelist: &FileList) {
        self.filelist.set(Some(filelist))
    }

    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.get_or_create_shadow_tree(cx, input).update(
            cx,
            self.value_for_shadow_dom(input),
            input.Multiple(),
        )
    }
}

/// <https://html.spec.whatwg.org/multipage/#attr-input-accept>
fn filter_from_accept(s: &DOMString) -> Vec<FilterPattern> {
    let mut filter = vec![];
    for p in split_commas(&s.str()) {
        let p = p.trim();
        if let Some('.') = p.chars().next() {
            filter.push(FilterPattern(p[1..].to_string()));
        } else if let Some(exts) = mime_guess::get_mime_extensions_str(p) {
            for ext in exts {
                filter.push(FilterPattern(ext.to_string()));
            }
        }
    }

    filter
}

#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// Contains references to the elements in the shadow tree for `<input type=file>`.
///
/// The shadow tree consists of the file selector button and a span for the chosen files text.
pub(crate) struct FileInputShadowTree {
    selector_button: Dom<Element>,
    value_container: Dom<Element>,
}

impl FileInputShadowTree {
    pub(crate) fn new(cx: &mut JSContext, shadow_root: &Node) -> Self {
        let selector_button = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("button")),
            None,
            &shadow_root.owner_document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        selector_button
            .downcast::<HTMLButtonElement>()
            .expect("This should be guaranteed by the element type used above")
            .SetType(DOMString::from("button"));

        selector_button
            .downcast::<HTMLElement>()
            .expect("This should be guaranteed by the element type used above")
            .SetTabIndex(-1, CanGc::from_cx(cx));

        let _ = shadow_root.AppendChild(cx, selector_button.upcast());

        selector_button
            .upcast::<Node>()
            .set_implemented_pseudo_element(PseudoElement::FileSelectorButton);

        let value_container = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("span")),
            None,
            &shadow_root.owner_document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        let _ = shadow_root.AppendChild(cx, value_container.upcast());

        Self {
            selector_button: selector_button.as_traced(),
            value_container: value_container.as_traced(),
        }
    }

    pub(crate) fn update(&self, cx: &mut JSContext, input_value: DOMString, multiple: bool) {
        if multiple {
            self.selector_button
                .upcast::<Node>()
                .set_text_content_for_element(
                    cx,
                    Some(DOMString::from(SELECTOR_BUTTON_MULTIPLE_TEXT)),
                );
        } else {
            self.selector_button
                .upcast::<Node>()
                .set_text_content_for_element(cx, Some(DOMString::from(SELECTOR_BUTTON_TEXT)));
        }

        self.value_container
            .upcast::<Node>()
            .set_text_content_for_element(cx, Some(input_value));
    }
}
