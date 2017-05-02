/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module provides rust bindings for the XPCOM string types.
//!
//! # TL;DR (what types should I use)
//!
//! Use `&{mut,} nsA[C]String` for functions in rust which wish to take or
//! mutate XPCOM strings. The other string types `Deref` to this type.
//!
//! Use `ns[C]String<'a>` (`ns[C]String` in C++) for string struct members, and
//! as an intermediate between rust string data structures (such as `String`,
//! `Vec<u16>`, `&str`, and `&[u16]`) and `&{mut,} nsA[C]String` (using
//! `ns[C]String::from(value)`). These conversions, when possible, will not
//! perform any allocations. When using this type in structs shared with C++,
//! the correct lifetime argument is usually `'static`.
//!
//! Use `nsFixed[C]String` or `ns_auto_[c]string!` for dynamic stack allocated
//! strings which are expected to hold short string values.
//!
//! Use `*{const,mut} nsA[C]String` (`{const,} nsA[C]String*` in C++) for
//! function arguments passed across the rust/C++ language boundary.
//!
//! # String Types
//!
//! ## `nsA[C]String`
//!
//! The core types in this module are `nsAString` and `nsACString`. These types
//! are zero-sized as far as rust is concerned, and are safe to pass around
//! behind both references (in rust code), and pointers (in C++ code). They
//! represent a handle to a XPCOM string which holds either `u16` or `u8`
//! characters respectively. The backing character buffer is guaranteed to live
//! as long as the reference to the `nsAString` or `nsACString`.
//!
//! These types in rust are simply used as dummy types. References to them
//! represent a pointer to the beginning of a variable-sized `#[repr(C)]` struct
//! which is common between both C++ and Rust implementations. In C++, their
//! corresponding types are also named `nsAString` or `nsACString`, and they are
//! defined within the `nsTSubstring.{cpp,h}` file.
//!
//! ### Valid Operations
//!
//! An `&nsA[C]String` acts like rust's `&str`, in that it is a borrowed
//! reference to the backing data. When used as an argument to other functions
//! on `&mut nsA[C]String`, optimizations can be performed to avoid copying
//! buffers, as information about the backing storage is preserved.
//!
//! An `&mut nsA[C]String` acts like rust's `&mut Cow<str>`, in that it is a
//! mutable reference to a potentially borrowed string, which when modified will
//! ensure that it owns its own backing storage. This type can be appended to
//! with the methods `.append`, `.append_utf{8,16}`, and with the `write!`
//! macro, and can be assigned to with `.assign`.
//!
//! ## `ns[C]String<'a>`
//!
//! This type is an maybe-owned string type. It acts similarially to a
//! `Cow<[{u8,u16}]>`. This type provides `Deref` and `DerefMut` implementations
//! to `nsA[C]String`, which provides the methods for manipulating this type.
//! This type's lifetime parameter, `'a`, represents the lifetime of the backing
//! storage. When modified this type may re-allocate in order to ensure that it
//! does not mutate its backing storage.
//!
//! `ns[C]String`s can be constructed either with `ns[C]String::new()`, which
//! creates an empty `ns[C]String<'static>`, or through one of the provided
//! `From` implementations. Both string types may be constructed `From<&'a
//! str>`, with `nsCString` having a `'a` lifetime, as the storage is shared
//! with the `str`, while `nsString` has a `'static` lifetime, as its storage
//! has to be transcoded.
//!
//! When passing this type by reference, prefer passing a `&nsA[C]String` or
//! `&mut nsA[C]String`. to passing this type.
//!
//! When passing this type across the language boundary, pass it as `*const
//! nsA[C]String` for an immutable reference, or `*mut nsA[C]String` for a
//! mutable reference. This struct may also be included in `#[repr(C)]`
//! structs shared with C++.
//!
//! ## `nsFixed[C]String<'a>`
//!
//! This type is a string type with fixed backing storage. It is created with
//! `nsFixed[C]String::new(buffer)`, passing a mutable reference to a buffer as
//! the argument. This buffer will be used as backing storage whenever the
//! resulting string will fit within it, falling back to heap allocations only
//! when the string size exceeds that of the backing buffer.
//!
//! Like `ns[C]String`, this type dereferences to `nsA[C]String` which provides
//! the methods for manipulating the type, and is not `#[repr(C)]`.
//!
//! When passing this type by reference, prefer passing a `&nsA[C]String` or
//! `&mut nsA[C]String`. to passing this type.
//!
//! When passing this type across the language boundary, pass it as `*const
//! nsA[C]String` for an immutable reference, or `*mut nsA[C]String` for a
//! mutable reference. This struct may also be included in `#[repr(C)]`
//! structs shared with C++, although `nsFixed[C]String` objects are uncommon
//! as struct members.
//!
//! ## `ns_auto_[c]string!($name)`
//!
//! This is a helper macro which defines a fixed size, (currently 64 character),
//! backing array on the stack, and defines a local variable with name `$name`
//! which is a `nsFixed[C]String` using this buffer as its backing storage.
//!
//! Usage of this macro is similar to the C++ type `nsAuto[C]String`, but could
//! not be implemented as a basic type due to the differences between rust and
//! C++'s move semantics.
//!
//! ## `ns[C]StringRepr`
//!
//! This crate also provides the type `ns[C]StringRepr` which acts conceptually
//! similar to an `ns[C]String<'static>`, however, it does not have a `Drop`
//! implementation.
//!
//! If this type is dropped in rust, it will not free its backing storage. This
//! can be useful when implementing FFI types which contain `ns[C]String` members
//! which invoke their member's destructors through C++ code.

