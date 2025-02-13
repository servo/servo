/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use indexmap::IndexMap;
use pixels::Image;
use script_traits::serializable::BlobImpl;

use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

/// <https://html.spec.whatwg.org/multipage/#the-drag-data-item-kind>
pub(crate) enum Kind {
    Text {
        data: DOMString,
        type_: DOMString,
    },
    File {
        bytes: Vec<u8>,
        name: DOMString,
        type_: String,
    },
}

impl Kind {
    pub(crate) fn type_(&self) -> DOMString {
        match self {
            Kind::Text { type_, .. } => type_.clone(),
            Kind::File { type_, .. } => DOMString::from(type_.clone()),
        }
    }

    pub(crate) fn as_string(&self) -> Option<String> {
        match self {
            Kind::Text { data, .. } => Some(data.to_string()),
            Kind::File { .. } => None,
        }
    }

    // TODO for now we create a new BlobImpl
    // since File constructor requires moving it.
    pub(crate) fn as_file(&self, global: &GlobalScope, can_gc: CanGc) -> Option<DomRoot<File>> {
        match self {
            Kind::Text { .. } => None,
            Kind::File { bytes, name, type_ } => Some(File::new(
                global,
                BlobImpl::new_from_bytes(bytes.clone(), type_.clone()),
                name.clone(),
                None,
                can_gc,
            )),
        }
    }

    fn text_type_matches(&self, text_type: &str) -> bool {
        matches!(self, Kind::Text { type_, .. } if type_.eq(text_type))
    }

    fn is_file(&self) -> bool {
        matches!(self, Kind::File { .. })
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
    ReadOnly,
    /// <https://html.spec.whatwg.org/multipage/#concept-dnd-p>
    Protected,
}

#[allow(dead_code)] // TODO some fields are used by DragEvent.
pub(crate) struct DragDataStore {
    /// <https://html.spec.whatwg.org/multipage/#drag-data-store-item-list>
    item_list: IndexMap<u16, Kind>,
    next_item_id: u16,
    /// <https://html.spec.whatwg.org/multipage/#drag-data-store-default-feedback>
    default_feedback: Option<String>,
    bitmap: Option<Bitmap>,
    mode: Mode,
    /// <https://html.spec.whatwg.org/multipage/#drag-data-store-allowed-effects-state>
    allowed_effects_state: String,
    pub clear_was_called: bool,
}

impl DragDataStore {
    /// <https://html.spec.whatwg.org/multipage/#create-a-drag-data-store>
    // We don't really need it since it's only instantiated by DataTransfer.
    #[allow(clippy::new_without_default)]
    pub(crate) fn new() -> DragDataStore {
        DragDataStore {
            item_list: IndexMap::new(),
            next_item_id: 0,
            default_feedback: None,
            bitmap: None,
            mode: Mode::Protected,
            allowed_effects_state: String::from("uninitialized"),
            clear_was_called: false,
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

        let has_files = self.item_list.values().fold(false, |has_files, item| {
            // Step 2.1 For each item in the item list whose kind is text,
            // add an entry to L consisting of the item's type string.
            match item {
                Kind::Text { type_, .. } => types.push(type_.clone()),
                Kind::File { .. } => return true,
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
            .values()
            .find(|item| item.text_type_matches(type_))
            .and_then(|item| item.as_string())
            .map(DOMString::from)
    }

    pub(crate) fn add(&mut self, kind: Kind) -> Fallible<u16> {
        if let Kind::Text { ref type_, .. } = kind {
            // Step 2.1 If there is already an item in the item list whose kind is text
            // and whose type string is equal to the method's second argument, throw "NotSupportedError".
            if self
                .item_list
                .values()
                .any(|item| item.text_type_matches(type_))
            {
                return Err(Error::NotSupported);
            }
        }

        let item_id = self.next_item_id;

        // Step 2.2
        self.item_list.insert(item_id, kind);

        self.next_item_id += 1;
        Ok(item_id)
    }

    pub(crate) fn set_data(&mut self, format: DOMString, data: DOMString) {
        // Step 3-4
        let type_ = normalize_mime(format);

        // Step 5 Remove the item in the drag data store item list whose kind is text
        // and whose type string is equal to format, if there is one.
        self.item_list
            .retain(|_, item| !item.text_type_matches(&type_));

        // Step 6 Add an item whose kind is text, whose type is format, and whose data is the method's second argument.
        self.item_list
            .insert(self.next_item_id, Kind::Text { data, type_ });
        self.next_item_id += 1;
    }

    pub(crate) fn clear_data(&mut self, format: Option<DOMString>) -> bool {
        let mut was_modified = false;

        if let Some(format) = format {
            // Step 4-5
            let type_ = normalize_mime(format);

            // Step 6 Remove the item in the item list whose kind is text and whose type is format.
            self.item_list.retain(|_, item| {
                let matches = item.text_type_matches(&type_);

                if matches {
                    was_modified = true;
                }
                !matches
            });
        } else {
            // Step 3 Remove each item in the item list whose kind is text.
            self.item_list.retain(|_, item| {
                let matches = item.is_file();

                if !matches {
                    was_modified = true;
                }
                matches
            });
        }

        was_modified
    }

    pub(crate) fn files(
        &self,
        global: &GlobalScope,
        can_gc: CanGc,
        file_list: &mut Vec<DomRoot<File>>,
    ) {
        // Step 3 If the data store is in the protected mode return the empty list.
        if self.mode == Mode::Protected {
            return;
        }

        // Step 4 For each item in the drag data store item list whose kind is File, add the item's data to the list L.
        self.item_list
            .values()
            .filter_map(|item| item.as_file(global, can_gc))
            .for_each(|file| file_list.push(file));
    }

    pub(crate) fn list_len(&self) -> usize {
        self.item_list.len()
    }

    pub(crate) fn iter_item_list(&self) -> indexmap::map::Values<'_, u16, Kind> {
        self.item_list.values()
    }

    pub(crate) fn get_by_index(&self, index: usize) -> Option<(&u16, &Kind)> {
        self.item_list.get_index(index)
    }

    pub(crate) fn get_by_id(&self, id: &u16) -> Option<&Kind> {
        self.item_list.get(id)
    }

    pub(crate) fn remove(&mut self, index: usize) {
        self.item_list.shift_remove_index(index);
    }

    pub(crate) fn clear_list(&mut self) {
        self.item_list.clear();
        self.clear_was_called = true;
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
