/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Deref;

use js::conversions::ToJSValConvertible;
use js::gc::{HandleId, HandleValue, MutableHandleValue};
use js::glue::{GetProxyReservedSlot, SetProxyReservedSlot};
use js::jsapi::{
    AtomToLinearString, ForwardingProxyHandler, GetLinearStringCharAt, GetLinearStringLength,
    JS_GetElement, JSAtom, JSContext, JSObject, NewArrayObject1, ObjectOpResult,
    PropertyDescriptor, PropertyKey, StringIsArrayIndex,
};
use js::jsid::VoidId;
use js::jsval::{ObjectValue, UndefinedValue};
use js::rust::wrappers::GetArrayLength;
use js::rust::{Handle, IntoHandle, IntoMutableHandle};

// Adapted from ffs dom/bindings/ObservableArrayProxyHandler.cpp
// https://github.com/mozilla-firefox/firefox/blob/d57308a2f57e36e77491b6fdd355dc18da0e94ed/dom/bindings/ObservableArrayProxyHandler.cpp

/// Get an array index from the given `jsid`. Returns `None` if the given
/// `jsid` is not an integer.
pub fn get_array_index_from_id(id: HandleId) -> Option<u32> {
    let raw_id = *id;
    if raw_id.is_int() {
        return Some(raw_id.to_int() as u32);
    }

    if raw_id.is_void() || !raw_id.is_string() {
        return None;
    }

    unsafe {
        let atom = raw_id.to_string() as *mut JSAtom;
        let s = AtomToLinearString(atom);
        if GetLinearStringLength(s) == 0 {
            return None;
        }

        let chars = [GetLinearStringCharAt(s, 0)];
        let first_char = char::decode_utf16(chars.iter().cloned())
            .next()
            .map_or('\0', |r| r.unwrap_or('\0'));
        if first_char.is_ascii_lowercase() {
            return None;
        }

        let mut i = 0;
        if StringIsArrayIndex(s, &mut i) {
            Some(i)
        } else {
            None
        }
    }

    /*let s = jsstr_to_string(cx, RUST_JSID_TO_STRING(raw_id));
    if s.len() == 0 {
        return None;
    }

    let first = s.chars().next().unwrap();
    if first.is_ascii_lowercase() {
        return None;
    }

    let mut i: u32 = 0;
    let is_array = if s.is_ascii() {
        let chars = s.as_bytes();
        StringIsArrayIndex1(chars.as_ptr() as *const _, chars.len() as u32, &mut i)
    } else {
        let chars = s.encode_utf16().collect::<Vec<u16>>();
        let slice = chars.as_slice();
        StringIsArrayIndex2(slice.as_ptr(), chars.len() as u32, &mut i)
    };

    if is_array {
        Some(i)
    } else {
        None
    }*/
}

pub trait ObservableArrayHandler {
    fn set_indexed_value(
        &self,
        cx: *mut JSContext,
        proxy: JSObject,
        backing_list: MutableHandleValue,
        index: u32,
        value: Handle<PropertyDescriptor>,
        result: &mut ObjectOpResult,
    ) -> Result<(), ()> {
        Err(())
    }

    fn on_delete_item(
        &self,
        cx: *mut JSContext,
        proxy: JSObject,
        value: HandleValue,
        index: u32,
    ) -> Result<(), ()> {
        Err(())
    }
}

struct ObservableArrayProxyHandler {
    inner: ForwardingProxyHandler,
    handler: Box<dyn ObservableArrayHandler>,
}

