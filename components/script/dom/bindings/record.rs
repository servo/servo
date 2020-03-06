/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Record` (open-ended dictionary) type.

use crate::dom::bindings::conversions::jsid_to_string;
use crate::dom::bindings::str::{ByteString, DOMString, USVString};
use indexmap::IndexMap;
use js::conversions::{ConversionResult, FromJSValConvertible, ToJSValConvertible};
use js::jsapi::HandleId as RawHandleId;
use js::jsapi::JSContext;
use js::jsapi::JS_NewPlainObject;
use js::jsapi::PropertyDescriptor;
use js::jsapi::JSITER_HIDDEN;
use js::jsapi::JSITER_OWNONLY;
use js::jsapi::JSITER_SYMBOLS;
use js::jsapi::JSPROP_ENUMERATE;
use js::jsval::ObjectValue;
use js::jsval::UndefinedValue;
use js::rust::wrappers::GetPropertyKeys;
use js::rust::wrappers::JS_DefineUCProperty2;
use js::rust::wrappers::JS_GetOwnPropertyDescriptorById;
use js::rust::wrappers::JS_GetPropertyById;
use js::rust::wrappers::JS_IdToValue;
use js::rust::HandleId;
use js::rust::HandleValue;
use js::rust::IdVector;
use js::rust::MutableHandleValue;
use std::cmp::Eq;
use std::hash::Hash;
use std::marker::Sized;
use std::ops::Deref;

pub trait RecordKey: Eq + Hash + Sized {
    fn to_utf16_vec(&self) -> Vec<u16>;
    unsafe fn from_id(cx: *mut JSContext, id: HandleId) -> Result<ConversionResult<Self>, ()>;
}

impl RecordKey for DOMString {
    fn to_utf16_vec(&self) -> Vec<u16> {
        self.encode_utf16().collect::<Vec<_>>()
    }

    unsafe fn from_id(cx: *mut JSContext, id: HandleId) -> Result<ConversionResult<Self>, ()> {
        match jsid_to_string(cx, id) {
            Some(s) => Ok(ConversionResult::Success(s)),
            None => Ok(ConversionResult::Failure("Failed to get DOMString".into())),
        }
    }
}

impl RecordKey for USVString {
    fn to_utf16_vec(&self) -> Vec<u16> {
        self.0.encode_utf16().collect::<Vec<_>>()
    }

    unsafe fn from_id(cx: *mut JSContext, id: HandleId) -> Result<ConversionResult<Self>, ()> {
        rooted!(in(cx) let mut jsid_value = UndefinedValue());
        let raw_id: RawHandleId = id.into();
        JS_IdToValue(cx, *raw_id.ptr, jsid_value.handle_mut());

        USVString::from_jsval(cx, jsid_value.handle(), ())
    }
}

impl RecordKey for ByteString {
    fn to_utf16_vec(&self) -> Vec<u16> {
        self.iter().map(|&x| x as u16).collect::<Vec<u16>>()
    }

    unsafe fn from_id(cx: *mut JSContext, id: HandleId) -> Result<ConversionResult<Self>, ()> {
        rooted!(in(cx) let mut jsid_value = UndefinedValue());
        let raw_id: RawHandleId = id.into();
        JS_IdToValue(cx, *raw_id.ptr, jsid_value.handle_mut());

        ByteString::from_jsval(cx, jsid_value.handle(), ())
    }
}

/// The `Record` (open-ended dictionary) type.
#[derive(Clone, JSTraceable)]
pub struct Record<K: RecordKey, V> {
    map: IndexMap<K, V>,
}

impl<K: RecordKey, V> Record<K, V> {
    /// Create an empty `Record`.
    pub fn new() -> Self {
        Record {
            map: IndexMap::new(),
        }
    }
}

impl<K: RecordKey, V> Deref for Record<K, V> {
    type Target = IndexMap<K, V>;

    fn deref(&self) -> &IndexMap<K, V> {
        &self.map
    }
}

impl<K, V, C> FromJSValConvertible for Record<K, V>
where
    K: RecordKey,
    V: FromJSValConvertible<Config = C>,
    C: Clone,
{
    type Config = C;
    unsafe fn from_jsval(
        cx: *mut JSContext,
        value: HandleValue,
        config: C,
    ) -> Result<ConversionResult<Self>, ()> {
        if !value.is_object() {
            return Ok(ConversionResult::Failure(
                "Record value was not an object".into(),
            ));
        }

        rooted!(in(cx) let object = value.to_object());
        let mut ids = IdVector::new(cx);
        if !GetPropertyKeys(
            cx,
            object.handle(),
            JSITER_OWNONLY | JSITER_HIDDEN | JSITER_SYMBOLS,
            ids.handle_mut(),
        ) {
            return Err(());
        }

        let mut map = IndexMap::new();
        for id in &*ids {
            rooted!(in(cx) let id = *id);
            rooted!(in(cx) let mut desc = PropertyDescriptor::default());

            if !JS_GetOwnPropertyDescriptorById(cx, object.handle(), id.handle(), desc.handle_mut())
            {
                return Err(());
            }

            if (JSPROP_ENUMERATE as u32) & desc.attrs == 0 {
                continue;
            }

            let key = match K::from_id(cx, id.handle())? {
                ConversionResult::Success(key) => key,
                ConversionResult::Failure(message) => {
                    return Ok(ConversionResult::Failure(message))
                },
            };

            rooted!(in(cx) let mut property = UndefinedValue());
            if !JS_GetPropertyById(cx, object.handle(), id.handle(), property.handle_mut()) {
                return Err(());
            }

            let property = match V::from_jsval(cx, property.handle(), config.clone())? {
                ConversionResult::Success(property) => property,
                ConversionResult::Failure(message) => {
                    return Ok(ConversionResult::Failure(message))
                },
            };
            map.insert(key, property);
        }

        Ok(ConversionResult::Success(Record { map: map }))
    }
}

impl<K, V> ToJSValConvertible for Record<K, V>
where
    K: RecordKey,
    V: ToJSValConvertible,
{
    #[inline]
    unsafe fn to_jsval(&self, cx: *mut JSContext, mut rval: MutableHandleValue) {
        rooted!(in(cx) let js_object = JS_NewPlainObject(cx));
        assert!(!js_object.handle().is_null());

        rooted!(in(cx) let mut js_value = UndefinedValue());
        for (key, value) in &self.map {
            let key = key.to_utf16_vec();
            value.to_jsval(cx, js_value.handle_mut());

            assert!(JS_DefineUCProperty2(
                cx,
                js_object.handle(),
                key.as_ptr(),
                key.len(),
                js_value.handle(),
                JSPROP_ENUMERATE as u32
            ));
        }

        rval.set(ObjectValue(js_object.handle().get()));
    }
}
