/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![expect(unsafe_code)]

#[cfg(feature = "webgpu")]
use std::ffi::c_void;
use std::marker::PhantomData;
#[cfg(feature = "webgpu")]
use std::ops::Range;
use std::ptr;
#[cfg(feature = "webgpu")]
use std::sync::Arc;

use js::context::JSContext;
use js::jsapi::{
    GetArrayBufferByteLength, Heap, IsArrayBufferObject, IsDetachedArrayBufferObject,
    JS_GetArrayBufferViewByteLength, JS_GetArrayBufferViewByteOffset, JS_GetArrayBufferViewType,
    JS_GetTypedArrayLength, JS_IsArrayBufferViewObject, JS_IsTypedArrayObject, JSObject, Type,
};
use js::jsval::{ObjectValue, UndefinedValue};
#[cfg(feature = "webgpu")]
use js::rust::wrappers2::NewExternalArrayBuffer;
use js::rust::wrappers2::{
    ArrayBufferClone, ArrayBufferCopyData, DetachArrayBuffer, HasDefinedArrayBufferDetachKey,
    JS_ClearPendingException, JS_GetArrayBufferViewBuffer, JS_GetPendingException,
    JS_NewBigInt64ArrayWithBuffer, JS_NewBigUint64ArrayWithBuffer, JS_NewDataView,
    JS_NewFloat16ArrayWithBuffer, JS_NewFloat32ArrayWithBuffer, JS_NewFloat64ArrayWithBuffer,
    JS_NewInt8ArrayWithBuffer, JS_NewInt16ArrayWithBuffer, JS_NewInt32ArrayWithBuffer,
    JS_NewUint8ArrayWithBuffer, JS_NewUint8ClampedArrayWithBuffer, JS_NewUint16ArrayWithBuffer,
    JS_NewUint32ArrayWithBuffer, NewArrayBuffer, NewArrayBufferWithContents,
    StealArrayBufferContents,
};
use js::rust::{
    CustomAutoRooterGuard, Handle, MutableHandleObject,
    MutableHandleValue as SafeMutableHandleValue,
};
#[cfg(feature = "webgpu")]
use js::typedarray::HeapArrayBuffer;
use js::typedarray::{
    ArrayBufferU8, ArrayBufferViewU8, CreateWith, TypedArray, TypedArrayElement,
    TypedArrayElementCreator,
};

use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::trace::RootedTraceableBox;

pub(crate) type RootedTypedArray<T> = RootedTraceableBox<TypedArray<T, Box<Heap<*mut JSObject>>>>;

/// Represents a `BufferSource` as defined in the WebIDL specification.
///
/// A `BufferSource` is either an `ArrayBuffer` or an `ArrayBufferView`, which
/// provides a view onto an `ArrayBuffer`.
///
/// See: <https://webidl.spec.whatwg.org/#BufferSource>
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum BufferSource {
    /// Represents an `ArrayBufferView` (e.g., `Uint8Array`, `DataView`).
    /// See: <https://webidl.spec.whatwg.org/#ArrayBufferView>
    ArrayBufferView(Box<Heap<*mut JSObject>>),

    /// Represents an `ArrayBuffer`, a fixed-length binary data buffer.
    /// See: <https://webidl.spec.whatwg.org/#idl-ArrayBuffer>
    ArrayBuffer(Box<Heap<*mut JSObject>>),
}

impl Clone for BufferSource {
    fn clone(&self) -> Self {
        match self {
            BufferSource::ArrayBufferView(heap) => {
                BufferSource::ArrayBufferView(Heap::boxed(heap.get()))
            },
            BufferSource::ArrayBuffer(heap) => BufferSource::ArrayBuffer(Heap::boxed(heap.get())),
        }
    }
}

