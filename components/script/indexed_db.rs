/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CString;
use std::iter::repeat_n;
use std::ptr;

use ipc_channel::ipc::IpcSender;
use js::conversions::jsstr_to_string;
use js::gc::MutableHandle;
use js::jsapi::{
    ClippedTime, ESClass, GetBuiltinClass, IsArrayBufferObject, JS_GetStringLength,
    JS_IsArrayBufferViewObject, JS_NewObject, NewDateObject,
};
use js::jsval::{DoubleValue, UndefinedValue};
use js::rust::wrappers::{IsArrayObject, JS_GetProperty, JS_HasOwnProperty};
use js::rust::{HandleValue, MutableHandleValue};
use net_traits::indexeddb_thread::{BackendResult, IndexedDBKeyRange, IndexedDBKeyType};
use profile_traits::ipc;
use profile_traits::ipc::IpcReceiver;
use serde::{Deserialize, Serialize};

use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::Bindings::FileBinding::FileMethods;
use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence as StrOrStringSequence;
use crate::dom::bindings::conversions::{
    SafeToJSValConvertible, get_property_jsval, root_from_handlevalue, root_from_object,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::SafeJSContext;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::utils::set_dictionary_property;
use crate::dom::blob::Blob;
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;
use crate::dom::idbkeyrange::IDBKeyRange;
use crate::dom::idbobjectstore::KeyPath;

pub fn create_channel<T>(
    global: DomRoot<GlobalScope>,
) -> (IpcSender<BackendResult<T>>, IpcReceiver<BackendResult<T>>)
where
    T: for<'a> Deserialize<'a> + Serialize,
{
    ipc::channel::<BackendResult<T>>(global.time_profiler_chan().clone()).unwrap()
}

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
        IndexedDBKeyType::Date(d) => {
            let time = js::jsapi::ClippedTime { t: *d };
            let date = unsafe { js::jsapi::NewDateObject(*cx, time) };
            date.safe_to_jsval(cx, result);
        },
        IndexedDBKeyType::Array(a) => {
            rooted_vec!(let mut values <- repeat_n(UndefinedValue(), a.len()));
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
        let key = unsafe { jsstr_to_string(*cx, string_ptr) };
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
                let key = structuredclone::write(cx, input, None)?;
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

/// The result of steps in
/// <https://www.w3.org/TR/IndexedDB-2/#evaluate-a-key-path-on-a-value>
pub(crate) enum EvaluationResult {
    Success,
    Failure,
}

/// <https://www.w3.org/TR/IndexedDB-2/#evaluate-a-key-path-on-a-value>
#[allow(unsafe_code)]
pub(crate) fn evaluate_key_path_on_value(
    cx: SafeJSContext,
    value: HandleValue,
    key_path: &KeyPath,
    mut return_val: MutableHandleValue,
) -> Result<EvaluationResult, Error> {
    match key_path {
        // Step 1. If keyPath is a list of strings, then:
        KeyPath::StringSequence(key_path) => {
            // Step 1.1. Let result be a new Array object created as if by the expression [].
            rooted!(in(*cx) let mut result = unsafe { JS_NewObject(*cx, ptr::null()) });

            // Step 1.2. Let i be 0.
            // Step 1.3. For each item in keyPath:
            for (i, item) in key_path.iter().enumerate() {
                // Step 1.3.1. Let key be the result of recursively running the steps to evaluate a key
                // path on a value using item as keyPath and value as value.
                // Step 1.3.2. Assert: key is not an abrupt completion.
                // Step 1.3.3. If key is failure, abort the overall algorithm and return failure.
                rooted!(in(*cx) let mut key = UndefinedValue());
                if let EvaluationResult::Failure = evaluate_key_path_on_value(
                    cx,
                    value,
                    &KeyPath::String(item.clone()),
                    key.handle_mut(),
                )? {
                    return Ok(EvaluationResult::Failure);
                };

                // Step 1.3.4. Let p be ! ToString(i).
                // Step 1.3.5. Let status be CreateDataProperty(result, p, key).
                // Step 1.3.6. Assert: status is true.
                set_dictionary_property(cx, result.handle(), &i.to_string(), key.handle())
                    .map_err(|_| Error::JSFailed)?;

                // Step 1.3.7. Increase i by 1.
                // Done by for loop with enumerate()
            }

            // Step 1.4. Return result.
            result.safe_to_jsval(cx, return_val);
        },
        KeyPath::String(key_path) => {
            // Step 2. If keyPath is the empty string, return value and skip the remaining steps.
            if key_path.is_empty() {
                return_val.set(*value);
                return Ok(EvaluationResult::Success);
            }

            // NOTE: Use current_value, instead of value described in spec, in the following steps.
            rooted!(in(*cx) let mut current_value = *value);

            // Step 3. Let identifiers be the result of strictly splitting keyPath on U+002E
            // FULL STOP characters (.).
            // Step 4. For each identifier of identifiers, jump to the appropriate step below:
            for identifier in key_path.split('.') {
                // If Type(value) is String, and identifier is "length"
                if identifier == "length" && current_value.is_string() {
                    // Let value be a Number equal to the number of elements in value.
                    rooted!(in(*cx) let string_value = current_value.to_string());
                    unsafe {
                        let string_length = JS_GetStringLength(*string_value) as u64;
                        string_length.safe_to_jsval(cx, current_value.handle_mut());
                    }
                    continue;
                }

                // If value is an Array and identifier is "length"
                if identifier == "length" {
                    unsafe {
                        let mut is_array = false;
                        if !IsArrayObject(*cx, current_value.handle(), &mut is_array) {
                            return Err(Error::JSFailed);
                        }
                        if is_array {
                            // Let value be ! ToLength(! Get(value, "length")).
                            rooted!(in(*cx) let object = current_value.to_object());
                            get_property_jsval(
                                cx,
                                object.handle(),
                                "length",
                                current_value.handle_mut(),
                            )?;

                            continue;
                        }
                    }
                }

                // If value is a Blob and identifier is "size"
                if identifier == "size" {
                    if let Ok(blob) = root_from_handlevalue::<Blob>(current_value.handle(), cx) {
                        // Let value be a Number equal to value’s size.
                        blob.Size().safe_to_jsval(cx, current_value.handle_mut());

                        continue;
                    }
                }

                // If value is a Blob and identifier is "type"
                if identifier == "type" {
                    if let Ok(blob) = root_from_handlevalue::<Blob>(current_value.handle(), cx) {
                        // Let value be a String equal to value’s type.
                        blob.Type().safe_to_jsval(cx, current_value.handle_mut());

                        continue;
                    }
                }

                // If value is a File and identifier is "name"
                if identifier == "name" {
                    if let Ok(file) = root_from_handlevalue::<File>(current_value.handle(), cx) {
                        // Let value be a String equal to value’s name.
                        file.name().safe_to_jsval(cx, current_value.handle_mut());

                        continue;
                    }
                }

                // If value is a File and identifier is "lastModified"
                if identifier == "lastModified" {
                    if let Ok(file) = root_from_handlevalue::<File>(current_value.handle(), cx) {
                        // Let value be a Number equal to value’s lastModified.
                        file.LastModified()
                            .safe_to_jsval(cx, current_value.handle_mut());

                        continue;
                    }
                }

                // If value is a File and identifier is "lastModifiedDate"
                if identifier == "lastModifiedDate" {
                    if let Ok(file) = root_from_handlevalue::<File>(current_value.handle(), cx) {
                        // Let value be a new Date object with [[DateValue]] internal slot equal to value’s lastModified.
                        let time = ClippedTime {
                            t: file.LastModified() as f64,
                        };
                        unsafe {
                            NewDateObject(*cx, time).safe_to_jsval(cx, current_value.handle_mut());
                        }

                        continue;
                    }
                }

                // Otherwise
                unsafe {
                    // If Type(value) is not Object, return failure.
                    if !current_value.is_object() {
                        return Ok(EvaluationResult::Failure);
                    }

                    rooted!(in(*cx) let object = current_value.to_object());
                    let identifier_name =
                        CString::new(identifier).expect("Failed to convert str to CString");

                    // Let hop be ! HasOwnProperty(value, identifier).
                    let mut hop = false;
                    if !JS_HasOwnProperty(*cx, object.handle(), identifier_name.as_ptr(), &mut hop)
                    {
                        return Err(Error::JSFailed);
                    }

                    // If hop is false, return failure.
                    if !hop {
                        return Ok(EvaluationResult::Failure);
                    }

                    // Let value be ! Get(value, identifier).
                    if !JS_GetProperty(
                        *cx,
                        object.handle(),
                        identifier_name.as_ptr(),
                        current_value.handle_mut(),
                    ) {
                        return Err(Error::JSFailed);
                    }

                    // If value is undefined, return failure.
                    if current_value.get().is_undefined() {
                        return Ok(EvaluationResult::Failure);
                    }
                }
            }

            // Step 5. Assert: value is not an abrupt completion.
            // Done within Step 4.

            // Step 6. Return value.
            return_val.set(*current_value);
        },
    }
    Ok(EvaluationResult::Success)
}

/// The result of steps in
/// <https://www.w3.org/TR/IndexedDB-2/#extract-a-key-from-a-value-using-a-key-path>
pub(crate) enum ExtractionResult {
    Key(IndexedDBKeyType),
    // NOTE: Invalid is not used for now. Remove the unused annotation when it is used.
    #[expect(unused)]
    Invalid,
    Failure,
}

/// <https://www.w3.org/TR/IndexedDB-2/#extract-a-key-from-a-value-using-a-key-path>
pub(crate) fn extract_key(
    cx: SafeJSContext,
    value: HandleValue,
    key_path: &KeyPath,
    multi_entry: Option<bool>,
) -> Result<ExtractionResult, Error> {
    // Step 1. Let r be the result of running the steps to evaluate a key path on a value with
    // value and keyPath. Rethrow any exceptions.
    // Step 2. If r is failure, return failure.
    rooted!(in(*cx) let mut r = UndefinedValue());
    if let EvaluationResult::Failure =
        evaluate_key_path_on_value(cx, value, key_path, r.handle_mut())?
    {
        return Ok(ExtractionResult::Failure);
    }

    // Step 3. Let key be the result of running the steps to convert a value to a key with r if the
    // multiEntry flag is unset, and the result of running the steps to convert a value to a
    // multiEntry key with r otherwise. Rethrow any exceptions.
    let key = match multi_entry {
        Some(true) => {
            // TODO: implement convert_value_to_multientry_key
            unimplemented!("multiEntry keys are not yet supported");
        },
        _ => convert_value_to_key(cx, r.handle(), None)?,
    };

    // TODO: Step 4. If key is invalid, return invalid.

    // Step 5. Return key.
    Ok(ExtractionResult::Key(key))
}
