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

/// <https://webidl.spec.whatwg.org/#BufferSource>
#[allow(dead_code)]
pub enum BufferSource {
    Int8Array(Box<Heap<*mut JSObject>>),
    Int16Array(Box<Heap<*mut JSObject>>),
    Int32Array(Box<Heap<*mut JSObject>>),
    Uint8Array(Box<Heap<*mut JSObject>>),
    Uint16Array(Box<Heap<*mut JSObject>>),
    Uint32Array(Box<Heap<*mut JSObject>>),
    Uint8ClampedArray(Box<Heap<*mut JSObject>>),
    BigInt64Array(Box<Heap<*mut JSObject>>),
    BigUint64Array(Box<Heap<*mut JSObject>>),
    Float32Array(Box<Heap<*mut JSObject>>),
    Float64Array(Box<Heap<*mut JSObject>>),
    DataView(Box<Heap<*mut JSObject>>),
    ArrayBuffer(Box<Heap<*mut JSObject>>),
    Default(Box<Heap<*mut JSObject>>),
}

pub struct HeapBufferSource<T> {
    buffer_source: BufferSource,
    phantom: PhantomData<T>,
}

unsafe impl<T> crate::dom::bindings::trace::JSTraceable for HeapBufferSource<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut js::jsapi::JSTracer) {
        match &self.buffer_source {
            BufferSource::Int8Array(buffer) |
            BufferSource::Int16Array(buffer) |
            BufferSource::Int32Array(buffer) |
            BufferSource::Uint8Array(buffer) |
            BufferSource::Uint16Array(buffer) |
            BufferSource::Uint32Array(buffer) |
            BufferSource::Uint8ClampedArray(buffer) |
            BufferSource::BigInt64Array(buffer) |
            BufferSource::BigUint64Array(buffer) |
            BufferSource::Float32Array(buffer) |
            BufferSource::Float64Array(buffer) |
            BufferSource::DataView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
            BufferSource::Default(buffer) => {
                buffer.trace(tracer);
            },
        }
    }
}

pub fn new_initialized_heap_buffer_source<T>(
    init: HeapTypedArrayInit,
) -> Result<HeapBufferSource<T>, ()>
where
    T: TypedArrayElement + TypedArrayElementCreator,
    T::Element: Clone + Copy,
{
    let heap_buffer_source = match init {
        HeapTypedArrayInit::Buffer(buffer_source) => HeapBufferSource {
            buffer_source,
            phantom: PhantomData,
        },
        HeapTypedArrayInit::Info { len, cx } => {
            rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());
            let typed_array_result =
                create_buffer_source_with_length::<T>(cx, len as usize, array.handle_mut());
            if typed_array_result.is_err() {
                return Err(());
            }
            let heap_buffer_source = HeapBufferSource::<T>::default();

            match &heap_buffer_source.buffer_source {
                BufferSource::Int8Array(buffer) |
                BufferSource::Int16Array(buffer) |
                BufferSource::Int32Array(buffer) |
                BufferSource::Uint8Array(buffer) |
                BufferSource::Uint16Array(buffer) |
                BufferSource::Uint32Array(buffer) |
                BufferSource::Uint8ClampedArray(buffer) |
                BufferSource::BigInt64Array(buffer) |
                BufferSource::BigUint64Array(buffer) |
                BufferSource::Float32Array(buffer) |
                BufferSource::Float64Array(buffer) |
                BufferSource::DataView(buffer) |
                BufferSource::ArrayBuffer(buffer) |
                BufferSource::Default(buffer) => {
                    buffer.set(*array);
                },
            }
            heap_buffer_source
        },
    };
    Ok(heap_buffer_source)
}

pub enum HeapTypedArrayInit {
    Buffer(BufferSource),
    Info { len: u32, cx: JSContext },
}

