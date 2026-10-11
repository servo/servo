/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Value containers for JS↔native communication (CEF-compatible CefValue,
//! CefDictionaryValue, CefListValue).

use std::collections::HashMap;
use std::ffi::{CStr, c_char};

use base64::Engine;
use servo_api::JSValue;

use crate::ServoError;

/// Value type constants matching CEF's CefValueType.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServoValueType {
    Invalid = 0,
    Null = 1,
    Bool = 2,
    Int = 3,
    Double = 4,
    String = 5,
    Binary = 6,
    Dictionary = 7,
    List = 8,
}

impl From<ServoValueType> for i32 {
    fn from(v: ServoValueType) -> i32 {
        v as i32
    }
}
/// The actual value data, mirroring JSValue but owned.
#[derive(Clone, Debug)]
enum ValueData {
    Null,
    Bool(bool),
    Int(i64),
    Double(f64),
    String(String),
    Binary(Vec<u8>),
    Dictionary(HashMap<String, ServoValue>),
    List(Vec<ServoValue>),
}

/// An opaque struct representing a generic value container.
/// Can hold: null, bool, int, double, string, binary, dictionary, list.
// cbindgen:opaque
#[derive(Clone, Debug)]
pub struct ServoValue {
    value: ValueData,
}

/// An opaque struct representing a dictionary value (key-value map).
// cbindgen:opaque
#[derive(Clone, Debug)]
pub struct ServoDictionaryValue {
    map: HashMap<String, ServoValue>,
}

/// An opaque struct representing a list value (ordered array).
// cbindgen:opaque
#[derive(Clone, Debug)]
pub struct ServoListValue {
    list: Vec<ServoValue>,
}

impl ServoValue {
    fn new(value: ValueData) -> Self {
        Self { value }
    }

    fn get_type(&self) -> ServoValueType {
        match &self.value {
            ValueData::Null => ServoValueType::Null,
            ValueData::Bool(_) => ServoValueType::Bool,
            ValueData::Int(_) => ServoValueType::Int,
            ValueData::Double(_) => ServoValueType::Double,
            ValueData::String(_) => ServoValueType::String,
            ValueData::Binary(_) => ServoValueType::Binary,
            ValueData::Dictionary(_) => ServoValueType::Dictionary,
            ValueData::List(_) => ServoValueType::List,
        }
    }

    fn as_dictionary(&self) -> Option<&HashMap<String, ServoValue>> {
        match &self.value {
            ValueData::Dictionary(map) => Some(map),
            _ => None,
        }
    }

    fn as_list(&self) -> Option<&Vec<ServoValue>> {
        match &self.value {
            ValueData::List(list) => Some(list),
            _ => None,
        }
    }

    fn as_dictionary_mut(&mut self) -> Option<&mut HashMap<String, ServoValue>> {
        match &mut self.value {
            ValueData::Dictionary(map) => Some(map),
            _ => None,
        }
    }

    fn as_list_mut(&mut self) -> Option<&mut Vec<ServoValue>> {
        match &mut self.value {
            ValueData::List(list) => Some(list),
            _ => None,
        }
    }

    fn set_bool(&mut self, val: bool) {
        self.value = ValueData::Bool(val);
    }

    fn set_int(&mut self, val: i64) {
        self.value = ValueData::Int(val);
    }

    fn set_double(&mut self, val: f64) {
        self.value = ValueData::Double(val);
    }

    fn set_string(&mut self, val: String) {
        self.value = ValueData::String(val);
    }

    fn set_binary(&mut self, val: Vec<u8>) {
        self.value = ValueData::Binary(val);
    }

