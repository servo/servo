/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! High-level, safe bindings for JS typed array APIs. Allows creating new
//! typed arrays or wrapping existing JS reflectors, and prevents reinterpreting
//! existing buffers as different types except in well-defined cases.

use dom::bindings::trace::{JSTraceable, RootedTraceableSet};

use js::jsapi::{JS_NewUint8Array, JS_NewUint16Array, JS_NewUint32Array, JS_NewInt8Array};
use js::jsapi::{JS_NewInt16Array, JS_NewInt32Array, JS_NewFloat32Array, JS_NewFloat64Array};
use js::jsapi::{JS_NewUint8ClampedArray, JS_GetUint8ArrayData, JS_GetUint16ArrayData};
use js::jsapi::{JS_GetUint32ArrayData, JS_GetInt8ArrayData, JS_GetInt16ArrayData, JSObject};
use js::jsapi::{JS_GetInt32ArrayData, JS_GetUint8ClampedArrayData, JS_GetFloat32ArrayData};
use js::jsapi::{JS_GetFloat64ArrayData, JSContext, Type, JSTracer, JS_CallUnbarrieredObjectTracer};
use js::jsapi::{UnwrapInt8Array, UnwrapInt16Array, UnwrapInt32Array, UnwrapUint8ClampedArray};
use js::jsapi::{UnwrapUint8Array, UnwrapUint16Array, UnwrapUint32Array, UnwrapArrayBufferView};
use js::jsapi::{UnwrapFloat32Array, UnwrapFloat64Array, GetUint8ArrayLengthAndData};
use js::jsapi::{JS_NewArrayBuffer, JS_GetArrayBufferData, JS_GetArrayBufferViewType};
use js::jsapi::{UnwrapArrayBuffer, GetArrayBufferLengthAndData, GetArrayBufferViewLengthAndData};
use js::jsapi::MutableHandleObject;
use js::glue::{GetUint8ClampedArrayLengthAndData, GetFloat32ArrayLengthAndData};
use js::glue::{GetUint16ArrayLengthAndData, GetUint32ArrayLengthAndData};
use js::glue::{GetInt8ArrayLengthAndData, GetFloat64ArrayLengthAndData};
use js::glue::{GetInt16ArrayLengthAndData, GetInt32ArrayLengthAndData};

use std::mem;
use std::ops::{Deref, DerefMut};
use std::slice;
use std::ptr;

#[derive(Debug)]
#[allow(raw_pointer_derive)]
struct TypedArrayObjectStorage {
    typed_obj: *mut JSObject,
}

/// Internal trait used to associate an element type with an underlying representation
/// and various functions required to manipulate typed arrays of that element type.
pub trait TypedArrayElement {
    /// Underlying primitive representation of this element type.
    type FundamentalType;
    /// Unwrap a typed array JS reflector for this element type.
    fn unwrap_array(obj: *mut JSObject) -> *mut JSObject;
    /// Retrieve the length and data of a typed array's buffer for this element type.
    fn length_and_data(obj: *mut JSObject) -> (u32, *mut Self::FundamentalType);
}

macro_rules! typed_array_element {
    ($t: ty, $fundamental: ty, $unwrap: ident, $length_and_data: ident) => (
        impl TypedArrayElement for $t {
            type FundamentalType = $fundamental;
            fn unwrap_array(obj: *mut JSObject) -> *mut JSObject {
                unsafe {
                    $unwrap(obj)
                }
            }

            fn length_and_data(obj: *mut JSObject) -> (u32, *mut Self::FundamentalType) {
                unsafe {
                    let mut len = 0;
                    let mut data = ptr::null_mut();
                    $length_and_data(obj, &mut len, &mut data);
                    (len, data)
                }
            }
        }
    );
    ($t: ty, $unwrap: ident, $length_and_data: ident) => {
        typed_array_element!($t, $t, $unwrap, $length_and_data);
    };
}

