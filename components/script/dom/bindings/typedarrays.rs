/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::borrow::BorrowMut;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr;
use std::sync::{Arc, Mutex};

use js::jsapi::{
    Heap, JSObject, JS_GetArrayBufferViewBuffer, JS_IsArrayBufferViewObject, NewExternalArrayBuffer,
};
use js::rust::wrappers::DetachArrayBuffer;
use js::rust::{CustomAutoRooterGuard, Handle, MutableHandleObject};
use js::typedarray::{CreateWith, TypedArray, TypedArrayElement, TypedArrayElementCreator};

use crate::script_runtime::JSContext;

pub struct HeapTypedArray<T> {
    internal: Box<Heap<*mut JSObject>>,
    phantom: PhantomData<T>,
}

unsafe impl<T> crate::dom::bindings::trace::JSTraceable for HeapTypedArray<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut js::jsapi::JSTracer) {
        self.internal.trace(tracer);
    }
}

pub fn new_initialized_heap_typed_array<T>(
    init: HeapTypedArrayInit,
) -> Result<HeapTypedArray<T>, ()>
where
    T: TypedArrayElement + TypedArrayElementCreator,
    T::Element: Clone + Copy,
{
    let heap_typed_array = match init {
        HeapTypedArrayInit::Object(js_object) => HeapTypedArray {
            internal: Heap::boxed(js_object),
            phantom: PhantomData::default(),
        },
        HeapTypedArrayInit::Info { len, cx } => {
            rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());
            let typed_array_result =
                create_typed_array_with_length::<T>(cx, len as usize, array.handle_mut());
            if typed_array_result.is_err() {
                return Err(());
            }
            let heap_typed_array = HeapTypedArray::<T>::default();
            heap_typed_array.internal.set(*array);
            heap_typed_array
        },
    };
    Ok(heap_typed_array)
}

pub enum HeapTypedArrayInit {
    Object(*mut JSObject),
    Info { len: u32, cx: JSContext },
}

impl<T> HeapTypedArray<T>
where
    T: TypedArrayElement + TypedArrayElementCreator,
    T::Element: Clone + Copy,
{
    pub fn default() -> HeapTypedArray<T> {
        HeapTypedArray {
            internal: Box::new(Heap::default()),
            phantom: PhantomData::default(),
        }
    }

    pub fn set_data(&self, cx: JSContext, data: &[T::Element]) -> Result<(), ()> {
        rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());
        let _: TypedArray<T, *mut JSObject> = create_typed_array(cx, data, array.handle_mut())?;
        self.internal.set(*array);
        Ok(())
    }

    pub fn acquire_data(&self, cx: JSContext) -> Result<Vec<T::Element>, ()> {
        assert!(self.is_initialized());
        typedarray!(in(*cx) let array: TypedArray = self.internal.get());
        let data = if let Ok(array) =
            array as Result<CustomAutoRooterGuard<'_, TypedArray<T, *mut JSObject>>, &mut ()>
        {
            let data = array.to_vec();
            let _ = self.detach_internal(cx);
            Ok(data)
        } else {
            Err(())
        };
        self.internal.set(ptr::null_mut());
        data
    }

    /// <https://tc39.es/ecma262/#sec-detacharraybuffer>
    pub fn detach_internal(&self, cx: JSContext) -> bool {
        assert!(self.is_initialized());
        let mut is_shared = false;
        unsafe {
            if JS_IsArrayBufferViewObject(*self.internal.handle()) {
                // If it is an ArrayBuffer view, get the buffer using JS_GetArrayBufferViewBuffer
                rooted!(in (*cx) let view_buffer =
                    JS_GetArrayBufferViewBuffer(*cx, self.internal.handle(), &mut is_shared));
                // This buffer is always created unshared
                debug_assert!(!is_shared);
                // Detach the ArrayBuffer
                DetachArrayBuffer(*cx, view_buffer.handle())
            } else {
                // If it's not an ArrayBuffer view, Detach the internal buffer directly
                DetachArrayBuffer(*cx, Handle::from_raw(self.internal.handle()))
            }
        }
    }

    pub fn copy_data_to(
        &self,
        cx: JSContext,
        dest: &mut [T::Element],
        source_start: usize,
        length: usize,
    ) -> Result<(), ()> {
        assert!(self.is_initialized());
        typedarray!(in(*cx) let array: TypedArray = self.internal.get());
        let Ok(array) = array as Result<CustomAutoRooterGuard<'_, TypedArray<T, *mut JSObject>>, &mut ()> else{
             return Err(())
        };
        unsafe {
            let slice = (*array).as_slice();
            dest.copy_from_slice(&slice[source_start..length]);
        }
        Ok(())
    }

    pub fn copy_data_from(
        &self,
        cx: JSContext,
        source: CustomAutoRooterGuard<TypedArray<T, *mut JSObject>>,
        dest_start: usize,
        length: usize,
    ) -> Result<(), ()> {
        assert!(self.is_initialized());
        typedarray!(in(*cx) let mut array: TypedArray = self.internal.get());
        let Ok(mut array) =  array as Result<CustomAutoRooterGuard<'_, TypedArray<T, *mut JSObject>>, &mut ()> else
        {
            return Err(())
        };
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

    pub fn get_internal(&self) -> Result<TypedArray<T, *mut JSObject>, ()> {
        TypedArray::from(self.internal.get())
    }

    pub fn internal_to_option(&self) -> Option<TypedArray<T, *mut JSObject>> {
        if self.is_initialized() {
            Some(self.get_internal().expect("Failed to get internal."))
        } else {
            warn!("Internal not initialized.");
            None
        }
    }
}