    pub(crate) fn from_jsvalue(value: &JSValue) -> Self {
        match value {
            JSValue::Undefined | JSValue::Null => Self::new(ValueData::Null),
            JSValue::Boolean(b) => Self::new(ValueData::Bool(*b)),
            JSValue::Number(n) => {
                if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                    Self::new(ValueData::Int(*n as i64))
                } else {
                    Self::new(ValueData::Double(*n))
                }
            },
            JSValue::String(s) => Self::new(ValueData::String(s.clone())),
            JSValue::Element(s)
            | JSValue::ShadowRoot(s)
            | JSValue::Frame(s)
            | JSValue::Window(s) => Self::new(ValueData::String(s.clone())),
            JSValue::Array(items) => {
                let list: Vec<ServoValue> = items.iter().map(ServoValue::from_jsvalue).collect();
                Self::new(ValueData::List(list))
            },
            JSValue::Object(entries) => {
                let mut map = HashMap::new();
                for (key, value) in entries {
                    map.insert(key.clone(), ServoValue::from_jsvalue(value));
                }
                Self::new(ValueData::Dictionary(map))
            },
        }
    }

    fn to_jsvalue(&self) -> JSValue {
        match &self.value {
            ValueData::Null => JSValue::Null,
            ValueData::Bool(b) => JSValue::Boolean(*b),
            ValueData::Int(i) => JSValue::Number(*i as f64),
            ValueData::Double(d) => JSValue::Number(*d),
            ValueData::String(s) => JSValue::String(s.clone()),
            ValueData::Binary(_) => JSValue::Null, // Binary not representable in JSValue
            ValueData::Dictionary(map) => {
                let entries: HashMap<String, JSValue> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_jsvalue()))
                    .collect();
                JSValue::Object(entries)
            },
            ValueData::List(list) => {
                let items: Vec<JSValue> = list.iter().map(|v| v.to_jsvalue()).collect();
                JSValue::Array(items)
            },
        }
    }

    fn to_json_string(&self) -> String {
        let mut output = String::new();
        self.write_json(&mut output);
        output
    }

    fn write_json(&self, output: &mut String) {
        match &self.value {
            ValueData::Null => output.push_str("null"),
            ValueData::Bool(b) => output.push_str(if *b { "true" } else { "false" }),
            ValueData::Int(i) => output.push_str(&i.to_string()),
            ValueData::Double(d) => {
                if d.is_finite() {
                    output.push_str(&d.to_string());
                } else {
                    output.push_str("null");
                }
            },
            ValueData::String(s) => write_json_string(s, output),
            ValueData::Binary(bytes) => {
                // Encode binary as base64 string in JSON
                output.push('"');
                output.push_str(&base64::engine::general_purpose::STANDARD.encode(bytes));
                output.push('"');
            },
            ValueData::Dictionary(map) => {
                output.push('{');
                for (index, (key, value)) in map.iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                    }
                    write_json_string(key, output);
                    output.push(':');
                    value.write_json(output);
                }
                output.push('}');
            },
            ValueData::List(list) => {
                output.push('[');
                for (index, item) in list.iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                    }
                    item.write_json(output);
                }
                output.push(']');
            },
        }
    }

    fn from_json_string(json: &str) -> Result<Self, serde_json::Error> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        Ok(Self::from_serde_value(&value))
    }

    fn from_serde_value(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::new(ValueData::Null),
            serde_json::Value::Bool(b) => Self::new(ValueData::Bool(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Self::new(ValueData::Int(i))
                } else if let Some(f) = n.as_f64() {
                    Self::new(ValueData::Double(f))
                } else {
                    Self::new(ValueData::Null)
                }
            },
            serde_json::Value::String(s) => Self::new(ValueData::String(s.clone())),
            serde_json::Value::Array(arr) => {
                let list: Vec<ServoValue> = arr.iter().map(Self::from_serde_value).collect();
                Self::new(ValueData::List(list))
            },
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (key, val) in obj {
                    map.insert(key.clone(), Self::from_serde_value(val));
                }
                Self::new(ValueData::Dictionary(map))
            },
        }
    }
}

impl ServoDictionaryValue {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn get_map(&self) -> &HashMap<String, ServoValue> {
        &self.map
    }