impl ObservableArrayProxyHandler {
    fn define_property(
        &self,
        cx: *mut JSContext,
        proxy: JSObject,
        id: Handle<PropertyKey>,
        desc: Handle<PropertyDescriptor>,
        result: &mut ObjectOpResult,
    ) -> Result<(), ()> {
        if id.get() == VoidId() {
            // if id.get().is_accesor_descriptor() {
            //     Err(result.fail_not_data_descriptor())
            // }
            if desc.hasConfigurable_() && desc.configurable_() {
                result.fail_invalid_descriptor();
                return Err(());
            }
            if desc.hasEnumerable_() && desc.enumerable_() {
                result.fail_invalid_descriptor();
                return Err(());
            }
            if desc.hasWritable_() && !desc.writable_() {
                result.fail_invalid_descriptor();
                return Err(());
            }
            if desc.hasValue_() {
                rooted!(in(cx) let mut backing_list_obj = UndefinedValue());
                self.get_backing_list_object(cx, proxy, backing_list_obj.handle_mut())?;
                return self.set_length_value(
                    cx,
                    proxy,
                    backing_list_obj.handle_mut(),
                    desc,
                    result,
                );
            }
            result.succeed();
            return Ok(());
        }
        let index = get_array_index_from_id(id);
        if let Some(index) = index {
            // if desc.is_accessor_descriptor() {
            //     result.fail_not_data_descriptor();
            //     return Err(());
            // }
            if desc.hasConfigurable_() && !desc.configurable_() {
                result.fail_invalid_descriptor();
                return Err(());
            }
            if desc.hasEnumerable_() && !desc.enumerable_() {
                result.fail_invalid_descriptor();
                return Err(());
            }
            if desc.hasWritable_() && !desc.writable_() {
                result.fail_invalid_descriptor();
                return Err(());
            }
            if desc.hasValue_() {
                rooted!(in(cx) let mut backing_list_obj = UndefinedValue());
                self.get_backing_list_object(cx, proxy, backing_list_obj.handle_mut())?;
                return self.handler.set_indexed_value(
                    cx,
                    proxy,
                    backing_list_obj.handle_mut(),
                    index,
                    desc,
                    result,
                );
            }
            result.succeed();
            return Ok(());
        }
        Err(())
        // self.define_property(cx, proxy, id, desc, result)
    }

    fn delete(
        &self,
        cx: *mut JSContext,
        proxy: JSObject,
        id: Handle<PropertyKey>,
        result: &mut ObjectOpResult,
    ) -> Result<(), ()> {
        if id.get() == VoidId() {
            result.fail_cant_delete();
            return Err(());
        }
        let index = get_array_index_from_id(id);
        if let Some(index) = index {
            rooted!(in(cx) let mut backing_list_obj = UndefinedValue());
            self.get_backing_list_object(cx, proxy, backing_list_obj.handle_mut())?;
            self.get_backing_list_object(cx, proxy, backing_list_obj.handle_mut())?;
            let mut old_len: u32 = 0;
            unsafe {
                rooted!(in(cx) let mut backing_list_obj = backing_list_obj.to_object());
                if !js::jsapi::GetArrayLength(
                    cx,
                    backing_list_obj.handle().into_handle(),
                    &mut old_len,
                ) {
                    return Err(());
                }

                if old_len != index + 1 {
                    result.fail_bad_index();
                    return Err(());
                }

                rooted!(in(cx) let mut value = UndefinedValue());
                if !js::jsapi::JS_GetElement(
                    cx,
                    backing_list_obj.handle().into_handle(),
                    index,
                    value.handle_mut().into_handle_mut(),
                ) {
                    return Err(());
                }
                self.handler
                    .on_delete_item(cx, proxy, value.handle(), index)?;
                if !js::jsapi::SetArrayLength(cx, backing_list_obj.handle().into_handle(), index) {
                    return Err(());
                }
                result.succeed();
                return Ok(());
            }
        }
        Err(())
        // self.delete(cx, proxy, id, result)
    }

    fn get(
        &self,
        cx: *mut JSContext,
        proxy: JSObject,
        receiver: HandleValue,
        id: Handle<PropertyKey>,
        mut vp: MutableHandleValue,
    ) -> Result<(), ()> {
        rooted!(in(cx) let mut backing_list_obj = UndefinedValue());
        self.get_backing_list_object(cx, proxy, backing_list_obj.handle_mut())?;
        let mut length: u32 = 0;
        unsafe {
            rooted!(in(cx) let mut backing_list_obj = backing_list_obj.to_object());
            if !js::jsapi::GetArrayLength(cx, backing_list_obj.handle().into_handle(), &mut length)
            {
                return Err(());
            }

            if id.get() == VoidId() {
                length.to_jsval(cx, vp);
                return Ok(());
            }
            let index = get_array_index_from_id(id);
            if let Some(index) = index {
                if index >= length {
                    vp.set(UndefinedValue());
                    return Ok(());
                }
                if !JS_GetElement(
                    cx,
                    backing_list_obj.handle().into_handle(),
                    index,
                    vp.into_handle_mut(),
                ) {
                    return Err(());
                }
                return Ok(());
            }
            Err(())
            // self.get(cx, proxy, receiver, id, vp)
        }
    }

