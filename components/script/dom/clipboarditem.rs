/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;

use constellation_traits::BlobImpl;
use data_url::mime::Mime;
use dom_struct::dom_struct;
use js::rust::{HandleObject, HandleValue as SafeHandleValue, MutableHandleValue};
use script_bindings::record::Record;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ClipboardBinding::{
    ClipboardItemMethods, ClipboardItemOptions, PresentationStyle,
};
use crate::dom::bindings::conversions::{
    ConversionResult, SafeFromJSValConvertible, StringificationBehavior,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::blob::Blob;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::window::Window;
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// The fulfillment handler for the reacting to representationDataPromise part of
/// <https://w3c.github.io/clipboard-apis/#dom-clipboarditem-gettype>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct RepresentationDataPromiseFulfillmentHandler {
    #[conditional_malloc_size_of]
    promise: Rc<Promise>,
    type_: String,
}

impl Callback for RepresentationDataPromiseFulfillmentHandler {
    /// Substeps of 8.1.2.1 If representationDataPromise was fulfilled with value v, then:
    fn callback(&self, cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // 1. If v is a DOMString, then follow the below steps:
        if v.get().is_string() {
            // 1.1 Let dataAsBytes be the result of UTF-8 encoding v.
            let data_as_bytes =
                match DOMString::safe_from_jsval(cx, v, StringificationBehavior::Default) {
                    Ok(ConversionResult::Success(s)) => s.as_bytes().to_owned(),
                    _ => return,
                };

            // 1.2 Let blobData be a Blob created using dataAsBytes with its type set to mimeType, serialized.
            let blob_data = Blob::new(
                &self.promise.global(),
                BlobImpl::new_from_bytes(data_as_bytes, self.type_.clone()),
                can_gc,
            );

            // 1.3 Resolve p with blobData.
            self.promise.resolve_native(&blob_data, can_gc);
        }
        // 2. If v is a Blob, then follow the below steps:
        else if DomRoot::<Blob>::safe_from_jsval(cx, v, ())
            .is_ok_and(|result| result.get_success_value().is_some())
        {
            // 2.1 Resolve p with v.
            self.promise.resolve(cx, v, can_gc);
        }
    }
}

/// The rejection handler for the reacting to representationDataPromise part of
/// <https://w3c.github.io/clipboard-apis/#dom-clipboarditem-gettype>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct RepresentationDataPromiseRejectionHandler {
    #[conditional_malloc_size_of]
    promise: Rc<Promise>,
}

impl Callback for RepresentationDataPromiseRejectionHandler {
    /// Substeps of 8.1.2.2 If representationDataPromise was rejected, then:
    fn callback(&self, _cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        // 1. Reject p with "NotFoundError" DOMException in realm.
        self.promise.reject_error(Error::NotFound(None), can_gc);
    }
}

/// <https://w3c.github.io/clipboard-apis/#web-custom-format>
const CUSTOM_FORMAT_PREFIX: &str = "web ";

/// <https://w3c.github.io/clipboard-apis/#representation>
#[derive(JSTraceable, MallocSizeOf)]
pub(super) struct Representation {
    #[no_trace]
    #[ignore_malloc_size_of = "Extern type"]
    pub mime_type: Mime,
    pub is_custom: bool,
    #[conditional_malloc_size_of]
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
            let (key, is_custom) = match key.str().strip_prefix(CUSTOM_FORMAT_PREFIX) {
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
    fn Types(&self, cx: SafeJSContext, can_gc: CanGc, retval: MutableHandleValue) {
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

    /// <https://w3c.github.io/clipboard-apis/#dom-clipboarditem-gettype>
    fn GetType(&self, type_: DOMString, can_gc: CanGc) -> Fallible<Rc<Promise>> {
        // Step 1 Let realm be this’s relevant realm.
        let global = self.global();

        // Step 2 Let isCustom be false.

        // Step 3 If type starts with `"web "` prefix, then:
        // Step 3.1 Remove `"web "` prefix and assign the remaining string to type.
        let (type_, is_custom) = match type_.strip_prefix(CUSTOM_FORMAT_PREFIX) {
            None => (type_.str(), false),
            // Step 3.2 Set isCustom to true.
            Some(stripped) => (stripped, true),
        };

        // Step 4 Let mimeType be the result of parsing a MIME type given type.
        // Step 5 If mimeType is failure, then throw a TypeError.
        let mime_type =
            Mime::from_str(type_).map_err(|_| Error::Type(String::from("Invalid mime type")))?;

        // Step 6 Let itemTypeList be this’s clipboard item’s list of representations.
        let item_type_list = self.representations.borrow();

        // Step 7 Let p be a new promise in realm.
        let p = Promise::new(&global, can_gc);

        // Step 8 For each representation in itemTypeList
        for representation in item_type_list.iter() {
            // Step 8.1 If representation’s MIME type is mimeType and representation’s isCustom is isCustom, then:
            if representation.mime_type == mime_type && representation.is_custom == is_custom {
                // Step 8.1.1 Let representationDataPromise be the representation’s data.
                let representation_data_promise = &representation.data;

                // Step 8.1.2 React to representationDataPromise:
                let fulfillment_handler = Box::new(RepresentationDataPromiseFulfillmentHandler {
                    promise: p.clone(),
                    type_: representation.mime_type.to_string(),
                });
                let rejection_handler =
                    Box::new(RepresentationDataPromiseRejectionHandler { promise: p.clone() });
                let handler = PromiseNativeHandler::new(
                    &global,
                    Some(fulfillment_handler),
                    Some(rejection_handler),
                    can_gc,
                );
                let realm = enter_realm(&*global);
                let comp = InRealm::Entered(&realm);
                representation_data_promise.append_native_handler(&handler, comp, can_gc);

                // Step 8.1.3 Return p.
                return Ok(p);
            }
        }

        // Step 9 Reject p with "NotFoundError" DOMException in realm.
        p.reject_error(Error::NotFound(None), can_gc);

        // Step 10 Return p.
        Ok(p)
    }

    /// <https://w3c.github.io/clipboard-apis/#dom-clipboarditem-supports>
    fn Supports(_: &Window, type_: DOMString) -> bool {
        // TODO Step 1 If type is in mandatory data types or optional data types, then return true.
        // Step 2 If not, then return false.
        // NOTE: We only supports text/plain
        type_ == "text/plain"
    }
}