    fn get_map_mut(&mut self) -> &mut HashMap<String, ServoValue> {
        &mut self.map
    }
}

impl ServoListValue {
    fn new() -> Self {
        Self {
            list: Vec::new(),
        }
    }

    fn get_list(&self) -> &Vec<ServoValue> {
        &self.list
    }

    fn get_list_mut(&mut self) -> &mut Vec<ServoValue> {
        &mut self.list
    }
}

fn write_json_string(value: &str, output: &mut String) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if (character as u32) < 0x20 => {
                output.push_str(&format!("\\u{:04x}", character as u32));
            },
            character => output.push(character),
        }
    }
    output.push('"');
}

/// Creates a new empty `ServoValue` (type = invalid/null).
///
/// Returns a newly allocated handle. The caller owns the handle and must
/// free it with `servo_value_free`. Returns `NULL` on allocation failure.
#[unsafe(no_mangle)]
pub extern "C" fn servo_value_create() -> *mut ServoValue {
    ServoError::catch_ffi_ptr(|| Box::into_raw(Box::new(ServoValue::new(ValueData::Null))))
}

/// Frees a `ServoValue` and its associated resources.
///
/// `value` is a handle to a `ServoValue` previously returned by
/// `servo_value_create`, `servo_value_from_json`, or obtained from a
/// dictionary/list. The ownership of `value` is transferred to this function.
///
/// # Safety
///
/// The caller must ensure that `value` is a non-null pointer previously
/// returned by a Servo value creation function and has not yet been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_free(value: *mut ServoValue) -> ServoError {
    ServoError::catch_ffi(|| {
        if value.is_null() {
            return ServoError::NullArgument;
        }
        unsafe {
            let _ = Box::from_raw(value);
        }
        ServoError::Ok
    })
}

/// Gets the type of `value`.
///
/// `value` is a handle to a `ServoValue`. The ownership remains with the caller.
/// `out_type` is a pointer to an `int32_t` that will be set to one of the
/// `SERVO_VALUE_TYPE_*` constants.
///
/// # Safety
///
/// The caller must ensure that `value` and `out_type` are valid non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_get_type(
    value: *const ServoValue,
    out_type: *mut i32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if value.is_null() {
            return ServoError::NullArgument;
        }
        if out_type.is_null() {
            return ServoError::NullArgument;
        }
        let value = unsafe { &*value };
        unsafe { *out_type = value.get_type().into() };
        ServoError::Ok
    })
}

/// Gets a boolean value from `value`.
///
/// Returns `ServoError::Ok` on success, or an error if the value is not a boolean.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_get_bool(
    value: *const ServoValue,
    out: *mut bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if value.is_null() {
            return ServoError::NullArgument;
        }
        if out.is_null() {
            return ServoError::NullArgument;
        }
        let value = unsafe { &*value };
        match &value.value {
            ValueData::Bool(b) => {
                unsafe { *out = *b };
                ServoError::Ok
            },
            _ => ServoError::InvalidState,
        }
    })
}

/// Gets an integer value from `value`.
///
/// Returns `ServoError::Ok` on success, or an error if the value is not an integer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_get_int(
    value: *const ServoValue,
    out: *mut i64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if value.is_null() {
            return ServoError::NullArgument;
        }
        if out.is_null() {
            return ServoError::NullArgument;
        }
        let value = unsafe { &*value };
        match &value.value {
            ValueData::Int(i) => {
                unsafe { *out = *i };
                ServoError::Ok
            },
            _ => ServoError::InvalidState,
        }
    })
}

/// Gets a double value from `value`.
///
/// Returns `ServoError::Ok` on success, or an error if the value is not a double.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_get_double(
    value: *const ServoValue,
    out: *mut f64,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if value.is_null() {
            return ServoError::NullArgument;
        }
        if out.is_null() {
            return ServoError::NullArgument;
        }
        let value = unsafe { &*value };
        match &value.value {
            ValueData::Double(d) => {
                unsafe { *out = *d };
                ServoError::Ok
            },
            _ => ServoError::InvalidState,
        }
    })
}