    fn get_own_property_descriptor(
        &self,
        cx: *mut JSContext,
        proxy: JSObject,
        id: Handle<PropertyKey>,
        mut desc: MutableHandleValue,
    ) -> Result<(), ()> {
        rooted!(in(cx) let mut backing_list_obj = UndefinedValue());
        self.get_backing_list_object(cx, proxy, backing_list_obj.handle_mut())?;
        rooted!(in(cx) let mut backing_list_obj = backing_list_obj.to_object());
        let mut length: u32 = 0;
        unsafe {
            if !GetArrayLength(cx, backing_list_obj.handle(), &mut length) {
                return Err(());
            }
        }
        if id.get() == VoidId() {
            rooted!(in(cx) let mut value = UndefinedValue());
            unsafe {
                length.to_jsval(cx, value.handle_mut());
            }
            let mut desc = PropertyDescriptor::default();
            desc.set_writable_(true);
            todo!();
        }
        let index = get_array_index_from_id(id);
        if let Some(index) = index {
            if index >= length {
                return Ok(());
            }
        }
        todo!();
    }

    fn has(
        &self,
        cx: *mut JSContext,
        proxy: JSObject,
        id: Handle<PropertyKey>,
    ) -> Result<bool, ()> {
        if id.get() == VoidId() {
            return Ok(true);
        }
        let index = get_array_index_from_id(id);
        if let Some(index) = index {
            let mut length: u32 = 0;
            self.get_backing_list_length(cx, proxy, &mut length)?;
            return Ok(index < length);
        }
        Err(())
        // self.has(cx, proxy, id)
    }

    // bool ObservableArrayProxyHandler::ownPropertyKeys(
    //     JSContext* aCx, JS::Handle<JSObject*> aProxy,
    //     JS::MutableHandleVector<jsid> aProps) const {
    //   uint32_t length = 0;
    //   if (!GetBackingListLength(aCx, aProxy, &length)) {
    //     return false;
    //   }
    //
    //   for (int32_t i = 0; i < int32_t(length); i++) {
    //     if (!aProps.append(JS::PropertyKey::Int(i))) {
    //       return false;
    //     }
    //   }
    //   return ForwardingProxyHandler::ownPropertyKeys(aCx, aProxy, aProps);
    // }

    fn prevent_extensions(
        &self,
        cx: *mut JSContext,
        proxy: JSObject,
        result: &mut ObjectOpResult,
    ) -> Result<(), ()> {
        result.fail_cant_prevent_extensions();
        Err(())
    }

    // bool ObservableArrayProxyHandler::set(JSContext* aCx,
    //                                       JS::Handle<JSObject*> aProxy,
    //                                       JS::Handle<JS::PropertyKey> aId,
    //                                       JS::Handle<JS::Value> aV,
    //                                       JS::Handle<JS::Value> aReceiver,
    //                                       JS::ObjectOpResult& aResult) const {
    //   if (aId.get() == s_length_id) {
    //     JS::Rooted<JSObject*> backingListObj(aCx);
    //     if (!GetBackingListObject(aCx, aProxy, &backingListObj)) {
    //       return false;
    //     }
    //
    //     return SetLength(aCx, aProxy, backingListObj, aV, aResult);
    //   }
    //   uint32_t index = GetArrayIndexFromId(aId);
    //   if (IsArrayIndex(index)) {
    //     JS::Rooted<JSObject*> backingListObj(aCx);
    //     if (!GetBackingListObject(aCx, aProxy, &backingListObj)) {
    //       return false;
    //     }
    //
    //     return SetIndexedValue(aCx, aProxy, backingListObj, index, aV, aResult);
    //   }
    //   return ForwardingProxyHandler::set(aCx, aProxy, aId, aV, aReceiver, aResult);
    // }
    fn get_backing_list_object(
        &self,
        cx: *mut JSContext,
        mut proxy: JSObject,
        mut backing_list_object: MutableHandleValue,
    ) -> Result<(), ()> {
        // Retrieve the backing list object from the reserved slot on the proxy
        // object. If it doesn't exist yet, create it.
        rooted!(in(cx) let mut slot_value = UndefinedValue());
        const OBSERVABLE_ARRAY_BACKING_LIST_OBJECT_SLOT: u32 = 1;
        unsafe {
            GetProxyReservedSlot(
                &mut proxy,
                OBSERVABLE_ARRAY_BACKING_LIST_OBJECT_SLOT,
                slot_value.as_mut(),
            );
        }
        if slot_value.is_undefined() {
            unsafe {
                rooted!(in(cx) let mut new_backing_list_obj = NewArrayObject1(cx, 0));
                if new_backing_list_obj.is_null() {
                    return Err(());
                }
                slot_value.set(ObjectValue(new_backing_list_obj.get()));
                SetProxyReservedSlot(
                    &mut proxy,
                    OBSERVABLE_ARRAY_BACKING_LIST_OBJECT_SLOT,
                    slot_value.as_mut(),
                );
            }
        }
        unsafe {
            slot_value.to_object().to_jsval(cx, backing_list_object);
        }
        Ok(())
    }

