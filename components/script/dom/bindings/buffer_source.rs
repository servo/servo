/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops::Range;
use std::ptr;
use std::sync::Arc;

use js::jsapi::{
    Heap, IsDetachedArrayBufferObject, JSObject, JS_GetArrayBufferViewBuffer,
    JS_IsArrayBufferViewObject, NewExternalArrayBuffer,
};
use js::rust::wrappers::DetachArrayBuffer;
use js::rust::{CustomAutoRooterGuard, Handle, MutableHandleObject};
use js::typedarray::{
    ArrayBuffer, ArrayBufferViewU8, CreateWith, HeapArrayBuffer, TypedArray, TypedArrayElement,
    TypedArrayElementCreator,
};

use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext;

/// <https://webidl.spec.whatwg.org/#BufferSource>
#[allow(dead_code)]
pub(crate) enum BufferSource {
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
    ArrayBufferView(Box<Heap<*mut JSObject>>),
    Default(Box<Heap<*mut JSObject>>),
}

pub(crate) fn new_initialized_heap_buffer_source<T>(
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
                BufferSource::ArrayBufferView(buffer) |
                BufferSource::Default(buffer) => {
                    buffer.set(*array);
                },
            }
            heap_buffer_source
        },
    };
    Ok(heap_buffer_source)
}

pub(crate) enum HeapTypedArrayInit {
    Buffer(BufferSource),
    Info { len: u32, cx: JSContext },
}

pub(crate) struct HeapBufferSource<T> {
    buffer_source: BufferSource,
    phantom: PhantomData<T>,
}

impl<T> HeapBufferSource<T>
where
    T: TypedArrayElement,
{
    pub(crate) fn new(buffer_source: BufferSource) -> HeapBufferSource<T> {
        HeapBufferSource {
            buffer_source,
            phantom: PhantomData,
        }
    }

    pub(crate) fn default() -> HeapBufferSource<T> {
        HeapBufferSource {
            buffer_source: BufferSource::Default(Box::default()),
            phantom: PhantomData,
        }
    }

    pub(crate) fn is_initialized(&self) -> bool {
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
            BufferSource::ArrayBufferView(buffer) |
            BufferSource::Default(buffer) => !buffer.get().is_null(),
        }
    }

    pub(crate) fn get_buffer(&self) -> Result<TypedArray<T, *mut JSObject>, ()> {
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
            BufferSource::ArrayBufferView(buffer) |
            BufferSource::Default(buffer) => buffer.get(),
        })
    }

    /// <https://tc39.es/ecma262/#sec-detacharraybuffer>
    pub(crate) fn detach_buffer(&self, cx: JSContext) -> bool {
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
            BufferSource::ArrayBufferView(buffer) |
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

    pub(crate) fn buffer_to_option(&self) -> Option<TypedArray<T, *mut JSObject>> {
        if self.is_initialized() {
            self.get_buffer().ok()
        } else {
            warn!("Buffer not initialized.");
            None
        }
    }

    pub(crate) fn is_detached_buffer(&self) -> bool {
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
            BufferSource::ArrayBufferView(buffer) |
            BufferSource::Default(buffer) => unsafe {
                IsDetachedArrayBufferObject(*buffer.handle())
            },
        }
    }

    fn len(&self) -> usize {
        self.get_buffer().unwrap().len()
    }

    pub fn array_buffer_byte_length(&self) -> usize {
        self.len() * std::mem::size_of::<T::Element>()
    }

    pub fn byte_length(&self) -> usize {
        self.len() * std::mem::size_of::<T::Element>()
    }

    pub fn array_length(&self) -> usize {
        self.len()
    }

    pub(crate) fn has_typed_array_name(&self) -> bool {
        match &self.buffer_source {
            BufferSource::Int8Array(_) |
            BufferSource::Int16Array(_) |
            BufferSource::Int32Array(_) |
            BufferSource::Uint8Array(_) |
            BufferSource::Uint16Array(_) |
            BufferSource::Uint32Array(_) |
            BufferSource::Uint8ClampedArray(_) |
            BufferSource::BigInt64Array(_) |
            BufferSource::BigUint64Array(_) |
            BufferSource::Float32Array(_) |
            BufferSource::Float64Array(_) |
            BufferSource::DataView(_) |
            BufferSource::ArrayBuffer(_) |
            BufferSource::Default(_) => true,
            BufferSource::ArrayBufferView(buffer) => {
                let buffer_view =
                    TypedArray::<ArrayBufferViewU8, *mut JSObject>::from(buffer.get()).unwrap();

                match buffer_view.get_array_type() {
                    js::jsapi::Type::Int8 |
                    js::jsapi::Type::Uint8 |
                    js::jsapi::Type::Int16 |
                    js::jsapi::Type::Uint16 |
                    js::jsapi::Type::Int32 |
                    js::jsapi::Type::Uint32 |
                    js::jsapi::Type::Float32 |
                    js::jsapi::Type::Float64 |
                    js::jsapi::Type::BigInt64 |
                    js::jsapi::Type::BigUint64 |
                    js::jsapi::Type::Uint8Clamped => true,

                    js::jsapi::Type::Float16 |
                    js::jsapi::Type::MaxTypedArrayViewType |
                    js::jsapi::Type::Int64 |
                    js::jsapi::Type::Simd128 => false,
                }
            },
        }
    }
}

