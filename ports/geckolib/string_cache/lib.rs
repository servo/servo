/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate cfg_if;
extern crate gecko_bindings;
extern crate heapsize;
extern crate serde;

use gecko_bindings::bindings::Gecko_AddRefAtom;
use gecko_bindings::bindings::Gecko_AtomEqualsUTF8IgnoreCase;
use gecko_bindings::bindings::Gecko_Atomize;
use gecko_bindings::bindings::Gecko_GetAtomAsUTF16;
use gecko_bindings::bindings::Gecko_ReleaseAtom;
use gecko_bindings::structs::nsIAtom;
use heapsize::HeapSizeOf;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::char;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::transmute;
use std::ops::Deref;
use std::slice;

#[macro_use]
pub mod atom_macro;

#[macro_export]
macro_rules! ns {
    () => { $crate::Namespace(atom!("")) };
}

#[derive(PartialEq, Eq)]
pub struct Atom(*mut nsIAtom);
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Namespace(pub Atom);

pub struct BorrowedAtom<'a> {
    weak_ptr: *mut nsIAtom,
    chain: PhantomData<&'a ()>,
}

impl<'a> BorrowedAtom<'a> {
    pub unsafe fn new(atom: *mut nsIAtom) ->  Self {
        BorrowedAtom {
            weak_ptr: atom,
            chain: PhantomData,
        }
    }
}

impl<'a> Deref for BorrowedAtom<'a> {
    type Target = Atom;
    fn deref(&self) -> &Atom {
        unsafe {
            transmute(self)
        }
    }
}

impl<'a> PartialEq<Atom> for BorrowedAtom<'a> {
    fn eq(&self, other: &Atom) -> bool {
        self.weak_ptr == other.as_ptr()
    }
}

pub struct BorrowedNamespace<'a> {
    weak_ptr: *mut nsIAtom,
    chain: PhantomData<&'a ()>,
}

impl<'a> BorrowedNamespace<'a> {
    pub unsafe fn new(atom: *mut nsIAtom) ->  Self {
        BorrowedNamespace {
            weak_ptr: atom,
            chain: PhantomData,
        }
    }
}

impl<'a> Deref for BorrowedNamespace<'a> {
    type Target = Namespace;
    fn deref(&self) -> &Namespace {
        unsafe {
            transmute(self)
        }
    }
}

impl<'a> PartialEq<Namespace> for BorrowedNamespace<'a> {
    fn eq(&self, other: &Namespace) -> bool {
        self.weak_ptr == other.0.as_ptr()
    }
}

unsafe impl Send for Atom {}
unsafe impl Sync for Atom {}

impl Atom {
    pub fn get_hash(&self) -> u32 {
        unsafe {
            (*self.0).mHash
        }
    }

    pub fn as_slice(&self) -> &[u16] {
        unsafe {
            let mut len = 0;
            let ptr = Gecko_GetAtomAsUTF16(self.0, &mut len);
            slice::from_raw_parts(ptr, len as usize)
        }
    }

    pub fn with_str<F, Output>(&self, cb: F) -> Output
                               where F: FnOnce(&str) -> Output {
        // FIXME(bholley): We should measure whether it makes more sense to
        // cache the UTF-8 version in the Gecko atom table somehow.
        let owned = String::from_utf16(self.as_slice()).unwrap();
        cb(&owned)
    }

    pub fn eq_str_ignore_ascii_case(&self, s: &str) -> bool {
        unsafe {
            Gecko_AtomEqualsUTF8IgnoreCase(self.0, s.as_ptr() as *const _, s.len() as u32)
        }
    }

    pub fn to_string(&self) -> String {
        String::from_utf16(self.as_slice()).unwrap()
    }

    pub fn as_ptr(&self) -> *mut nsIAtom {
        self.0
    }

    pub unsafe fn with<F>(ptr: *mut nsIAtom, callback: &mut F) where F: FnMut(&Atom) {
        callback(transmute(&ptr))
    }

    // Static atoms have a dummy AddRef/Release, so we don't bother calling
    // AddRef() here. This would cause memory corruption with non-static atoms
    // both because (a) we wouldn't hold the atom alive, and (b) we can't avoid
    // calling Release() when the Atom is dropped, since we can't tell the
    // difference between static and non-static atoms without bloating the
    // size of Atom beyond word-size.
    pub unsafe fn from_static(ptr: *mut nsIAtom) -> Atom {
        Atom(ptr)
    }
}

impl Hash for Atom {
    fn hash<H>(&self, state: &mut H)
        where H: Hasher
    {
        state.write_u32(self.get_hash());
    }
}

impl Clone for Atom {
    #[inline(always)]
    fn clone(&self) -> Atom {
        unsafe {
            Gecko_AddRefAtom(self.0);
        }
        Atom(self.0)
    }
}

impl Drop for Atom {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            Gecko_ReleaseAtom(self.0);
        }
    }
}

impl HeapSizeOf for Atom {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl HeapSizeOf for Namespace {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl Serialize for Atom {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        self.with_str(|s| s.serialize(serializer))
    }
}

impl Deserialize for Atom {
    fn deserialize<D>(deserializer: &mut D) -> Result<Atom, D::Error> where D: Deserializer {
        let string: String = try!(Deserialize::deserialize(deserializer));
        Ok(Atom::from(&*string))
    }
}

impl fmt::Debug for Atom {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "Gecko Atom {:p}", self.0)
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        for c in char::decode_utf16(self.as_slice().iter().cloned()) {
            try!(write!(w, "{}", c.unwrap_or(char::REPLACEMENT_CHARACTER)))
        }
        Ok(())
    }
}

impl<'a> From<&'a str> for Atom {
    #[inline]
    fn from(string: &str) -> Atom {
        assert!(string.len() <= u32::max_value() as usize);
        Atom(unsafe {
            Gecko_Atomize(string.as_ptr() as *const _, string.len() as u32)
        })
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
    fn from(ptr: *mut nsIAtom) -> Atom {
        unsafe {
            Gecko_AddRefAtom(ptr);
            Atom(ptr)
        }
    }
}
