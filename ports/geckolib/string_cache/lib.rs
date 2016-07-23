/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use] #[no_link]
extern crate cfg_if;
extern crate gecko_bindings;
extern crate heapsize;
extern crate selectors;
extern crate serde;

use gecko_bindings::bindings::Gecko_AddRefAtom;
use gecko_bindings::bindings::Gecko_AtomEqualsUTF8IgnoreCase;
use gecko_bindings::bindings::Gecko_Atomize;
use gecko_bindings::bindings::Gecko_GetAtomAsUTF16;
use gecko_bindings::bindings::Gecko_ReleaseAtom;
use gecko_bindings::structs::nsIAtom;
use heapsize::HeapSizeOf;
use selectors::bloom::BloomHash;
use selectors::parser::FromCowStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::{Cow, Borrow};
use std::char::{self, DecodeUtf16};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::Cloned;
use std::mem;
use std::ops::Deref;
use std::slice;

#[macro_use]
pub mod atom_macro;

#[macro_export]
macro_rules! ns {
    () => { atom!("") }
}

pub type Namespace = Atom;

#[allow(non_snake_case)]
#[inline]
pub fn Namespace(atom: Atom) -> Atom {
    atom
}

/// A strong reference to a Gecko atom.
#[derive(PartialEq, Eq)]
pub struct Atom(*mut WeakAtom);

/// An atom *without* a strong reference.
///
/// Only usable as `&'a WeakAtom`,
/// where `'a` is the lifetime of something that holds a strong reference to that atom.
pub struct WeakAtom(nsIAtom);

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
    #[inline]
    pub unsafe fn new<'a>(atom: *mut nsIAtom) -> &'a mut Self {
        &mut *(atom as *mut WeakAtom)
    }

    #[inline]
    pub fn clone(&self) -> Atom {
        Atom::from(self.as_ptr())
    }

    pub fn get_hash(&self) -> u32 {
        self.0.mHash
    }

    pub fn as_slice(&self) -> &[u16] {
        unsafe {
            let mut len = 0;
            let ptr = Gecko_GetAtomAsUTF16(self.as_ptr(), &mut len);
            slice::from_raw_parts(ptr, len as usize)
        }
    }

    pub fn chars(&self) -> DecodeUtf16<Cloned<slice::Iter<u16>>> {
        char::decode_utf16(self.as_slice().iter().cloned())
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
            Gecko_AtomEqualsUTF8IgnoreCase(self.as_ptr(), s.as_ptr() as *const _, s.len() as u32)
        }
    }

    pub fn to_string(&self) -> String {
        String::from_utf16(self.as_slice()).unwrap()
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut nsIAtom {
        let const_ptr: *const nsIAtom = &self.0;
        const_ptr as *mut nsIAtom
    }
}

impl Atom {
    pub unsafe fn with<F>(ptr: *mut nsIAtom, callback: &mut F) where F: FnMut(&Atom) {
        let atom = Atom(WeakAtom::new(ptr));
        callback(&atom);
        mem::forget(atom);
     }
}

impl BloomHash for Atom {
    #[inline]
    fn bloom_hash(&self) -> u32 {
        self.get_hash()
    }
}

impl BloomHash for WeakAtom {
    #[inline]
    fn bloom_hash(&self) -> u32 {
        self.get_hash()
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
        unsafe {
            Gecko_ReleaseAtom(self.as_ptr());
        }
    }
}

impl Default for Atom {
    #[inline]
    fn default() -> Self {
        atom!("")
    }
}

impl HeapSizeOf for Atom {
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

impl PartialEq<str> for Atom {
    fn eq(&self, other: &str) -> bool {
        self.chars().eq(other.chars().map(Ok))
    }
}

impl PartialEq<Atom> for str {
    fn eq(&self, other: &Atom) -> bool {
        other == self
    }
}

impl<'a> From<&'a str> for Atom {
    #[inline]
    fn from(string: &str) -> Atom {
        assert!(string.len() <= u32::max_value() as usize);
        unsafe {
            Atom(WeakAtom::new(
                Gecko_Atomize(string.as_ptr() as *const _, string.len() as u32)
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

impl FromCowStr for Atom {
    #[inline]
    fn from_cow_str(string: Cow<str>) -> Atom {
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
        unsafe {
            Gecko_AddRefAtom(ptr);
            Atom(WeakAtom::new(ptr))
        }
    }
}
