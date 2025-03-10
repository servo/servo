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
    ArrayBufferClone, ArrayBufferCopyData, GetArrayBufferByteLength,
    HasDefinedArrayBufferDetachKey, Heap, IsArrayBufferObject, IsDetachedArrayBufferObject,
    JS_ClearPendingException, JS_GetArrayBufferViewBuffer, JS_GetArrayBufferViewByteLength,
    JS_GetArrayBufferViewByteOffset, JS_GetArrayBufferViewType, JS_GetPendingException,
    JS_GetTypedArrayLength, JS_IsArrayBufferViewObject, JS_IsTypedArrayObject,
    JS_NewBigInt64ArrayWithBuffer, JS_NewBigUint64ArrayWithBuffer, JS_NewDataView,
    JS_NewFloat16ArrayWithBuffer, JS_NewFloat32ArrayWithBuffer, JS_NewFloat64ArrayWithBuffer,
    JS_NewInt8ArrayWithBuffer, JS_NewInt16ArrayWithBuffer, JS_NewInt32ArrayWithBuffer,
    JS_NewUint8ArrayWithBuffer, JS_NewUint8ClampedArrayWithBuffer, JS_NewUint16ArrayWithBuffer,
    JS_NewUint32ArrayWithBuffer, JSObject, NewArrayBuffer, NewArrayBufferWithContents,
    StealArrayBufferContents, Type,
};
use js::jsval::{ObjectValue, UndefinedValue};
use js::rust::wrappers::DetachArrayBuffer;
use js::rust::{
    CustomAutoRooterGuard, Handle, MutableHandleObject,
    MutableHandleValue as SafeMutableHandleValue,
};
use js::typedarray::{
    ArrayBuffer, ArrayBufferU8, ArrayBufferView, ArrayBufferViewU8, CreateWith, HeapArrayBuffer,
    TypedArray, TypedArrayElement, TypedArrayElementCreator,
};

use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
#[cfg(feature = "webgpu")]
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CanGc, JSContext};

// Represents a `BufferSource` as defined in the WebIDL specification.
///
/// A `BufferSource` is either an `ArrayBuffer` or an `ArrayBufferView`, which
/// provides a view onto an `ArrayBuffer`.
///
/// See: <https://webidl.spec.whatwg.org/#BufferSource>
#[derive(PartialEq)]
pub(crate) enum BufferSource {
    /// Represents an `ArrayBufferView` (e.g., `Uint8Array`, `DataView`).
    /// See: <https://webidl.spec.whatwg.org/#ArrayBufferView>
    ArrayBufferView(Box<Heap<*mut JSObject>>),

    /// Represents an `ArrayBuffer`, a fixed-length binary data buffer.
    /// See: <https://webidl.spec.whatwg.org/#idl-ArrayBuffer>
    ArrayBuffer(Box<Heap<*mut JSObject>>),
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

            HeapBufferSource::<T>::new(BufferSource::ArrayBufferView(Heap::boxed(*array.handle())))
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

impl<T> Eq for HeapBufferSource<T> where T: TypedArrayElement {}

impl<T> PartialEq for HeapBufferSource<T>
where
    T: TypedArrayElement,
{
    fn eq(&self, other: &Self) -> bool {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(heap) | BufferSource::ArrayBuffer(heap) => match &other
                .buffer_source
            {
                BufferSource::ArrayBufferView(from_heap) | BufferSource::ArrayBuffer(from_heap) => unsafe {
                    heap.handle() == from_heap.handle()
                },
            },
        }
    }
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

    pub(crate) fn from_view(
        chunk: CustomAutoRooterGuard<ArrayBufferView>,
    ) -> HeapBufferSource<ArrayBufferViewU8> {
        HeapBufferSource::<ArrayBufferViewU8>::new(BufferSource::ArrayBufferView(Heap::boxed(
            unsafe { *chunk.underlying_object() },
        )))
    }

