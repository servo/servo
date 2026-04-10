/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CString;
use std::ptr;

use itertools::Itertools;
use js::context::JSContext;
use js::conversions::{ToJSValConvertible, jsstr_to_string};
use js::jsapi::{
    ClippedTime, HandleValueArray, IsArrayBufferObject, IsDetachedArrayBufferObject,
    JS_GetArrayBufferViewBuffer, JS_GetStringLength, JS_IsArrayBufferViewObject, NewArrayObject,
    PropertyKey,
};
use js::jsval::{DoubleValue, ObjectValue, UndefinedValue};
use js::rust::wrappers::SameValue;
use js::rust::wrappers2::{
    GetArrayLength, IsArrayObject, JS_HasOwnPropertyById, JS_IndexToId, JS_IsIdentifier,
    JS_NewObject, NewDateObject, ObjectIsDate,
};
use js::rust::{HandleValue, MutableHandleValue};
use js::typedarray::{ArrayBuffer, ArrayBufferView, CreateWith};
use storage_traits::indexeddb::{BackendError, IndexedDBKeyRange, IndexedDBKeyType};

use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::Bindings::FileBinding::FileMethods;
use crate::dom::bindings::codegen::UnionTypes::StringOrStringSequence as StrOrStringSequence;
use crate::dom::bindings::conversions::{
    get_property_jsval, root_from_handlevalue, root_from_object,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::utils::{
    define_dictionary_property, get_dictionary_property, has_own_property,
};
use crate::dom::blob::Blob;
use crate::dom::file::File;
use crate::dom::idbkeyrange::IDBKeyRange;
use crate::dom::idbobjectstore::KeyPath;
use crate::script_runtime::CanGc;

// https://www.w3.org/TR/IndexedDB-3/#convert-key-to-value
#[expect(unsafe_code)]
pub fn key_type_to_jsval(
    cx: &mut JSContext,
    key: &IndexedDBKeyType,
    mut result: MutableHandleValue,
) {
    // Step 1. Let type be key’s type.
    // Step 2. Let value be key’s value.
    // Step 3. Switch on type:
    match key {
        // Step 3. If type is number, return an ECMAScript Number value equal to value.
        IndexedDBKeyType::Number(n) => result.set(DoubleValue(*n)),

        // Step 3. If type is string, return an ECMAScript String value equal to value.
        IndexedDBKeyType::String(s) => s.safe_to_jsval(cx, result),

        IndexedDBKeyType::Date(d) => unsafe {
            // Step 3.1. Let date be the result of executing the ECMAScript Date
            // constructor with the single argument value.
            let date = NewDateObject(cx, ClippedTime { t: *d });

            // Step 3.2. Assert: date is not an abrupt completion.
            assert!(
                !date.is_null(),
                "Failed to convert IndexedDB date key into a Date"
            );

            // Step 3.3. Return date.
            date.safe_to_jsval(cx, result);
        },

        IndexedDBKeyType::Binary(b) => unsafe {
            // Step 3.1. Let len be value’s length.
            let len = b.len();

            // Step 3.2. Let buffer be the result of executing the ECMAScript
            // ArrayBuffer constructor with len.
            rooted!(&in(cx) let mut buffer = ptr::null_mut::<js::jsapi::JSObject>());
            assert!(
                ArrayBuffer::create(cx.raw_cx(), CreateWith::Length(len), buffer.handle_mut())
                    .is_ok(),
                "Failed to convert IndexedDB binary key into an ArrayBuffer"
            );

            // Step 3.3. Assert: buffer is not an abrupt completion.

            // Step 3.4. Set the entries in buffer’s [[ArrayBufferData]] internal slot to the
            // entries in value.
            let mut array_buffer = ArrayBuffer::from(buffer.get())
                .expect("ArrayBuffer::create should create an ArrayBuffer object");
            array_buffer.as_mut_slice().copy_from_slice(b);

            // Step 3.5. Return buffer.
            result.set(ObjectValue(buffer.get()));
        },

        IndexedDBKeyType::Array(a) => unsafe {
            // Step 3.1. Let array be the result of executing the ECMAScript Array
            // constructor with no arguments.
            let empty_args = HandleValueArray::empty();
            rooted!(&in(cx) let array = NewArrayObject(cx.raw_cx(), &empty_args));

            // Step 3.2. Assert: array is not an abrupt completion.
            assert!(
                !array.get().is_null(),
                "Failed to convert IndexedDB array key into an Array"
            );

            // Step 3.3. Let len be value’s size.
            let len = a.len();

            // Step 3.4. Let index be 0.
            let mut index = 0;

            // Step 3.5. While index is less than len:
            while index < len {
                // Step 3.5.1. Let entry be the result of converting a key to a value with
                // value[index].
                rooted!(&in(cx) let mut entry = UndefinedValue());
                key_type_to_jsval(cx, &a[index], entry.handle_mut());

                // Step 3.5.2. Let status be CreateDataProperty(array, index, entry).
                let index_property = CString::new(index.to_string());
                assert!(
                    index_property.is_ok(),
                    "Failed to convert IndexedDB array index to CString"
                );
                let index_property = index_property.unwrap();
                let status = define_dictionary_property(
                    cx.into(),
                    array.handle(),
                    index_property.as_c_str(),
                    entry.handle(),
                );

                // Step 3.5.3. Assert: status is true.
                assert!(
                    status.is_ok(),
                    "CreateDataProperty on a fresh JS array should not fail"
                );

                // Step 3.5.4. Increase index by 1.
                index += 1;
            }

            // Step 3.6. Return array.
            result.set(ObjectValue(array.get()));
        },
    }
}

/// <https://www.w3.org/TR/IndexedDB-3/#valid-key-path>
pub(crate) fn is_valid_key_path(
    cx: &mut JSContext,
    key_path: &StrOrStringSequence,
) -> Result<bool, Error> {
    // <https://tc39.es/ecma262/#prod-IdentifierName>
    #[expect(unsafe_code)]
    let is_identifier_name = |cx: &mut JSContext, name: &str| -> Result<bool, Error> {
        rooted!(&in(cx) let mut value = UndefinedValue());
        name.safe_to_jsval(cx, value.handle_mut());
        rooted!(&in(cx) let string = value.to_string());

        unsafe {
            let mut is_identifier = false;
            if !JS_IsIdentifier(cx, string.handle(), &mut is_identifier) {
                return Err(Error::JSFailed);
            }
            Ok(is_identifier)
        }
    };

    // A valid key path is one of:
    let is_valid = |cx: &mut JSContext, path: &DOMString| -> Result<bool, Error> {
        // An empty string.
        let is_empty_string = path.is_empty();

        // An identifier, which is a string matching the IdentifierName production from the
        // ECMAScript Language Specification [ECMA-262].
        let is_identifier = is_identifier_name(cx, &path.str())?;

        // A string consisting of two or more identifiers separated by periods (U+002E FULL STOP).
        let is_identifier_list = path
            .str()
            .split('.')
            .map(|s| is_identifier_name(cx, s))
            .try_collect::<bool, Vec<bool>, Error>()?
            .iter()
            .all(|&value| value);

        Ok(is_empty_string || is_identifier || is_identifier_list)
    };

    match key_path {
        StrOrStringSequence::StringSequence(paths) => {
            // A non-empty list containing only strings conforming to the above requirements.
            if paths.is_empty() {
                Ok(false)
            } else {
                Ok(paths
                    .iter()
                    .map(|s| is_valid(cx, s))
                    .try_collect::<bool, Vec<bool>, Error>()?
                    .iter()
                    .all(|&value| value))
            }
        },
        StrOrStringSequence::String(path) => is_valid(cx, path),
    }
}

pub(crate) enum ConversionResult {
    Valid(IndexedDBKeyType),
    Invalid,
}

impl ConversionResult {
    pub fn into_result(self) -> Result<IndexedDBKeyType, Error> {
        match self {
            ConversionResult::Valid(key) => Ok(key),
            ConversionResult::Invalid => Err(Error::Data(None)),
        }
    }
}

// https://www.w3.org/TR/IndexedDB-3/#convert-value-to-key
#[expect(unsafe_code)]
pub fn convert_value_to_key(
    cx: &mut JSContext,
    input: HandleValue,
    seen: Option<Vec<HandleValue>>,
) -> Result<ConversionResult, Error> {
    // Step 1: If seen was not given, then let seen be a new empty set.
    let mut seen = seen.unwrap_or_default();

    // Step 2: If seen contains input, then return "invalid value".
    for seen_input in &seen {
        let mut same = false;
        if unsafe { !SameValue(cx.raw_cx(), *seen_input, input, &mut same) } {
            return Err(Error::JSFailed);
        }
        if same {
            return Ok(ConversionResult::Invalid);
        }
    }

    // Step 3. Jump to the appropriate step below.

    // If Type(input) is Number:
    if input.is_number() {
        // 3.1. If input is NaN then return "invalid value".
        if input.to_number().is_nan() {
            return Ok(ConversionResult::Invalid);
        }
        // 3.2. Otherwise, return a new key with type number and value input.
        return Ok(ConversionResult::Valid(IndexedDBKeyType::Number(
            input.to_number(),
        )));
    }

    // If Type(input) is String:
    if input.is_string() {
        // 3.1. Return a new key with type string and value input.
        let string_ptr = std::ptr::NonNull::new(input.to_string()).unwrap();
        let key = unsafe { jsstr_to_string(cx.raw_cx(), string_ptr) };
        return Ok(ConversionResult::Valid(IndexedDBKeyType::String(key)));
    }

    if input.is_object() {
        rooted!(&in(cx) let object = input.to_object());
        unsafe {
            let mut is_date = false;
            if !ObjectIsDate(cx, object.handle(), &mut is_date) {
                return Err(Error::JSFailed);
            }

            // If input is a Date (has a [[DateValue]] internal slot):
            if is_date {
                // 3.1. Let ms be the value of input's [[DateValue]] internal slot.
                let mut ms = f64::NAN;
                if !js::rust::wrappers2::DateGetMsecSinceEpoch(cx, object.handle(), &mut ms) {
                    return Err(Error::JSFailed);
                }
                // 3.2. If ms is NaN then return "invalid value".
                if ms.is_nan() {
                    return Ok(ConversionResult::Invalid);
                }
                // 3.3. Otherwise, return a new key with type date and value ms.
                return Ok(ConversionResult::Valid(IndexedDBKeyType::Date(ms)));
            }

            // If input is a buffer source type:
            if IsArrayBufferObject(*object) || JS_IsArrayBufferViewObject(*object) {
                let is_detached = if IsArrayBufferObject(*object) {
                    IsDetachedArrayBufferObject(*object)
                } else {
                    // Shared ArrayBuffers are not supported here, so this stays false.
                    let mut is_shared = false;
                    rooted!(
                        in (cx.raw_cx()) let view_buffer =
                            JS_GetArrayBufferViewBuffer(
                                cx.raw_cx(),
                                object.handle().into(),
                                &mut is_shared
                            )
                    );
                    !is_shared && IsDetachedArrayBufferObject(*view_buffer.handle())
                };
                // 3.1. If input is detached then return "invalid value".
                if is_detached {
                    return Ok(ConversionResult::Invalid);
                }
                // 3.2. Let bytes be the result of getting a copy of the bytes held
                // by the buffer source input.
                let bytes = if IsArrayBufferObject(*object) {
                    let array_buffer = ArrayBuffer::from(*object).map_err(|()| Error::JSFailed)?;
                    array_buffer.to_vec()
                } else {
                    let array_buffer_view =
                        ArrayBufferView::from(*object).map_err(|()| Error::JSFailed)?;
                    array_buffer_view.to_vec()
                };
                // 3.3. Return a new key with type binary and value bytes.
                return Ok(ConversionResult::Valid(IndexedDBKeyType::Binary(bytes)));
            }

            // If input is an Array exotic object:
            let mut is_array = false;
            if !IsArrayObject(cx, input, &mut is_array) {
                return Err(Error::JSFailed);
            }
            if is_array {
                // 3.1. Let len be ? ToLength( ? Get(input, "length")).
                let mut len = 0;
                if !GetArrayLength(cx, object.handle(), &mut len) {
                    return Err(Error::JSFailed);
                }
                // 3.2. Append input to seen.
                seen.push(input);
                // 3.3. Let keys be a new empty list.
                let mut keys = vec![];
                // 3.4. Let index be 0.
                let mut index: u32 = 0;
                // 3.5. While index is less than len:
                while index < len {
                    rooted!(&in(cx) let mut id: PropertyKey);
                    if !JS_IndexToId(cx, index, id.handle_mut()) {
                        return Err(Error::JSFailed);
                    }
                    // 3.5.1. Let hop be ? HasOwnProperty(input, index).
                    let mut hop = false;
                    if !JS_HasOwnPropertyById(cx, object.handle(), id.handle(), &mut hop) {
                        return Err(Error::JSFailed);
                    }
                    // 3.5.2. If hop is false, return "invalid value".
                    if !hop {
                        return Ok(ConversionResult::Invalid);
                    }
                    // 3.5.3. Let entry be ? Get(input, index).
                    rooted!(&in(cx) let mut entry = UndefinedValue());
                    if !js::rust::wrappers2::JS_GetPropertyById(
                        cx,
                        object.handle(),
                        id.handle(),
                        entry.handle_mut(),
                    ) {
                        return Err(Error::JSFailed);
                    }

                    // 3.5.4. Let key be the result of converting a value to a key
                    //        with arguments entry and seen.
                    // 3.5.5. ReturnIfAbrupt(key).
                    let key = match convert_value_to_key(cx, entry.handle(), Some(seen.clone()))? {
                        ConversionResult::Valid(key) => key,
                        // 3.5.6. If key is "invalid value" or "invalid type"
                        //        abort these steps and return "invalid value".
                        ConversionResult::Invalid => return Ok(ConversionResult::Invalid),
                    };
                    // 3.5.7. Append key to keys.
                    keys.push(key);
                    // 3.5.8. Increase index by 1.
                    index += 1;
                }
                // 3.6. Return a new array key with value keys.
                return Ok(ConversionResult::Valid(IndexedDBKeyType::Array(keys)));
            }
        }
    }

    // Otherwise, return "invalid type".
    Ok(ConversionResult::Invalid)
}

/// <https://www.w3.org/TR/IndexedDB-3/#convert-a-value-to-a-key-range>
#[expect(unsafe_code)]
pub fn convert_value_to_key_range(
    cx: &mut JSContext,
    input: HandleValue,
    null_disallowed: Option<bool>,
) -> Result<IndexedDBKeyRange, Error> {
    // Step 1. If value is a key range, return value.
    if input.is_object() {
        rooted!(&in(cx) let object = input.to_object());
        unsafe {
            if let Ok(obj) = root_from_object::<IDBKeyRange>(object.get(), cx.raw_cx()) {
                let obj = obj.inner().clone();
                return Ok(obj);
            }
        }
    }

    // Step 2. If value is undefined or is null, then throw a "DataError" DOMException if null
    // disallowed flag is set, or return an unbounded key range otherwise.
    if input.get().is_undefined() || input.get().is_null() {
        if null_disallowed.is_some_and(|flag| flag) {
            return Err(Error::Data(None));
        } else {
            return Ok(IndexedDBKeyRange {
                lower: None,
                upper: None,
                lower_open: Default::default(),
                upper_open: Default::default(),
            });
        }
    }

    // Step 3. Let key be the result of running the steps to convert a value to a key with value.
    // Rethrow any exceptions.
    let key = convert_value_to_key(cx, input, None)?;

    // Step 4. If key is invalid, throw a "DataError" DOMException.
    let key = key.into_result()?;

    // Step 5. Return a key range containing only key.
    Ok(IndexedDBKeyRange::only(key))
}

pub(crate) fn map_backend_error_to_dom_error(error: BackendError) -> Error {
    match error {
        BackendError::QuotaExceeded => Error::QuotaExceeded {
            quota: None,
            requested: None,
        },
        BackendError::DbErr(details) => {
            Error::Operation(Some(format!("IndexedDB open failed: {details}")))
        },
        other => Error::Operation(Some(format!("IndexedDB open failed: {other:?}"))),
    }
}

/// The result of steps in
/// <https://www.w3.org/TR/IndexedDB-3/#evaluate-a-key-path-on-a-value>
pub(crate) enum EvaluationResult {
    Success,
    Failure,
}

/// <https://www.w3.org/TR/IndexedDB-3/#evaluate-a-key-path-on-a-value>
#[expect(unsafe_code)]
pub(crate) fn evaluate_key_path_on_value(
    cx: &mut JSContext,
    value: HandleValue,
    key_path: &KeyPath,
    mut return_val: MutableHandleValue,
) -> Result<EvaluationResult, Error> {
    match key_path {
        // Step 1. If keyPath is a list of strings, then:
        KeyPath::StringSequence(key_path) => {
            // Step 1.1. Let result be a new Array object created as if by the expression [].
            rooted!(&in(cx) let mut result = unsafe { JS_NewObject(cx, ptr::null()) });

            // Step 1.2. Let i be 0.
            // Step 1.3. For each item in keyPath:
            for (i, item) in key_path.iter().enumerate() {
                // Step 1.3.1. Let key be the result of recursively running the steps to evaluate a key
                // path on a value using item as keyPath and value as value.
                // Step 1.3.2. Assert: key is not an abrupt completion.
                // Step 1.3.3. If key is failure, abort the overall algorithm and return failure.
                rooted!(&in(cx) let mut key = UndefinedValue());
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
                let i_cstr = std::ffi::CString::new(i.to_string()).unwrap();
                define_dictionary_property(
                    cx.into(),
                    result.handle(),
                    i_cstr.as_c_str(),
                    key.handle(),
                )
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
            rooted!(&in(cx) let mut current_value = *value);

            // Step 3. Let identifiers be the result of strictly splitting keyPath on U+002E
            // FULL STOP characters (.).
            // Step 4. For each identifier of identifiers, jump to the appropriate step below:
            for identifier in key_path.str().split('.') {
                // If Type(value) is String, and identifier is "length"
                if identifier == "length" && current_value.is_string() {
                    // Let value be a Number equal to the number of elements in value.
                    rooted!(&in(cx) let string_value = current_value.to_string());
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
                        if !IsArrayObject(cx, current_value.handle(), &mut is_array) {
                            return Err(Error::JSFailed);
                        }
                        if is_array {
                            // Let value be ! ToLength(! Get(value, "length")).
                            rooted!(&in(cx) let object = current_value.to_object());
                            get_property_jsval(
                                cx.into(),
                                object.handle(),
                                c"length",
                                current_value.handle_mut(),
                            )?;

                            continue;
                        }
                    }
                }

                // If value is a Blob and identifier is "size"
                if identifier == "size" {
                    if let Ok(blob) =
                        root_from_handlevalue::<Blob>(current_value.handle(), cx.into())
                    {
                        // Let value be a Number equal to value’s size.
                        blob.Size().safe_to_jsval(cx, current_value.handle_mut());

                        continue;
                    }
                }

                // If value is a Blob and identifier is "type"
                if identifier == "type" {
                    if let Ok(blob) =
                        root_from_handlevalue::<Blob>(current_value.handle(), cx.into())
                    {
                        // Let value be a String equal to value’s type.
                        blob.Type().safe_to_jsval(cx, current_value.handle_mut());

                        continue;
                    }
                }

                // If value is a File and identifier is "name"
                if identifier == "name" {
                    if let Ok(file) =
                        root_from_handlevalue::<File>(current_value.handle(), cx.into())
                    {
                        // Let value be a String equal to value’s name.
                        file.name().safe_to_jsval(cx, current_value.handle_mut());

                        continue;
                    }
                }

                // If value is a File and identifier is "lastModified"
                if identifier == "lastModified" {
                    if let Ok(file) =
                        root_from_handlevalue::<File>(current_value.handle(), cx.into())
                    {
                        // Let value be a Number equal to value’s lastModified.
                        file.LastModified()
                            .safe_to_jsval(cx, current_value.handle_mut());

                        continue;
                    }
                }

                // If value is a File and identifier is "lastModifiedDate"
                if identifier == "lastModifiedDate" {
                    if let Ok(file) =
                        root_from_handlevalue::<File>(current_value.handle(), cx.into())
                    {
                        // Let value be a new Date object with [[DateValue]] internal slot equal to value’s lastModified.
                        let time = ClippedTime {
                            t: file.LastModified() as f64,
                        };
                        unsafe {
                            NewDateObject(cx, time).safe_to_jsval(cx, current_value.handle_mut());
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

                    rooted!(&in(cx) let object = current_value.to_object());
                    let identifier_name =
                        CString::new(identifier).expect("Failed to convert str to CString");

                    // Let hop be ! HasOwnProperty(value, identifier).
                    let hop =
                        has_own_property(cx.into(), object.handle(), identifier_name.as_c_str())
                            .map_err(|_| Error::JSFailed)?;

                    // If hop is false, return failure.
                    if !hop {
                        return Ok(EvaluationResult::Failure);
                    }

                    // Let value be ! Get(value, identifier).
                    match get_dictionary_property(
                        cx.raw_cx(),
                        object.handle(),
                        identifier_name.as_c_str(),
                        current_value.handle_mut(),
                        CanGc::deprecated_note(),
                    ) {
                        Ok(true) => {},
                        Ok(false) => return Ok(EvaluationResult::Failure),
                        Err(()) => return Err(Error::JSFailed),
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
/// <https://www.w3.org/TR/IndexedDB-3/#extract-a-key-from-a-value-using-a-key-path>
pub(crate) enum ExtractionResult {
    Key(IndexedDBKeyType),
    Invalid,
    Failure,
}

/// <https://w3c.github.io/IndexedDB/#check-that-a-key-could-be-injected-into-a-value>
#[expect(unsafe_code)]
pub(crate) fn can_inject_key_into_value(
    cx: &mut JSContext,
    value: HandleValue,
    key_path: &DOMString,
) -> Result<bool, Error> {
    // Step 1. Let identifiers be the result of strictly splitting keyPath on U+002E FULL STOP
    // characters (.).
    let key_path_string = key_path.str();
    let mut identifiers: Vec<&str> = key_path_string.split('.').collect();

    // Step 2. Assert: identifiers is not empty.
    let Some(_) = identifiers.pop() else {
        return Ok(false);
    };

    rooted!(&in(cx) let mut current_value = *value);

    // Step 3. For each remaining identifier of identifiers:
    for identifier in identifiers {
        // Step 3.1. If value is not an Object or an Array, return false.
        if !current_value.is_object() {
            return Ok(false);
        }

        rooted!(&in(cx) let current_object = current_value.to_object());
        let identifier_name =
            CString::new(identifier).expect("Failed to convert key path identifier to CString");

        // Step 3.2. Let hop be ? HasOwnProperty(value, identifier).
        let hop = has_own_property(
            cx.into(),
            current_object.handle(),
            identifier_name.as_c_str(),
        )
        .map_err(|_| Error::JSFailed)?;

        // Step 3.3. If hop is false, set value to a new Object created as if by the expression
        // ({}).
        // We avoid mutating `value` during this check and can return true immediately because the
        // remaining path can be created from scratch.
        if !hop {
            return Ok(true);
        }

        // Step 3.4. Set value to ? Get(value, identifier).
        match unsafe {
            get_dictionary_property(
                cx.raw_cx(),
                current_object.handle(),
                identifier_name.as_c_str(),
                current_value.handle_mut(),
                CanGc::deprecated_note(),
            )
        } {
            Ok(true) => {},
            Ok(false) => return Ok(false),
            Err(()) => return Err(Error::JSFailed),
        }
    }

    // Step 4. Return true if value is an Object or an Array, and false otherwise.
    Ok(current_value.is_object())
}

/// <https://w3c.github.io/IndexedDB/#inject-a-key-into-a-value-using-a-key-path>
#[expect(unsafe_code)]
pub(crate) fn inject_key_into_value(
    cx: &mut JSContext,
    value: HandleValue,
    key: &IndexedDBKeyType,
    key_path: &DOMString,
) -> Result<bool, Error> {
    // Step 1. Let identifiers be the result of strictly splitting keyPath on U+002E FULL STOP characters (.).
    let key_path_string = key_path.str();
    let mut identifiers: Vec<&str> = key_path_string.split('.').collect();

    // Step 2. Assert: identifiers is not empty.
    let Some(last) = identifiers.pop() else {
        return Ok(false);
    };

    // Step 3. Let last be the last item of identifiers and remove it from the list.
    // Done by `pop()` above.

    rooted!(&in(cx) let mut current_value = *value);

    // Step 4. For each remaining identifier of identifiers:
    for identifier in identifiers {
        // Step 4.1 Assert: value is an Object or an Array.
        if !current_value.is_object() {
            return Ok(false);
        }

        rooted!(&in(cx) let current_object = current_value.to_object());
        let identifier_name =
            CString::new(identifier).expect("Failed to convert key path identifier to CString");

        // Step 4.2 Let hop be ! HasOwnProperty(value, identifier).
        let hop = has_own_property(
            cx.into(),
            current_object.handle(),
            identifier_name.as_c_str(),
        )
        .map_err(|_| Error::JSFailed)?;

        // Step 4.3 If hop is false, then:
        if !hop {
            // Step 4.3.1 Let o be a new Object created as if by the expression ({}).
            rooted!(&in(cx) let o = unsafe { JS_NewObject(cx, ptr::null()) });
            rooted!(&in(cx) let mut o_value = UndefinedValue());
            o.safe_to_jsval(cx, o_value.handle_mut());

            // Step 4.3.2 Let status be CreateDataProperty(value, identifier, o).
            define_dictionary_property(
                cx.into(),
                current_object.handle(),
                identifier_name.as_c_str(),
                o_value.handle(),
            )
            .map_err(|_| Error::JSFailed)?;

            // Step 4.3.3 Assert: status is true.
        }

        // Step 4.3 Let value be ! Get(value, identifier).
        match unsafe {
            get_dictionary_property(
                cx.raw_cx(),
                current_object.handle(),
                identifier_name.as_c_str(),
                current_value.handle_mut(),
                CanGc::deprecated_note(),
            )
        } {
            Ok(true) => {},
            Ok(false) => return Ok(false),
            Err(()) => return Err(Error::JSFailed),
        }

        // Step 5 "Assert: value is an Object or an Array."
        if !current_value.is_object() {
            return Ok(false);
        }
    }

    // Step 6. Let keyValue be the result of converting a key to a value with key.
    rooted!(&in(cx) let mut key_value = UndefinedValue());
    key_type_to_jsval(cx, key, key_value.handle_mut());

    // `current_value` is the parent object where `last` will be defined.
    if !current_value.is_object() {
        return Ok(false);
    }
    rooted!(&in(cx) let parent_object = current_value.to_object());
    let last_name = CString::new(last).expect("Failed to convert final key path identifier");

    // Step 7. Let status be CreateDataProperty(value, last, keyValue).
    define_dictionary_property(
        cx.into(),
        parent_object.handle(),
        last_name.as_c_str(),
        key_value.handle(),
    )
    .map_err(|_| Error::JSFailed)?;

    // Step 8. Assert: status is true.
    // The JS_DefineProperty success check above enforces this assertion.
    // "NOTE: Assertions can be made in the above steps because this algorithm is only applied to values that are the output of StructuredDeserialize, and the steps to check that a key could be injected into a value have been run."
    Ok(true)
}

/// <https://www.w3.org/TR/IndexedDB-3/#extract-a-key-from-a-value-using-a-key-path>
pub(crate) fn extract_key(
    cx: &mut JSContext,
    value: HandleValue,
    key_path: &KeyPath,
    multi_entry: Option<bool>,
) -> Result<ExtractionResult, Error> {
    // Step 1. Let r be the result of running the steps to evaluate a key path on a value with
    // value and keyPath. Rethrow any exceptions.
    // Step 2. If r is failure, return failure.
    rooted!(&in(cx) let mut r = UndefinedValue());
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
        _ => match convert_value_to_key(cx, r.handle(), None)? {
            ConversionResult::Valid(key) => key,
            // Step 4. If key is invalid, return invalid.
            ConversionResult::Invalid => return Ok(ExtractionResult::Invalid),
        },
    };

    // Step 5. Return key.
    Ok(ExtractionResult::Key(key))
}