#[derive(Clone, Copy)]
/// Wrapped for u8 to allow different utility methods for Uint8ClampedArray objects.
pub struct ClampedU8(pub u8);

#[derive(Clone, Copy)]
/// Wrapped for u8 to allow different utility methods for ArrayBuffer objects.
pub struct ArrayBufferU8(pub u8);

#[derive(Clone, Copy)]
/// Wrapped for u8 to allow different utility methods for ArrayBufferView objects.
pub struct ArrayBufferViewU8(pub u8);

typed_array_element!(u8, UnwrapUint8Array, GetUint8ArrayLengthAndData);
typed_array_element!(u16, UnwrapUint16Array, GetUint16ArrayLengthAndData);
typed_array_element!(u32, UnwrapUint32Array, GetUint32ArrayLengthAndData);
typed_array_element!(i8, UnwrapInt8Array, GetInt8ArrayLengthAndData);
typed_array_element!(i16, UnwrapInt16Array, GetInt16ArrayLengthAndData);
typed_array_element!(i32, UnwrapInt32Array, GetInt32ArrayLengthAndData);
typed_array_element!(f32, UnwrapFloat32Array, GetFloat32ArrayLengthAndData);
typed_array_element!(f64, UnwrapFloat64Array, GetFloat64ArrayLengthAndData);
typed_array_element!(ClampedU8, u8, UnwrapUint8ClampedArray, GetUint8ClampedArrayLengthAndData);
typed_array_element!(ArrayBufferU8, u8, UnwrapArrayBuffer, GetArrayBufferLengthAndData);
typed_array_element!(ArrayBufferViewU8, u8, UnwrapArrayBufferView, GetArrayBufferViewLengthAndData);

/// The underlying buffer representation for a typed array.
pub struct TypedArrayBuffer<'a, T: TypedArrayElement> {
    buffer: *mut T::FundamentalType,
    elements: u32,
    _typed_array: &'a TypedArrayObjectStorage,
}

impl<'owner, T: TypedArrayElement> TypedArrayBuffer<'owner, T> {
    /// Return the underlying buffer as a slice of the element type.
    pub fn as_slice(&self) -> &[T::FundamentalType] {
        unsafe {
            slice::from_raw_parts(self.buffer, self.elements as usize)
        }
    }

    /// Return the underlying buffer as a mutable slice of the element type.
    /// Creating multiple mutable slices of the same buffer simultaneously
    /// will result in undefined behaviour.
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T::FundamentalType] {
        slice::from_raw_parts_mut(self.buffer, self.elements as usize)
    }

    fn data(&self) -> *mut T::FundamentalType {
        self.buffer
    }

    fn length(&self) -> u32 {
        self.elements
    }
}

impl<'owner> TypedArrayBuffer<'owner, ArrayBufferViewU8> {
    unsafe fn as_slice_transmute<T: TypedArrayElement>(&self) -> &[T] {
        slice::from_raw_parts(self.buffer as *mut T, self.elements as usize / mem::size_of::<T>())
    }
}

/// The base representation for all typed arrays.
pub struct TypedArrayBase<'a, T: 'a + TypedArrayElement> {
    /// The JS wrapper storage.
    storage: TypedArrayObjectStorage,
    /// The underlying memory buffer and number of contained elements, if computed
    computed: Option<(*mut T::FundamentalType, u32)>,
    rooter: &'a mut TypedArrayRooter,
}

impl<'a, T: TypedArrayElement> TypedArrayBase<'a, T> {
    /// Create a typed array representation that wraps an existing JS reflector.
    /// This operation will fail if attempted on a JS object that does not match
    /// the expected typed array details.
    pub fn from(obj: *mut JSObject, rooter: &'a mut TypedArrayRooter)
                -> Result<TypedArrayBase<'a, T>, ()> {
        let typed_obj = T::unwrap_array(obj);
        if typed_obj.is_null() {
            return Err(());
        }

        Ok(TypedArrayBase {
            storage: TypedArrayObjectStorage {
                typed_obj: obj,
            },
            computed: None,
            rooter: rooter,
        })
    }