#![allow(non_camel_case_types)]
#![deny(warnings)]

use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;
use std::borrow;
use std::slice;
use std::ptr;
use std::mem;
use std::fmt;
use std::cmp;
use std::str;
use std::u32;
use std::os::raw::c_void;

//////////////////////////////////
// Internal Implemenation Flags //
//////////////////////////////////

const F_NONE: u32 = 0; // no flags

// data flags are in the lower 16-bits
const F_OWNED: u32 = 1 << 3; // mData points to a heap-allocated, raw buffer

// class flags are in the upper 16-bits
const F_CLASS_FIXED: u32 = 1 << 16; // indicates that |this| is of type nsTFixedString

////////////////////////////////////
// Generic String Bindings Macros //
////////////////////////////////////

macro_rules! define_string_types {
    {
        char_t = $char_t: ty;

        AString = $AString: ident;
        String = $String: ident;
        FixedString = $FixedString: ident;

        StringLike = $StringLike: ident;
        StringAdapter = $StringAdapter: ident;

        StringRepr = $StringRepr: ident;

        drop = $drop: ident;
        assign = $assign: ident, $fallible_assign: ident;
        append = $append: ident, $fallible_append: ident;
        set_length = $set_length: ident, $fallible_set_length: ident;
    } => {
        /// The representation of a ns[C]String type in C++. This type is
        /// used internally by our definition of ns[C]String to ensure layout
        /// compatibility with the C++ ns[C]String type.
        ///
        /// This type may also be used in place of a C++ ns[C]String inside of
        /// struct definitions which are shared with C++, as it has identical
        /// layout to our ns[C]String type.
        ///
        /// This struct will leak its data if dropped from rust. See the module
        /// documentation for more information on this type.
        #[repr(C)]
        #[derive(Debug)]
        pub struct $StringRepr {
            data: *const $char_t,
            length: u32,
            flags: u32,
        }

        impl Deref for $StringRepr {
            type Target = $AString;
            fn deref(&self) -> &$AString {
                unsafe {
                    mem::transmute(self)
                }
            }
        }

        impl DerefMut for $StringRepr {
            fn deref_mut(&mut self) -> &mut $AString {
                unsafe {
                    mem::transmute(self)
                }
            }
        }

        /// This type is the abstract type which is used for interacting with
        /// strings in rust. Each string type can derefence to an instance of
        /// this type, which provides the useful operations on strings.
        ///
        /// NOTE: Rust thinks this type has a size of 0, because the data
        /// associated with it is not necessarially safe to move. It is not safe
        /// to construct a nsAString yourself, unless it is received by
        /// dereferencing one of these types.
        ///
        /// NOTE: The `[u8; 0]` member is zero sized, and only exists to prevent
        /// the construction by code outside of this module. It is used instead
        /// of a private `()` member because the `improper_ctypes` lint complains
        /// about some ZST members in `extern "C"` function declarations.
        #[repr(C)]
        pub struct $AString {
            _prohibit_constructor: [u8; 0],
        }

        impl $AString {
            /// Assign the value of `other` into self, overwriting any value
            /// currently stored. Performs an optimized assignment when possible
            /// if `other` is a `nsA[C]String`.
            pub fn assign<T: $StringLike + ?Sized>(&mut self, other: &T) {
                unsafe { $assign(self, other.adapt().as_ptr()) };
            }

            /// Assign the value of `other` into self, overwriting any value
            /// currently stored. Performs an optimized assignment when possible
            /// if `other` is a `nsA[C]String`.
            ///
            /// Returns Ok(()) on success, and Err(()) if the allocation failed.
            pub fn fallible_assign<T: $StringLike + ?Sized>(&mut self, other: &T) -> Result<(), ()> {
                if unsafe { $fallible_assign(self, other.adapt().as_ptr()) } {
                    Ok(())
                } else {
                    Err(())
                }
            }

            /// Append the value of `other` into self.
            pub fn append<T: $StringLike + ?Sized>(&mut self, other: &T) {
                unsafe { $append(self, other.adapt().as_ptr()) };
            }

            /// Append the value of `other` into self.
            ///
            /// Returns Ok(()) on success, and Err(()) if the allocation failed.
            pub fn fallible_append<T: $StringLike + ?Sized>(&mut self, other: &T) -> Result<(), ()> {
                if unsafe { $fallible_append(self, other.adapt().as_ptr()) } {
                    Ok(())
                } else {
                    Err(())
                }
            }

            /// Set the length of the string to the passed-in length, and expand
            /// the backing capacity to match. This method is unsafe as it can
            /// expose uninitialized memory when len is greater than the current
            /// length of the string.
            pub unsafe fn set_length(&mut self, len: u32) {
                $set_length(self, len);
            }

            /// Set the length of the string to the passed-in length, and expand
            /// the backing capacity to match. This method is unsafe as it can
            /// expose uninitialized memory when len is greater than the current
            /// length of the string.
            ///
            /// Returns Ok(()) on success, and Err(()) if the allocation failed.
            pub unsafe fn fallible_set_length(&mut self, len: u32) -> Result<(), ()> {
                if $fallible_set_length(self, len) {
                    Ok(())
                } else {
                    Err(())
                }
            }

            pub fn truncate(&mut self) {
                unsafe {
                    self.set_length(0);
                }
            }
        }

        impl Deref for $AString {
            type Target = [$char_t];
            fn deref(&self) -> &[$char_t] {
                unsafe {
                    // All $AString values point to a struct prefix which is
                    // identical to $StringRepr, this we can transmute `self`
                    // into $StringRepr to get the reference to the underlying
                    // data.
                    let this: &$StringRepr = mem::transmute(self);
                    if this.data.is_null() {
                        debug_assert!(this.length == 0);
                        // Use an arbitrary non-null value as the pointer
                        slice::from_raw_parts(0x1 as *const $char_t, 0)
                    } else {
                        slice::from_raw_parts(this.data, this.length as usize)
                    }
                }
            }
        }

        impl AsRef<[$char_t]> for $AString {
            fn as_ref(&self) -> &[$char_t] {
                self
            }
        }

        impl cmp::PartialEq for $AString {
            fn eq(&self, other: &$AString) -> bool {
                &self[..] == &other[..]
            }
        }

        impl cmp::PartialEq<[$char_t]> for $AString {
            fn eq(&self, other: &[$char_t]) -> bool {
                &self[..] == other
            }
        }

        impl<'a> cmp::PartialEq<$String<'a>> for $AString {
            fn eq(&self, other: &$String<'a>) -> bool {
                self.eq(&**other)
            }
        }

        impl<'a> cmp::PartialEq<$FixedString<'a>> for $AString {
            fn eq(&self, other: &$FixedString<'a>) -> bool {
                self.eq(&**other)
            }
        }

        #[repr(C)]
        pub struct $String<'a> {
            hdr: $StringRepr,
            _marker: PhantomData<&'a [$char_t]>,
        }

        impl $String<'static> {
            pub fn new() -> $String<'static> {
                $String {
                    hdr: $StringRepr {
                        data: ptr::null(),
                        length: 0,
                        flags: F_NONE,
                    },
                    _marker: PhantomData,
                }
            }
        }

        impl<'a> Drop for $String<'a> {
            fn drop(&mut self) {
                unsafe {
                    $drop(&mut **self);
                }
            }
        }

        impl<'a> Deref for $String<'a> {
            type Target = $AString;
            fn deref(&self) -> &$AString {
                &self.hdr
            }
        }

        impl<'a> DerefMut for $String<'a> {
            fn deref_mut(&mut self) -> &mut $AString {
                &mut self.hdr
            }
        }

        impl<'a> AsRef<[$char_t]> for $String<'a> {
            fn as_ref(&self) -> &[$char_t] {
                &self
            }
        }

        impl<'a> From<&'a String> for $String<'a> {
            fn from(s: &'a String) -> $String<'a> {
                $String::from(&s[..])
            }
        }

        impl<'a> From<&'a Vec<$char_t>> for $String<'a> {
            fn from(s: &'a Vec<$char_t>) -> $String<'a> {
                $String::from(&s[..])
            }
        }

        impl<'a> From<&'a [$char_t]> for $String<'a> {
            fn from(s: &'a [$char_t]) -> $String<'a> {
                assert!(s.len() < (u32::MAX as usize));
                $String {
                    hdr: $StringRepr {
                        data: if s.is_empty() { ptr::null() } else { s.as_ptr() },
                        length: s.len() as u32,
                        flags: F_NONE,
                    },
                    _marker: PhantomData,
                }
            }
        }

        impl From<Box<[$char_t]>> for $String<'static> {
            fn from(s: Box<[$char_t]>) -> $String<'static> {
                assert!(s.len() < (u32::MAX as usize));
                if s.is_empty() {
                    return $String::new();
                }

                // SAFETY NOTE: This method produces an F_OWNED ns[C]String from
                // a Box<[$char_t]>. this is only safe because in the Gecko
                // tree, we use the same allocator for Rust code as for C++
                // code, meaning that our box can be legally freed with
                // libc::free().
                let length = s.len() as u32;
                let ptr = s.as_ptr();
                mem::forget(s);
                unsafe {
                    Gecko_IncrementStringAdoptCount(ptr as *mut _);
                }
                $String {
                    hdr: $StringRepr {
                        data: ptr,
                        length: length,
                        flags: F_OWNED,
                    },
                    _marker: PhantomData,
                }
            }
        }

        impl From<Vec<$char_t>> for $String<'static> {
            fn from(s: Vec<$char_t>) -> $String<'static> {
                s.into_boxed_slice().into()
            }
        }

        impl<'a> From<&'a $AString> for $String<'static> {
            fn from(s: &'a $AString) -> $String<'static> {
                let mut string = $String::new();
                string.assign(s);
                string
            }
        }

        impl<'a> fmt::Write for $String<'a> {
            fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
                $AString::write_str(self, s)
            }
        }

        impl<'a> fmt::Display for $String<'a> {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                <$AString as fmt::Display>::fmt(self, f)
            }
        }

        impl<'a> fmt::Debug for $String<'a> {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                <$AString as fmt::Debug>::fmt(self, f)
            }
        }

        impl<'a> cmp::PartialEq for $String<'a> {
            fn eq(&self, other: &$String<'a>) -> bool {
                $AString::eq(self, other)
            }
        }

        impl<'a> cmp::PartialEq<[$char_t]> for $String<'a> {
            fn eq(&self, other: &[$char_t]) -> bool {
                $AString::eq(self, other)
            }
        }

        impl<'a, 'b> cmp::PartialEq<&'b [$char_t]> for $String<'a> {
            fn eq(&self, other: &&'b [$char_t]) -> bool {
                $AString::eq(self, *other)
            }
        }

        impl<'a> cmp::PartialEq<str> for $String<'a> {
            fn eq(&self, other: &str) -> bool {
                $AString::eq(self, other)
            }
        }

        impl<'a, 'b> cmp::PartialEq<&'b str> for $String<'a> {
            fn eq(&self, other: &&'b str) -> bool {
                $AString::eq(self, *other)
            }
        }

        /// A nsFixed[C]String is a string which uses a fixed size mutable
        /// backing buffer for storing strings which will fit within that
        /// buffer, rather than using heap allocations.
        #[repr(C)]
        pub struct $FixedString<'a> {
            base: $String<'a>,
            capacity: u32,
            buffer: *mut $char_t,
            _marker: PhantomData<&'a mut [$char_t]>,
        }

        impl<'a> $FixedString<'a> {
            pub fn new(buf: &'a mut [$char_t]) -> $FixedString<'a> {
                let len = buf.len();
                assert!(len < (u32::MAX as usize));
                let buf_ptr = buf.as_mut_ptr();
                $FixedString {
                    base: $String {
                        hdr: $StringRepr {
                            data: ptr::null(),
                            length: 0,
                            flags: F_CLASS_FIXED,
                        },
                        _marker: PhantomData,
                    },
                    capacity: len as u32,
                    buffer: buf_ptr,
                    _marker: PhantomData,
                }
            }
        }

        impl<'a> Deref for $FixedString<'a> {
            type Target = $AString;
            fn deref(&self) -> &$AString {
                &self.base
            }
        }

        impl<'a> DerefMut for $FixedString<'a> {
            fn deref_mut(&mut self) -> &mut $AString {
                &mut self.base
            }
        }

        impl<'a> AsRef<[$char_t]> for $FixedString<'a> {
            fn as_ref(&self) -> &[$char_t] {
                &self
            }
        }

        impl<'a> fmt::Write for $FixedString<'a> {
            fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
                $AString::write_str(self, s)
            }
        }

        impl<'a> fmt::Display for $FixedString<'a> {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                <$AString as fmt::Display>::fmt(self, f)
            }
        }

        impl<'a> fmt::Debug for $FixedString<'a> {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                <$AString as fmt::Debug>::fmt(self, f)
            }
        }

        impl<'a> cmp::PartialEq for $FixedString<'a> {
            fn eq(&self, other: &$FixedString<'a>) -> bool {
                $AString::eq(self, other)
            }
        }

        impl<'a> cmp::PartialEq<[$char_t]> for $FixedString<'a> {
            fn eq(&self, other: &[$char_t]) -> bool {
                $AString::eq(self, other)
            }
        }

        impl<'a, 'b> cmp::PartialEq<&'b [$char_t]> for $FixedString<'a> {
            fn eq(&self, other: &&'b [$char_t]) -> bool {
                $AString::eq(self, *other)
            }
        }

        impl<'a> cmp::PartialEq<str> for $FixedString<'a> {
            fn eq(&self, other: &str) -> bool {
                $AString::eq(self, other)
            }
        }

        impl<'a, 'b> cmp::PartialEq<&'b str> for $FixedString<'a> {
            fn eq(&self, other: &&'b str) -> bool {
                $AString::eq(self, *other)
            }
        }

        /// An adapter type to allow for passing both types which coerce to
        /// &[$char_type], and &$AString to a function, while still performing
        /// optimized operations when passed the $AString.
        pub enum $StringAdapter<'a> {
            Borrowed($String<'a>),
            Abstract(&'a $AString),
        }

        impl<'a> $StringAdapter<'a> {
            fn as_ptr(&self) -> *const $AString {
                &**self
            }
        }

        impl<'a> Deref for $StringAdapter<'a> {
            type Target = $AString;

            fn deref(&self) -> &$AString {
                match *self {
                    $StringAdapter::Borrowed(ref s) => s,
                    $StringAdapter::Abstract(ref s) => s,
                }
            }
        }

        /// This trait is implemented on types which are `ns[C]String`-like, in
        /// that they can at very low cost be converted to a borrowed
        /// `&nsA[C]String`. Unfortunately, the intermediate type
        /// `ns[C]StringAdapter` is required as well due to types like `&[u8]`
        /// needing to be (cheaply) wrapped in a `nsCString` on the stack to
        /// create the `&nsACString`.
        ///
        /// This trait is used to DWIM when calling the methods on
        /// `nsA[C]String`.
        pub trait $StringLike {
            fn adapt(&self) -> $StringAdapter;
        }

        impl<'a, T: $StringLike + ?Sized> $StringLike for &'a T {
            fn adapt(&self) -> $StringAdapter {
                <T as $StringLike>::adapt(*self)
            }
        }

        impl<'a, T> $StringLike for borrow::Cow<'a, T>
            where T: $StringLike + borrow::ToOwned + ?Sized {
            fn adapt(&self) -> $StringAdapter {
                <T as $StringLike>::adapt(self.as_ref())
            }
        }

        impl $StringLike for $AString {
            fn adapt(&self) -> $StringAdapter {
                $StringAdapter::Abstract(self)
            }
        }

        impl<'a> $StringLike for $String<'a> {
            fn adapt(&self) -> $StringAdapter {
                $StringAdapter::Abstract(self)
            }
        }

        impl<'a> $StringLike for $FixedString<'a> {
            fn adapt(&self) -> $StringAdapter {
                $StringAdapter::Abstract(self)
            }
        }

        impl $StringLike for [$char_t] {
            fn adapt(&self) -> $StringAdapter {
                $StringAdapter::Borrowed($String::from(self))
            }
        }

        impl $StringLike for Vec<$char_t> {
            fn adapt(&self) -> $StringAdapter {
                $StringAdapter::Borrowed($String::from(&self[..]))
            }
        }

        impl $StringLike for Box<[$char_t]> {
            fn adapt(&self) -> $StringAdapter {
                $StringAdapter::Borrowed($String::from(&self[..]))
            }
        }
    }
}