    pub(crate) fn default() -> Self {
        HeapBufferSource {
            buffer_source: BufferSource::ArrayBufferView(Heap::boxed(std::ptr::null_mut())),
            phantom: PhantomData,
        }
    }

    pub(crate) fn is_initialized(&self) -> bool {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer) => {
                !buffer.get().is_null()
            },
        }
    }

    pub(crate) fn get_typed_array(&self) -> Result<TypedArray<T, *mut JSObject>, ()> {
        TypedArray::from(match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer) => {
                buffer.get()
            },
        })
    }

    pub(crate) fn get_buffer_view_value(
        &self,
        cx: JSContext,
        mut handle_mut: SafeMutableHandleValue,
    ) {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => {
                rooted!(in(*cx) let value = ObjectValue(buffer.get()));
                handle_mut.set(*value);
            },
            BufferSource::ArrayBuffer(_) => {
                unreachable!("BufferSource::ArrayBuffer does not have a view buffer.")
            },
        }
    }

    pub(crate) fn get_array_buffer_view_buffer(
        &self,
        cx: JSContext,
    ) -> HeapBufferSource<ArrayBufferU8> {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => unsafe {
                let mut is_shared = false;
                rooted!(in (*cx) let view_buffer =
                         JS_GetArrayBufferViewBuffer(*cx, buffer.handle(), &mut is_shared));

                HeapBufferSource::<ArrayBufferU8>::new(BufferSource::ArrayBuffer(Heap::boxed(
                    *view_buffer.handle(),
                )))
            },
            BufferSource::ArrayBuffer(_) => {
                unreachable!("BufferSource::ArrayBuffer does not have a view buffer.")
            },
        }
    }

    /// <https://tc39.es/ecma262/#sec-detacharraybuffer>
    pub(crate) fn detach_buffer(&self, cx: JSContext) -> bool {
        assert!(self.is_initialized());
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => {
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

    pub(crate) fn typed_array_to_option(&self) -> Option<TypedArray<T, *mut JSObject>> {
        if self.is_initialized() {
            self.get_typed_array().ok()
        } else {
            warn!("Buffer not initialized.");
            None
        }
    }

    pub(crate) fn is_detached_buffer(&self, cx: JSContext) -> bool {
        assert!(self.is_initialized());
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => {
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
            BufferSource::ArrayBufferView(buffer) => {
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
            BufferSource::ArrayBufferView(buffer) => unsafe {
                JS_GetArrayBufferViewByteLength(*buffer.handle())
            },
            BufferSource::ArrayBuffer(buffer) => unsafe {
                GetArrayBufferByteLength(*buffer.handle())
            },
        }
    }

    pub(crate) fn get_byte_offset(&self) -> usize {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => unsafe {
                JS_GetArrayBufferViewByteOffset(*buffer.handle())
            },
            BufferSource::ArrayBuffer(_) => {
                unreachable!("BufferSource::ArrayBuffer does not have a byte offset.")
            },
        }
    }

    pub(crate) fn get_typed_array_length(&self) -> usize {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => unsafe {
                JS_GetTypedArrayLength(*buffer.handle())
            },
            BufferSource::ArrayBuffer(_) => {
                unreachable!("BufferSource::ArrayBuffer does not have a length.")
            },
        }
    }

    /// <https://tc39.es/ecma262/#typedarray>
    pub(crate) fn has_typed_array_name(&self) -> bool {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => unsafe {
                JS_IsTypedArrayObject(*buffer.handle())
            },
            BufferSource::ArrayBuffer(_) => false,
        }
    }

    pub(crate) fn get_array_buffer_view_type(&self) -> Type {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => unsafe {
                JS_GetArrayBufferViewType(*buffer.handle())
            },
            BufferSource::ArrayBuffer(_) => unreachable!("ArrayBuffer does not have a view type."),
        }
    }

    pub(crate) fn is_array_buffer_object(&self) -> bool {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(heap) | BufferSource::ArrayBuffer(heap) => unsafe {
                IsArrayBufferObject(*heap.handle())
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
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer)
            => {
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
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer) => {
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
            BufferSource::ArrayBufferView(buffer) |  BufferSource::ArrayBuffer(buffer)
            => {
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
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer)
            => {
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
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer) => {
                buffer.set(*array);
            },
        }
        Ok(())
    }

    /// <https://tc39.es/ecma262/#sec-clonearraybuffer>
    pub(crate) fn clone_array_buffer(
        &self,
        cx: JSContext,
        byte_offset: usize,
        byte_length: usize,
    ) -> Option<HeapBufferSource<ArrayBufferU8>> {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(heap) | BufferSource::ArrayBuffer(heap) => {
                let result =
                    unsafe { ArrayBufferClone(*cx, heap.handle(), byte_offset, byte_length) };
                if result.is_null() {
                    None
                } else {
                    Some(HeapBufferSource::<ArrayBufferU8>::new(
                        BufferSource::ArrayBuffer(Heap::boxed(result)),
                    ))
                }
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-cancopydatablockbytes>
    // CanCopyDataBlockBytes(descriptorBuffer, destStart, queueBuffer, queueByteOffset, bytesToCopy)
    pub(crate) fn can_copy_data_block_bytes(
        &self,
        cx: JSContext,
        to_index: usize,
        from_buffer: &HeapBufferSource<ArrayBufferU8>,
        from_index: usize,
        bytes_to_copy: usize,
    ) -> bool {
        // Assert: toBuffer is an Object.
        // Assert: toBuffer has an [[ArrayBufferData]] internal slot.
        assert!(self.is_array_buffer_object());

        // Assert: fromBuffer is an Object.
        // Assert: fromBuffer has an [[ArrayBufferData]] internal slot.
        assert!(from_buffer.is_array_buffer_object());

        // If toBuffer is fromBuffer, return false.
        match &self.buffer_source {
            BufferSource::ArrayBufferView(heap) | BufferSource::ArrayBuffer(heap) => {
                match &from_buffer.buffer_source {
                    BufferSource::ArrayBufferView(from_heap) |
                    BufferSource::ArrayBuffer(from_heap) => {
                        unsafe {
                            if heap.handle() == from_heap.handle() {
                                return false;
                            }
                        };
                    },
                }
            },
        }

        // If ! IsDetachedBuffer(toBuffer) is true, return false.
        if self.is_detached_buffer(cx) {
            return false;
        }

        // If ! IsDetachedBuffer(fromBuffer) is true, return false.
        if from_buffer.is_detached_buffer(cx) {
            return false;
        }

        // If toIndex + count > toBuffer.[[ArrayBufferByteLength]], return false.
        if to_index + bytes_to_copy > self.byte_length() {
            return false;
        }

        // If fromIndex + count > fromBuffer.[[ArrayBufferByteLength]], return false.
        if from_index + bytes_to_copy > from_buffer.byte_length() {
            return false;
        }

        // Return true.
        true
    }

    pub(crate) fn copy_data_block_bytes(
        &self,
        cx: JSContext,
        dest_start: usize,
        from_buffer: &HeapBufferSource<ArrayBufferU8>,
        from_byte_offset: usize,
        bytes_to_copy: usize,
    ) -> bool {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(heap) | BufferSource::ArrayBuffer(heap) => unsafe {
                match &from_buffer.buffer_source {
                    BufferSource::ArrayBufferView(from_heap) |
                    BufferSource::ArrayBuffer(from_heap) => ArrayBufferCopyData(
                        *cx,
                        heap.handle(),
                        dest_start,
                        from_heap.handle(),
                        from_byte_offset,
                        bytes_to_copy,
                    ),
                }
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#can-transfer-array-buffer>
    pub(crate) fn can_transfer_array_buffer(&self, cx: JSContext) -> bool {
        // Assert: O is an Object.
        // Assert: O has an [[ArrayBufferData]] internal slot.
        assert!(self.is_array_buffer_object());

        // If ! IsDetachedBuffer(O) is true, return false.
        if self.is_detached_buffer(cx) {
            return false;
        }

        // If SameValue(O.[[ArrayBufferDetachKey]], undefined) is false, return false.
        // Return true.
        let mut is_defined = false;
        match &self.buffer_source {
            BufferSource::ArrayBufferView(heap) | BufferSource::ArrayBuffer(heap) => unsafe {
                if !HasDefinedArrayBufferDetachKey(*cx, heap.handle(), &mut is_defined) {
                    return false;
                }
            },
        }

        !is_defined
    }

    /// <https://streams.spec.whatwg.org/#transfer-array-buffer>
    pub(crate) fn transfer_array_buffer(
        &self,
        cx: JSContext,
    ) -> Fallible<HeapBufferSource<ArrayBufferU8>> {
        assert!(self.is_array_buffer_object());

        // Assert: ! IsDetachedBuffer(O) is false.
        assert!(!self.is_detached_buffer(cx));

        // Let arrayBufferByteLength be O.[[ArrayBufferByteLength]].
        // Step 3 (Reordered)
        let buffer_length = self.byte_length();

        // Let arrayBufferData be O.[[ArrayBufferData]].
        // Step 2 (Reordered)
        let buffer_data = match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer) => unsafe {
                StealArrayBufferContents(*cx, buffer.handle())
            },
        };

        // Perform ? DetachArrayBuffer(O).
        // This will throw an exception if O has an [[ArrayBufferDetachKey]] that is not undefined,
        // such as a WebAssembly.Memory’s buffer. [WASM-JS-API-1]
        if !self.detach_buffer(cx) {
            rooted!(in(*cx) let mut rval = UndefinedValue());
            unsafe {
                assert!(JS_GetPendingException(*cx, rval.handle_mut().into()));
                JS_ClearPendingException(*cx)
            };

            Err(Error::Type("can't transfer array buffer".to_owned()))
        } else {
            // Return a new ArrayBuffer object, created in the current Realm,
            // whose [[ArrayBufferData]] internal slot value is arrayBufferData and
            // whose [[ArrayBufferByteLength]] internal slot value is arrayBufferByteLength.
            Ok(HeapBufferSource::<ArrayBufferU8>::new(
                BufferSource::ArrayBuffer(Heap::boxed(unsafe {
                    NewArrayBufferWithContents(*cx, buffer_length, buffer_data)
                })),
            ))
        }
    }
}

unsafe impl<T> crate::dom::bindings::trace::JSTraceable for HeapBufferSource<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut js::jsapi::JSTracer) {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer) => {
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

pub(crate) fn byte_size(byte_type: Type) -> u64 {
    match byte_type {
        Type::Int8 | Type::Uint8 | Type::Uint8Clamped => 1,
        Type::Int16 | Type::Uint16 | Type::Float16 => 2,
        Type::Int32 | Type::Uint32 | Type::Float32 => 4,
        Type::Int64 | Type::Float64 | Type::BigInt64 | Type::BigUint64 => 8,
        Type::Simd128 => 16,
        _ => unreachable!("invalid scalar type"),
    }
}

#[derive(Clone, Eq, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum Constructor {
    DataView,
    Name(
        #[ignore_malloc_size_of = "mozjs"]
        #[no_trace]
        Type,
    ),
}

