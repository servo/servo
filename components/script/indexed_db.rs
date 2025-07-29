/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::iter::repeat;
use std::ptr;

use js::gc::MutableHandle;
use js::jsapi::{
    ESClass, GetBuiltinClass, IsArrayBufferObject, JS_DeleteUCProperty,
    JS_GetOwnUCPropertyDescriptor, JS_GetStringLength, JS_IsArrayBufferViewObject, JSObject,
    ObjectOpResult, ObjectOpResult_SpecialCodes, PropertyDescriptor,
};
use js::jsval::{DoubleValue, UndefinedValue};
use js::rust::{HandleValue, MutableHandleValue};
use net_traits::indexeddb_thread::{IndexedDBKeyRange, IndexedDBKeyType};
use script_bindings::conversions::{SafeToJSValConvertible, root_from_object};
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence as StrOrStringSequence;
use crate::dom::bindings::conversions::jsstring_to_str;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::SafeJSContext;
use crate::dom::bindings::structuredclone;
use crate::dom::idbkeyrange::IDBKeyRange;
use crate::dom::idbobjectstore::KeyPath;

// https://www.w3.org/TR/IndexedDB-2/#convert-key-to-value
#[allow(unsafe_code)]
pub fn key_type_to_jsval(
    cx: SafeJSContext,
    key: &IndexedDBKeyType,
    mut result: MutableHandleValue,
) {
    match key {
        IndexedDBKeyType::Number(n) => result.set(DoubleValue(*n)),
        IndexedDBKeyType::String(s) => s.safe_to_jsval(cx, result),
        IndexedDBKeyType::Binary(b) => b.safe_to_jsval(cx, result),
        IndexedDBKeyType::Date(_d) => {
            // TODO: implement this when Date's representation is finalized.
            result.set(UndefinedValue());
        },
        IndexedDBKeyType::Array(a) => {
            rooted_vec!(let mut values <- repeat(UndefinedValue()).take(a.len()));
            for (key, value) in a.iter().zip(unsafe {
                values
                    .iter_mut()
                    .map(|v| MutableHandle::from_marked_location(v))
            }) {
                key_type_to_jsval(cx, key, value);
            }
            values.safe_to_jsval(cx, result);
        },
    }
}

// https://www.w3.org/TR/IndexedDB-2/#valid-key-path
pub fn is_valid_key_path(key_path: &StrOrStringSequence) -> bool {
    fn is_identifier(_s: &str) -> bool {
        // FIXME: (arihant2math)
        true
    }

    let is_valid = |path: &DOMString| {
        path.is_empty() || is_identifier(path) || path.split(".").all(is_identifier)
    };

    match key_path {
        StrOrStringSequence::StringSequence(paths) => {
            if paths.is_empty() {
                return false;
            }

            paths.iter().all(is_valid)
        },
        StrOrStringSequence::String(path) => is_valid(path),
    }
}