///////////////////////////////////////////
// Bindings for nsCString (u8 char type) //
///////////////////////////////////////////

define_string_types! {
    char_t = u8;

    AString = nsACString;
    String = nsCString;
    FixedString = nsFixedCString;

    StringLike = nsCStringLike;
    StringAdapter = nsCStringAdapter;

    StringRepr = nsCStringRepr;

    drop = Gecko_FinalizeCString;
    assign = Gecko_AssignCString, Gecko_FallibleAssignCString;
    append = Gecko_AppendCString, Gecko_FallibleAppendCString;
    set_length = Gecko_SetLengthCString, Gecko_FallibleSetLengthCString;
}

impl nsACString {
    pub fn assign_utf16<T: nsStringLike + ?Sized>(&mut self, other: &T) {
        self.truncate();
        self.append_utf16(other);
    }

    pub fn fallible_assign_utf16<T: nsStringLike + ?Sized>(&mut self, other: &T) -> Result<(), ()> {
        self.truncate();
        self.fallible_append_utf16(other)
    }

    pub fn append_utf16<T: nsStringLike + ?Sized>(&mut self, other: &T) {
        unsafe {
            Gecko_AppendUTF16toCString(self, other.adapt().as_ptr());
        }
    }

    pub fn fallible_append_utf16<T: nsStringLike + ?Sized>(&mut self, other: &T) -> Result<(), ()> {
        if unsafe { Gecko_FallibleAppendUTF16toCString(self, other.adapt().as_ptr()) } {
            Ok(())
        } else {
            Err(())
        }
    }