/// <https://webidl.spec.whatwg.org/#dfn-get-buffer-source-copy>
///
/// Spec steps and how they're covered:
///
/// - **Convert to JS value**: Handled by WebIDL bindings before this function
///   receives the typed `ArrayBufferViewOrArrayBuffer` union.
///
/// - **ArrayBufferView offset/length**: `view.to_vec()` (mozjs) calls
///   `GetArrayBufferViewLengthAndData`, which respects the view's
///   `[[ByteOffset]]` and `[[ByteLength]]` — only the viewed bytes are copied.
///
/// - **ArrayBuffer**: `buffer.to_vec()` (mozjs) calls
///   `GetArrayBufferLengthAndData`, copying the entire buffer contents.
///
/// - **Detached buffer**: When a buffer is detached, SpiderMonkey's
///   `GetArrayBuffer(LengthAndData|ViewLengthAndData)` returns a null pointer
///   and zero length. `to_vec()` thus produces an empty `Vec<u8>`.
///
/// - **SharedArrayBuffer**: Not applicable — `ArrayBufferViewOrArrayBuffer`
///   does not include a SharedArrayBuffer variant.
pub(crate) fn get_buffer_source_copy(source: &ArrayBufferViewOrArrayBuffer) -> Vec<u8> {
    match source {
        ArrayBufferViewOrArrayBuffer::ArrayBufferView(view) => view.to_vec(),
        ArrayBufferViewOrArrayBuffer::ArrayBuffer(buffer) => buffer.to_vec(),
    }
}

pub(crate) fn create_heap_buffer_source_with_length<T>(
    cx: &mut JSContext,
    len: u32,
) -> Fallible<RootedTraceableBox<HeapBufferSource<T>>>
where
    T: TypedArrayElement + TypedArrayElementCreator + 'static,
    T::Element: Clone + Copy,
{
    rooted!(&in(cx) let mut array = ptr::null_mut::<JSObject>());
    let typed_array_result =
        create_buffer_source_with_length::<T>(cx, len as usize, array.handle_mut());
    if typed_array_result.is_err() {
        return Err(Error::JSFailed);
    }

    Ok(RootedTraceableBox::new(HeapBufferSource::<T>::new(
        array.handle(),
    )))
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
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
                BufferSource::ArrayBufferView(from_heap) | BufferSource::ArrayBuffer(from_heap) => {
                    std::ptr::eq(heap.get(), from_heap.get())
                },
            },
        }
    }
}