impl<T> HeapBufferSource<T>
where
    T: TypedArrayElement + TypedArrayElementCreator,
    T::Element: Clone + Copy,
{
    pub fn default() -> HeapBufferSource<T> {
        HeapBufferSource {
            buffer_source: BufferSource::Default(Box::default()),
            phantom: PhantomData,
        }
    }

    pub fn set_data(&self, cx: JSContext, data: &[T::Element]) -> Result<(), ()> {
        rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());
        let _: TypedArray<T, *mut JSObject> = create_buffer_source(cx, data, array.handle_mut())?;

        match &self.buffer_source {
            BufferSource::Int8Array(buffer) |
            BufferSource::Int16Array(buffer) |
            BufferSource::Int32Array(buffer) |
            BufferSource::Uint8Array(buffer) |
            BufferSource::Uint16Array(buffer) |
            BufferSource::Uint32Array(buffer) |
            BufferSource::Uint8ClampedArray(buffer) |
            BufferSource::BigInt64Array(buffer) |
            BufferSource::BigUint64Array(buffer) |
            BufferSource::Float32Array(buffer) |
            BufferSource::Float64Array(buffer) |
            BufferSource::DataView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
            BufferSource::Default(buffer) => {
                buffer.set(*array);
            },
        }
        Ok(())
    }

    pub fn acquire_data(&self, cx: JSContext) -> Result<Vec<T::Element>, ()> {
        assert!(self.is_initialized());

        typedarray!(in(*cx) let array: TypedArray = match &self.buffer_source {
            BufferSource::Int8Array(buffer) |
            BufferSource::Int16Array(buffer) |
            BufferSource::Int32Array(buffer) |
            BufferSource::Uint8Array(buffer) |
            BufferSource::Uint16Array(buffer) |
            BufferSource::Uint32Array(buffer) |
            BufferSource::Uint8ClampedArray(buffer) |
            BufferSource::BigInt64Array(buffer) |
            BufferSource::BigUint64Array(buffer) |
            BufferSource::Float32Array(buffer) |
            BufferSource::Float64Array(buffer) |
            BufferSource::DataView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
            BufferSource::Default(buffer) => {
                buffer.get()
            },
        });
        let data = if let Ok(array) =
            array as Result<CustomAutoRooterGuard<'_, TypedArray<T, *mut JSObject>>, &mut ()>
        {
            let data = array.to_vec();
            let _ = self.detach_buffer(cx);
            Ok(data)
        } else {
            Err(())
        };

        match &self.buffer_source {
            BufferSource::Int8Array(buffer) |
            BufferSource::Int16Array(buffer) |
            BufferSource::Int32Array(buffer) |
            BufferSource::Uint8Array(buffer) |
            BufferSource::Uint16Array(buffer) |
            BufferSource::Uint32Array(buffer) |
            BufferSource::Uint8ClampedArray(buffer) |
            BufferSource::BigInt64Array(buffer) |
            BufferSource::BigUint64Array(buffer) |
            BufferSource::Float32Array(buffer) |
            BufferSource::Float64Array(buffer) |
            BufferSource::DataView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
            BufferSource::Default(buffer) => {
                buffer.set(ptr::null_mut());
            },
        }
        data
    }

    /// <https://tc39.es/ecma262/#sec-detacharraybuffer>
    pub fn detach_buffer(&self, cx: JSContext) -> bool {
        match &self.buffer_source {
            BufferSource::Int8Array(buffer) |
            BufferSource::Int16Array(buffer) |
            BufferSource::Int32Array(buffer) |
            BufferSource::Uint8Array(buffer) |
            BufferSource::Uint16Array(buffer) |
            BufferSource::Uint32Array(buffer) |
            BufferSource::Uint8ClampedArray(buffer) |
            BufferSource::BigInt64Array(buffer) |
            BufferSource::BigUint64Array(buffer) |
            BufferSource::Float32Array(buffer) |
            BufferSource::Float64Array(buffer) |
            BufferSource::DataView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
            BufferSource::Default(buffer) => {
                assert!(self.is_initialized());
                let mut is_shared = false;
                unsafe {
                    if JS_IsArrayBufferViewObject(*buffer.handle()) {
                        // If it is an ArrayBuffer view, get the buffer using JS_GetArrayBufferViewBuffer
                        rooted!(in (*cx) let view_buffer =
                            JS_GetArrayBufferViewBuffer(*cx, buffer.handle(), &mut is_shared));
                        // This buffer is always created unshared
                        debug_assert!(!is_shared);
                        // Detach the ArrayBuffer
                        DetachArrayBuffer(*cx, view_buffer.handle())
                    } else {
                        // If it's not an ArrayBuffer view, Detach the buffer directly
                        DetachArrayBuffer(*cx, Handle::from_raw(buffer.handle()))
                    }
                }
            },
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
        typedarray!(in(*cx) let array: TypedArray = match &self.buffer_source {
            BufferSource::Int8Array(buffer) |
            BufferSource::Int16Array(buffer) |
            BufferSource::Int32Array(buffer) |
            BufferSource::Uint8Array(buffer) |
            BufferSource::Uint16Array(buffer) |
            BufferSource::Uint32Array(buffer) |
            BufferSource::Uint8ClampedArray(buffer) |
            BufferSource::BigInt64Array(buffer) |
            BufferSource::BigUint64Array(buffer) |
            BufferSource::Float32Array(buffer) |
            BufferSource::Float64Array(buffer) |
            BufferSource::DataView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
            BufferSource::Default(buffer) => {
                buffer.get()
            },
        });
        let Ok(array) =
            array as Result<CustomAutoRooterGuard<'_, TypedArray<T, *mut JSObject>>, &mut ()>
        else {
            return Err(());
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
        typedarray!(in(*cx) let mut array: TypedArray = match &self.buffer_source {
            BufferSource::Int8Array(buffer) |
            BufferSource::Int16Array(buffer) |
            BufferSource::Int32Array(buffer) |
            BufferSource::Uint8Array(buffer) |
            BufferSource::Uint16Array(buffer) |
            BufferSource::Uint32Array(buffer) |
            BufferSource::Uint8ClampedArray(buffer) |
            BufferSource::BigInt64Array(buffer) |
            BufferSource::BigUint64Array(buffer) |
            BufferSource::Float32Array(buffer) |
            BufferSource::Float64Array(buffer) |
            BufferSource::DataView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
            BufferSource::Default(buffer) => {
                buffer.get()
            },
        });
        let Ok(mut array) =
            array as Result<CustomAutoRooterGuard<'_, TypedArray<T, *mut JSObject>>, &mut ()>
        else {
            return Err(());
        };
        unsafe {
            let slice = (*array).as_mut_slice();
            let (_, dest) = slice.split_at_mut(dest_start);
            dest[0..length].copy_from_slice(&source.as_slice()[0..length])
        }
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        match &self.buffer_source {
            BufferSource::Int8Array(buffer) |
            BufferSource::Int16Array(buffer) |
            BufferSource::Int32Array(buffer) |
            BufferSource::Uint8Array(buffer) |
            BufferSource::Uint16Array(buffer) |
            BufferSource::Uint32Array(buffer) |
            BufferSource::Uint8ClampedArray(buffer) |
            BufferSource::BigInt64Array(buffer) |
            BufferSource::BigUint64Array(buffer) |
            BufferSource::Float32Array(buffer) |
            BufferSource::Float64Array(buffer) |
            BufferSource::DataView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
            BufferSource::Default(buffer) => !buffer.get().is_null(),
        }
    }

    pub fn get_buffer(&self) -> Result<TypedArray<T, *mut JSObject>, ()> {
        TypedArray::from(match &self.buffer_source {
            BufferSource::Int8Array(buffer) |
            BufferSource::Int16Array(buffer) |
            BufferSource::Int32Array(buffer) |
            BufferSource::Uint8Array(buffer) |
            BufferSource::Uint16Array(buffer) |
            BufferSource::Uint32Array(buffer) |
            BufferSource::Uint8ClampedArray(buffer) |
            BufferSource::BigInt64Array(buffer) |
            BufferSource::BigUint64Array(buffer) |
            BufferSource::Float32Array(buffer) |
            BufferSource::Float64Array(buffer) |
            BufferSource::DataView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
            BufferSource::Default(buffer) => buffer.get(),
        })
    }

    pub fn buffer_to_option(&self) -> Option<TypedArray<T, *mut JSObject>> {
        if self.is_initialized() {
            Some(self.get_buffer().expect("Failed to get buffer."))
        } else {
            warn!("Buffer not initialized.");
            None
        }
    }
}

/// <https://webidl.spec.whatwg.org/#arraybufferview-create>
pub fn create_buffer_source<T>(
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

fn create_buffer_source_with_length<T>(
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
) -> HeapBufferSource<T>
where
    T: TypedArrayElement + TypedArrayElementCreator,
    T::Element: Clone + Copy,
{
    /// `freeFunc()` must be threadsafe, should be safely callable from any thread
    /// without causing conflicts , unexpected behavior.
    /// <https://github.com/servo/mozjs/blob/main/mozjs-sys/mozjs/js/public/ArrayBuffer.h#L89>
    unsafe extern "C" fn free_func(_contents: *mut c_void, free_user_data: *mut c_void) {
        // Clippy warns about "creating a `Arc` from a void raw pointer" here, but suggests
        // the exact same line to fix it. Doing the cast is tricky because of the use of
        // a generic type in this parameter.
        #[allow(clippy::from_raw_with_void_ptr)]
        let _ = Arc::from_raw(free_user_data as *const _);
    }

    unsafe {
        let mapping_slice_ptr = mapping.lock().unwrap().borrow_mut()[offset..m_end].as_mut_ptr();

        // rooted! is needed to ensure memory safety and prevent potential garbage collection issues.
        // https://github.com/mozilla-spidermonkey/spidermonkey-embedding-examples/blob/esr78/docs/GC%20Rooting%20Guide.md#performance-tweaking
        rooted!(in(*cx) let array_buffer = NewExternalArrayBuffer(
            *cx,
            range_size,
            mapping_slice_ptr as _,
            Some(free_func),
            Arc::into_raw(mapping) as _,
        ));

        HeapBufferSource {
            buffer_source: BufferSource::ArrayBuffer(Heap::boxed(*array_buffer)),
            phantom: PhantomData,
        }
    }
}
