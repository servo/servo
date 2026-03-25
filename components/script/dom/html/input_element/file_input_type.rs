/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::path::PathBuf;
use std::str::FromStr;

use embedder_traits::{EmbedderControlRequest, FilePickerRequest, FilterPattern, SelectedFile};
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::FileListBinding::FileListMethods;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::domstring::DOMString;
use script_bindings::inheritance::Castable;
use script_bindings::script_runtime::CanGc;
use style::str::split_commas;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::document_embedder_controls::ControlElement;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventComposed};
use crate::dom::eventtarget::EventTarget;
use crate::dom::file::File;
use crate::dom::filelist::FileList;
use crate::dom::htmlinputelement::text_value_widget::TextValueWidget;
use crate::dom::input_element::HTMLInputElement;
use crate::dom::input_element::input_type::SpecificInputType;
use crate::dom::node::NodeTraits;

const DEFAULT_FILE_INPUT_VALUE: &str = "No file chosen";
const DEFAULT_FILE_INPUT_MULTIPLE_VALUE: &str = "No files chosen";

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct FileInputType {
    filelist: MutNullableDom<FileList>,
    text_value_widget: DomRefCell<TextValueWidget>,
}

impl FileInputType {
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
        self.text_value_widget
            .borrow()
            .update_shadow_tree(cx, input)
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