    pub unsafe fn as_str_unchecked(&self) -> &str {
        str::from_utf8_unchecked(self)
    }
}

impl<'a> From<&'a str> for nsCString<'a> {
    fn from(s: &'a str) -> nsCString<'a> {
        s.as_bytes().into()
    }
}

impl From<Box<str>> for nsCString<'static> {
    fn from(s: Box<str>) -> nsCString<'static> {
        s.into_string().into()
    }
}

impl From<String> for nsCString<'static> {
    fn from(s: String) -> nsCString<'static> {
        s.into_bytes().into()
    }
}

// Support for the write!() macro for appending to nsACStrings
impl fmt::Write for nsACString {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.append(&nsCString::from(s));
        Ok(())
    }
}

impl fmt::Display for nsACString {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&String::from_utf8_lossy(&self[..]), f)
    }
}

impl fmt::Debug for nsACString {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&String::from_utf8_lossy(&self[..]), f)
    }
}

impl cmp::PartialEq<str> for nsACString {
    fn eq(&self, other: &str) -> bool {
        &self[..] == other.as_bytes()
    }
}

impl nsCStringLike for str {
    fn adapt(&self) -> nsCStringAdapter {
        nsCStringAdapter::Borrowed(nsCString::from(self))
    }
}

impl nsCStringLike for String {
    fn adapt(&self) -> nsCStringAdapter {
        nsCStringAdapter::Borrowed(nsCString::from(&self[..]))
    }
}

