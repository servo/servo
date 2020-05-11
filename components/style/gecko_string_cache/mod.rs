/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]
// This is needed for the constants in atom_macro.rs, because we have some
// atoms whose names differ only by case, e.g. datetime and dateTime.
#![allow(non_upper_case_globals)]

//! A drop-in replacement for string_cache, but backed by Gecko `nsAtom`s.

use crate::gecko_bindings::bindings::Gecko_AddRefAtom;
use crate::gecko_bindings::bindings::Gecko_Atomize;
use crate::gecko_bindings::bindings::Gecko_Atomize16;
use crate::gecko_bindings::bindings::Gecko_ReleaseAtom;
use crate::gecko_bindings::structs::root::mozilla::detail::gGkAtoms;
use crate::gecko_bindings::structs::root::mozilla::detail::kGkAtomsArrayOffset;
use crate::gecko_bindings::structs::root::mozilla::detail::GkAtoms_Atoms_AtomsCount;
use crate::gecko_bindings::structs::{nsAtom, nsDynamicAtom, nsStaticAtom};
use nsstring::{nsAString, nsStr};
use precomputed_hash::PrecomputedHash;
use std::borrow::{Borrow, Cow};
use std::char::{self, DecodeUtf16};
use std::fmt::{self, Write};
use std::hash::{Hash, Hasher};
use std::iter::Cloned;
use std::mem::{self, ManuallyDrop};
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::{slice, str};
use style_traits::SpecifiedValueInfo;
use to_shmem::{self, SharedMemoryBuilder, ToShmem};

#[macro_use]
#[allow(improper_ctypes, non_camel_case_types, missing_docs)]
pub mod atom_macro {
    include!(concat!(env!("OUT_DIR"), "/gecko/atom_macro.rs"));
}

#[macro_use]
pub mod namespace;

pub use self::namespace::{Namespace, WeakNamespace};

macro_rules! local_name {
    ($s:tt) => {
        atom!($s)
    };
}

/// A handle to a Gecko atom. This is a type that can represent either:
///
///  * A strong reference to a dynamic atom (an `nsAtom` pointer), in which case
///    the `usize` just holds the pointer value.
///
///  * A byte offset from `gGkAtoms` to the `nsStaticAtom` object (shifted to
///    the left one bit, and with the lower bit set to `1` to differentiate it
///    from the above), so `(offset << 1 | 1)`.
///
#[derive(Eq, PartialEq)]
#[repr(C)]
pub struct Atom(NonZeroUsize);

/// An atom *without* a strong reference.
///
/// Only usable as `&'a WeakAtom`,
/// where `'a` is the lifetime of something that holds a strong reference to that atom.
pub struct WeakAtom(nsAtom);

/// The number of static atoms we have.
const STATIC_ATOM_COUNT: usize = GkAtoms_Atoms_AtomsCount as usize;

/// Returns the Gecko static atom array.
///
/// We have this rather than use rust-bindgen to generate
/// mozilla::detail::gGkAtoms and then just reference gGkAtoms.mAtoms, so we
/// avoid a problem with lld-link.exe on Windows.
///
/// https://bugzilla.mozilla.org/show_bug.cgi?id=1517685
#[inline]
fn static_atoms() -> &'static [nsStaticAtom; STATIC_ATOM_COUNT] {
    unsafe {
        let addr = &gGkAtoms as *const _ as usize + kGkAtomsArrayOffset as usize;
        &*(addr as *const _)
    }
}

/// Returns whether the specified address points to one of the nsStaticAtom
/// objects in the Gecko static atom array.
#[inline]
fn valid_static_atom_addr(addr: usize) -> bool {
    unsafe {
        let atoms = static_atoms();
        let start = atoms.as_ptr();
        let end = atoms.get_unchecked(STATIC_ATOM_COUNT) as *const _;
        let in_range = addr >= start as usize && addr < end as usize;
        let aligned = addr % mem::align_of::<nsStaticAtom>() == 0;
        in_range && aligned
    }
}

impl Deref for Atom {
    type Target = WeakAtom;

