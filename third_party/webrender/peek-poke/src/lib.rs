// Copyright 2019 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Fast binary serialization and deserialization for types with a known maximum size.
//!
//! ## Binary Encoding Scheme
//!
//! ## Usage
//!
//! ## Comparison to bincode

#[cfg(feature = "derive")]
pub use peek_poke_derive::*;

use core::{marker::PhantomData, mem::size_of, slice};
use crate::{slice_ext::*, vec_ext::*};

mod slice_ext;
mod vec_ext;

union MaybeUninitShim<T: Copy> {
    uninit: (),
    init: T,
}

/// Peek helper for constructing a `T` by `Copy`ing into an uninitialized stack
/// allocation.
pub unsafe fn peek_from_uninit<T: Copy + Peek>(bytes: *const u8) -> (T, *const u8) {
    let mut val = MaybeUninitShim { uninit: () };
    let bytes = <T>::peek_from(bytes, &mut val.init);
    (val.init, bytes)
}

/// Peek helper for constructing a `T` by `Default` initialized stack
/// allocation.
pub unsafe fn peek_from_default<T: Default + Peek>(bytes: *const u8) -> (T, *const u8) {
    let mut val = T::default();
    let bytes = <T>::peek_from(bytes, &mut val);
    (val, bytes)
}

/// Peek inplace a `T` from a slice of bytes, returning a slice of the remaining
/// bytes. `src` must contain at least `T::max_size()` bytes.
///
/// [`ensure_red_zone`] can be used to add required padding.
pub fn peek_from_slice<'a, T: Peek>(src: &'a [u8], dst: &mut T) -> &'a [u8] {
    unsafe {
        // If src.len() == T::max_size() then src is at the start of the red-zone.
        assert!(T::max_size() < src.len(), "WRDL: unexpected end of display list");
        let end_ptr = T::peek_from(src.as_ptr(), dst);
        let len = end_ptr as usize - src.as_ptr() as usize;
        // Did someone break the T::peek_from() can't read more than T::max_size()
        // bytes contract?
        assert!(len <= src.len(), "WRDL: Peek::max_size was wrong");
        slice::from_raw_parts(end_ptr, src.len() - len)
    }
}

/// Poke helper to insert a serialized version of `src` at the beginning for `dst`.
pub fn poke_inplace_slice<T: Poke>(src: &T, dst: &mut [u8]) {
    assert!(T::max_size() <= dst.len(),  "WRDL: buffer too small to write into");
    unsafe {
        src.poke_into(dst.as_mut_ptr());
    }
}

/// Poke helper to append a serialized version of `src` to the end of `dst`.
pub fn poke_into_vec<T: Poke>(src: &T, dst: &mut Vec<u8>) {
    dst.reserve(T::max_size());
    unsafe {
        let ptr = dst.as_end_mut_ptr();
        let end_ptr = src.poke_into(ptr);
        dst.set_end_ptr(end_ptr);
    }
}

// TODO: Is returning the len of the iterator of any practical use?
pub fn poke_extend_vec<I>(src: I, dst: &mut Vec<u8>) -> usize
where
    I: ExactSizeIterator,
    I::Item: Poke,
{
    let len = src.len();
    let max_size = len * I::Item::max_size();
    dst.reserve(max_size);
    unsafe {
        let ptr = dst.as_end_mut_ptr();
        // Guard against the possibility of a misbehaved implementation of
        // ExactSizeIterator by writing at most `len` items.
        let end_ptr = src.take(len).fold(ptr, |ptr, item| item.poke_into(ptr));
        dst.set_end_ptr(end_ptr);
    }

    len
}

/// Add `T::max_size()` "red zone" (padding of zeroes) to the end of the vec of
/// `bytes`. This allows deserialization to assert that at least `T::max_size()`
/// bytes exist at all times.
pub fn ensure_red_zone<T: Poke>(bytes: &mut Vec<u8>) {
    bytes.reserve(T::max_size());
    unsafe {
        let end_ptr = bytes.as_end_mut_ptr();
        end_ptr.write_bytes(0, T::max_size());
        bytes.set_end_ptr(end_ptr.add(T::max_size()));
    }
}