pub(crate) fn create_buffer_source_with_constructor(
    cx: JSContext,
    constructor: &Constructor,
    buffer_source: &HeapBufferSource<ArrayBufferU8>,
    byte_offset: usize,
    byte_length: usize,
) -> Fallible<HeapBufferSource<ArrayBufferViewU8>> {
    let buffer = unsafe {
        Heap::boxed(
            *buffer_source
                .get_typed_array()
                .expect("Failed to get typed array")
                .underlying_object(),
        )
        .handle()
    };

    match constructor {
        Constructor::DataView => Ok(HeapBufferSource::new(BufferSource::ArrayBufferView(
            Heap::boxed(unsafe { JS_NewDataView(*cx, buffer, byte_offset, byte_length) }),
        ))),
        Constructor::Name(name_type) => construct_typed_array(
            cx,
            name_type,
            buffer_source,
            byte_offset,
            byte_length as i64,
        ),
    }
}

/// Helper function to construct different TypedArray views
fn construct_typed_array(
    cx: JSContext,
    name_type: &Type,
    buffer_source: &HeapBufferSource<ArrayBufferU8>,
    byte_offset: usize,
    byte_length: i64,
) -> Fallible<HeapBufferSource<ArrayBufferViewU8>> {
    let buffer = unsafe {
        Heap::boxed(
            *buffer_source
                .get_typed_array()
                .expect("Failed to get typed array")
                .underlying_object(),
        )
        .handle()
    };
    let array_view = match name_type {
        Type::Int8 => unsafe { JS_NewInt8ArrayWithBuffer(*cx, buffer, byte_offset, byte_length) },
        Type::Uint8 => unsafe { JS_NewUint8ArrayWithBuffer(*cx, buffer, byte_offset, byte_length) },
        Type::Uint16 => unsafe {
            JS_NewUint16ArrayWithBuffer(*cx, buffer, byte_offset, byte_length)
        },
        Type::Int16 => unsafe { JS_NewInt16ArrayWithBuffer(*cx, buffer, byte_offset, byte_length) },
        Type::Int32 => unsafe { JS_NewInt32ArrayWithBuffer(*cx, buffer, byte_offset, byte_length) },
        Type::Uint32 => unsafe {
            JS_NewUint32ArrayWithBuffer(*cx, buffer, byte_offset, byte_length)
        },
        Type::Float32 => unsafe {
            JS_NewFloat32ArrayWithBuffer(*cx, buffer, byte_offset, byte_length)
        },
        Type::Float64 => unsafe {
            JS_NewFloat64ArrayWithBuffer(*cx, buffer, byte_offset, byte_length)
        },
        Type::Uint8Clamped => unsafe {
            JS_NewUint8ClampedArrayWithBuffer(*cx, buffer, byte_offset, byte_length)
        },
        Type::BigInt64 => unsafe {
            JS_NewBigInt64ArrayWithBuffer(*cx, buffer, byte_offset, byte_length)
        },
        Type::BigUint64 => unsafe {
            JS_NewBigUint64ArrayWithBuffer(*cx, buffer, byte_offset, byte_length)
        },
        Type::Float16 => unsafe {
            JS_NewFloat16ArrayWithBuffer(*cx, buffer, byte_offset, byte_length)
        },
        Type::Int64 | Type::Simd128 | Type::MaxTypedArrayViewType => {
            unreachable!("Invalid TypedArray type")
        },
    };

    Ok(HeapBufferSource::new(BufferSource::ArrayBufferView(
        Heap::boxed(array_view),
    )))
}

pub(crate) fn create_array_buffer_with_size(
    cx: JSContext,
    size: usize,
) -> Fallible<HeapBufferSource<ArrayBufferU8>> {
    let result = unsafe { NewArrayBuffer(*cx, size) };
    if result.is_null() {
        rooted!(in(*cx) let mut rval = UndefinedValue());
        unsafe {
            assert!(JS_GetPendingException(*cx, rval.handle_mut().into()));
            JS_ClearPendingException(*cx)
        };

        Err(Error::Type("can't create array buffer".to_owned()))
    } else {
        Ok(HeapBufferSource::<ArrayBufferU8>::new(
            BufferSource::ArrayBuffer(Heap::boxed(result)),
        ))
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
