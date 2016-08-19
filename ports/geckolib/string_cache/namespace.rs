/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gecko_bindings::structs::nsIAtom;
use selectors::bloom::BloomHash;
use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;
use {Atom, WeakAtom};

#[macro_export]
macro_rules! ns {
    () => { $crate::Namespace(atom!("")) }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct Namespace(pub Atom);
pub struct WeakNamespace(WeakAtom);

impl Deref for Namespace {
    type Target = WeakNamespace;

    #[inline]
    fn deref(&self) -> &WeakNamespace {
        let weak: *const WeakAtom = &*self.0;
        unsafe {
            &*(weak as *const WeakNamespace)
        }
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(w)
    }
}

impl Borrow<WeakNamespace> for Namespace {
    #[inline]
    fn borrow(&self) -> &WeakNamespace {
        self
    }
}

impl WeakNamespace {
    #[inline]
    pub unsafe fn new<'a>(atom: *mut nsIAtom) -> &'a Self {
        &*(atom as *const WeakNamespace)
    }

    #[inline]
    pub fn clone(&self) -> Namespace {
        Namespace(self.0.clone())
    }
}

impl Eq for WeakNamespace {}
impl PartialEq for WeakNamespace {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let weak: *const WeakNamespace = self;
        let other: *const WeakNamespace = other;
        weak == other
    }
}

impl BloomHash for Namespace {
    #[inline]
    fn bloom_hash(&self) -> u32 {
        self.0.get_hash()
    }
}

impl BloomHash for WeakNamespace {
    #[inline]
    fn bloom_hash(&self) -> u32 {
        self.0.get_hash()
    }
}