impl nsCStringLike for Box<str> {
    fn adapt(&self) -> nsCStringAdapter {
        nsCStringAdapter::Borrowed(nsCString::from(&self[..]))
    }
}

#[macro_export]
macro_rules! ns_auto_cstring {
    ($name:ident) => {
        let mut buf: [u8; 64] = [0; 64];
        let mut $name = $crate::nsFixedCString::new(&mut buf);
    }
}

///////////////////////////////////////////
// Bindings for nsString (u16 char type) //
///////////////////////////////////////////

define_string_types! {
    char_t = u16;

    AString = nsAString;
    String = nsString;
    FixedString = nsFixedString;

    StringLike = nsStringLike;
    StringAdapter = nsStringAdapter;

    StringRepr = nsStringRepr;

    drop = Gecko_FinalizeString;
    assign = Gecko_AssignString, Gecko_FallibleAssignString;
    append = Gecko_AppendString, Gecko_FallibleAppendString;
    set_length = Gecko_SetLengthString, Gecko_FallibleSetLengthString;
}

impl nsAString {
    pub fn assign_utf8<T: nsCStringLike + ?Sized>(&mut self, other: &T) {
        self.truncate();
        self.append_utf8(other);
    }

    pub fn fallible_assign_utf8<T: nsCStringLike + ?Sized>(&mut self, other: &T) -> Result<(), ()> {
        self.truncate();
        self.fallible_append_utf8(other)
    }