    fn get_backing_list_length(
        &self,
        cx: *mut JSContext,
        proxy: JSObject,
        length: &mut u32,
    ) -> Result<(), ()> {
        rooted!(in(cx) let mut backing_list_obj = UndefinedValue());
        self.get_backing_list_object(cx, proxy, backing_list_obj.handle_mut())?;
        unsafe {
            rooted!(in(cx) let mut backing_list_obj = backing_list_obj.to_object());
            if !js::jsapi::GetArrayLength(cx, backing_list_obj.handle().into_handle(), length) {
                return Err(());
            }
        }
        Ok(())
    }

    fn set_length(
        &self,
        cx: *mut JSContext,
        proxy: JSObject,
        backing_list: MutableHandleValue,
        length: u32,
        result: &mut ObjectOpResult,
    ) -> Result<(), ()> {
        let mut old_len: u32 = 0;
        unsafe {
            rooted!(in(cx) let mut backing_list_obj = backing_list.to_object());
            if !js::jsapi::GetArrayLength(cx, backing_list_obj.handle().into_handle(), &mut old_len)
            {
                return Err(());
            }

            if length > old_len {
                result.fail_bad_array_length();
                return Err(());
            }

            let mut ok = true;
            let mut len = old_len;
            for _ in length..old_len {
                let index_to_delete = len - 1;
                rooted!(in(cx) let mut value = UndefinedValue());
                if !JS_GetElement(
                    cx,
                    backing_list_obj.handle().into_handle(),
                    index_to_delete,
                    value.handle_mut().into_handle_mut(),
                ) {
                    ok = false;
                    break;
                }

                if self
                    .handler
                    .on_delete_item(cx, proxy, value.handle(), index_to_delete)
                    .is_err()
                {
                    ok = false;
                    break;
                }
                len -= 1;
            }

            if !js::jsapi::SetArrayLength(cx, backing_list_obj.handle().into_handle(), len) {
                return Err(());
            }

            if ok {
                result.succeed();
                Ok(())
            } else {
                Err(())
            }
        }
    }

    fn set_length_value(
        &self,
        cx: *mut JSContext,
        proxy: JSObject,
        backing_list: MutableHandleValue,
        value: Handle<PropertyDescriptor>,
        result: &mut ObjectOpResult,
    ) -> Result<(), ()> {
        warn!("unimplemented ObservableArrayProxyHandler::set_length_value");
        return Ok(());
        // self.set_length(cx, proxy, backing_list, 0, result);
    }
}

impl Deref for ObservableArrayProxyHandler {
    type Target = ForwardingProxyHandler;

    fn deref(&self) -> &ForwardingProxyHandler {
        &self.inner
    }
}

