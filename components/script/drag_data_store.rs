/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use pixels::Image;
use script_traits::serializable::BlobImpl;

use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

/// <https://html.spec.whatwg.org/multipage/#the-drag-data-item-kind>
#[derive(Clone)]
pub(crate) enum Kind {
    Text(PlainString),
    File(Binary),
}

#[derive(Clone)]
pub(crate) struct PlainString {
    data: DOMString,
    type_: DOMString,
}

impl PlainString {
    pub(crate) fn new(data: DOMString, type_: DOMString) -> Self {
        Self { data, type_ }
    }
}

#[derive(Clone)]
pub(crate) struct Binary {
    bytes: Vec<u8>,
    name: DOMString,
    type_: String,
}

impl Binary {
    pub(crate) fn new(bytes: Vec<u8>, name: DOMString, type_: String) -> Self {
        Self { bytes, name, type_ }
    }
}

impl Kind {
    pub(crate) fn type_(&self) -> DOMString {
        match self {
            Kind::Text(string) => string.type_.clone(),
            Kind::File(binary) => DOMString::from(binary.type_.clone()),
        }
    }

    pub(crate) fn as_string(&self) -> Option<DOMString> {
        match self {
            Kind::Text(string) => Some(string.data.clone()),
            Kind::File(_) => None,
        }
    }

    // TODO for now we create a new BlobImpl
    // since File constructor requires moving it.
    pub(crate) fn as_file(&self, global: &GlobalScope) -> Option<DomRoot<File>> {
        match self {
            Kind::Text(_) => None,
            Kind::File(binary) => Some(File::new(
                global,
                BlobImpl::new_from_bytes(binary.bytes.clone(), binary.type_.clone()),
                binary.name.clone(),
                None,
                CanGc::note(),
            )),
        }
    }

    fn text_type_matches(&self, type_: &str) -> bool {
        matches!(self, Kind::Text(string) if string.type_.eq(type_))
    }

    fn is_file(&self) -> bool {
        matches!(self, Kind::File(_))
    }
}

/// <https://html.spec.whatwg.org/multipage/#drag-data-store-bitmap>
#[allow(dead_code)] // TODO this used by DragEvent.
struct Bitmap {
    image: Option<Arc<Image>>,
    x: i32,
    y: i32,
}

/// Control the behaviour of the drag data store
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Mode {
    /// <https://html.spec.whatwg.org/multipage/#concept-dnd-rw>
    ReadWrite,
    /// <https://html.spec.whatwg.org/multipage/#concept-dnd-ro>
    #[allow(dead_code)] // TODO this used by ClipboardEvent.
    ReadOnly,
    /// <https://html.spec.whatwg.org/multipage/#concept-dnd-p>
    Protected,
}

#[allow(dead_code)] // TODO some fields are used by DragEvent.
pub(crate) struct DragDataStore {
    /// <https://html.spec.whatwg.org/multipage/#drag-data-store-item-list>
    item_list: Vec<Kind>,
    /// <https://html.spec.whatwg.org/multipage/#drag-data-store-default-feedback>
    default_feedback: Option<String>,
    bitmap: Option<Bitmap>,
    mode: Mode,
    /// <https://html.spec.whatwg.org/multipage/#drag-data-store-allowed-effects-state>
    allowed_effects_state: String,
}

impl DragDataStore {
    /// <https://html.spec.whatwg.org/multipage/#create-a-drag-data-store>
    // We don't really need it since it's only instantiated by DataTransfer.
    #[allow(clippy::new_without_default)]
    pub(crate) fn new() -> DragDataStore {
        DragDataStore {
            item_list: Vec::new(),
            default_feedback: None,
            bitmap: None,
            mode: Mode::Protected,
            allowed_effects_state: String::from("uninitialized"),
        }
    }

    /// Get the drag data store mode
    pub(crate) fn mode(&self) -> Mode {
        self.mode
    }

