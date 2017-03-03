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

        StringRepr = $StringRepr: ident;
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
                        data: if s.is_empty() { ptr::null() } else { s.as_ptr()},
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

    StringRepr = nsCStringRepr;
}

impl nsACString {
    pub fn assign<T: AsRef<[u8]> + ?Sized>(&mut self, other: &T) {
        let s = nsCString::from(other.as_ref());
        unsafe {
            Gecko_AssignCString(self, &*s);
        }
    }

    pub fn assign_utf16<T: AsRef<[u16]> + ?Sized>(&mut self, other: &T) {
        self.assign(&[]);
        self.append_utf16(other);
    }

    pub fn append<T: AsRef<[u8]> + ?Sized>(&mut self, other: &T) {
        let s = nsCString::from(other.as_ref());
        unsafe {
            Gecko_AppendCString(self, &*s);
        }
    }

    pub fn append_utf16<T: AsRef<[u16]> + ?Sized>(&mut self, other: &T) {
        let s = nsString::from(other.as_ref());
        unsafe {
            Gecko_AppendUTF16toCString(self, &*s);
        }
    }

    pub unsafe fn as_str_unchecked(&self) -> &str {
        str::from_utf8_unchecked(self)
    }

    pub fn truncate(&mut self) {
        unsafe {
            Gecko_TruncateCString(self);
        }
    }
}

impl<'a> Drop for nsCString<'a> {
    fn drop(&mut self) {
        unsafe {
            Gecko_FinalizeCString(&mut **self);
        }
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

    StringRepr = nsStringRepr;
}

impl nsAString {
    pub fn assign<T: AsRef<[u16]> + ?Sized>(&mut self, other: &T) {
        let s = nsString::from(other.as_ref());
        unsafe {
            Gecko_AssignString(self, &*s);
        }
    }

    pub fn assign_utf8<T: AsRef<[u8]> + ?Sized>(&mut self, other: &T) {
        self.assign(&[]);
        self.append_utf8(other);
    }

    pub fn append<T: AsRef<[u16]> + ?Sized>(&mut self, other: &T) {
        let s = nsString::from(other.as_ref());
        unsafe {
            Gecko_AppendString(self, &*s);
        }
    }

    pub fn append_utf8<T: AsRef<[u8]> + ?Sized>(&mut self, other: &T) {
        let s = nsCString::from(other.as_ref());
        unsafe {
            Gecko_AppendUTF8toString(self, &*s);
        }
    }

    pub fn truncate(&mut self) {
        unsafe {
            Gecko_TruncateString(self);
        }
    }
}

impl<'a> Drop for nsString<'a> {
    fn drop(&mut self) {
        unsafe {
            Gecko_FinalizeString(&mut **self);
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

#[cfg(not(debug_assertions))]
#[allow(non_snake_case)]
unsafe fn Gecko_IncrementStringAdoptCount(_: *mut c_void) {}

// NOTE: These bindings currently only expose infallible operations. Perhaps
// consider allowing for fallible methods?
extern "C" {
    #[cfg(debug_assertions)]
    fn Gecko_IncrementStringAdoptCount(data: *mut c_void);

    // Gecko implementation in nsSubstring.cpp
    fn Gecko_FinalizeCString(this: *mut nsACString);
    fn Gecko_AssignCString(this: *mut nsACString, other: *const nsACString);
    fn Gecko_AppendCString(this: *mut nsACString, other: *const nsACString);
    fn Gecko_TruncateCString(this: *mut nsACString);

    fn Gecko_FinalizeString(this: *mut nsAString);
    fn Gecko_AssignString(this: *mut nsAString, other: *const nsAString);
    fn Gecko_AppendString(this: *mut nsAString, other: *const nsAString);
    fn Gecko_TruncateString(this: *mut nsAString);

    // Gecko implementation in nsReadableUtils.cpp
    fn Gecko_AppendUTF16toCString(this: *mut nsACString, other: *const nsAString);
    fn Gecko_AppendUTF8toString(this: *mut nsAString, other: *const nsACString);
}