impl<T> Clone for HeapBufferSource<T>
where
    T: TypedArrayElement,
{
    fn clone(&self) -> Self {
        HeapBufferSource {
            buffer_source: self.buffer_source.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T> HeapBufferSource<T>
where
    T: TypedArrayElement,
{
    /// Create a buffer source from a rooted ArrayBuffer or ArrayBufferView.
    pub(crate) fn new(object: Handle<*mut JSObject>) -> HeapBufferSource<T> {
        let object = object.get();
        assert!(!object.is_null());

        HeapBufferSource {
            buffer_source: if unsafe { IsArrayBufferObject(object) } {
                BufferSource::ArrayBuffer(Heap::boxed(object))
            } else {
                assert!(unsafe { JS_IsArrayBufferViewObject(object) });
                BufferSource::ArrayBufferView(Heap::boxed(object))
            },
            phantom: PhantomData,
        }
    }

    pub(crate) fn from_view(
        cx: &mut JSContext,
        chunk: CustomAutoRooterGuard<TypedArray<T, *mut JSObject>>,
    ) -> RootedTraceableBox<HeapBufferSource<T>> {
        rooted!(&in(cx) let object = unsafe { *chunk.underlying_object() });
        RootedTraceableBox::new(HeapBufferSource::<T>::new(object.handle()))
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

    pub(crate) fn get_typed_array(&self) -> Result<RootedTypedArray<T>, ()> {
        TypedArray::from(match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer) => {
                buffer.get()
            },
        })
        .map(RootedTraceableBox::new)
    }

    pub(crate) fn get_buffer_view_value(
        &self,
        cx: &mut JSContext,
        mut handle_mut: SafeMutableHandleValue,
    ) {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => {
                rooted!(&in(cx) let value = ObjectValue(buffer.get()));
                handle_mut.set(*value);
            },
            BufferSource::ArrayBuffer(_) => {
                unreachable!("BufferSource::ArrayBuffer does not have a view buffer.")
            },
        }
    }

    pub(crate) fn get_array_buffer_view_buffer(
        &self,
        cx: &mut JSContext,
    ) -> RootedTraceableBox<HeapBufferSource<ArrayBufferU8>> {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => unsafe {
                let mut is_shared = false;
                rooted!(&in(cx) let view_buffer =
                         JS_GetArrayBufferViewBuffer(cx, Handle::from_raw(buffer.handle()), &mut is_shared));

                RootedTraceableBox::new(HeapBufferSource::<ArrayBufferU8>::new(
                    view_buffer.handle(),
                ))
            },
            BufferSource::ArrayBuffer(_) => {
                unreachable!("BufferSource::ArrayBuffer does not have a view buffer.")
            },
        }
    }

    /// <https://tc39.es/ecma262/#sec-detacharraybuffer>
    pub(crate) fn detach_buffer(&self, cx: &mut JSContext) -> bool {
        assert!(self.is_initialized());
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => {
                let mut is_shared = false;
                unsafe {
                    // assert buffer is an ArrayBuffer view
                    assert!(JS_IsArrayBufferViewObject(*buffer.handle()));
                    rooted!(&in(cx) let view_buffer =
                            JS_GetArrayBufferViewBuffer(cx, Handle::from_raw(buffer.handle()), &mut is_shared));
                    // This buffer is always created unshared
                    debug_assert!(!is_shared);
                    // Detach the ArrayBuffer
                    DetachArrayBuffer(cx, view_buffer.handle())
                }
            },
            BufferSource::ArrayBuffer(buffer) => unsafe {
                DetachArrayBuffer(cx, Handle::from_raw(buffer.handle()))
            },
        }
    }

    pub(crate) fn typed_array_to_option(&self) -> Option<RootedTypedArray<T>> {
        if self.is_initialized() {
            self.get_typed_array().ok()
        } else {
            warn!("Buffer not initialized.");
            None
        }
    }

    pub(crate) fn is_detached_buffer(&self, cx: &mut JSContext) -> bool {
        assert!(self.is_initialized());
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => {
                let mut is_shared = false;
                unsafe {
                    assert!(JS_IsArrayBufferViewObject(*buffer.handle()));
                    rooted!(&in(cx) let view_buffer =
                            JS_GetArrayBufferViewBuffer(cx, Handle::from_raw(buffer.handle()), &mut is_shared));
                    debug_assert!(!is_shared);
                    IsDetachedArrayBufferObject(*view_buffer.handle())
                }
            },
            BufferSource::ArrayBuffer(buffer) => unsafe {
                IsDetachedArrayBufferObject(*buffer.handle())
            },
        }
    }

    pub(crate) fn viewed_buffer_array_byte_length(&self, cx: &mut JSContext) -> usize {
        assert!(self.is_initialized());
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => {
                let mut is_shared = false;
                unsafe {
                    assert!(JS_IsArrayBufferViewObject(*buffer.handle()));
                    rooted!(&in(cx) let view_buffer =
                            JS_GetArrayBufferViewBuffer(cx, Handle::from_raw(buffer.handle()), &mut is_shared));
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

    /// <https://tc39.es/ecma262/#sec-clonearraybuffer>
    pub(crate) fn clone_array_buffer(
        &self,
        cx: &mut JSContext,
        byte_offset: usize,
        byte_length: usize,
    ) -> Fallible<RootedTraceableBox<HeapBufferSource<ArrayBufferU8>>> {
        let result = match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => {
                let mut is_shared = false;
                rooted!(&in(cx) let view_buffer =
                    unsafe { JS_GetArrayBufferViewBuffer(cx, Handle::from_raw(buffer.handle()), &mut is_shared) });
                debug_assert!(!is_shared);

                unsafe { ArrayBufferClone(cx, view_buffer.handle(), byte_offset, byte_length) }
            },
            BufferSource::ArrayBuffer(buffer) => unsafe {
                ArrayBufferClone(
                    cx,
                    Handle::from_raw(buffer.handle()),
                    byte_offset,
                    byte_length,
                )
            },
        };

        rooted!(&in(cx) let result = result);

        if result.is_null() {
            // Normalize SpiderMonkey failure: consume pending exception and
            // map it to a DOM Error.
            rooted!(&in(cx) let mut _ex = UndefinedValue());
            unsafe {
                // If SpiderMonkey set an exception, clear it so callers see a clean cx.
                if JS_GetPendingException(cx, _ex.handle_mut()) {
                    JS_ClearPendingException(cx);
                }
            }

            Err(Error::Type(c"can't clone array buffer".to_owned()))
        } else {
            Ok(RootedTraceableBox::new(
                HeapBufferSource::<ArrayBufferU8>::new(result.handle()),
            ))
        }
    }
    /// <https://streams.spec.whatwg.org/#abstract-opdef-cloneasuint8array>
    #[expect(unsafe_code)]
    pub(crate) fn clone_as_uint8_array(
        &self,
        cx: &mut JSContext,
    ) -> Fallible<RootedTraceableBox<HeapBufferSource<ArrayBufferViewU8>>> {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) => {
                // Assert: O is an Object.
                // Assert: O has an [[ViewedArrayBuffer]] internal slot.
                assert!(unsafe { JS_IsArrayBufferViewObject(*buffer.handle()) });

                // Assert: ! IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is false.
                assert!(!self.is_detached_buffer(cx));

                // Let buffer be ? CloneArrayBuffer(O.[[ViewedArrayBuffer]],
                // O.[[ByteOffset]], O.[[ByteLength]], %ArrayBuffer%).
                let byte_offset = self.get_byte_offset();
                let byte_length = self.byte_length();

                let buffer = self.clone_array_buffer(cx, byte_offset, byte_length)?;

                // Let array be ! Construct(%Uint8Array%, « buffer »).
                // Return array.
                construct_typed_array(cx, &Type::Uint8, &buffer, 0, byte_length as i64)
            },
            BufferSource::ArrayBuffer(_buffer) => {
                unreachable!("BufferSource::ArrayBuffer does not have a view buffer.")
            },
        }
    }

    pub(crate) fn is_undefined(&self) -> bool {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer) => {
                buffer.get().is_null()
            },
        }
    }
}

