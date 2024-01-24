/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::ptr;

use js::jsapi::{Heap, JSObject, JS_GetArrayBufferViewBuffer};
use js::rust::wrappers::DetachArrayBuffer;
use js::rust::{CustomAutoRooterGuard, MutableHandleObject};
use js::typedarray::{
    CreateWith, Float32Array, JSObjectStorage, TypedArray, TypedArrayElement,
    TypedArrayElementCreator,
};

use crate::script_runtime::JSContext;

#[derive(Default, JSTraceable)]
pub struct HeapFloat32Array {
    internal: Box<Heap<*mut JSObject>>,
}

impl HeapFloat32Array {
    pub fn set_data(&self, cx: JSContext, data: &[f32]) -> Result<(), ()> {
        rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());
        let _: Float32Array = create_typed_array(cx, data, array.handle_mut())?;
        self.internal.set(*array);
        Ok(())
    }

    pub fn acquire_data(&self, cx: JSContext) -> Result<Vec<f32>, ()> {
        assert!(self.is_initialized());
        typedarray!(in(*cx) let array: Float32Array = self.internal.get());
        let data = if let Ok(array) = array {
            let data = array.to_vec();
            let mut is_shared = false;
            unsafe {
                rooted!(in (*cx) let view_buffer =
                    JS_GetArrayBufferViewBuffer(*cx, self.internal.handle(), &mut is_shared));
                // This buffer is always created unshared
                debug_assert!(!is_shared);
                let _ = DetachArrayBuffer(*cx, view_buffer.handle());
            }
            Ok(data)
        } else {
            Err(())
        };
        self.internal.set(ptr::null_mut());
        data
    }

    pub fn copy_data_to(
        &self,
        cx: JSContext,
        dest: &mut [f32],
        source_start: usize,
        length: usize,
    ) -> Result<(), ()> {
        assert!(self.is_initialized());
        typedarray!(in(*cx) let array: Float32Array = self.internal.get());
        let Ok(array) = array else { return Err(()) };
        unsafe {
            let slice = (*array).as_slice();
            dest.copy_from_slice(&slice[source_start..length]);
        }
        Ok(())
    }

    pub fn copy_data_from(
        &self,
        cx: JSContext,
        source: CustomAutoRooterGuard<Float32Array>,
        dest_start: usize,
        length: usize,
    ) -> Result<(), ()> {
        assert!(self.is_initialized());
        typedarray!(in(*cx) let mut array: Float32Array = self.internal.get());
        let Ok(mut array) = array else { return Err(()) };
        unsafe {
            let slice = (*array).as_mut_slice();
            let (_, dest) = slice.split_at_mut(dest_start);
            dest[0..length].copy_from_slice(&source.as_slice()[0..length])
        }
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        !self.internal.get().is_null()
    }

    pub fn get_internal(&self) -> Result<Float32Array, ()> {
        Float32Array::from(self.internal.get())
    }

    pub fn internal_to_option(&self) -> Option<Float32Array> {
        if self.is_initialized() {
            Some(self.get_internal().expect("Failed to get internal."))
        } else {
            warn!("Internal not initialized.");
            None
        }
    }
}

pub fn create_typed_array<T, S>(
    cx: JSContext,
    data: &[T::Element],
    dest: MutableHandleObject,
) -> Result<TypedArray<T, S>, ()>
where
    T: TypedArrayElementCreator + TypedArrayElement,
    S: JSObjectStorage,
{
    let res = unsafe { TypedArray::<T, S>::create(*cx, CreateWith::Slice(data), dest) };

    if res.is_err() {
        Err(())
    } else {
        TypedArray::from(dest.get())
    }
}
