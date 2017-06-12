/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! A drop-in replacement for string_cache, but backed by Gecko `nsIAtom`s.

use gecko_bindings::bindings::Gecko_AddRefAtom;
use gecko_bindings::bindings::Gecko_Atomize;
use gecko_bindings::bindings::Gecko_Atomize16;
use gecko_bindings::bindings::Gecko_ReleaseAtom;
use gecko_bindings::structs::nsIAtom;
use nsstring::{nsAString, nsString};
use precomputed_hash::PrecomputedHash;
use std::ascii::AsciiExt;
use std::borrow::{Cow, Borrow};
use std::char::{self, DecodeUtf16};
use std::fmt::{self, Write};
use std::hash::{Hash, Hasher};
use std::iter::Cloned;
use std::mem;
use std::ops::Deref;
use std::slice;

#[macro_use]
#[allow(improper_ctypes, non_camel_case_types, missing_docs)]
pub mod atom_macro {
    include!(concat!(env!("OUT_DIR"), "/gecko/atom_macro.rs"));
}

#[macro_use]
pub mod namespace;

pub use self::namespace::{Namespace, WeakNamespace};

macro_rules! local_name {
    ($s: tt) => { atom!($s) }
}

/// A strong reference to a Gecko atom.
#[derive(PartialEq, Eq)]
pub struct Atom(*mut WeakAtom);

/// An atom *without* a strong reference.
///
/// Only usable as `&'a WeakAtom`,
/// where `'a` is the lifetime of something that holds a strong reference to that atom.
pub struct WeakAtom(nsIAtom);

/// A BorrowedAtom for Gecko is just a weak reference to a `nsIAtom`, that
/// hasn't been bumped.
pub type BorrowedAtom<'a> = &'a WeakAtom;

impl Deref for Atom {
    type Target = WeakAtom;

    #[inline]
    fn deref(&self) -> &WeakAtom {
        unsafe {
            &*self.0
        }
    }
}

impl PrecomputedHash for Atom {
    #[inline]
    fn precomputed_hash(&self) -> u32 {
        self.get_hash()
    }
}

impl Borrow<WeakAtom> for Atom {
    #[inline]
    fn borrow(&self) -> &WeakAtom {
        self
    }
}

impl Eq for WeakAtom {}
impl PartialEq for WeakAtom {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let weak: *const WeakAtom = self;
        let other: *const WeakAtom = other;
        weak == other
    }
}

unsafe impl Send for Atom {}
unsafe impl Sync for Atom {}
unsafe impl Sync for WeakAtom {}

impl WeakAtom {
    /// Construct a `WeakAtom` from a raw `nsIAtom`.
    #[inline]
    pub unsafe fn new<'a>(atom: *mut nsIAtom) -> &'a mut Self {
        &mut *(atom as *mut WeakAtom)
    }

    /// Clone this atom, bumping the refcount if the atom is not static.
    #[inline]
    pub fn clone(&self) -> Atom {
        Atom::from(self.as_ptr())
    }

    /// Get the atom hash.
    #[inline]
    pub fn get_hash(&self) -> u32 {
        self.0.mHash
    }

    /// Get the atom as a slice of utf-16 chars.
    #[inline]
    pub fn as_slice(&self) -> &[u16] {
        unsafe {
            slice::from_raw_parts((*self.as_ptr()).mString, self.len() as usize)
        }
    }

    // NOTE: don't expose this, since it's slow, and easy to be misused.
    fn chars(&self) -> DecodeUtf16<Cloned<slice::Iter<u16>>> {
        char::decode_utf16(self.as_slice().iter().cloned())
    }

    /// Execute `cb` with the string that this atom represents.
    ///
    /// Find alternatives to this function when possible, please, since it's
    /// pretty slow.
    pub fn with_str<F, Output>(&self, cb: F) -> Output
        where F: FnOnce(&str) -> Output
    {
        // FIXME(bholley): We should measure whether it makes more sense to
        // cache the UTF-8 version in the Gecko atom table somehow.
        let owned = self.to_string();
        cb(&owned)
    }

    /// Convert this Atom into a string, decoding the UTF-16 bytes.
    ///
    /// Find alternatives to this function when possible, please, since it's
    /// pretty slow.
    #[inline]
    pub fn to_string(&self) -> String {
        String::from_utf16(self.as_slice()).unwrap()
    }

    /// Returns whether this atom is static.
    #[inline]
    pub fn is_static(&self) -> bool {
        unsafe {
            (*self.as_ptr()).mIsStatic() != 0
        }
    }

    /// Returns the length of the atom string.
    #[inline]
    pub fn len(&self) -> u32 {
        // FIXME(emilio): re-introduce bitfield accessors:
        //
        // https://github.com/servo/rust-bindgen/issues/519
        unsafe {
            (*self.as_ptr())._bitfield_1 & 0x7FFFFFFF
        }
    }