/// Gets a string value from `value` (buffer-query pattern).
///
/// Writes the string into `buffer` as a NUL-terminated UTF-8 string.
/// Returns the number of bytes required (including NUL terminator).
/// If `buffer` is `NULL` or `buffer_len` is too small, nothing is written
/// and the required length is returned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_get_string(
    value: *const ServoValue,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if value.is_null() {
        return 0;
    }
    let value = unsafe { &*value };
    match &value.value {
        ValueData::String(s) => unsafe { write_c_string(s, buffer, buffer_len) },
        _ => 0,
    }
}

/// Gets a binary value from `value` (buffer-query pattern).
///
/// Writes the binary data into `buffer`.
/// Returns the number of bytes required.
/// If `buffer` is `NULL` or `buffer_len` is too small, nothing is written
/// and the required length is returned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_get_binary(
    value: *const ServoValue,
    buffer: *mut u8,
    buffer_len: usize,
) -> usize {
    if value.is_null() {
        return 0;
    }
    let value = unsafe { &*value };
    match &value.value {
        ValueData::Binary(data) => {
            let required = data.len();
            if buffer.is_null() || buffer_len < required {
                return required;
            }
            unsafe {
                std::ptr::copy_nonoverlapping(data.as_ptr(), buffer, data.len());
            }
            required
        },
        _ => 0,
    }
}

/// Sets `value` to a boolean.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_set_bool(value: *mut ServoValue, val: bool) -> ServoError {
    ServoError::catch_ffi(|| {
        if value.is_null() {
            return ServoError::NullArgument;
        }
        let value = unsafe { &mut *value };
        value.set_bool(val);
        ServoError::Ok
    })
}

/// Sets `value` to an integer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_set_int(value: *mut ServoValue, val: i64) -> ServoError {
    ServoError::catch_ffi(|| {
        if value.is_null() {
            return ServoError::NullArgument;
        }
        let value = unsafe { &mut *value };
        value.set_int(val);
        ServoError::Ok
    })
}

/// Sets `value` to a double.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_set_double(value: *mut ServoValue, val: f64) -> ServoError {
    ServoError::catch_ffi(|| {
        if value.is_null() {
            return ServoError::NullArgument;
        }
        let value = unsafe { &mut *value };
        value.set_double(val);
        ServoError::Ok
    })
}

/// Sets `value` to a string (copies the string).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_set_string(
    value: *mut ServoValue,
    str: *const c_char,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if value.is_null() {
            return ServoError::NullArgument;
        }
        if str.is_null() {
            return ServoError::NullArgument;
        }
        let s = match unsafe { CStr::from_ptr(str) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };
        let value = unsafe { &mut *value };
        value.set_string(s);
        ServoError::Ok
    })
}

/// Sets `value` to binary data (copies the data).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_set_binary(
    value: *mut ServoValue,
    data: *const u8,
    len: usize,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if value.is_null() {
            return ServoError::NullArgument;
        }
        if data.is_null() && len > 0 {
            return ServoError::NullArgument;
        }
        let vec = unsafe { std::slice::from_raw_parts(data, len).to_vec() };
        let value = unsafe { &mut *value };
        value.set_binary(vec);
        ServoError::Ok
    })
}

/// Creates a new empty dictionary value.
#[unsafe(no_mangle)]
pub extern "C" fn servo_dictionary_create() -> *mut ServoDictionaryValue {
    ServoError::catch_ffi_ptr(|| Box::into_raw(Box::new(ServoDictionaryValue::new())))
}

/// Frees a `ServoDictionaryValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_dictionary_free(dict: *mut ServoDictionaryValue) -> ServoError {
    ServoError::catch_ffi(|| {
        if dict.is_null() {
            return ServoError::NullArgument;
        }
        unsafe {
            let _ = Box::from_raw(dict);
        }
        ServoError::Ok
    })
}

