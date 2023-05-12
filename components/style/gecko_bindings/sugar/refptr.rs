/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A rust helper to ease the use of Gecko's refcounted types.

use crate::gecko_bindings::{bindings, structs};
use crate::Atom;
use servo_arc::Arc;
use std::fmt::Write;
use std::marker::PhantomData;
use std::ops::Deref;
use std::{fmt, mem, ptr};

/// Trait for all objects that have Addref() and Release
/// methods and can be placed inside RefPtr<T>
pub unsafe trait RefCounted {
    /// Bump the reference count.
    fn addref(&self);
    /// Decrease the reference count.
    unsafe fn release(&self);
}

/// Trait for types which can be shared across threads in RefPtr.
pub unsafe trait ThreadSafeRefCounted: RefCounted {}

/// A custom RefPtr implementation to take into account Drop semantics and
/// a bit less-painful memory management.
pub struct RefPtr<T: RefCounted> {
    ptr: *mut T,
    _marker: PhantomData<T>,
}

impl<T: RefCounted> fmt::Debug for RefPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("RefPtr { ")?;
        self.ptr.fmt(f)?;
        f.write_char('}')
    }
}

impl<T: RefCounted> RefPtr<T> {
    /// Create a new RefPtr from an already addrefed pointer obtained from FFI.
    ///
    /// The pointer must be valid, non-null and have been addrefed.
    pub unsafe fn from_addrefed(ptr: *mut T) -> Self {
        debug_assert!(!ptr.is_null());
        RefPtr {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Returns whether the current pointer is null.
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Returns a null pointer.
    pub fn null() -> Self {
        Self {
            ptr: ptr::null_mut(),
            _marker: PhantomData,
        }
    }

    /// Create a new RefPtr from a pointer obtained from FFI.
    ///
    /// This method calls addref() internally
    pub unsafe fn new(ptr: *mut T) -> Self {
        let ret = RefPtr {
            ptr,
            _marker: PhantomData,
        };
        ret.addref();
        ret
    }

    /// Produces an FFI-compatible RefPtr that can be stored in style structs.
    ///
    /// structs::RefPtr does not have a destructor, so this may leak
    pub fn forget(self) -> structs::RefPtr<T> {
        let ret = structs::RefPtr {
            mRawPtr: self.ptr,
            _phantom_0: PhantomData,
        };
        mem::forget(self);
        ret
    }

    /// Returns the raw inner pointer to be fed back into FFI.
    pub fn get(&self) -> *mut T {
        self.ptr
    }

    /// Addref the inner data, obviously leaky on its own.
    pub fn addref(&self) {
        if !self.ptr.is_null() {
            unsafe {
                (*self.ptr).addref();
            }
        }
    }

    /// Release the inner data.
    ///
    /// Call only when the data actually needs releasing.
    pub unsafe fn release(&self) {
        if !self.ptr.is_null() {
            (*self.ptr).release();
        }
    }
}

impl<T: RefCounted> Deref for RefPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        debug_assert!(!self.ptr.is_null());
        unsafe { &*self.ptr }
    }
}

impl<T: RefCounted> structs::RefPtr<T> {
    /// Produces a Rust-side RefPtr from an FFI RefPtr, bumping the refcount.
    ///
    /// Must be called on a valid, non-null structs::RefPtr<T>.
    pub unsafe fn to_safe(&self) -> RefPtr<T> {
        let r = RefPtr {
            ptr: self.mRawPtr,
            _marker: PhantomData,
        };
        r.addref();
        r
    }
    /// Produces a Rust-side RefPtr, consuming the existing one (and not bumping
    /// the refcount).
    pub unsafe fn into_safe(self) -> RefPtr<T> {
        debug_assert!(!self.mRawPtr.is_null());
        RefPtr {
            ptr: self.mRawPtr,
            _marker: PhantomData,
        }
    }

    /// Replace a structs::RefPtr<T> with a different one, appropriately
    /// addref/releasing.
    ///
    /// Both `self` and `other` must be valid, but can be null.
    ///
    /// Safe when called on an aliased pointer because the refcount in that case
    /// needs to be at least two.
    pub unsafe fn set(&mut self, other: &Self) {
        self.clear();
        if !other.mRawPtr.is_null() {
            *self = other.to_safe().forget();
        }
    }