// bool ObservableArrayProxyHandler::defineProperty(
//     JSContext* aCx, JS::Handle<JSObject*> aProxy,
//     JS::Handle<JS::PropertyKey> aId, JS::Handle<JS::PropertyDescriptor> aDesc,
//     JS::ObjectOpResult& aResult) const {
//   if (aId.get() == s_length_id) {
//     if (aDesc.isAccessorDescriptor()) {
//       return aResult.failNotDataDescriptor();
//     }
//     if (aDesc.hasConfigurable() && aDesc.configurable()) {
//       return aResult.failInvalidDescriptor();
//     }
//     if (aDesc.hasEnumerable() && aDesc.enumerable()) {
//       return aResult.failInvalidDescriptor();
//     }
//     if (aDesc.hasWritable() && !aDesc.writable()) {
//       return aResult.failInvalidDescriptor();
//     }
//     if (aDesc.hasValue()) {
//       JS::Rooted<JSObject*> backingListObj(aCx);
//       if (!GetBackingListObject(aCx, aProxy, &backingListObj)) {
//         return false;
//       }
//
//       return SetLength(aCx, aProxy, backingListObj, aDesc.value(), aResult);
//     }
//     return aResult.succeed();
//   }
//   uint32_t index = GetArrayIndexFromId(aId);
//   if (IsArrayIndex(index)) {
//     if (aDesc.isAccessorDescriptor()) {
//       return aResult.failNotDataDescriptor();
//     }
//     if (aDesc.hasConfigurable() && !aDesc.configurable()) {
//       return aResult.failInvalidDescriptor();
//     }
//     if (aDesc.hasEnumerable() && !aDesc.enumerable()) {
//       return aResult.failInvalidDescriptor();
//     }
//     if (aDesc.hasWritable() && !aDesc.writable()) {
//       return aResult.failInvalidDescriptor();
//     }
//     if (aDesc.hasValue()) {
//       JS::Rooted<JSObject*> backingListObj(aCx);
//       if (!GetBackingListObject(aCx, aProxy, &backingListObj)) {
//         return false;
//       }
//
//       return SetIndexedValue(aCx, aProxy, backingListObj, index, aDesc.value(),
//                              aResult);
//     }
//     return aResult.succeed();
//   }
//
//   return ForwardingProxyHandler::defineProperty(aCx, aProxy, aId, aDesc,
//                                                 aResult);
// }
//
// bool ObservableArrayProxyHandler::delete_(JSContext* aCx,
//                                           JS::Handle<JSObject*> aProxy,
//                                           JS::Handle<JS::PropertyKey> aId,
//                                           JS::ObjectOpResult& aResult) const {
//   if (aId.get() == s_length_id) {
//     return aResult.failCantDelete();
//   }
//   uint32_t index = GetArrayIndexFromId(aId);
//   if (IsArrayIndex(index)) {
//     JS::Rooted<JSObject*> backingListObj(aCx);
//     if (!GetBackingListObject(aCx, aProxy, &backingListObj)) {
//       return false;
//     }
//
//     uint32_t oldLen = 0;
//     if (!JS::GetArrayLength(aCx, backingListObj, &oldLen)) {
//       return false;
//     }
//
//     // We do not follow the spec (step 3.3 in
//     // https://webidl.spec.whatwg.org/#es-observable-array-deleteProperty)
//     // is because `oldLen - 1` could be `-1` if the backing list is empty, but
//     // `oldLen` is `uint32_t` in practice. See also
//     // https://github.com/whatwg/webidl/issues/1049.
//     if (oldLen != index + 1) {
//       return aResult.failBadIndex();
//     }
//
//     JS::Rooted<JS::Value> value(aCx);
//     if (!JS_GetElement(aCx, backingListObj, index, &value)) {
//       return false;
//     }
//
//     if (!OnDeleteItem(aCx, aProxy, value, index)) {
//       return false;
//     }
//
//     if (!JS::SetArrayLength(aCx, backingListObj, index)) {
//       return false;
//     }
//
//     return aResult.succeed();
//   }
//   return ForwardingProxyHandler::delete_(aCx, aProxy, aId, aResult);
// }
//
// bool ObservableArrayProxyHandler::get(JSContext* aCx,
//                                       JS::Handle<JSObject*> aProxy,
//                                       JS::Handle<JS::Value> aReceiver,
//                                       JS::Handle<JS::PropertyKey> aId,
//                                       JS::MutableHandle<JS::Value> aVp) const {
//   JS::Rooted<JSObject*> backingListObj(aCx);
//   if (!GetBackingListObject(aCx, aProxy, &backingListObj)) {
//     return false;
//   }
//
//   uint32_t length = 0;
//   if (!JS::GetArrayLength(aCx, backingListObj, &length)) {
//     return false;
//   }
//
//   if (aId.get() == s_length_id) {
//     return ToJSValue(aCx, length, aVp);
//   }
//   uint32_t index = GetArrayIndexFromId(aId);
//   if (IsArrayIndex(index)) {
//     if (index >= length) {
//       aVp.setUndefined();
//       return true;
//     }
//     return JS_GetElement(aCx, backingListObj, index, aVp);
//   }
//   return ForwardingProxyHandler::get(aCx, aProxy, aReceiver, aId, aVp);
// }
//
// bool ObservableArrayProxyHandler::getOwnPropertyDescriptor(
//     JSContext* aCx, JS::Handle<JSObject*> aProxy,
//     JS::Handle<JS::PropertyKey> aId,
//     JS::MutableHandle<Maybe<JS::PropertyDescriptor>> aDesc) const {
//   JS::Rooted<JSObject*> backingListObj(aCx);
//   if (!GetBackingListObject(aCx, aProxy, &backingListObj)) {
//     return false;
//   }
//
//   uint32_t length = 0;
//   if (!JS::GetArrayLength(aCx, backingListObj, &length)) {
//     return false;
//   }
//
//   if (aId.get() == s_length_id) {
//     JS::Rooted<JS::Value> value(aCx, JS::NumberValue(length));
//     aDesc.set(Some(JS::PropertyDescriptor::Data(
//         value, {JS::PropertyAttribute::Writable})));
//     return true;
//   }
//   uint32_t index = GetArrayIndexFromId(aId);
//   if (IsArrayIndex(index)) {
//     if (index >= length) {
//       return true;
//     }
//
//     JS::Rooted<JS::Value> value(aCx);
//     if (!JS_GetElement(aCx, backingListObj, index, &value)) {
//       return false;
//     }
//
//     aDesc.set(Some(JS::PropertyDescriptor::Data(
//         value,
//         {JS::PropertyAttribute::Configurable, JS::PropertyAttribute::Writable,
//          JS::PropertyAttribute::Enumerable})));
//     return true;
//   }
//   return ForwardingProxyHandler::getOwnPropertyDescriptor(aCx, aProxy, aId,
//                                                           aDesc);
// }
//
// bool ObservableArrayProxyHandler::has(JSContext* aCx,
//                                       JS::Handle<JSObject*> aProxy,
//                                       JS::Handle<JS::PropertyKey> aId,
//                                       bool* aBp) const {
//   if (aId.get() == s_length_id) {
//     *aBp = true;
//     return true;
//   }
//   uint32_t index = GetArrayIndexFromId(aId);
//   if (IsArrayIndex(index)) {
//     uint32_t length = 0;
//     if (!GetBackingListLength(aCx, aProxy, &length)) {
//       return false;
//     }
//
//     *aBp = (index < length);
//     return true;
//   }
//   return ForwardingProxyHandler::has(aCx, aProxy, aId, aBp);
// }
//
// bool ObservableArrayProxyHandler::ownPropertyKeys(
//     JSContext* aCx, JS::Handle<JSObject*> aProxy,
//     JS::MutableHandleVector<jsid> aProps) const {
//   uint32_t length = 0;
//   if (!GetBackingListLength(aCx, aProxy, &length)) {
//     return false;
//   }
//
//   for (int32_t i = 0; i < int32_t(length); i++) {
//     if (!aProps.append(JS::PropertyKey::Int(i))) {
//       return false;
//     }
//   }
//   return ForwardingProxyHandler::ownPropertyKeys(aCx, aProxy, aProps);
// }
//
// bool ObservableArrayProxyHandler::preventExtensions(
//     JSContext* aCx, JS::Handle<JSObject*> aProxy,
//     JS::ObjectOpResult& aResult) const {
//   return aResult.failCantPreventExtensions();
// }
//
// bool ObservableArrayProxyHandler::set(JSContext* aCx,
//                                       JS::Handle<JSObject*> aProxy,
//                                       JS::Handle<JS::PropertyKey> aId,
//                                       JS::Handle<JS::Value> aV,
//                                       JS::Handle<JS::Value> aReceiver,
//                                       JS::ObjectOpResult& aResult) const {
//   if (aId.get() == s_length_id) {
//     JS::Rooted<JSObject*> backingListObj(aCx);
//     if (!GetBackingListObject(aCx, aProxy, &backingListObj)) {
//       return false;
//     }
//
//     return SetLength(aCx, aProxy, backingListObj, aV, aResult);
//   }
//   uint32_t index = GetArrayIndexFromId(aId);
//   if (IsArrayIndex(index)) {
//     JS::Rooted<JSObject*> backingListObj(aCx);
//     if (!GetBackingListObject(aCx, aProxy, &backingListObj)) {
//       return false;
//     }
//
//     return SetIndexedValue(aCx, aProxy, backingListObj, index, aV, aResult);
//   }
//   return ForwardingProxyHandler::set(aCx, aProxy, aId, aV, aReceiver, aResult);
// }
//
// bool ObservableArrayProxyHandler::GetBackingListObject(
//     JSContext* aCx, JS::Handle<JSObject*> aProxy,
//     JS::MutableHandle<JSObject*> aBackingListObject) const {
//   // Retrieve the backing list object from the reserved slot on the proxy
//   // object. If it doesn't exist yet, create it.
//   JS::Rooted<JS::Value> slotValue(aCx);
//   slotValue = js::GetProxyReservedSlot(
//       aProxy, OBSERVABLE_ARRAY_BACKING_LIST_OBJECT_SLOT);
//   if (slotValue.isUndefined()) {
//     JS::Rooted<JSObject*> newBackingListObj(aCx);
//     newBackingListObj.set(JS::NewArrayObject(aCx, 0));
//     if (NS_WARN_IF(!newBackingListObj)) {
//       return false;
//     }
//     slotValue = JS::ObjectValue(*newBackingListObj);
//     js::SetProxyReservedSlot(aProxy, OBSERVABLE_ARRAY_BACKING_LIST_OBJECT_SLOT,
//                              slotValue);
//   }
//   aBackingListObject.set(&slotValue.toObject());
//   return true;
// }
//
// bool ObservableArrayProxyHandler::GetBackingListLength(
//     JSContext* aCx, JS::Handle<JSObject*> aProxy, uint32_t* aLength) const {
//   JS::Rooted<JSObject*> backingListObj(aCx);
//   if (!GetBackingListObject(aCx, aProxy, &backingListObj)) {
//     return false;
//   }
//
//   return JS::GetArrayLength(aCx, backingListObj, aLength);
// }
//
// bool ObservableArrayProxyHandler::SetLength(JSContext* aCx,
//                                             JS::Handle<JSObject*> aProxy,
//                                             uint32_t aLength) const {
//   JS::Rooted<JSObject*> backingListObj(aCx);
//   if (!GetBackingListObject(aCx, aProxy, &backingListObj)) {
//     return false;
//   }
//
//   JS::ObjectOpResult result;
//   if (!SetLength(aCx, aProxy, backingListObj, aLength, result)) {
//     return false;
//   }
//
//   return result ? true : result.reportError(aCx, aProxy);
// }
//
// bool ObservableArrayProxyHandler::SetLength(JSContext* aCx,
//                                             JS::Handle<JSObject*> aProxy,
//                                             JS::Handle<JSObject*> aBackingList,
//                                             uint32_t aLength,
//                                             JS::ObjectOpResult& aResult) const {
//   uint32_t oldLen;
//   if (!JS::GetArrayLength(aCx, aBackingList, &oldLen)) {
//     return false;
//   }
//
//   if (aLength > oldLen) {
//     return aResult.failBadArrayLength();
//   }
//
//   bool ok = true;
//   uint32_t len = oldLen;
//   for (; len > aLength; len--) {
//     uint32_t indexToDelete = len - 1;
//     JS::Rooted<JS::Value> value(aCx);
//     if (!JS_GetElement(aCx, aBackingList, indexToDelete, &value)) {
//       ok = false;
//       break;
//     }
//
//     if (!OnDeleteItem(aCx, aProxy, value, indexToDelete)) {
//       ok = false;
//       break;
//     }
//   }
//
//   return JS::SetArrayLength(aCx, aBackingList, len) && ok ? aResult.succeed()
//                                                           : false;
// }
//
// bool ObservableArrayProxyHandler::SetLength(JSContext* aCx,
//                                             JS::Handle<JSObject*> aProxy,
//                                             JS::Handle<JSObject*> aBackingList,
//                                             JS::Handle<JS::Value> aValue,
//                                             JS::ObjectOpResult& aResult) const {
//   uint32_t uint32Len;
//   if (!ToUint32(aCx, aValue, &uint32Len)) {
//     return false;
//   }
//
//   double numberLen;
//   if (!ToNumber(aCx, aValue, &numberLen)) {
//     return false;
//   }
//
//   if (uint32Len != numberLen) {
//     JS_ReportErrorNumberASCII(aCx, js::GetErrorMessage, nullptr,
//                               JSMSG_BAD_INDEX);
//     return false;
//   }
//
//   return SetLength(aCx, aProxy, aBackingList, uint32Len, aResult);
// }
