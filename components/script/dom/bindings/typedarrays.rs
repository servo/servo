/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::ptr;

use js::jsapi::{Heap, JSObject, JS_GetArrayBufferViewBuffer};
use js::rust::wrappers::DetachArrayBuffer;
use js::rust::CustomAutoRooterGuard;
use js::typedarray::{CreateWith, Float32Array};

use crate::script_runtime::JSContext;

#[derive(Default, JSTraceable)]
pub struct Float32ArrayOnHeap {
    internal: Heap<*mut JSObject>,
}

impl Float32ArrayOnHeap {
    pub fn set_data(&self, cx: JSContext, data: &[f32]) -> Result<(), ()> {
        rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());

        let res =
            unsafe { Float32Array::create(*cx, CreateWith::Slice(&data), array.handle_mut()) };

        if res.is_err() {
            return Err(());
        }

        self.internal.set(*array);
        Ok(())
    }

    pub fn acquire_data(&self, cx: JSContext) -> Result<Vec<f32>, ()> {
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
        start: usize,
        end: usize,
    ) -> Result<(), ()> {
        assert!(self.is_set());
        typedarray!(in(*cx) let array: Float32Array = self.internal.get());
        if array.is_err() {
            return Err(());
        }
        let array = array.unwrap();
        unsafe {
            let slice = (*array).as_slice();
            dest.clone_from_slice(&slice[start..end]);
        }
        Ok(())
    }

    pub fn copy_data_from(
        &self,
        cx: JSContext,
        source: CustomAutoRooterGuard<Float32Array>,
        start: usize,
        end: usize,
    ) -> Result<(), ()> {
        assert!(self.is_set());
        typedarray!(in(*cx) let mut array: Float32Array = self.internal.get());
        if array.is_err() {
            return Err(());
        }
        let mut array = array.unwrap();
        unsafe {
            let slice = (*array).as_mut_slice();
            let (_, dest) = slice.split_at_mut(start);
            dest[0..end].copy_from_slice(&source.as_slice()[0..end])
        }
        Ok(())
    }

    pub fn is_set(&self) -> bool {
        !self.internal.get().is_null()
    }

    pub fn get_internal(&self) -> Result<Float32Array, ()> {
        Float32Array::from(self.internal.get())
    }
}