impl<T> HeapBufferSource<T>
where
    T: TypedArrayElement + TypedArrayElementCreator + 'static,
    T::Element: Clone + Copy,
{
    pub(crate) fn acquire_data(&self, cx: &mut JSContext) -> Result<Vec<T::Element>, ()> {
        assert!(self.is_initialized());

        typedarray!(&in(cx) let array: TypedArray = match &self.buffer_source {
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
        cx: &mut JSContext,
        dest: &mut [T::Element],
        source_start: usize,
        length: usize,
    ) -> Result<(), ()> {
        assert!(self.is_initialized());
        typedarray!(&in(cx) let array: TypedArray = match &self.buffer_source {
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
        let slice = (*array).as_slice_safe(cx.no_gc());
        dest.copy_from_slice(&slice[source_start..length]);
        Ok(())
    }

    pub(crate) fn copy_data_from(
        &self,
        cx: &mut JSContext,
        source: CustomAutoRooterGuard<TypedArray<T, *mut JSObject>>,
        dest_start: usize,
        length: usize,
    ) -> Result<(), ()> {
        assert!(self.is_initialized());
        typedarray!(&in(cx) let mut array: TypedArray = match &self.buffer_source {
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
        let slice = (*array).as_mut_slice_safe(cx.no_gc());
        let (_, dest) = slice.split_at_mut(dest_start);
        dest[0..length].copy_from_slice(&source.as_slice_safe(cx.no_gc())[0..length]);
        Ok(())
    }

    pub(crate) fn set_data(&self, cx: &mut JSContext, data: &[T::Element]) -> Result<(), ()> {
        rooted!(&in(cx) let mut array = ptr::null_mut::<JSObject>());
        let _ = create_buffer_source::<T>(cx, data, array.handle_mut())?;

        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer) => {
                buffer.set(*array);
            },
        }
        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-cancopydatablockbytes>
    // CanCopyDataBlockBytes(descriptorBuffer, destStart, queueBuffer, queueByteOffset, bytesToCopy)
    pub(crate) fn can_copy_data_block_bytes(
        &self,
        cx: &mut JSContext,
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
                        if std::ptr::eq(heap.get(), from_heap.get()) {
                            return false;
                        }
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
        cx: &mut JSContext,
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
                        cx,
                        Handle::from_raw(heap.handle()),
                        dest_start,
                        Handle::from_raw(from_heap.handle()),
                        from_byte_offset,
                        bytes_to_copy,
                    ),
                }
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#can-transfer-array-buffer>
    pub(crate) fn can_transfer_array_buffer(&self, cx: &mut JSContext) -> bool {
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
                if !HasDefinedArrayBufferDetachKey(
                    cx,
                    Handle::from_raw(heap.handle()),
                    &mut is_defined,
                ) {
                    return false;
                }
            },
        }

        !is_defined
    }

    /// <https://streams.spec.whatwg.org/#transfer-array-buffer>
    pub(crate) fn transfer_array_buffer(
        &self,
        cx: &mut JSContext,
    ) -> Fallible<RootedTraceableBox<HeapBufferSource<ArrayBufferU8>>> {
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
                StealArrayBufferContents(cx, Handle::from_raw(buffer.handle()))
            },
        };

        // Perform ? DetachArrayBuffer(O).
        // This will throw an exception if O has an [[ArrayBufferDetachKey]] that is not undefined,
        // such as a WebAssembly.Memory’s buffer. [WASM-JS-API-1]
        if !self.detach_buffer(cx) {
            rooted!(&in(cx) let mut rval = UndefinedValue());
            unsafe {
                assert!(JS_GetPendingException(cx, rval.handle_mut()));
                JS_ClearPendingException(cx)
            };

            Err(Error::Type(c"can't transfer array buffer".to_owned()))
        } else {
            // Return a new ArrayBuffer object, created in the current Realm,
            // whose [[ArrayBufferData]] internal slot value is arrayBufferData and
            // whose [[ArrayBufferByteLength]] internal slot value is arrayBufferByteLength.
            rooted!(&in(cx) let result = unsafe {
                NewArrayBufferWithContents(cx, buffer_length, buffer_data)
            });
            if result.is_null() {
                return Err(Error::JSFailed);
            }
            Ok(RootedTraceableBox::new(
                HeapBufferSource::<ArrayBufferU8>::new(result.handle()),
            ))
        }
    }
}