    fn inited(&self) -> bool {
        self.rooter.inited()
    }

    /// Initialize the rooting for this wrapper.
    pub fn init(&mut self) {
        assert!(!self.inited());
        self.rooter.init(&mut self.storage);
    }

    /// Retrieve a usable buffer from this typed array.
    pub fn extract(&mut self) -> TypedArrayBuffer<T> {
        assert!(self.inited());
        let (data, length) = match self.computed.as_ref() {
            None => {
                let (length, data) = T::length_and_data(self.storage.typed_obj);
                self.computed = Some((data, length));
                (data, length)
            }
            Some(&(data, length)) => (data, length)
        };
        TypedArrayBuffer {
            buffer: data,
            elements: length,
            _typed_array: &self.storage
        }
    }

}

impl<'a, T: TypedArrayElement> Drop for TypedArrayBase<'a, T> {
    fn drop(&mut self) {
        // Ensure this typed array wrapper wasn't moved during its lifetime.
        let storage = &mut self.storage as *mut _;
        let original = self.rooter.typed_array;
        assert_eq!(storage, original);
    }
}

trait TypedArrayElementCreator: TypedArrayElement {
    fn create_new(cx: *mut JSContext, length: u32) -> *mut JSObject;
    fn get_data(obj: *mut JSObject) -> *mut Self::FundamentalType;
}

macro_rules! typed_array_element_creator {
    ($t: ty, $create_new: ident, $get_data: ident) => (
        impl TypedArrayElementCreator for $t {
            fn create_new(cx: *mut JSContext, length: u32) -> *mut JSObject {
                unsafe {
                    $create_new(cx, length)
                }
            }

            fn get_data(obj: *mut JSObject) -> *mut Self::FundamentalType {
                unsafe {
                    $get_data(obj, ptr::null_mut())
                }
            }
        }
    )
}

typed_array_element_creator!(u8, JS_NewUint8Array, JS_GetUint8ArrayData);
typed_array_element_creator!(u16, JS_NewUint16Array, JS_GetUint16ArrayData);
typed_array_element_creator!(u32, JS_NewUint32Array, JS_GetUint32ArrayData);
typed_array_element_creator!(i8, JS_NewInt8Array, JS_GetInt8ArrayData);
typed_array_element_creator!(i16, JS_NewInt16Array, JS_GetInt16ArrayData);
typed_array_element_creator!(i32, JS_NewInt32Array, JS_GetInt32ArrayData);
typed_array_element_creator!(f32, JS_NewFloat32Array, JS_GetFloat32ArrayData);
typed_array_element_creator!(f64, JS_NewFloat64Array, JS_GetFloat64ArrayData);
typed_array_element_creator!(ClampedU8, JS_NewUint8ClampedArray, JS_GetUint8ClampedArrayData);
typed_array_element_creator!(ArrayBufferU8, JS_NewArrayBuffer, JS_GetArrayBufferData);

/// A wrapper that can be used to create a new typed array from scratch, or
/// initialized from an existing JS reflector as necessary.
pub struct TypedArray<'a, T: 'a + TypedArrayElement> {
    base: TypedArrayBase<'a, T>,
}

impl<'a, T: TypedArrayElement> Deref for TypedArray<'a, T> {
    type Target = TypedArrayBase<'a, T>;
    fn deref<'b>(&'b self) -> &'b TypedArrayBase<'a, T> {
        &self.base
    }
}

impl<'a, T: TypedArrayElement> DerefMut for TypedArray<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut TypedArrayBase<'a, T> {
        &mut self.base
    }
}

impl<'a, T: TypedArrayElementCreator + TypedArrayElement> TypedArray<'a, T> {
    /// Create an uninitialized wrapper around a future typed array.
    pub fn from(obj: *mut JSObject, rooter: &'a mut TypedArrayRooter)
                -> Result<TypedArray<'a, T>, ()> {
        Ok(TypedArray {
            base: try!(TypedArrayBase::from(obj, rooter)),
        })
    }