pub fn create_typed_array<T>(
    cx: JSContext,
    data: &[T::Element],
    dest: MutableHandleObject,
) -> Result<TypedArray<T, *mut JSObject>, ()>
where
    T: TypedArrayElement + TypedArrayElementCreator,
{
    let res = unsafe { TypedArray::<T, *mut JSObject>::create(*cx, CreateWith::Slice(data), dest) };

    if res.is_err() {
        Err(())
    } else {
        TypedArray::from(dest.get())
    }
}

fn create_typed_array_with_length<T>(
    cx: JSContext,
    len: usize,
    dest: MutableHandleObject,
) -> Result<TypedArray<T, *mut JSObject>, ()>
where
    T: TypedArrayElement + TypedArrayElementCreator,
{
    let res = unsafe { TypedArray::<T, *mut JSObject>::create(*cx, CreateWith::Length(len), dest) };

    if res.is_err() {
        Err(())
    } else {
        TypedArray::from(dest.get())
    }
}

pub fn create_new_external_array_buffer<T>(
    cx: JSContext,
    mapping: Arc<Mutex<Vec<T::Element>>>,
    offset: usize,
    range_size: usize,
    m_end: usize,
) -> HeapTypedArray<T>
where
    T: TypedArrayElement + TypedArrayElementCreator,
    T::Element: Clone + Copy,
{
    /// `freeFunc()` must be threadsafe, should be safely callable from any thread
    /// without causing conflicts or unexpected behavior.
    /// <https://github.com/servo/mozjs/blob/main/mozjs-sys/mozjs/js/public/ArrayBuffer.h#L89>
    unsafe extern "C" fn free_func(_contents: *mut c_void, free_user_data: *mut c_void) {
        let _ = Arc::from_raw(free_user_data as _);
    }

    unsafe {
        let mapping_slice_ptr =
            mapping.lock().unwrap().borrow_mut()[offset as usize..m_end as usize].as_mut_ptr();

        // rooted! is needed to ensure memory safety and prevent potential garbage collection issues.
        // https://github.com/mozilla-spidermonkey/spidermonkey-embedding-examples/blob/esr78/docs/GC%20Rooting%20Guide.md#performance-tweaking
        rooted!(in(*cx) let array_buffer = NewExternalArrayBuffer(
            *cx,
            range_size as usize,
            mapping_slice_ptr as _,
            Some(free_func),
            Arc::into_raw(mapping) as _,
        ));

        HeapTypedArray {
            internal: Heap::boxed(*array_buffer),
            phantom: PhantomData::default(),
        }
    }
}