    /// Set the drag data store mode
    pub(crate) fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub(crate) fn set_bitmap(&mut self, image: Option<Arc<Image>>, x: i32, y: i32) {
        self.bitmap = Some(Bitmap { image, x, y });
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-datatransfer-types>
    pub(crate) fn types(&self) -> Vec<DOMString> {
        let mut types = Vec::new();

        let has_files = self.item_list.iter().fold(false, |has_files, item| {
            // Step 2.1 For each item in the item list whose kind is text,
            // add an entry to L consisting of the item's type string.
            match item {
                Kind::Text(string) => types.push(string.type_.clone()),
                Kind::File(_) => return true,
            }

            has_files
        });

        // Step 2.2 If there are any items in the item list whose kind is File,
        // add an entry to L consisting of the string "Files".
        if has_files {
            types.push(DOMString::from("Files"));
        }
        types
    }

    pub(crate) fn find_matching_text(&self, type_: &str) -> Option<DOMString> {
        self.item_list
            .iter()
            .find(|item| item.text_type_matches(type_))
            .and_then(|item| item.as_string())
    }

    pub(crate) fn add(&mut self, kind: Kind) -> Fallible<()> {
        if let Kind::Text(ref string) = kind {
            // Step 2.1 If there is already an item in the item list whose kind is text
            // and whose type string is equal to the method's second argument, throw "NotSupportedError".
            if self
                .item_list
                .iter()
                .any(|item| item.text_type_matches(&string.type_))
            {
                return Err(Error::NotSupported);
            }
        }

        // Step 2.2
        self.item_list.push(kind);

        Ok(())
    }

    pub(crate) fn set_data(&mut self, format: DOMString, data: DOMString) {
        // Step 3-4
        let type_ = normalize_mime(format);

        // Step 5 Remove the item in the drag data store item list whose kind is text
        // and whose type string is equal to format, if there is one.
        self.item_list
            .retain(|item| !item.text_type_matches(&type_));

        // Step 6 Add an item whose kind is text, whose type is format, and whose data is the method's second argument.
        self.item_list.push(Kind::Text(PlainString { data, type_ }));
    }

    pub(crate) fn clear_data(&mut self, format: Option<DOMString>) -> bool {
        let mut was_modified = false;

        if let Some(format) = format {
            // Step 4-5
            let type_ = normalize_mime(format);

            // Step 6 Remove the item in the item list whose kind is text and whose type is format.
            self.item_list.retain(|item| {
                let matches = item.text_type_matches(&type_);

                if matches {
                    was_modified = true;
                }
                !matches
            });
        } else {
            // Step 3 Remove each item in the item list whose kind is text.
            self.item_list.retain(|item| {
                let matches = item.is_file();

                if !matches {
                    was_modified = true;
                }
                matches
            });
        }

        was_modified
    }

    pub(crate) fn files(&self, global: &GlobalScope, file_list: &mut Vec<DomRoot<File>>) {
        // Step 3 If the data store is in the protected mode return the empty list.
        if self.mode == Mode::Protected {
            return;
        }

        // Step 4 For each item in the drag data store item list whose kind is File, add the item's data to the list L.
        self.item_list
            .iter()
            .filter_map(|item| item.as_file(global))
            .for_each(|file| file_list.push(file));
    }

    pub(crate) fn list_len(&self) -> usize {
        self.item_list.len()
    }

    pub(crate) fn get_item(&self, index: usize) -> Option<Kind> {
        self.item_list.get(index).cloned()
    }

    pub(crate) fn remove(&mut self, index: usize) {
        self.item_list.remove(index);
    }

    pub(crate) fn clear_list(&mut self) {
        self.item_list.clear();
    }
}

fn normalize_mime(mut format: DOMString) -> DOMString {
    // Convert format to ASCII lowercase.
    format.make_ascii_lowercase();

    match format.as_ref() {
        // If format equals "text", change it to "text/plain".
        "text" => DOMString::from("text/plain"),
        // If format equals "url", change it to "text/uri-list".
        "url" => DOMString::from("text/uri-list"),
        s => DOMString::from(s),
    }
}
