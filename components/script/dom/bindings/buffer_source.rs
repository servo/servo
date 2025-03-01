/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

#[cfg(feature = "webgpu")]
use std::ffi::c_void;
use std::marker::PhantomData;
#[cfg(feature = "webgpu")]
use std::ops::Range;
use std::ptr;
#[cfg(feature = "webgpu")]
use std::sync::Arc;

#[cfg(feature = "webgpu")]
use js::jsapi::NewExternalArrayBuffer;
use js::jsapi::{
    GetArrayBufferByteLength, Heap, IsDetachedArrayBufferObject, JSObject,
    JS_GetArrayBufferViewBuffer, JS_GetArrayBufferViewByteLength, JS_IsArrayBufferViewObject,
    JS_IsTypedArrayObject,
};
use js::rust::wrappers::DetachArrayBuffer;
use js::rust::{CustomAutoRooterGuard, Handle, MutableHandleObject};
#[cfg(feature = "webgpu")]
use js::typedarray::{ArrayBuffer, HeapArrayBuffer};
use js::typedarray::{CreateWith, TypedArray, TypedArrayElement, TypedArrayElementCreator};

#[cfg(feature = "webgpu")]
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext};

// Represents a `BufferSource` as defined in the WebIDL specification.
///
/// A `BufferSource` is either an `ArrayBuffer` or an `ArrayBufferView`, which
/// provides a view onto an `ArrayBuffer`.
///
/// See: <https://webidl.spec.whatwg.org/#BufferSource>
pub(crate) enum BufferSource {
    /// Represents an `ArrayBufferView` (e.g., `Uint8Array`, `DataView`).
    /// See: <https://webidl.spec.whatwg.org/#ArrayBufferView>
    ArrayBufferView(Box<Heap<*mut JSObject>>),

    /// Represents an `ArrayBuffer`, a fixed-length binary data buffer.
    /// See: <https://webidl.spec.whatwg.org/#idl-ArrayBuffer>
    #[allow(dead_code)]
    ArrayBuffer(Box<Heap<*mut JSObject>>),

    /// Default variant, used as a placeholder in initialization.
    Default(Box<Heap<*mut JSObject>>),
}

