/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]
#![deny(missing_docs)]

//! A handle that encapsulate a reference to a given data along with its owner.

use owning_ref::StableAddress;
use std::ops::{Deref, DerefMut};

/// `OwningHandle` is a complement to `OwningRef`. Where `OwningRef` allows
/// consumers to pass around an owned object and a dependent reference,
/// `OwningHandle` contains an owned object and a dependent _object_.
///
/// `OwningHandle` can encapsulate a `RefMut` along with its associated
/// `RefCell`, or an `RwLockReadGuard` along with its associated `RwLock`.
/// However, the API is completely generic and there are no restrictions on
/// what types of owning and dependent objects may be used.
///
/// `OwningHandle` is created by passing an owner object (which dereferences
/// to a stable address) along with a callback which receives a pointer to
/// that stable location. The callback may then dereference the pointer and
/// mint a dependent object, with the guarantee that the returned object will
/// not outlive the referent of the pointer.
///
/// This does foist some unsafety onto the callback, which needs an `unsafe`
/// block to dereference the pointer. It would be almost good enough for
/// OwningHandle to pass a transmuted &'static reference to the callback
/// since the lifetime is infinite as far as the minted handle is concerned.
/// However, even an `Fn` callback can still allow the reference to escape
/// via a `StaticMutex` or similar, which technically violates the safety
/// contract. Some sort of language support in the lifetime system could
/// make this API a bit nicer.
pub struct OwningHandle<O, H>
    where O: StableAddress,
          H: Deref,
{
    handle: H,
    _owner: O,
}

impl<O, H> Deref for OwningHandle<O, H>
    where O: StableAddress,
          H: Deref,
{
    type Target = H::Target;
    fn deref(&self) -> &H::Target {
        self.handle.deref()
    }
}

unsafe impl<O, H> StableAddress for OwningHandle<O, H>
    where O: StableAddress,
          H: StableAddress,
{}

impl<O, H> DerefMut for OwningHandle<O, H>
    where O: StableAddress,
          H: DerefMut,
{
    fn deref_mut(&mut self) -> &mut H::Target {
        self.handle.deref_mut()
    }
}

impl<O, H> OwningHandle<O, H>
    where O: StableAddress,
          H: Deref,
{
    /// Create a new OwningHandle. The provided callback will be invoked with
    /// a pointer to the object owned by `o`, and the returned value is stored
    /// as the object to which this `OwningHandle` will forward `Deref` and
    /// `DerefMut`.
    pub fn new<F>(o: O, f: F) -> Self
        where F: Fn(*const O::Target) -> H,
    {
        let h: H;
        {
            h = f(o.deref() as *const O::Target);
        }

        OwningHandle {
          handle: h,
          _owner: o,
        }
    }
}
