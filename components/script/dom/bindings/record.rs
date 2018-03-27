/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The `Record` (open-ended dictionary) type.

use dom::bindings::conversions::jsid_to_string;
use dom::bindings::str::{ByteString, DOMString, USVString};
use js::conversions::{FromJSValConvertible, ToJSValConvertible, ConversionResult};
use js::jsapi::GetPropertyKeys;
use js::jsapi::HandleId;
use js::jsapi::HandleValue;
use js::jsapi::JSContext;
use js::jsapi::JSITER_OWNONLY;
use js::jsapi::JSPROP_ENUMERATE;
use js::jsapi::JS_DefineUCProperty2;
use js::jsapi::JS_GetOwnPropertyDescriptorById;
use js::jsapi::JS_GetPropertyById;
use js::jsapi::JS_IdToValue;
use js::jsapi::JS_NewPlainObject;
use js::jsapi::MutableHandle;
use js::jsapi::MutableHandleValue;
use js::jsapi::PropertyDescriptor;
use js::jsval::ObjectValue;
use js::jsval::UndefinedValue;
use js::rust::IdVector;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::Sized;
use std::ops::Deref;

pub trait RecordKey : Eq + Hash + Sized {
    fn to_utf16_vec(&self) -> Vec<u16>;
    unsafe fn from_id(cx: *mut JSContext, id: HandleId) -> Option<Self>;
}

impl RecordKey for DOMString {
    fn to_utf16_vec(&self) -> Vec<u16> {
        self.encode_utf16().collect::<Vec<_>>()
    }

    unsafe fn from_id(cx: *mut JSContext, id: HandleId) -> Option<DOMString> {
        jsid_to_string(cx, id)
    }
}

impl RecordKey for USVString {
    fn to_utf16_vec(&self) -> Vec<u16> {
        self.0.encode_utf16().collect::<Vec<_>>()
    }

    unsafe fn from_id(cx: *mut JSContext, id: HandleId) -> Option<USVString>      {
        rooted!(in(cx) let mut jsid_value = UndefinedValue());
        JS_IdToValue(cx, *id.ptr, jsid_value.handle_mut());

        match USVString::from_jsval(cx, jsid_value.handle(), ()).unwrap() {
            ConversionResult::Success(s) => return Some(s),
            ConversionResult::Failure(_) => return None
        }
    }
}

impl RecordKey for ByteString {
    fn to_utf16_vec(&self) -> Vec<u16> {
        self.iter().map(|&x| x as u16).collect::<Vec<u16>>()
    }

    unsafe fn from_id(cx: *mut JSContext, id: HandleId) -> Option<ByteString> {
        rooted!(in(cx) let mut jsid_value = UndefinedValue());
        JS_IdToValue(cx, *id.ptr, jsid_value.handle_mut());

        match ByteString::from_jsval(cx, jsid_value.handle(), ()).unwrap() {
            ConversionResult::Success(s) => return Some(s),
            ConversionResult::Failure(_) => return None
        }
    }
}

/// The `Record` (open-ended dictionary) type.
#[derive(Clone, JSTraceable)]
pub struct Record<K: RecordKey, V> {
    map: HashMap<K, V>,
}

impl<K: RecordKey, V> Record<K, V> {
    /// Create an empty `Record`.
    pub fn new() -> Self {
        Record {
            map: HashMap::new(),
        }
    }
}

impl<K: RecordKey, V> Deref for Record<K, V> {
    type Target = HashMap<K, V>;

    fn deref(&self) -> &HashMap<K, V> {
        &self.map
    }
}

impl<K, V, C> FromJSValConvertible for Record<K, V>
    where K: RecordKey,
          V: FromJSValConvertible<Config=C>,
          C: Clone,
{
    type Config = C;
    unsafe fn from_jsval(cx: *mut JSContext, value: HandleValue, config: C)
                         -> Result<ConversionResult<Self>, ()> {
        if !value.is_object() {
            return Ok(ConversionResult::Failure("Record value was not an object".into()));
        }

        rooted!(in(cx) let object = value.to_object());
        let ids = IdVector::new(cx);
        assert!(GetPropertyKeys(cx, object.handle(), JSITER_OWNONLY, ids.get()));

        let mut map = HashMap::new();
        for id in &*ids {
            rooted!(in(cx) let id = *id);
            rooted!(in(cx) let mut desc = PropertyDescriptor::default());

            if !JS_GetOwnPropertyDescriptorById(cx, object.handle(), id.handle(), desc.handle_mut()) {
                return Err(());
            }

            if JSPROP_ENUMERATE & desc.attrs == 0 {
                continue;
            }

            rooted!(in(cx) let mut property = UndefinedValue());
            if !JS_GetPropertyById(cx, object.handle(), id.handle(), property.handle_mut()) {
                return Err(());
            }

            let property = match V::from_jsval(cx, property.handle(), config.clone())? {
                ConversionResult::Success(property) => property,
                ConversionResult::Failure(message) => return Ok(ConversionResult::Failure(message)),
            };

            let key = K::from_id(cx, id.handle()).unwrap();
            map.insert(key, property);
        }

        Ok(ConversionResult::Success(Record {
            map: map,
        }))
    }
}

impl<K, V> ToJSValConvertible for Record<K, V>
    where K: RecordKey,
          V: ToJSValConvertible  {
    #[inline]
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {
        rooted!(in(cx) let js_object = JS_NewPlainObject(cx));
        assert!(!js_object.handle().is_null());

        rooted!(in(cx) let mut js_value = UndefinedValue());
        for (key, value) in &self.map {
            let key = key.to_utf16_vec();
            value.to_jsval(cx, js_value.handle_mut());

            assert!(JS_DefineUCProperty2(cx,
                                         js_object.handle(),
                                         key.as_ptr(),
                                         key.len(),
                                         js_value.handle(),
                                         JSPROP_ENUMERATE,
                                         None,
                                         None));
        }

        rval.set(ObjectValue(js_object.handle().get()));
    }
}
