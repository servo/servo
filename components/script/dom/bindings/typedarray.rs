/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! High-level, safe bindings for JS typed array APIs. Allows creating new
//! typed arrays or wrapping existing JS reflectors, and prevents reinterpreting
//! existing buffers as different types except in well-defined cases.

use js::jsapi::{JS_NewUint8Array, JS_NewUint16Array, JS_NewUint32Array, JS_NewInt8Array};
use js::jsapi::{JS_NewInt16Array, JS_NewInt32Array, JS_NewFloat32Array, JS_NewFloat64Array};
use js::jsapi::{JS_NewUint8ClampedArray, JS_GetUint8ArrayData, JS_GetUint16ArrayData};
use js::jsapi::{JS_GetUint32ArrayData, JS_GetInt8ArrayData, JS_GetInt16ArrayData, JSObject};
use js::jsapi::{JS_GetInt32ArrayData, JS_GetUint8ClampedArrayData, JS_GetFloat32ArrayData};
use js::jsapi::{JS_GetFloat64ArrayData, JSContext, Type};
use js::jsapi::{UnwrapInt8Array, UnwrapInt16Array, UnwrapInt32Array, UnwrapUint8ClampedArray};
use js::jsapi::{UnwrapUint8Array, UnwrapUint16Array, UnwrapUint32Array, UnwrapArrayBufferView};
use js::jsapi::{UnwrapFloat32Array, UnwrapFloat64Array, GetUint8ArrayLengthAndData};
use js::jsapi::{JS_NewArrayBuffer, JS_GetArrayBufferData, JS_GetArrayBufferViewType};
use js::jsapi::{UnwrapArrayBuffer, GetArrayBufferLengthAndData, GetArrayBufferViewLengthAndData};
use js::glue::{GetUint8ClampedArrayLengthAndData, GetFloat32ArrayLengthAndData};
use js::glue::{GetUint16ArrayLengthAndData, GetUint32ArrayLengthAndData};
use js::glue::{GetInt8ArrayLengthAndData, GetFloat64ArrayLengthAndData};
use js::glue::{GetInt16ArrayLengthAndData, GetInt32ArrayLengthAndData};

use core::nonzero::NonZero;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::slice;
use std::ptr;

struct TypedArrayObjectStorage {
    typed_obj: *mut JSObject,
    wrapped_obj: *mut JSObject,
}

impl TypedArrayObjectStorage {
    fn new() -> TypedArrayObjectStorage {
        TypedArrayObjectStorage {
            typed_obj: ptr::null_mut(),
            wrapped_obj: ptr::null_mut(),
        }
    }
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

/// The base representation for all typed arrays.
pub struct TypedArrayBase<T: TypedArrayElement> {
    /// The JS wrapper storage.
    storage: TypedArrayObjectStorage,
    /// The underlying memory buffer.
    data: *mut T::FundamentalType,
    /// The number of elements that make up the buffer.
    length: u32,
    /// True if the length and data of this typed array have been computed.
    /// Any attempt to interact with the underlying buffer must compute it first,
    /// to minimize the window of potential GC hazards.
    computed: bool,
}

impl<T: TypedArrayElement> TypedArrayBase<T> {
    /// Create an uninitialized typed array representation.
    pub fn new() -> TypedArrayBase<T> {
        TypedArrayBase {
            storage: TypedArrayObjectStorage::new(),
            data: ptr::null_mut(),
            length: 0,
            computed: false,
        }
    }

    fn inited(&self) -> bool {
        !self.storage.typed_obj.is_null()
    }

    /// Initialize this typed array representaiton from an existing JS reflector
    /// for a typed array. This operation will fail if attempted on a JS object
    /// that does not match the expected typed array details.
    pub fn init(&mut self, obj: *mut JSObject) -> Result<(), ()> {
        assert!(!self.inited());
        self.storage.typed_obj = T::unwrap_array(obj);
        self.storage.wrapped_obj = self.storage.typed_obj;
        if self.inited() {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Return the underlying buffer as a slice of the element type.
    /// Length and data must be computed first.
    pub fn as_slice(&self) -> &[T::FundamentalType] {
        assert!(self.computed);
        unsafe {
            slice::from_raw_parts(self.data, self.length as usize)
        }
    }

    /// Return the underlying buffer as a mutable slice of the element type.
    /// Length and data must be computed first.
    pub fn as_mut_slice(&mut self) -> &mut [T::FundamentalType] {
        assert!(self.computed);
        unsafe {
            slice::from_raw_parts_mut(self.data, self.length as usize)
        }
    }

    /// Compute the length and data of this typed array.
    /// This operation that must only be performed once, as soon as
    /// possible before making use of the resulting buffer.
    pub fn compute_length_and_data(&mut self) {
        assert!(self.inited());
        assert!(!self.computed);
        let (length, data) = T::length_and_data(self.storage.typed_obj);
        self.length = length;
        self.data = data;
        self.computed = true;
    }
}

impl TypedArrayBase<ArrayBufferViewU8> {
    unsafe fn as_slice_transmute<T>(&self) -> &[T] {
        assert!(self.computed);
        slice::from_raw_parts(self.data as *mut T, self.length as usize / mem::size_of::<T>())
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
pub struct TypedArray<T: TypedArrayElement> {
    base: TypedArrayBase<T>,
}

impl<T: TypedArrayElement> Deref for TypedArray<T> {
    type Target = TypedArrayBase<T>;
    fn deref<'a>(&'a self) -> &'a TypedArrayBase<T> {
        &self.base
    }
}

impl<T: TypedArrayElement> DerefMut for TypedArray<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut TypedArrayBase<T> {
        &mut self.base
    }
}

impl<T: TypedArrayElementCreator + TypedArrayElement> TypedArray<T> {
    /// Create an uninitialized wrapper around a future typed array.
    pub fn new() -> TypedArray<T> {
        TypedArray {
            base: TypedArrayBase::new(),
        }
    }

