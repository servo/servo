/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;

use data_url::mime::Mime;
use dom_struct::dom_struct;
use js::rust::{HandleObject, MutableHandleValue};
use script_bindings::record::Record;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ClipboardBinding::{
    ClipboardItemMethods, ClipboardItemOptions, PresentationStyle,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext};

/// <https://w3c.github.io/clipboard-apis/#web-custom-format>
const CUSTOM_FORMAT_PREFIX: &str = "web ";

/// <https://w3c.github.io/clipboard-apis/#representation>
#[derive(JSTraceable, MallocSizeOf)]
pub(super) struct Representation {
    #[no_trace]
    #[ignore_malloc_size_of = "Extern type"]
    pub mime_type: Mime,
    pub is_custom: bool,
    #[ignore_malloc_size_of = "Rc is hard"]
    pub data: Rc<Promise>,
}

#[dom_struct]
pub(crate) struct ClipboardItem {
    reflector_: Reflector,
    representations: DomRefCell<Vec<Representation>>,
    presentation_style: DomRefCell<PresentationStyle>,
    #[ignore_malloc_size_of = "mozjs"]
    frozen_types: CachedFrozenArray,
}

impl ClipboardItem {
    fn new_inherited() -> ClipboardItem {
        ClipboardItem {
            reflector_: Reflector::new(),
            representations: Default::default(),
            presentation_style: Default::default(),
            frozen_types: CachedFrozenArray::new(),
        }
    }

    fn new(window: &Window, proto: Option<HandleObject>, can_gc: CanGc) -> DomRoot<ClipboardItem> {
        reflect_dom_object_with_proto(
            Box::new(ClipboardItem::new_inherited()),
            window,
            proto,
            can_gc,
        )
    }
}

impl ClipboardItemMethods<crate::DomTypeHolder> for ClipboardItem {
    /// <https://w3c.github.io/clipboard-apis/#dom-clipboarditem-clipboarditem>
    fn Constructor(
        global: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        items: Record<DOMString, Rc<Promise>>,
        options: &ClipboardItemOptions,
    ) -> Fallible<DomRoot<ClipboardItem>> {
        // Step 1 If items is empty, then throw a TypeError.
        if items.is_empty() {
            return Err(Error::Type(String::from("No item provided")));
        }

        // Step 2 If options is empty, then set options["presentationStyle"] = "unspecified".
        // NOTE: This is done inside bindings

        // Step 3 Set this's clipboard item to a new clipboard item.
        let clipboard_item = ClipboardItem::new(global, proto, can_gc);

        // Step 4 Set this's clipboard item's presentation style to options["presentationStyle"].
        *clipboard_item.presentation_style.borrow_mut() = options.presentationStyle;

        // Step 6 For each (key, value) in items:
        for (key, value) in items.deref() {
            // Step 6.2 Let isCustom be false.

            // Step 6.3 If key starts with `"web "` prefix, then
            // Step 6.3.1 Remove `"web "` prefix and assign the remaining string to key.
            let (key, is_custom) = match key.strip_prefix(CUSTOM_FORMAT_PREFIX) {
                None => (key.str(), false),
                // Step 6.3.2 Set isCustom true
                Some(stripped) => (stripped, true),
            };

            // Step 6.5 Let mimeType be the result of parsing a MIME type given key.
            // Step 6.6 If mimeType is failure, then throw a TypeError.
            let mime_type =
                Mime::from_str(key).map_err(|_| Error::Type(String::from("Invalid mime type")))?;

            // Step 6.7 If this's clipboard item's list of representations contains a representation
            // whose MIME type is mimeType and whose [representation/isCustom] is isCustom, then throw a TypeError.
            if clipboard_item
                .representations
                .borrow()
                .iter()
                .any(|representation| {
                    representation.mime_type == mime_type && representation.is_custom == is_custom
                })
            {
                return Err(Error::Type(String::from("Tried to add a duplicate mime")));
            }

            // Step 6.1 Let representation be a new representation.
            // Step 6.4 Set representation’s isCustom flag to isCustom.
            // Step 6.8 Set representation’s MIME type to mimeType.
            // Step 6.9 Set representation’s data to value.
            let representation = Representation {
                mime_type,
                is_custom,
                data: value.clone(),
            };

            // Step 6.10 Append representation to this's clipboard item's list of representations.
            clipboard_item
                .representations
                .borrow_mut()
                .push(representation);
        }

        // NOTE: The steps for creating a frozen array from the list of mimeType are done in the Types() method

        Ok(clipboard_item)
    }

    /// <https://w3c.github.io/clipboard-apis/#dom-clipboarditem-presentationstyle>
    fn PresentationStyle(&self) -> PresentationStyle {
        *self.presentation_style.borrow()
    }

    /// <https://w3c.github.io/clipboard-apis/#dom-clipboarditem-types>
    fn Types(&self, cx: JSContext, can_gc: CanGc, retval: MutableHandleValue) {
        self.frozen_types.get_or_init(
            || {
                // Step 5 Let types be a list of DOMString.
                let mut types = Vec::new();

                self.representations
                    .borrow()
                    .iter()
                    .for_each(|representation| {
                        // Step 6.11 Let mimeTypeString be the result of serializing a MIME type with mimeType.
                        let mime_type_string = representation.mime_type.to_string();

                        // Step 6.12 If isCustom is true, prefix mimeTypeString with `"web "`.
                        let mime_type_string = if representation.is_custom {
                            format!("{}{}", CUSTOM_FORMAT_PREFIX, mime_type_string)
                        } else {
                            mime_type_string
                        };

                        // Step 6.13 Add mimeTypeString to types.
                        types.push(DOMString::from(mime_type_string));
                    });
                types
            },
            cx,
            retval,
            can_gc,
        );
    }
}