    /// Create a new JS typed array, optionally providing initial data that will
    /// be copied into the newly-allocated buffer. Returns the new JS reflector.
    pub fn create(cx: *mut JSContext,
                  length: u32,
                  data: Option<&[T::FundamentalType]>,
                  result: MutableHandleObject)
                  -> Result<(), ()> {
        result.set(T::create_new(cx, length));
        if result.get().is_null() {
            return Err(());
        }

        if let Some(data) = data {
            assert!(data.len() <= length as usize);
            let buf = T::get_data(result.get());
            unsafe {
                ptr::copy_nonoverlapping(data.as_ptr(), buf, data.len());
            }
        }

        Ok(())
    }
}

/// A wrapper around a JS ArrayBufferView object.
pub struct ArrayBufferViewBase<'a, T: 'a + TypedArrayElement> {
    typed_array: TypedArrayBase<'a, T>,
    type_: Type,
}

trait ArrayBufferViewType {
    fn get_view_type(obj: *mut JSObject) -> Type;
}

macro_rules! array_buffer_view_type {
    ($t: ty, $get_view_type: ident) => (
        impl ArrayBufferViewType for $t {
            fn get_view_type(obj: *mut JSObject) -> Type {
                unsafe {
                    $get_view_type(obj)
                }
            }
        }
    )
}

array_buffer_view_type!(ArrayBufferViewU8, JS_GetArrayBufferViewType);

/// Internal trait used to perform compile-time reflection of specific
/// typed array element types.
pub trait ArrayBufferViewOutput {
    /// Fetch the compile-type type representation for this type.
    fn get_view_type() -> Type;
}

macro_rules! array_buffer_view_output {
    ($t: ty, $scalar: ident) => (
        impl ArrayBufferViewOutput for $t {
            fn get_view_type() -> Type { Type::$scalar }
        }
    )
}

array_buffer_view_output!(u8, Uint8);
array_buffer_view_output!(i8, Int8);
array_buffer_view_output!(u16, Uint16);
array_buffer_view_output!(i16, Int16);
array_buffer_view_output!(u32, Uint32);
array_buffer_view_output!(i32, Int32);
array_buffer_view_output!(f32, Float32);
array_buffer_view_output!(f64, Float64);
array_buffer_view_output!(ClampedU8, Uint8Clamped);

/// The underlying buffer representation for a typed array.
pub struct ArrayBufferBuffer<'owner, T: TypedArrayElement> {
    base: TypedArrayBuffer<'owner, T>,
    type_: Type,
}

impl<'owner, T: TypedArrayElement> ArrayBufferBuffer<'owner, T> {
    /// Return a slice of the bytes of the underlying buffer.
    pub fn as_untyped_slice(&self) -> &[u8] {
        unsafe {
            let num_bytes = self.base.length() as usize * mem::size_of::<T::FundamentalType>();
            slice::from_raw_parts(self.base.data() as *const _ as *const u8, num_bytes)
        }
    }

    /// Return a mutable slice of the bytes of the underlying buffer.
    /// Creating multiple mutable slices of the same buffer simultaneously
    /// will result in undefined behaviour.
    pub unsafe fn as_mut_untyped_slice(&mut self) -> &mut [u8] {
        let num_bytes = self.base.length() as usize * mem::size_of::<T::FundamentalType>();
        slice::from_raw_parts_mut(self.base.data() as *mut u8, num_bytes)
    }
}

impl<'a, T: TypedArrayElement + ArrayBufferViewType> ArrayBufferViewBase<'a, T> {
    /// Create an ArrayBufferView wrapper for a JS reflector.
    /// Fails if the JS reflector is not an ArrayBufferView, or if it is not
    /// one of the primitive numeric types (ie. SharedArrayBufferView, SIMD, etc.).
    pub fn from(obj: *mut JSObject, rooter: &'a mut TypedArrayRooter)
                -> Result<ArrayBufferViewBase<'a, T>, ()> {
        let typed_array_base = try!(TypedArrayBase::from(obj, rooter));

        let scalar_type = T::get_view_type(obj);
        if (scalar_type as u32) >= Type::MaxTypedArrayViewType as u32 {
            return Err(());
        }

        Ok(ArrayBufferViewBase {
            typed_array: typed_array_base,
            type_: scalar_type,
        })
    }