// https://www.w3.org/TR/IndexedDB-2/#convert-value-to-key
#[allow(unsafe_code)]
pub fn convert_value_to_key(
    cx: SafeJSContext,
    input: HandleValue,
    seen: Option<Vec<HandleValue>>,
) -> Result<IndexedDBKeyType, Error> {
    // Step 1: If seen was not given, then let seen be a new empty set.
    let _seen = seen.unwrap_or_default();

    // Step 2: If seen contains input, then return invalid.
    // FIXME:(arihant2math) implement this
    // Check if we have seen this key
    // Does not currently work with HandleValue,
    // as it does not implement PartialEq

    // Step 3
    // FIXME:(arihant2math) Accept array as well
    if input.is_number() {
        if input.to_number().is_nan() {
            return Err(Error::Data);
        }
        return Ok(IndexedDBKeyType::Number(input.to_number()));
    }

    if input.is_string() {
        let string_ptr = std::ptr::NonNull::new(input.to_string()).unwrap();
        let key = unsafe { jsstring_to_str(*cx, string_ptr).str().to_string() };
        return Ok(IndexedDBKeyType::String(key));
    }

    if input.is_object() {
        rooted!(in(*cx) let object = input.to_object());
        unsafe {
            let mut built_in_class = ESClass::Other;

            if !GetBuiltinClass(*cx, object.handle().into(), &mut built_in_class) {
                return Err(Error::Data);
            }

            if let ESClass::Date = built_in_class {
                let mut f = f64::NAN;
                if !js::jsapi::DateGetMsecSinceEpoch(*cx, object.handle().into(), &mut f) {
                    return Err(Error::Data);
                }
                if f.is_nan() {
                    return Err(Error::Data);
                }
                return Ok(IndexedDBKeyType::Date(f));
            }

            if IsArrayBufferObject(*object) || JS_IsArrayBufferViewObject(*object) {
                // FIXME:(arihant2math) implement it the correct way (is this correct?)
                let key = structuredclone::write(cx, input, None).expect("Could not serialize key");
                return Ok(IndexedDBKeyType::Binary(key.serialized.clone()));
            }

            if let ESClass::Array = built_in_class {
                // FIXME:(arihant2math)
                error!("Arrays as keys is currently unsupported");
                return Err(Error::NotSupported);
            }
        }
    }

    Err(Error::Data)
}

// https://www.w3.org/TR/IndexedDB-2/#convert-a-value-to-a-key-range
#[allow(unsafe_code)]
#[expect(unused)]
pub fn convert_value_to_key_range(
    cx: SafeJSContext,
    input: HandleValue,
    null_disallowed: Option<bool>,
) -> Result<IndexedDBKeyRange, Error> {
    let null_disallowed = null_disallowed.unwrap_or(false);
    // Step 1.
    if input.is_object() {
        rooted!(in(*cx) let object = input.to_object());
        unsafe {
            if let Ok(obj) = root_from_object::<IDBKeyRange>(object.get(), *cx) {
                let obj = obj.inner().clone();
                return Ok(obj);
            }
        }
    }
    // Step 2.
    if (input.get().is_undefined() || input.get().is_null()) && null_disallowed {
        return Err(Error::Data);
    }
    let key = convert_value_to_key(cx, input, None)?;
    Ok(IndexedDBKeyRange::only(key))
}

