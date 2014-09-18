/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Provides a wrapper around the Atom type in the string cache
//! crate. It's needed so that it can implement the Encodable
//! trait which is required by Servo.

use str::DOMString;

use serialize::{Encoder, Encodable};
use std::fmt;
use std::hash::Hash;
use string_cache::atom;

#[deriving(Clone, Eq, Hash, PartialEq)]
pub struct Atom {
    /// Public for use by pattern macros
    pub atom: atom::Atom,
}

impl Atom {
    #[inline(always)]
    pub fn from_slice(slice: &str) -> Atom {
        Atom {
            atom: atom::Atom::from_slice(slice)
        }
    }

    #[inline(always)]
    pub fn from_option_domstring(s: &Option<DOMString>) -> Atom {
        match *s {
            None => satom!(""),
            Some(ref s) => Atom::from_slice(s.as_slice()),
        }
    }

    #[inline(always)]
    pub fn as_slice<'t>(&'t self) -> &'t str {
        self.atom.as_slice()
    }
}

impl fmt::Show for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:s}", self.atom.as_slice())
    }
}

impl<E, S: Encoder<E>> Encodable<S, E> for Atom {
    fn encode(&self, _s: &mut S) -> Result<(), E> {
        Ok(())
    }
}