unsafe impl<T> crate::dom::bindings::trace::JSTraceable for HeapBufferSource<T> {
    #[inline]
    unsafe fn trace(&self, tracer: *mut js::jsapi::JSTracer) {
        match &self.buffer_source {
            BufferSource::ArrayBufferView(buffer) | BufferSource::ArrayBuffer(buffer) => {
                unsafe { buffer.trace(tracer) };
            },
        }
    }
}

/// <https://webidl.spec.whatwg.org/#arraybufferview-create>
pub(crate) fn create_buffer_source<T>(
    cx: &mut JSContext,
    data: &[T::Element],
    mut dest: MutableHandleObject,
) -> Result<RootedTypedArray<T>, ()>
where
    T: TypedArrayElement + TypedArrayElementCreator,
{
    let res = unsafe {
        TypedArray::<T, *mut JSObject>::create(
            cx.raw_cx(),
            CreateWith::Slice(data),
            dest.reborrow(),
        )
    };

    if res.is_err() {
        Err(())
    } else {
        TypedArray::from(dest.get()).map(RootedTraceableBox::new)
    }
}

fn create_buffer_source_with_length<T>(
    cx: &mut JSContext,
    len: usize,
    mut dest: MutableHandleObject,
) -> Result<RootedTypedArray<T>, ()>
where
    T: TypedArrayElement + TypedArrayElementCreator,
{
    let res = unsafe {
        TypedArray::<T, *mut JSObject>::create(
            cx.raw_cx(),
            CreateWith::Length(len),
            dest.reborrow(),
        )
    };

    if res.is_err() {
        Err(())
    } else {
        TypedArray::from(dest.get()).map(RootedTraceableBox::new)
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
    cx: &mut JSContext,
    constructor: &Constructor,
    buffer_source: &HeapBufferSource<ArrayBufferU8>,
    byte_offset: usize,
    byte_length: usize,
) -> Fallible<RootedTraceableBox<HeapBufferSource<ArrayBufferViewU8>>> {
    match &buffer_source.buffer_source {
        BufferSource::ArrayBuffer(heap) => match constructor {
            Constructor::DataView => {
                rooted!(&in(cx) let view = unsafe {
                    JS_NewDataView(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    )
                });
                if view.is_null() {
                    return Err(Error::JSFailed);
                }
                Ok(RootedTraceableBox::new(HeapBufferSource::new(
                    view.handle(),
                )))
            },
            Constructor::Name(name_type) => construct_typed_array(
                cx,
                name_type,
                buffer_source,
                byte_offset,
                byte_length as i64,
            ),
        },
        BufferSource::ArrayBufferView(_) => {
            unreachable!("Can not create a new ArrayBufferView from an existing ArrayBufferView");
        },
    }
}

/// Helper function to construct different TypedArray views
fn construct_typed_array(
    cx: &mut JSContext,
    name_type: &Type,
    buffer_source: &HeapBufferSource<ArrayBufferU8>,
    byte_offset: usize,
    byte_length: i64,
) -> Fallible<RootedTraceableBox<HeapBufferSource<ArrayBufferViewU8>>> {
    match &buffer_source.buffer_source {
        BufferSource::ArrayBuffer(heap) => {
            rooted!(&in(cx) let array_view = unsafe {
                match name_type {
                    Type::Int8 => JS_NewInt8ArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::Uint8 => JS_NewUint8ArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::Uint16 => JS_NewUint16ArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::Int16 => JS_NewInt16ArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::Int32 => JS_NewInt32ArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::Uint32 => JS_NewUint32ArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::Float32 => JS_NewFloat32ArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::Float64 => JS_NewFloat64ArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::Uint8Clamped => JS_NewUint8ClampedArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::BigInt64 => JS_NewBigInt64ArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::BigUint64 => JS_NewBigUint64ArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::Float16 => JS_NewFloat16ArrayWithBuffer(
                        cx,
                        Handle::from_raw(heap.handle()),
                        byte_offset,
                        byte_length,
                    ),
                    Type::Int64 | Type::Simd128 | Type::MaxTypedArrayViewType => {
                        unreachable!("Invalid TypedArray type")
                    },
                }
            });
            if array_view.is_null() {
                return Err(Error::JSFailed);
            }

            Ok(RootedTraceableBox::new(HeapBufferSource::new(
                array_view.handle(),
            )))
        },
        BufferSource::ArrayBufferView(_) => {
            unreachable!("Can not create a new ArrayBufferView from an existing ArrayBufferView");
        },
    }
}

pub(crate) fn create_array_buffer_with_size(
    cx: &mut JSContext,
    size: usize,
) -> Fallible<RootedTraceableBox<HeapBufferSource<ArrayBufferU8>>> {
    rooted!(&in(cx) let result = unsafe { NewArrayBuffer(cx, size) });
    if result.is_null() {
        rooted!(&in(cx) let mut rval = UndefinedValue());
        unsafe {
            assert!(JS_GetPendingException(cx, rval.handle_mut()));
            JS_ClearPendingException(cx)
        };

        Err(Error::Type(c"can't create array buffer".to_owned()))
    } else {
        Ok(RootedTraceableBox::new(
            HeapBufferSource::<ArrayBufferU8>::new(result.handle()),
        ))
    }
}

#[cfg(feature = "webgpu")]
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DataBlock {
    #[conditional_malloc_size_of]
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
    pub(crate) fn view(
        &mut self,
        cx: &mut JSContext,
        range: Range<usize>,
    ) -> Result<&DataView, ()> {
        if self
            .data_views
            .iter()
            .any(|view| range_overlap(&view.range, &range))
        {
            return Err(());
        }
        let range_len = range
            .end
            .checked_sub(range.start)
            .expect("range end must be >= range start");
        assert!(range.end <= self.data.len());

        /// `freeFunc()` must be threadsafe, should be safely callable from any thread
        /// without causing conflicts, unexpected behavior.
        unsafe extern "C" fn free_func(_contents: *mut c_void, free_user_data: *mut c_void) {
            let raw: *const Box<[u8]> = free_user_data.cast();
            // SAFETY: `free_func` is called by SM and returns ownership of the Arc we
            // leaked below with `into_raw`. Hence it is safe to reconstruct the Arc,
            // and destroy it to release the reference count.
            drop(unsafe { Arc::from_raw(raw) });
        }
        let raw: *const Box<[u8]> = Arc::into_raw(Arc::clone(&self.data));
        // SAFETY: We leaked the Arc, so the underlying slice will stay alive
        // until `free_func` is called. `range.start..range.end` is inside
        // the valid range of the slice.
        let data_ptr = unsafe { (**raw).as_ptr().add(range.start) };
        rooted!(&in(cx) let object = unsafe {
            NewExternalArrayBuffer(
                cx,
                range_len,
                // FIXME(jschwe): I believe casting to a mutable pointer is unsound.
                // We would need interior mutability.
                data_ptr.cast_mut().cast(),
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
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DataView {
    #[no_trace]
    range: Range<usize>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    buffer: HeapArrayBuffer,
}

#[cfg(feature = "webgpu")]
impl DataView {
    pub(crate) fn array_buffer(&self) -> RootedTraceableBox<HeapArrayBuffer> {
        RootedTraceableBox::new(unsafe {
            HeapArrayBuffer::from(self.buffer.underlying_object().get()).unwrap()
        })
    }
}

#[cfg(feature = "webgpu")]
impl Drop for DataView {
    #[expect(unsafe_code)]
    fn drop(&mut self) {
        // TODO: https://github.com/servo/servo/issues/44640
        let mut cx = unsafe { script_bindings::script_runtime::temp_cx() };
        assert!(unsafe {
            DetachArrayBuffer(
                &mut cx,
                Handle::from_raw(self.buffer.underlying_object().handle()),
            )
        })
    }
}