pub(crate) fn new_initialized_heap_buffer_source<T>(
    init: HeapTypedArrayInit,
    can_gc: CanGc,
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
                create_buffer_source_with_length::<T>(cx, len as usize, array.handle_mut(), can_gc);
            if typed_array_result.is_err() {
                return Err(());
            }
            let heap_buffer_source = HeapBufferSource::<T>::default();

            match &heap_buffer_source.buffer_source {
                BufferSource::ArrayBufferView(buffer) |
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
            BufferSource::ArrayBufferView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
            BufferSource::Default(buffer) => !buffer.get().is_null(),
        }
    }

    pub(crate) fn get_buffer(&self) -> Result<TypedArray<T, *mut JSObject>, ()> {
        TypedArray::from(match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
            BufferSource::Default(buffer) => buffer.get(),
        })
    }

    /// <https://tc39.es/ecma262/#sec-detacharraybuffer>
    pub(crate) fn detach_buffer(&self, cx: JSContext) -> bool {
        assert!(self.is_initialized());
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::Default(buffer) => {
                let mut is_shared = false;
                unsafe {
                    // assert buffer is an ArrayBuffer view
                    assert!(JS_IsArrayBufferViewObject(*buffer.handle()));
                    rooted!(in (*cx) let view_buffer =
                            JS_GetArrayBufferViewBuffer(*cx, buffer.handle(), &mut is_shared));
                    // This buffer is always created unshared
                    debug_assert!(!is_shared);
                    // Detach the ArrayBuffer
                    DetachArrayBuffer(*cx, view_buffer.handle())
                }
            },
            BufferSource::ArrayBuffer(buffer) => unsafe {
                DetachArrayBuffer(*cx, Handle::from_raw(buffer.handle()))
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

    pub(crate) fn is_detached_buffer(&self, cx: JSContext) -> bool {
        assert!(self.is_initialized());
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::Default(buffer) => {
                let mut is_shared = false;
                unsafe {
                    assert!(JS_IsArrayBufferViewObject(*buffer.handle()));
                    rooted!(in (*cx) let view_buffer =
                            JS_GetArrayBufferViewBuffer(*cx, buffer.handle(), &mut is_shared));
                    debug_assert!(!is_shared);
                    IsDetachedArrayBufferObject(*view_buffer.handle())
                }
            },
            BufferSource::ArrayBuffer(buffer) => unsafe {
                IsDetachedArrayBufferObject(*buffer.handle())
            },
        }
    }

    pub(crate) fn viewed_buffer_array_byte_length(&self, cx: JSContext) -> usize {
        assert!(self.is_initialized());
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::Default(buffer) => {
                let mut is_shared = false;
                unsafe {
                    assert!(JS_IsArrayBufferViewObject(*buffer.handle()));
                    rooted!(in (*cx) let view_buffer =
                            JS_GetArrayBufferViewBuffer(*cx, buffer.handle(), &mut is_shared));
                    debug_assert!(!is_shared);
                    GetArrayBufferByteLength(*view_buffer.handle())
                }
            },
            BufferSource::ArrayBuffer(buffer) => unsafe {
                GetArrayBufferByteLength(*buffer.handle())
            },
        }
    }

    pub(crate) fn byte_length(&self) -> usize {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::Default(buffer) => unsafe {
                JS_GetArrayBufferViewByteLength(*buffer.handle())
            },
            BufferSource::ArrayBuffer(buffer) => unsafe {
                GetArrayBufferByteLength(*buffer.handle())
            },
        }
    }

    pub(crate) fn array_length(&self) -> usize {
        self.get_buffer().unwrap().len()
    }

    /// <https://tc39.es/ecma262/#typedarray>
    pub(crate) fn has_typed_array_name(&self) -> bool {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::Default(buffer) => unsafe {
                JS_IsTypedArrayObject(*buffer.handle())
            },
            BufferSource::ArrayBuffer(_) => false,
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
            BufferSource::ArrayBufferView(buffer) |
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
            BufferSource::ArrayBufferView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
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
            BufferSource::ArrayBufferView(buffer) |
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

    pub(crate) fn copy_data_from(
        &self,
        cx: JSContext,
        source: CustomAutoRooterGuard<TypedArray<T, *mut JSObject>>,
        dest_start: usize,
        length: usize,
    ) -> Result<(), ()> {
        assert!(self.is_initialized());
        typedarray!(in(*cx) let mut array: TypedArray = match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) |
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

    pub(crate) fn set_data(
        &self,
        cx: JSContext,
        data: &[T::Element],
        can_gc: CanGc,
    ) -> Result<(), ()> {
        rooted!(in (*cx) let mut array = ptr::null_mut::<JSObject>());
        let _: TypedArray<T, *mut JSObject> =
            create_buffer_source(cx, data, array.handle_mut(), can_gc)?;

        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
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
            BufferSource::ArrayBufferView(buffer) |
            BufferSource::ArrayBuffer(buffer) |
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
    _can_gc: CanGc,
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
    _can_gc: CanGc,
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

#[cfg(feature = "webgpu")]
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct DataBlock {
    #[ignore_malloc_size_of = "Arc"]
    data: Arc<Box<[u8]>>,
    /// Data views (mutable subslices of data)
    data_views: Vec<DataView>,
}

/// Returns true if two non-inclusive ranges overlap
// https://stackoverflow.com/questions/3269434/whats-the-most-efficient-way-to-test-if-two-ranges-overlap
#[cfg(feature = "webgpu")]
fn range_overlap<T: std::cmp::PartialOrd>(range1: &Range<T>, range2: &Range<T>) -> bool {
    range1.start < range2.end && range2.start < range1.end
}

#[cfg(feature = "webgpu")]
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
    pub(crate) fn view(&mut self, range: Range<usize>, _can_gc: CanGc) -> Result<&DataView, ()> {
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

#[cfg(feature = "webgpu")]
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct DataView {
    #[no_trace]
    range: Range<usize>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    buffer: HeapArrayBuffer,
}

#[cfg(feature = "webgpu")]
impl DataView {
    pub(crate) fn array_buffer(&self) -> ArrayBuffer {
        unsafe { ArrayBuffer::from(self.buffer.underlying_object().get()).unwrap() }
    }
}

#[cfg(feature = "webgpu")]
impl Drop for DataView {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        let cx = GlobalScope::get_cx();
        assert!(unsafe {
            js::jsapi::DetachArrayBuffer(*cx, self.buffer.underlying_object().handle())
        })
    }
}