#[inline]
unsafe fn read_verbatim<T>(src: *const u8, dst: *mut T) -> *const u8 {
    *dst = (src as *const T).read_unaligned();
    src.add(size_of::<T>())
}

#[inline]
unsafe fn write_verbatim<T>(src: T, dst: *mut u8) -> *mut u8 {
    (dst as *mut T).write_unaligned(src);
    dst.add(size_of::<T>())
}

#[cfg(feature = "extras")]
mod euclid;

/// A trait for values that provide serialization into buffers of bytes.
///
/// # Example
///
/// ```no_run
/// use peek_poke::Poke;
///
/// struct Bar {
///     a: u32,
///     b: u8,
///     c: i16,
/// }
///
/// unsafe impl Poke for Bar {
///     fn max_size() -> usize {
///         <u32>::max_size() + <u8>::max_size() + <i16>::max_size()
///     }
///     unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
///         let bytes = self.a.poke_into(bytes);
///         let bytes = self.b.poke_into(bytes);
///         self.c.poke_into(bytes)
///     }
/// }
/// ```
///
/// # Safety
///
/// The `Poke` trait is an `unsafe` trait for the reasons, and implementors must
/// ensure that they adhere to these contracts:
///
/// * `max_size()` query and calculations in general must be correct.  Callers
///    of this trait are expected to rely on the contract defined on each
///    method, and implementors must ensure such contracts remain true.
pub unsafe trait Poke {
    /// Return the maximum number of bytes that the serialized version of `Self`
    /// will occupy.
    ///
    /// # Safety
    ///
    /// Implementors of `Poke` guarantee to not write more than the result of
    /// calling `max_size()` into the buffer pointed to by `bytes` when
    /// `poke_into()` is called.
    fn max_size() -> usize;
    /// Serialize into the buffer pointed to by `bytes`.
    ///
    /// Returns a pointer to the next byte after the serialized representation of `Self`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because undefined behavior can result if the
    /// caller does not ensure all of the following:
    ///
    /// * `bytes` must denote a valid pointer to a block of memory.
    ///
    /// * `bytes` must pointer to at least the number of bytes returned by
    ///   `max_size()`.
    unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8;
}

/// A trait for values that provide deserialization from buffers of bytes.
///
/// # Example
///
/// ```ignore
/// use peek_poke::Peek;
///
/// struct Bar {
///     a: u32,
///     b: u8,
///     c: i16,
/// }
///
/// ...
///
/// impl Peek for Bar {
///     unsafe fn peek_from(&mut self, bytes: *const u8) -> *const u8 {
///         let bytes = self.a.peek_from(bytes);
///         let bytes = self.b.peek_from(bytes);
///         self.c.peek_from(bytes)
///     }
/// }
/// ```
///
/// # Safety
///
/// The `Peek` trait contains unsafe methods for the following reasons, and
/// implementors must ensure that they adhere to these contracts:
///
/// * Callers of this trait are expected to rely on the contract defined on each
///   method, and implementors must ensure that `peek_from()` doesn't read more
///   bytes from `bytes` than is returned by `Peek::max_size()`.
pub trait Peek: Poke {
    /// Deserialize from the buffer pointed to by `bytes`.
    ///
    /// Returns a pointer to the next byte after the unconsumed bytes not used
    /// to deserialize the representation of `Self`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because undefined behavior can result if the
    /// caller does not ensure all of the following:
    ///
    /// * `bytes` must denote a valid pointer to a block of memory.
    ///
    /// * `bytes` must pointer to at least the number of bytes returned by
    ///   `Poke::max_size()`.
    unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8;
}

macro_rules! impl_poke_for_deref {
    (<$($desc:tt)+) => {
        unsafe impl <$($desc)+ {
            #[inline(always)]
            fn max_size() -> usize {
                <T>::max_size()
            }
            unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
                (**self).poke_into(bytes)
            }
        }
    }
}