    /// Initialize this wrapper.
    pub fn init(&mut self) {
        self.typed_array.init();
    }

    /// Retrieve a usable buffer from this typed array.
    pub fn extract(&mut self) -> ArrayBufferBuffer<T> {
        ArrayBufferBuffer {
            base: self.typed_array.extract(),
            type_: self.type_,
        }
    }

    /// Return the element type of the wrapped ArrayBufferView.
    pub fn element_type(&self) -> Type {
        self.type_
    }
}

impl<'owner> ArrayBufferBuffer<'owner, ArrayBufferViewU8> {
    /// Return a slice of the underlying buffer reinterpreted as a different element type.
    pub fn as_slice<U>(&self) -> Result<&[U], ()>
                              where U: ArrayBufferViewOutput + TypedArrayElement {
        if U::get_view_type() as u32 == self.type_ as u32 {
            unsafe {
                Ok(self.base.as_slice_transmute())
            }
        } else {
            Err(())
        }
    }
}

#[allow(missing_docs)]
mod typedefs {
    use super::{TypedArray, ArrayBufferViewBase, ClampedU8, ArrayBufferViewU8, ArrayBufferU8};
    pub type Uint8ClampedArray<'a> = TypedArray<'a, ClampedU8>;
    pub type Uint8Array<'a> = TypedArray<'a, u8>;
    pub type Int8Array<'a> = TypedArray<'a, i8>;
    pub type Uint16Array<'a> = TypedArray<'a, u16>;
    pub type Int16Array<'a> = TypedArray<'a, i16>;
    pub type Uint32Array<'a> = TypedArray<'a, u32>;
    pub type Int32Array<'a> = TypedArray<'a, i32>;
    pub type Float32Array<'a> = TypedArray<'a, f32>;
    pub type Float64Array<'a> = TypedArray<'a, f64>;
    pub type ArrayBufferView<'a> = ArrayBufferViewBase<'a, ArrayBufferViewU8>;
    pub type ArrayBuffer<'a> = TypedArray<'a, ArrayBufferU8>;
}

pub use self::typedefs::{Uint8ClampedArray, Uint8Array, Uint16Array, Uint32Array};
pub use self::typedefs::{Int8Array, Int16Array, Int32Array, Float64Array, Float32Array};
pub use self::typedefs::{ArrayBuffer, ArrayBufferView};

/// A GC root for a typed array wrapper.
pub struct TypedArrayRooter {
    typed_array: *mut TypedArrayObjectStorage,
}

impl TypedArrayRooter {
    /// Create a new GC root for a forthcoming typed array wrapper.
    pub fn new() -> TypedArrayRooter {
        TypedArrayRooter {
            typed_array: ptr::null_mut(),
        }
    }

    fn inited(&self) -> bool {
        !self.typed_array.is_null()
    }

    fn init(&mut self, array: &mut TypedArrayObjectStorage) {
        assert!(self.typed_array.is_null());
        self.typed_array = array;
        RootedTraceableSet::add(self);
    }
}

impl JSTraceable for TypedArrayRooter {
    fn trace(&self, trc: *mut JSTracer) {
        unsafe {
            let storage = &mut *self.typed_array;
            if !storage.typed_obj.is_null() {
                JS_CallUnbarrieredObjectTracer(trc,
                                               &mut storage.typed_obj,
                                               b"TypedArray.typed_obj\0".as_ptr() as *const _);
            }
        }
    }
}

impl Drop for TypedArrayRooter {
    fn drop(&mut self) {
        RootedTraceableSet::remove(self);
    }
}