    /// Clear an instance of the structs::RefPtr<T>, by releasing
    /// it and setting its contents to null.
    ///
    /// `self` must be valid, but can be null.
    pub unsafe fn clear(&mut self) {
        if !self.mRawPtr.is_null() {
            (*self.mRawPtr).release();
            self.mRawPtr = ptr::null_mut();
        }
    }

    /// Replace a `structs::RefPtr<T>` with a `RefPtr<T>`,
    /// consuming the `RefPtr<T>`, and releasing the old
    /// value in `self` if necessary.
    ///
    /// `self` must be valid, possibly null.
    pub fn set_move(&mut self, other: RefPtr<T>) {
        if !self.mRawPtr.is_null() {
            unsafe {
                (*self.mRawPtr).release();
            }
        }
        *self = other.forget();
    }
}

impl<T> structs::RefPtr<T> {
    /// Returns a new, null refptr.
    pub fn null() -> Self {
        Self {
            mRawPtr: ptr::null_mut(),
            _phantom_0: PhantomData,
        }
    }

    /// Create a new RefPtr from an arc.
    pub fn from_arc(s: Arc<T>) -> Self {
        Self {
            mRawPtr: Arc::into_raw(s) as *mut _,
            _phantom_0: PhantomData,
        }
    }

    /// Sets the contents to an Arc<T>.
    pub fn set_arc(&mut self, other: Arc<T>) {
        unsafe {
            if !self.mRawPtr.is_null() {
                let _ = Arc::from_raw(self.mRawPtr);
            }
            self.mRawPtr = Arc::into_raw(other) as *mut _;
        }
    }
}

impl<T: RefCounted> Drop for RefPtr<T> {
    fn drop(&mut self) {
        unsafe { self.release() }
    }
}

impl<T: RefCounted> Clone for RefPtr<T> {
    fn clone(&self) -> Self {
        self.addref();
        RefPtr {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: RefCounted> PartialEq for RefPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

unsafe impl<T: ThreadSafeRefCounted> Send for RefPtr<T> {}
unsafe impl<T: ThreadSafeRefCounted> Sync for RefPtr<T> {}

macro_rules! impl_refcount {
    ($t:ty, $addref:path, $release:path) => {
        unsafe impl RefCounted for $t {
            #[inline]
            fn addref(&self) {
                unsafe { $addref(self as *const _ as *mut _) }
            }

            #[inline]
            unsafe fn release(&self) {
                $release(self as *const _ as *mut _)
            }
        }
    };
}

// Companion of NS_DECL_THREADSAFE_FFI_REFCOUNTING.
//
// Gets you a free RefCounted impl implemented via FFI.
macro_rules! impl_threadsafe_refcount {
    ($t:ty, $addref:path, $release:path) => {
        impl_refcount!($t, $addref, $release);
        unsafe impl ThreadSafeRefCounted for $t {}
    };
}

impl_threadsafe_refcount!(
    structs::mozilla::URLExtraData,
    bindings::Gecko_AddRefURLExtraDataArbitraryThread,
    bindings::Gecko_ReleaseURLExtraDataArbitraryThread
);
impl_threadsafe_refcount!(
    structs::nsIURI,
    bindings::Gecko_AddRefnsIURIArbitraryThread,
    bindings::Gecko_ReleasensIURIArbitraryThread
);
impl_threadsafe_refcount!(
    structs::SheetLoadDataHolder,
    bindings::Gecko_AddRefSheetLoadDataHolderArbitraryThread,
    bindings::Gecko_ReleaseSheetLoadDataHolderArbitraryThread
);

#[inline]
unsafe fn addref_atom(atom: *mut structs::nsAtom) {
    mem::forget(Atom::from_raw(atom));
}

#[inline]
unsafe fn release_atom(atom: *mut structs::nsAtom) {
    let _ = Atom::from_addrefed(atom);
}
impl_threadsafe_refcount!(structs::nsAtom, addref_atom, release_atom);