/// Sets a key in the dictionary to `value`.
///
/// `dict` is a dictionary handle. `key` is a NUL-terminated UTF-8 string.
/// `value` is a `ServoValue` handle. The dictionary takes ownership of the
/// value handle (the caller must not free it after this call).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_dictionary_set_key(
    dict: *mut ServoDictionaryValue,
    key: *const c_char,
    value: *mut ServoValue,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if dict.is_null() {
            return ServoError::NullArgument;
        }
        if key.is_null() {
            return ServoError::NullArgument;
        }
        if value.is_null() {
            return ServoError::NullArgument;
        }
        let key_str = match unsafe { CStr::from_ptr(key) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };
        // Take ownership of the value
        let value = unsafe { Box::from_raw(value) };
        let dict = unsafe { &mut *dict };
        dict.get_map_mut().insert(key_str, *value);
        ServoError::Ok
    })
}

/// Gets a value from the dictionary by key.
///
/// Returns a newly allocated `ServoValue` handle in `out_value` on success.
/// The caller owns the returned handle and must free it with `servo_value_free`.
/// Returns `ServoError::NotFound` if the key does not exist.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_dictionary_get_key(
    dict: *const ServoDictionaryValue,
    key: *const c_char,
    out_value: *mut *mut ServoValue,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if dict.is_null() {
            return ServoError::NullArgument;
        }
        if key.is_null() {
            return ServoError::NullArgument;
        }
        if out_value.is_null() {
            return ServoError::NullArgument;
        }
        let key_str = match unsafe { CStr::from_ptr(key) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };
        let dict = unsafe { &*dict };
        let map = dict.get_map();
        match map.get(&key_str) {
            Some(value) => {
                let new_value = Box::new(value.clone());
                unsafe { *out_value = Box::into_raw(new_value) };
                ServoError::Ok
            },
            None => ServoError::NotFound,
        }
    })
}

/// Checks if the dictionary has a key.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_dictionary_has_key(
    dict: *const ServoDictionaryValue,
    key: *const c_char,
    out: *mut bool,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if dict.is_null() {
            return ServoError::NullArgument;
        }
        if key.is_null() {
            return ServoError::NullArgument;
        }
        if out.is_null() {
            return ServoError::NullArgument;
        }
        let key_str = match unsafe { CStr::from_ptr(key) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidUtf8,
        };
        let dict = unsafe { &*dict };
        let map = dict.get_map();
        unsafe { *out = map.contains_key(&key_str) };
        ServoError::Ok
    })
}

/// Gets the number of keys in the dictionary.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_dictionary_get_size(
    dict: *const ServoDictionaryValue,
    out: *mut usize,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if dict.is_null() {
            return ServoError::NullArgument;
        }
        if out.is_null() {
            return ServoError::NullArgument;
        }
        let dict = unsafe { &*dict };
        let map = dict.get_map();
        unsafe { *out = map.len() };
        ServoError::Ok
    })
}

/// Gets all keys as a JSON array string (buffer-query pattern).
///
/// Returns a JSON array of strings, e.g., `["key1","key2"]`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_dictionary_get_keys(
    dict: *const ServoDictionaryValue,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if dict.is_null() {
        return 0;
    }
    let dict = unsafe { &*dict };
    let map = dict.get_map();
    let keys: Vec<&String> = map.keys().collect();
    let mut output = String::new();
    output.push('[');
    for (index, key) in keys.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        write_json_string(key, &mut output);
    }
    output.push(']');
    unsafe { write_c_string(&output, buffer, buffer_len) }
}

/// Creates a new empty list value.
#[unsafe(no_mangle)]
pub extern "C" fn servo_list_create() -> *mut ServoListValue {
    ServoError::catch_ffi_ptr(|| Box::into_raw(Box::new(ServoListValue::new())))
}