    pub fn append_utf8<T: nsCStringLike + ?Sized>(&mut self, other: &T) {
        unsafe {
            Gecko_AppendUTF8toString(self, other.adapt().as_ptr());
        }
    }

    pub fn fallible_append_utf8<T: nsCStringLike + ?Sized>(&mut self, other: &T) -> Result<(), ()> {
        if unsafe { Gecko_FallibleAppendUTF8toString(self, other.adapt().as_ptr()) } {
            Ok(())
        } else {
            Err(())
        }
    }
}

// NOTE: The From impl for a string slice for nsString produces a <'static>
// lifetime, as it allocates.
impl<'a> From<&'a str> for nsString<'static> {
    fn from(s: &'a str) -> nsString<'static> {
        s.encode_utf16().collect::<Vec<u16>>().into()
    }
}

// Support for the write!() macro for writing to nsStrings
impl fmt::Write for nsAString {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        // Directly invoke gecko's routines for appending utf8 strings to
        // nsAString values, to avoid as much overhead as possible
        self.append_utf8(&nsCString::from(s));
        Ok(())
    }
}

impl fmt::Display for nsAString {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&String::from_utf16_lossy(&self[..]), f)
    }
}

impl fmt::Debug for nsAString {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&String::from_utf16_lossy(&self[..]), f)
    }
}