// https://www.w3.org/TR/IndexedDB-2/#evaluate-a-key-path-on-a-value
#[allow(unsafe_code)]
pub fn evaluate_key_path_on_value(
    cx: SafeJSContext,
    value: HandleValue,
    mut return_val: MutableHandleValue,
    key_path: &KeyPath,
) {
    // The implementation is translated from gecko:
    // https://github.com/mozilla/gecko-dev/blob/master/dom/indexedDB/KeyPath.cpp
    return_val.set(*value);

    rooted!(in(*cx) let mut target_object = ptr::null_mut::<JSObject>());
    rooted!(in(*cx) let mut current_val = *value);
    rooted!(in(*cx) let mut object = ptr::null_mut::<JSObject>());

    let mut target_object_prop_name: Option<String> = None;

    match key_path {
        KeyPath::String(path) => {
            // Step 3
            let path_as_string = path.to_string();
            let mut tokenizer = path_as_string.split('.').peekable();

            while let Some(token) = tokenizer.next() {
                if target_object.get().is_null() {
                    if token == "length" && tokenizer.peek().is_none() && current_val.is_string() {
                        rooted!(in(*cx) let input_val = current_val.to_string());
                        unsafe {
                            let string_len = JS_GetStringLength(*input_val) as u64;
                            string_len.safe_to_jsval(cx, return_val);
                        }
                        break;
                    }

                    if !current_val.is_object() {
                        // FIXME:(rasviitanen) Return a proper error
                        return;
                    }

                    object.handle_mut().set(current_val.to_object());
                    rooted!(in(*cx) let mut desc = PropertyDescriptor::default());
                    rooted!(in(*cx) let mut intermediate = UndefinedValue());

                    // So rust says that this value is never read, but it is.
                    #[allow(unused)]
                    let mut has_prop = false;

                    unsafe {
                        let prop_name_as_utf16: Vec<u16> = token.encode_utf16().collect();
                        let mut is_descriptor_none: bool = false;
                        let ok = JS_GetOwnUCPropertyDescriptor(
                            *cx,
                            object.handle().into(),
                            prop_name_as_utf16.as_ptr(),
                            prop_name_as_utf16.len(),
                            desc.handle_mut().into(),
                            &mut is_descriptor_none,
                        );

                        if !ok {
                            // FIXME:(arihant2math) Handle this
                            return;
                        }

                        if desc.hasWritable_() || desc.hasValue_() {
                            intermediate.handle_mut().set(desc.handle().value_);
                            has_prop = true;
                        } else {
                            // If we get here it means the object doesn't have the property or the
                            // property is available through a getter. We don't want to call any
                            // getters to avoid potential re-entrancy.
                            // The blob object is special since its properties are available
                            // only through getters but we still want to support them for key
                            // extraction. So they need to be handled manually.
                            unimplemented!("Blob tokens are not yet supported");
                        }
                    }

                    if has_prop {
                        // Treat undefined as an error
                        if intermediate.is_undefined() {
                            // FIXME:(rasviitanen) Throw/return error
                            return;
                        }

                        if tokenizer.peek().is_some() {
                            // ...and walk to it if there are more steps...
                            current_val.handle_mut().set(*intermediate);
                        } else {
                            // ...otherwise use it as key
                            return_val.set(*intermediate);
                        }
                    } else {
                        target_object.handle_mut().set(*object);
                        target_object_prop_name = Some(token.to_string());
                    }
                }

                if !target_object.get().is_null() {
                    // We have started inserting new objects or are about to just insert
                    // the first one.
                    // FIXME:(rasviitanen) Implement this piece
                    unimplemented!("keyPath tokens that requires insertion are not supported.");
                }
            } // All tokens processed

            if !target_object.get().is_null() {
                // If this fails, we lose, and the web page sees a magical property
                // appear on the object :-(
                unsafe {
                    let prop_name_as_utf16: Vec<u16> =
                        target_object_prop_name.unwrap().encode_utf16().collect();
                    #[allow(clippy::cast_enum_truncation)]
                    let mut succeeded = ObjectOpResult {
                        code_: ObjectOpResult_SpecialCodes::Uninitialized as usize,
                    };
                    if !JS_DeleteUCProperty(
                        *cx,
                        target_object.handle().into(),
                        prop_name_as_utf16.as_ptr(),
                        prop_name_as_utf16.len(),
                        &mut succeeded,
                    ) {
                        // FIXME:(rasviitanen) Throw/return error
                        // return;
                    }
                }
            }
        },
        KeyPath::StringSequence(_) => {
            unimplemented!("String sequence keyPath is currently unsupported");
        },
    }
}

// https://www.w3.org/TR/IndexedDB-2/#extract-a-key-from-a-value-using-a-key-path
pub fn extract_key(
    cx: SafeJSContext,
    input: HandleValue,
    key_path: &KeyPath,
    multi_entry: Option<bool>,
) -> Result<IndexedDBKeyType, Error> {
    // Step 1: Evaluate key path
    // FIXME:(rasviitanen) Do this propertly
    rooted!(in(*cx) let mut r = UndefinedValue());
    evaluate_key_path_on_value(cx, input, r.handle_mut(), key_path);

    if let Some(_multi_entry) = multi_entry {
        // FIXME:(rasviitanen) handle multi_entry cases
        unimplemented!("multiEntry keys are not yet supported");
    } else {
        convert_value_to_key(cx, r.handle(), None)
    }
}