    /// Create a new JS typed array, optionally providing initial data that will
    /// be copied into the newly-allocated buffer. Returns the new JS reflector.
    pub fn create(cx: *mut JSContext, length: u32, data: Option<&[T::FundamentalType]>)
                  -> Option<NonZero<*mut JSObject>> {
        let obj = T::create_new(cx, length);
        if obj.is_null() {
            return None;
        }

        if let Some(data) = data {
            assert!(data.len() <= length as usize);
            let buf = T::get_data(obj);
            unsafe {
                ptr::copy_nonoverlapping(data.as_ptr(), buf, data.len());
            }
        }

        unsafe {
            Some(NonZero::new(obj))
        }
    }
}

/// A wrapper around a JS ArrayBufferView object.
pub struct ArrayBufferViewBase<T: TypedArrayElement> {
    typed_array: TypedArrayBase<T>,
    type_: Option<Type>,
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

impl<T: TypedArrayElement + ArrayBufferViewType> ArrayBufferViewBase<T> {
    /// Create an uninitilized ArrayBufferView wrapper.
    pub fn new() -> ArrayBufferViewBase<T> {
        ArrayBufferViewBase {
            typed_array: TypedArrayBase::new(),
            type_: None,
        }
    }

    /// Initialize this wrapper with a JS ArrayBufferView reflector. Fails
    /// if the JS reflector is not an ArrayBufferView, or if it is not
    /// one of the primitive numeric types (ie. SharedArrayBufferView, SIMD, etc.).
    pub fn init(&mut self, obj: *mut JSObject) -> Result<(), ()> {
        try!(self.typed_array.init(obj));
        let scalar_type = T::get_view_type(obj);
        if (scalar_type as u32) < Type::MaxTypedArrayViewType as u32 {
            self.type_ = Some(scalar_type);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Compute the length and data of this typed array.
    /// This operation that must only be performed once, as soon as
    /// possible before making use of the resulting buffer.
    pub fn compute_length_and_data(&mut self) {
        self.typed_array.compute_length_and_data();
    }

    /// Return the element type of the wrapped ArrayBufferView.
    pub fn element_type(&self) -> Type {
        assert!(self.type_.is_some());
        self.type_.unwrap()
    }

    /// Return a slice of the bytes of the underlying buffer.
    /// Length and data must be computed first.
    pub fn as_untyped_slice(&self) -> &[u8] {
        unsafe { &*(self.typed_array.as_slice() as *const [T::FundamentalType] as *const [u8]) }
    }

    /// Return a mutable slice of the bytes of the underlying buffer.
    /// Length and data must be computed first.
    pub fn as_mut_untyped_slice(&mut self) -> &mut [u8] {
        unsafe { &mut *(self.typed_array.as_mut_slice() as *mut [T::FundamentalType] as *mut [u8]) }
    }
}

impl ArrayBufferViewBase<ArrayBufferViewU8> {
    /// Return a slice of the underlying buffer reinterpreted as a different element type.
    /// Length and data must be computed first.
    pub fn as_slice<U: ArrayBufferViewOutput>(&mut self) -> Result<&[U], ()> {
        assert!(self.type_.is_some());
        if U::get_view_type() as u32 == self.type_.unwrap() as u32 {
            unsafe {
                Ok(self.typed_array.as_slice_transmute())
            }
        } else {
            Err(())
        }
    }
}

#[allow(missing_docs)]
mod typedefs {
    use super::{TypedArray, ArrayBufferViewBase, ClampedU8, ArrayBufferViewU8, ArrayBufferU8};
    pub type Uint8ClampedArray = TypedArray<ClampedU8>;
    pub type Uint8Array = TypedArray<u8>;
    pub type Int8Array = TypedArray<i8>;
    pub type Uint16Array = TypedArray<u16>;
    pub type Int16Array = TypedArray<i16>;
    pub type Uint32Array = TypedArray<u32>;
    pub type Int32Array = TypedArray<i32>;
    pub type Float32Array = TypedArray<f32>;
    pub type Float64Array = TypedArray<f64>;
    pub type ArrayBufferView = ArrayBufferViewBase<ArrayBufferViewU8>;
    pub type ArrayBuffer = TypedArray<ArrayBufferU8>;
}

pub use self::typedefs::{Uint8ClampedArray, Uint8Array, Uint16Array, Uint32Array};
pub use self::typedefs::{Int8Array, Int16Array, Int32Array, Float64Array, Float32Array};
pub use self::typedefs::{ArrayBuffer, ArrayBufferView};