    #[inline]
    fn deref(&self) -> &WeakAtom {
        unsafe {
            let addr = if self.is_static() {
                (&gGkAtoms as *const _ as usize) + (self.0.get() >> 1)
            } else {
                self.0.get()
            };
            debug_assert!(!self.is_static() || valid_static_atom_addr(addr));
            WeakAtom::new(addr as *const nsAtom)
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

impl ToShmem for Atom {
    fn to_shmem(&self, _builder: &mut SharedMemoryBuilder) -> to_shmem::Result<Self> {
        if !self.is_static() {
            return Err(format!(
                "ToShmem failed for Atom: must be a static atom: {}",
                self
            ));
        }

        Ok(ManuallyDrop::new(Atom(self.0)))
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
    /// Construct a `WeakAtom` from a raw `nsAtom`.
    #[inline]
    pub unsafe fn new<'a>(atom: *const nsAtom) -> &'a mut Self {
        &mut *(atom as *mut WeakAtom)
    }

    /// Clone this atom, bumping the refcount if the atom is not static.
    #[inline]
    pub fn clone(&self) -> Atom {
        unsafe { Atom::from_raw(self.as_ptr()) }
    }

    /// Get the atom hash.
    #[inline]
    pub fn get_hash(&self) -> u32 {
        self.0.mHash
    }

    /// Get the atom as a slice of utf-16 chars.
    #[inline]
    pub fn as_slice(&self) -> &[u16] {
        let string = if self.is_static() {
            let atom_ptr = self.as_ptr() as *const nsStaticAtom;
            let string_offset = unsafe { (*atom_ptr).mStringOffset };
            let string_offset = -(string_offset as isize);
            let u8_ptr = atom_ptr as *const u8;
            // It is safe to use offset() here because both addresses are within
            // the same struct, e.g. mozilla::detail::gGkAtoms.
            unsafe { u8_ptr.offset(string_offset) as *const u16 }
        } else {
            let atom_ptr = self.as_ptr() as *const nsDynamicAtom;
            // Dynamic atom chars are stored at the end of the object.
            unsafe { atom_ptr.offset(1) as *const u16 }
        };
        unsafe { slice::from_raw_parts(string, self.len() as usize) }
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
    where
        F: FnOnce(&str) -> Output,
    {
        let mut buffer = mem::MaybeUninit::<[u8; 64]>::uninit();
        let buffer = unsafe { &mut *buffer.as_mut_ptr() };

        // The total string length in utf16 is going to be less than or equal
        // the slice length (each utf16 character is going to take at least one
        // and at most 2 items in the utf16 slice).
        //
        // Each of those characters will take at most four bytes in the utf8
        // one. Thus if the slice is less than 64 / 4 (16) we can guarantee that
        // we'll decode it in place.
        let owned_string;
        let len = self.len();
        let utf8_slice = if len <= 16 {
            let mut total_len = 0;

            for c in self.chars() {
                let c = c.unwrap_or(char::REPLACEMENT_CHARACTER);
                let utf8_len = c.encode_utf8(&mut buffer[total_len..]).len();
                total_len += utf8_len;
            }

            let slice = unsafe { str::from_utf8_unchecked(&buffer[..total_len]) };
            debug_assert_eq!(slice, String::from_utf16_lossy(self.as_slice()));
            slice
        } else {
            owned_string = String::from_utf16_lossy(self.as_slice());
            &*owned_string
        };

        cb(utf8_slice)
    }

    /// Returns whether this atom is static.
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.mIsStatic() != 0
    }

    /// Returns whether this atom is ascii lowercase.
    #[inline]
    fn is_ascii_lowercase(&self) -> bool {
        self.0.mIsAsciiLowercase() != 0
    }

    /// Returns the length of the atom string.
    #[inline]
    pub fn len(&self) -> u32 {
        self.0.mLength()
    }

    /// Returns whether this atom is the empty string.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the atom as a mutable pointer.
    #[inline]
    pub fn as_ptr(&self) -> *mut nsAtom {
        let const_ptr: *const nsAtom = &self.0;
        const_ptr as *mut nsAtom
    }

    /// Convert this atom to ASCII lower-case
    pub fn to_ascii_lowercase(&self) -> Atom {
        if self.is_ascii_lowercase() {
            return self.clone();
        }

        let slice = self.as_slice();
        let mut buffer = mem::MaybeUninit::<[u16; 64]>::uninit();
        let buffer = unsafe { &mut *buffer.as_mut_ptr() };
        let mut vec;
        let mutable_slice = if let Some(buffer_prefix) = buffer.get_mut(..slice.len()) {
            buffer_prefix.copy_from_slice(slice);
            buffer_prefix
        } else {
            vec = slice.to_vec();
            &mut vec
        };
        for char16 in &mut *mutable_slice {
            if *char16 <= 0x7F {
                *char16 = (*char16 as u8).to_ascii_lowercase() as u16
            }
        }
        Atom::from(&*mutable_slice)
    }

    /// Return whether two atoms are ASCII-case-insensitive matches
    #[inline]
    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        if self == other {
            return true;
        }

        // If we know both atoms are ascii-lowercase, then we can stick with
        // pointer equality.
        if self.is_ascii_lowercase() && other.is_ascii_lowercase() {
            debug_assert!(!self.eq_ignore_ascii_case_slow(other));
            return false;
        }

        self.eq_ignore_ascii_case_slow(other)
    }

    fn eq_ignore_ascii_case_slow(&self, other: &Self) -> bool {
        let a = self.as_slice();
        let b = other.as_slice();

        if a.len() != b.len() {
            return false;
        }

        a.iter().zip(b).all(|(&a16, &b16)| {
            if a16 <= 0x7F && b16 <= 0x7F {
                (a16 as u8).eq_ignore_ascii_case(&(b16 as u8))
            } else {
                a16 == b16
            }
        })
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
            w.write_char(c.unwrap_or(char::REPLACEMENT_CHARACTER))?
        }
        Ok(())
    }
}

#[inline]
unsafe fn make_handle(ptr: *const nsAtom) -> NonZeroUsize {
    debug_assert!(!ptr.is_null());
    if !WeakAtom::new(ptr).is_static() {
        NonZeroUsize::new_unchecked(ptr as usize)
    } else {
        make_static_handle(ptr as *mut nsStaticAtom)
    }
}

#[inline]
unsafe fn make_static_handle(ptr: *const nsStaticAtom) -> NonZeroUsize {
    // FIXME(heycam): Use offset_from once it's stabilized.
    // https://github.com/rust-lang/rust/issues/41079
    debug_assert!(valid_static_atom_addr(ptr as usize));
    let base = &gGkAtoms as *const _;
    let offset = ptr as usize - base as usize;
    NonZeroUsize::new_unchecked((offset << 1) | 1)
}

impl Atom {
    #[inline]
    fn is_static(&self) -> bool {
        self.0.get() & 1 == 1
    }

    /// Execute a callback with the atom represented by `ptr`.
    pub unsafe fn with<F, R>(ptr: *const nsAtom, callback: F) -> R
    where
        F: FnOnce(&Atom) -> R,
    {
        let atom = Atom(make_handle(ptr as *mut nsAtom));
        let ret = callback(&atom);
        mem::forget(atom);
        ret
    }

    /// Creates a static atom from its index in the static atom table, without
    /// checking.
    #[inline]
    pub const unsafe fn from_index_unchecked(index: u16) -> Self {
        // FIXME(emilio): No support for debug_assert! in const fn for now. Note
        // that violating this invariant will debug-assert in the `Deref` impl
        // though.
        //
        // debug_assert!((index as usize) < STATIC_ATOM_COUNT);
        let offset =
            (index as usize) * std::mem::size_of::<nsStaticAtom>() + kGkAtomsArrayOffset as usize;
        Atom(NonZeroUsize::new_unchecked((offset << 1) | 1))
    }

    /// Creates an atom from an atom pointer.
    #[inline(always)]
    pub unsafe fn from_raw(ptr: *mut nsAtom) -> Self {
        let atom = Atom(make_handle(ptr));
        if !atom.is_static() {
            Gecko_AddRefAtom(ptr);
        }
        atom
    }

    /// Creates an atom from an atom pointer that has already had AddRef
    /// called on it. This may be a static or dynamic atom.
    #[inline]
    pub unsafe fn from_addrefed(ptr: *mut nsAtom) -> Self {
        assert!(!ptr.is_null());
        Atom(make_handle(ptr))
    }

    /// Convert this atom into an addrefed nsAtom pointer.
    #[inline]
    pub fn into_addrefed(self) -> *mut nsAtom {
        let ptr = self.as_ptr();
        mem::forget(self);
        ptr
    }
}

impl Hash for Atom {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write_u32(self.get_hash());
    }
}

impl Hash for WeakAtom {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write_u32(self.get_hash());
    }
}

impl Clone for Atom {
    #[inline(always)]
    fn clone(&self) -> Atom {
        unsafe {
            let atom = Atom(self.0);
            if !atom.is_static() {
                Gecko_AddRefAtom(atom.as_ptr());
            }
            atom
        }
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
        write!(w, "Atom(0x{:08x}, {})", self.0, self)
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(w)
    }
}

impl<'a> From<&'a str> for Atom {
    #[inline]
    fn from(string: &str) -> Atom {
        debug_assert!(string.len() <= u32::max_value() as usize);
        unsafe {
            Atom::from_addrefed(Gecko_Atomize(
                string.as_ptr() as *const _,
                string.len() as u32,
            ))
        }
    }
}

impl<'a> From<&'a [u16]> for Atom {
    #[inline]
    fn from(slice: &[u16]) -> Atom {
        Atom::from(&*nsStr::from(slice))
    }
}

impl<'a> From<&'a nsAString> for Atom {
    #[inline]
    fn from(string: &nsAString) -> Atom {
        unsafe { Atom::from_addrefed(Gecko_Atomize16(string)) }
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

malloc_size_of_is_0!(Atom);

impl SpecifiedValueInfo for Atom {}