impl_poke_for_deref!(<'a, T: Poke> Poke for &'a T);
impl_poke_for_deref!(<'a, T: Poke> Poke for &'a mut T);

macro_rules! impl_for_primitive {
    ($($ty:ty)+) => {
        $(unsafe impl Poke for $ty {
            #[inline(always)]
            fn max_size() -> usize {
                size_of::<Self>()
            }
            #[inline(always)]
            unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
                write_verbatim(*self, bytes)
            }
        }
        impl Peek for $ty {
            #[inline(always)]
            unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
                read_verbatim(bytes, output)
            }
        })+
    };
}

impl_for_primitive! {
    i8 i16 i32 i64 isize
    u8 u16 u32 u64 usize
    f32 f64
}

unsafe impl Poke for bool {
    #[inline(always)]
    fn max_size() -> usize {
        u8::max_size()
    }
    #[inline]
    unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
        (*self as u8).poke_into(bytes)
    }
}

impl Peek for bool {
    #[inline]
    unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
        let mut int_bool = 0u8;
        let ptr = <u8>::peek_from(bytes, &mut int_bool);
        *output = int_bool != 0;
        ptr
    }
}

unsafe impl<T> Poke for PhantomData<T> {
    #[inline(always)]
    fn max_size() -> usize {
        0
    }
    #[inline(always)]
    unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
        bytes
    }
}

impl<T> Peek for PhantomData<T> {
    #[inline(always)]
    unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
        *output = PhantomData;
        bytes
    }
}

unsafe impl<T: Poke> Poke for Option<T> {
    #[inline(always)]
    fn max_size() -> usize {
        u8::max_size() + T::max_size()
    }

    #[inline]
    unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
        match self {
            None => 0u8.poke_into(bytes),
            Some(ref v) => {
                let bytes = 1u8.poke_into(bytes);
                let bytes = v.poke_into(bytes);
                bytes
            }
        }
    }
}

impl<T: Default + Peek> Peek for Option<T> {
    #[inline]
    unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
        let (variant, bytes) = peek_from_default::<u8>(bytes);
        match variant {
            0 => {
                *output = None;
                bytes
            }
            1 => {
                let (val, bytes) = peek_from_default(bytes);
                *output = Some(val);
                bytes
            }
            _ => unreachable!(),
        }
    }
}

macro_rules! impl_for_arrays {
    ($($len:tt)+) => {
        $(unsafe impl<T: Poke> Poke for [T; $len] {
            fn max_size() -> usize {
                $len * T::max_size()
            }
            unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
                self.iter().fold(bytes, |bytes, e| e.poke_into(bytes))
            }
        }
        impl<T: Peek> Peek for [T; $len] {
            unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
                (&mut *output).iter_mut().fold(bytes, |bytes, e| <T>::peek_from(bytes, e))
            }
        })+
    }
}

impl_for_arrays! {
    01 02 03 04 05 06 07 08 09 10
    11 12 13 14 15 16 17 18 19 20
    21 22 23 24 25 26 27 28 29 30
    31 32
}

unsafe impl Poke for () {
    fn max_size() -> usize {
        0
    }
    unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
        bytes
    }
}
impl Peek for () {
    unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
        *output = ();
        bytes
    }
}

macro_rules! impl_for_tuple {
    ($($n:tt: $ty:ident),+) => {
        unsafe impl<$($ty: Poke),+> Poke for ($($ty,)+) {
            #[inline(always)]
            fn max_size() -> usize {
                0 $(+ <$ty>::max_size())+
            }
            unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
                $(let bytes = self.$n.poke_into(bytes);)+
                bytes
            }
        }
        impl<$($ty: Peek),+> Peek for ($($ty,)+) {
            unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
                $(let bytes = $ty::peek_from(bytes, &mut (*output).$n);)+
                bytes
            }
        }
    }
}

impl_for_tuple!(0: A);
impl_for_tuple!(0: A, 1: B);
impl_for_tuple!(0: A, 1: B, 2: C);
impl_for_tuple!(0: A, 1: B, 2: C, 3: D);
impl_for_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);
