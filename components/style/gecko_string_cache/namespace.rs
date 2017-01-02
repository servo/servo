/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A type to represent a namespace.

use gecko_bindings::structs::nsIAtom;
use std::borrow::{Borrow, Cow};
use std::fmt;
use std::ops::Deref;
use string_cache::{Atom, WeakAtom};

#[macro_export]
macro_rules! ns {
    () => { $crate::string_cache::Namespace(atom!("")) }
}

/// A Gecko namespace is just a wrapped atom.
#[derive(Debug, PartialEq, Eq, Clone, Default, Hash)]
pub struct Namespace(pub Atom);

/// A Gecko WeakNamespace is a wrapped WeakAtom.
#[derive(Hash)]
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

impl<'a> From<Cow<'a, str>> for Namespace {
    fn from(s: Cow<'a, str>) -> Self {
        Namespace(Atom::from(s))
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
    /// Trivially construct a WeakNamespace.
    #[inline]
    pub unsafe fn new<'a>(atom: *mut nsIAtom) -> &'a Self {
        &*(atom as *const WeakNamespace)
    }

    /// Clone this WeakNamespace to obtain a strong reference to the same
    /// underlying namespace.
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