/// Frees a `ServoListValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_list_free(list: *mut ServoListValue) -> ServoError {
    ServoError::catch_ffi(|| {
        if list.is_null() {
            return ServoError::NullArgument;
        }
        unsafe {
            let _ = Box::from_raw(list);
        }
        ServoError::Ok
    })
}

/// Sets a value at `index` in the list.
///
/// If `index` >= current size, the list is extended with null values up to `index`.
/// The list takes ownership of `value` (caller must not free it).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_list_set_value(
    list: *mut ServoListValue,
    index: usize,
    value: *mut ServoValue,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if list.is_null() {
            return ServoError::NullArgument;
        }
        if value.is_null() {
            return ServoError::NullArgument;
        }
        let value = unsafe { Box::from_raw(value) };
        let list = unsafe { &mut *list };
        let vec = list.get_list_mut();
        if index >= vec.len() {
            vec.resize(index + 1, ServoValue::new(ValueData::Null));
        }
        vec[index] = *value;
        ServoError::Ok
    })
}

/// Gets a value at `index` from the list.
///
/// Returns a newly allocated `ServoValue` handle in `out_value`.
/// The caller owns the returned handle and must free it with `servo_value_free`.
/// Returns `ServoError::NotFound` if index is out of bounds.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_list_get_value(
    list: *const ServoListValue,
    index: usize,
    out_value: *mut *mut ServoValue,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if list.is_null() {
            return ServoError::NullArgument;
        }
        if out_value.is_null() {
            return ServoError::NullArgument;
        }
        let list = unsafe { &*list };
        let vec = list.get_list();
        if index >= vec.len() {
            return ServoError::NotFound;
        }
        let new_value = Box::new(vec[index].clone());
        unsafe { *out_value = Box::into_raw(new_value) };
        ServoError::Ok
    })
}

/// Gets the size of the list.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_list_get_size(
    list: *const ServoListValue,
    out: *mut usize,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if list.is_null() {
            return ServoError::NullArgument;
        }
        if out.is_null() {
            return ServoError::NullArgument;
        }
        let list = unsafe { &*list };
        let vec = list.get_list();
        unsafe { *out = vec.len() };
        ServoError::Ok
    })
}

/// Creates a `ServoValue` from a JSON string.
///
/// Parses the JSON string and returns a corresponding `ServoValue` handle.
/// Returns `NULL` on parse error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_from_json(json_str: *const c_char) -> *mut ServoValue {
    ServoError::catch_ffi_ptr(|| {
        if json_str.is_null() {
            return std::ptr::null_mut();
        }
        let json = match unsafe { CStr::from_ptr(json_str) }.to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        match ServoValue::from_json_string(json) {
            Ok(value) => Box::into_raw(Box::new(value)),
            Err(_) => std::ptr::null_mut(),
        }
    })
}

/// Serializes a `ServoValue` to a JSON string (buffer-query pattern).
///
/// Writes the JSON string into `buffer`. Returns the number of bytes required
/// (including NUL terminator). If `buffer` is `NULL` or `buffer_len` is too
/// small, nothing is written and the required length is returned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_value_to_json(
    value: *const ServoValue,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if value.is_null() {
        return 0;
    }
    let value = unsafe { &*value };
    let json = value.to_json_string();
    unsafe { write_c_string(&json, buffer, buffer_len) }
}

/// Writes `value` into a caller-provided NUL terminated UTF-8 buffer.
///
/// Returns the number of bytes required to store `value` including the
/// trailing NUL terminator. This allows the embedder to first query the
/// required length by passing `NULL` (or a too-small buffer) and then call
/// again with a large enough buffer.
///
/// # Safety
///
/// If `buffer` is non-`NULL`, it must point to at least `buffer_len` writable
/// bytes.
unsafe fn write_c_string(value: &str, buffer: *mut c_char, buffer_len: usize) -> usize {
    let required = value.len() + 1;
    if buffer.is_null() || buffer_len < required {
        return required;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(value.as_ptr(), buffer.cast::<u8>(), value.len());
        *buffer.add(value.len()) = 0;
    }
    required
}