    /// Returns whether this atom is the empty string.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the atom as a mutable pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut nsIAtom {
        let const_ptr: *const nsIAtom = &self.0;
        const_ptr as *mut nsIAtom
    }

    /// Convert this atom to ASCII lower-case
    pub fn to_ascii_lowercase(&self) -> Atom {
        let slice = self.as_slice();
        match slice.iter().position(|&char16| (b'A' as u16) <= char16 && char16 <= (b'Z' as u16)) {
            None => self.clone(),
            Some(i) => {
                let mut buffer: [u16; 64] = unsafe { mem::uninitialized() };
                let mut vec;
                let mutable_slice = if let Some(buffer_prefix) = buffer.get_mut(..slice.len()) {
                    buffer_prefix.copy_from_slice(slice);
                    buffer_prefix
                } else {
                    vec = slice.to_vec();
                    &mut vec
                };
                for char16 in &mut mutable_slice[i..] {
                    if *char16 <= 0x7F {
                        *char16 = (*char16 as u8).to_ascii_lowercase() as u16
                    }
                }
                Atom::from(&*mutable_slice)
            }
        }
    }

    /// Return whether two atoms are ASCII-case-insensitive matches
    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        if self == other {
            return true;
        }

        let a = self.as_slice();
        let b = other.as_slice();
        a.len() == b.len() && a.iter().zip(b).all(|(&a16, &b16)| {
            if a16 <= 0x7F && b16 <= 0x7F {
                (a16 as u8).eq_ignore_ascii_case(&(b16 as u8))
            } else {
                a16 == b16
            }
        })
    }

    /// Return whether this atom is an ASCII-case-insensitive match for the given string
    pub fn eq_str_ignore_ascii_case(&self, other: &str) -> bool {
        self.chars().map(|r| r.map(|c: char| c.to_ascii_lowercase()))
        .eq(other.chars().map(|c: char| Ok(c.to_ascii_lowercase())))
    }
}

impl fmt::Debug for WeakAtom {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "Gecko WeakAtom({:p}, {})", self, self)
    }
}

impl fmt::Display for WeakAtom {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        for c in self.chars() {
            try!(w.write_char(c.unwrap_or(char::REPLACEMENT_CHARACTER)))
        }
        Ok(())
    }
}

impl Atom {
    /// Execute a callback with the atom represented by `ptr`.
    pub unsafe fn with<F, R>(ptr: *mut nsIAtom, callback: F) -> R where F: FnOnce(&Atom) -> R {
        let atom = Atom(WeakAtom::new(ptr));
        let ret = callback(&atom);
        mem::forget(atom);
        ret
    }

    /// Creates an atom from an static atom pointer without checking in release
    /// builds.
    ///
    /// Right now it's only used by the atom macro, and ideally it should keep
    /// that way, now we have sugar for is_static, creating atoms using
    /// Atom::from should involve almost no overhead.
    #[inline]
    unsafe fn from_static(ptr: *mut nsIAtom) -> Self {
        let atom = Atom(ptr as *mut WeakAtom);
        debug_assert!(atom.is_static(),
                      "Called from_static for a non-static atom!");
        atom
    }

    /// Creates an atom from a dynamic atom pointer that has already had AddRef
    /// called on it.
    #[inline]
    pub unsafe fn from_addrefed(ptr: *mut nsIAtom) -> Self {
        debug_assert!(!ptr.is_null());
        unsafe {
            Atom(WeakAtom::new(ptr))
        }
    }

    /// Convert this atom into an addrefed nsIAtom pointer.
    #[inline]
    pub fn into_addrefed(self) -> *mut nsIAtom {
        let ptr = self.as_ptr();
        mem::forget(self);
        ptr
    }
}

impl Hash for Atom {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        state.write_u32(self.get_hash());
    }
}

impl Hash for WeakAtom {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        state.write_u32(self.get_hash());
    }
}

impl Clone for Atom {
    #[inline(always)]
    fn clone(&self) -> Atom {
        Atom::from(self.as_ptr())
    }
}

impl Drop for Atom {
    #[inline]
    fn drop(&mut self) {
        if !self.is_static() {
            unsafe {
                Gecko_ReleaseAtom(self.as_ptr());
            }
        }
    }
}

impl Default for Atom {
    #[inline]
    fn default() -> Self {
        atom!("")
    }
}

impl fmt::Debug for Atom {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "Gecko Atom({:p}, {})", self.0, self)
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            (&*self.0).fmt(w)
        }
    }
}

impl<'a> From<&'a str> for Atom {
    #[inline]
    fn from(string: &str) -> Atom {
        debug_assert!(string.len() <= u32::max_value() as usize);
        unsafe {
            Atom(WeakAtom::new(
                Gecko_Atomize(string.as_ptr() as *const _, string.len() as u32)
            ))
        }
    }
}

impl<'a> From<&'a [u16]> for Atom {
    #[inline]
    fn from(slice: &[u16]) -> Atom {
        Atom::from(&*nsString::from(slice))
    }
}

impl<'a> From<&'a nsAString> for Atom {
    #[inline]
    fn from(string: &nsAString) -> Atom {
        unsafe {
            Atom(WeakAtom::new(
                Gecko_Atomize16(string)
            ))
        }
    }
}

impl<'a> From<Cow<'a, str>> for Atom {
    #[inline]
    fn from(string: Cow<'a, str>) -> Atom {
        Atom::from(&*string)
    }
}

impl From<String> for Atom {
    #[inline]
    fn from(string: String) -> Atom {
        Atom::from(&*string)
    }
}

impl From<*mut nsIAtom> for Atom {
    #[inline]
    fn from(ptr: *mut nsIAtom) -> Atom {
        debug_assert!(!ptr.is_null());
        unsafe {
            let ret = Atom(WeakAtom::new(ptr));
            if !ret.is_static() {
                Gecko_AddRefAtom(ptr);
            }
            ret
        }
    }
}