impl cmp::PartialEq<str> for nsAString {
    fn eq(&self, other: &str) -> bool {
        other.encode_utf16().eq(self.iter().cloned())
    }
}

#[macro_export]
macro_rules! ns_auto_string {
    ($name:ident) => {
        let mut buf: [u16; 64] = [0; 64];
        let mut $name = $crate::nsFixedString::new(&mut buf);
    }
}

#[cfg(not(feature = "gecko_debug"))]
#[allow(non_snake_case)]
unsafe fn Gecko_IncrementStringAdoptCount(_: *mut c_void) {}

extern "C" {
    #[cfg(feature = "gecko_debug")]
    fn Gecko_IncrementStringAdoptCount(data: *mut c_void);

    // Gecko implementation in nsSubstring.cpp
    fn Gecko_FinalizeCString(this: *mut nsACString);

    fn Gecko_AssignCString(this: *mut nsACString, other: *const nsACString);
    fn Gecko_AppendCString(this: *mut nsACString, other: *const nsACString);
    fn Gecko_SetLengthCString(this: *mut nsACString, length: u32);
    fn Gecko_FallibleAssignCString(this: *mut nsACString, other: *const nsACString) -> bool;
    fn Gecko_FallibleAppendCString(this: *mut nsACString, other: *const nsACString) -> bool;
    fn Gecko_FallibleSetLengthCString(this: *mut nsACString, length: u32) -> bool;

    fn Gecko_FinalizeString(this: *mut nsAString);

    fn Gecko_AssignString(this: *mut nsAString, other: *const nsAString);
    fn Gecko_AppendString(this: *mut nsAString, other: *const nsAString);
    fn Gecko_SetLengthString(this: *mut nsAString, length: u32);
    fn Gecko_FallibleAssignString(this: *mut nsAString, other: *const nsAString) -> bool;
    fn Gecko_FallibleAppendString(this: *mut nsAString, other: *const nsAString) -> bool;
    fn Gecko_FallibleSetLengthString(this: *mut nsAString, length: u32) -> bool;

    // Gecko implementation in nsReadableUtils.cpp
    fn Gecko_AppendUTF16toCString(this: *mut nsACString, other: *const nsAString);
    fn Gecko_AppendUTF8toString(this: *mut nsAString, other: *const nsACString);
    fn Gecko_FallibleAppendUTF16toCString(this: *mut nsACString, other: *const nsAString) -> bool;
    fn Gecko_FallibleAppendUTF8toString(this: *mut nsAString, other: *const nsACString) -> bool;
}