impl<T> HeapBufferSource<T>
where
    T: TypedArrayElement + TypedArrayElementCreator,
    T::Element: Clone + Copy,
{
    pub(crate) fn acquire_data(&self, cx: JSContext) -> Result<Vec<T::Element>, ()> {
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
             BufferSource::ArrayBufferView(buffer) |
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
            BufferSource::ArrayBufferView(buffer) |
            BufferSource::Default(buffer) => {
                buffer.set(ptr::null_mut());
            },
        }
        data
    }

    pub(crate) fn copy_data_to(
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
            BufferSource::ArrayBufferView(buffer) |
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

    pub(crate) fn copy_data_from(
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
            BufferSource::ArrayBufferView(buffer) |
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

    pub(crate) fn set_data(&self, cx: JSContext, data: &[T::Element]) -> Result<(), ()> {
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
            BufferSource::ArrayBufferView(buffer) |
            BufferSource::Default(buffer) => {
                buffer.set(*array);
            },
        }
        Ok(())
    }
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
            BufferSource::ArrayBufferView(buffer) |
            BufferSource::Default(buffer) => {
                buffer.trace(tracer);
            },
        }
    }
}

/// <https://webidl.spec.whatwg.org/#arraybufferview-create>
pub(crate) fn create_buffer_source<T>(
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

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct DataBlock {
    #[ignore_malloc_size_of = "Arc"]
    data: Arc<Box<[u8]>>,
    /// Data views (mutable subslices of data)
    data_views: Vec<DataView>,
}

/// Returns true if two non-inclusive ranges overlap
// https://stackoverflow.com/questions/3269434/whats-the-most-efficient-way-to-test-if-two-ranges-overlap
fn range_overlap<T: std::cmp::PartialOrd>(range1: &Range<T>, range2: &Range<T>) -> bool {
    range1.start < range2.end && range2.start < range1.end
}

impl DataBlock {
    pub(crate) fn new_zeroed(size: usize) -> Self {
        let data = vec![0; size];
        Self {
            data: Arc::new(data.into_boxed_slice()),
            data_views: Vec::new(),
        }
    }

    /// Panics if there is any active view or src data is not same length
    pub(crate) fn load(&mut self, src: &[u8]) {
        // `Arc::get_mut` ensures there are no views
        Arc::get_mut(&mut self.data).unwrap().clone_from_slice(src)
    }

    /// Panics if there is any active view
    pub(crate) fn data(&mut self) -> &mut [u8] {
        // `Arc::get_mut` ensures there are no views
        Arc::get_mut(&mut self.data).unwrap()
    }

    pub(crate) fn clear_views(&mut self) {
        self.data_views.clear()
    }

    /// Returns error if requested range is already mapped
    pub(crate) fn view(&mut self, range: Range<usize>) -> Result<&DataView, ()> {
        if self
            .data_views
            .iter()
            .any(|view| range_overlap(&view.range, &range))
        {
            return Err(());
        }
        let cx = GlobalScope::get_cx();
        /// `freeFunc()` must be threadsafe, should be safely callable from any thread
        /// without causing conflicts, unexpected behavior.
        unsafe extern "C" fn free_func(_contents: *mut c_void, free_user_data: *mut c_void) {
            // Clippy warns about "creating a `Arc` from a void raw pointer" here, but suggests
            // the exact same line to fix it. Doing the cast is tricky because of the use of
            // a generic type in this parameter.
            #[allow(clippy::from_raw_with_void_ptr)]
            drop(Arc::from_raw(free_user_data as *const _));
        }
        let raw: *mut Box<[u8]> = Arc::into_raw(Arc::clone(&self.data)) as _;
        rooted!(in(*cx) let object = unsafe {
            NewExternalArrayBuffer(
                *cx,
                range.end - range.start,
                // SAFETY: This is safe because we have checked there is no overlapping view
                (*raw)[range.clone()].as_mut_ptr() as _,
                Some(free_func),
                raw as _,
            )
        });
        self.data_views.push(DataView {
            range,
            buffer: HeapArrayBuffer::from(*object).unwrap(),
        });
        Ok(self.data_views.last().unwrap())
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct DataView {
    #[no_trace]
    range: Range<usize>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    buffer: HeapArrayBuffer,
}

impl DataView {
    pub(crate) fn array_buffer(&self) -> ArrayBuffer {
        unsafe { ArrayBuffer::from(self.buffer.underlying_object().get()).unwrap() }
    }
}

impl Drop for DataView {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        let cx = GlobalScope::get_cx();
        assert!(unsafe {
            js::jsapi::DetachArrayBuffer(*cx, self.buffer.underlying_object().handle())
        })
    }
}
