/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gecko_bindings::structs;
use heapsize::HeapSizeOf;
use std::{mem, ptr};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Trait for all objects that have Addref() and Release
/// methods and can be placed inside RefPtr<T>
pub unsafe trait RefCounted {
    fn addref(&self);
    unsafe fn release(&self);
}

/// Trait for types which can be shared across threads in RefPtr
pub unsafe trait ThreadSafeRefCounted: RefCounted {}

#[derive(Debug)]
pub struct RefPtr<T: RefCounted> {
    ptr: *mut T,
    _marker: PhantomData<T>,
}

/// A RefPtr that we know is uniquely owned
///
/// This is basically Box<T>, with the additional
/// guarantee that the box can be safely interpreted
/// as a RefPtr<T> (with refcount 1)
///
/// This is useful when you wish to create a refptr
/// and mutate it temporarily, while it is still
/// uniquely owned.
pub struct UniqueRefPtr<T: RefCounted>(RefPtr<T>);

// There is no safe conversion from &T to RefPtr<T> (like Gecko has)
// because this lets you break UniqueRefPtr's guarantee

impl<T: RefCounted> RefPtr<T> {
    /// Create a new RefPtr from an already addrefed
    /// pointer obtained from FFI. Pointer
    /// must be valid, non-null and have been addrefed
    pub unsafe fn from_addrefed(ptr: *mut T) -> Self {
        debug_assert!(!ptr.is_null());
        RefPtr {
            ptr: ptr,
            _marker: PhantomData,
        }
    }

    /// Create a new RefPtr from a pointer obtained
    /// from FFI. Pointer must be valid and non null.
    /// This method calls addref() internally
    pub unsafe fn new(ptr: *mut T) -> Self {
        debug_assert!(!ptr.is_null());
        let ret = RefPtr {
            ptr: ptr,
            _marker: PhantomData,
        };
        ret.addref();
        ret
    }

    /// Produces an FFI-compatible RefPtr that can be stored in
    /// style structs.
    ///
    /// structs::RefPtr does not have a destructor, so this may leak
    pub fn forget(self) -> structs::RefPtr<T> {
        let ret = structs::RefPtr {
            mRawPtr: self.ptr,
        };
        mem::forget(self);
        ret
    }

    /// Returns the raw inner pointer
    /// to be fed back into FFI
    pub fn get(&self) -> *mut T {
        self.ptr
    }

    /// Addref the inner data
    ///
    /// Leaky on its own
    pub fn addref(&self) {
        unsafe { (*self.ptr).addref(); }
    }

    /// Release the inner data
    ///
    /// Call only when the data actuall needs releasing
    pub unsafe fn release(&self) {
        (*self.ptr).release();
    }
}

impl<T: RefCounted> UniqueRefPtr<T> {
    /// Create a unique refptr from an already addrefed
    /// pointer obtained from FFI. The refcount must be one.
    /// The pointer must be valid and non null
    pub unsafe fn from_addrefed(ptr: *mut T) -> Self {
        UniqueRefPtr(RefPtr::from_addrefed(ptr))
    }

    /// Convert to a RefPtr so that it can be used
    pub fn get(self) -> RefPtr<T> {
        self.0
    }
}

impl<T: RefCounted> Deref for RefPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T: RefCounted> Deref for UniqueRefPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.0.ptr }
    }
}

impl<T: RefCounted> DerefMut for UniqueRefPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.ptr }
    }
}

impl<T: RefCounted> structs::RefPtr<T> {
    /// Produces a Rust-side RefPtr from an FFI RefPtr, bumping the refcount
    ///
    /// Must be called on a valid, non-null structs::RefPtr<T>
    pub unsafe fn to_safe(&self) -> RefPtr<T> {
        debug_assert!(!self.mRawPtr.is_null());
        let r = RefPtr {
            ptr: self.mRawPtr,
            _marker: PhantomData,
        };
        r.addref();
        r
    }
    /// Produces a Rust-side RefPtr, consuming the existing one (and not bumping the refcount)
    pub unsafe fn into_safe(self) -> RefPtr<T> {
        debug_assert!(!self.mRawPtr.is_null());
        RefPtr {
            ptr: self.mRawPtr,
            _marker: PhantomData,
        }
    }

    /// Replace a structs::RefPtr<T> with a different one, appropriately addref/releasing
    ///
    /// Both `self` and `other` must be valid, but can be null
    pub unsafe fn set(&mut self, other: &Self) {
        self.clear();
        if !other.mRawPtr.is_null() {
            *self = other.to_safe().forget();
        }
    }

    /// Clear an instance of the structs::RefPtr<T>, by releasing
    /// it and setting its contents to null
    ///
    /// `self` must be valid, but can be null
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
    /// `self` must be valid, possibly null
    pub fn set_move(&mut self, other: RefPtr<T>) {
        if !self.mRawPtr.is_null() {
            unsafe { (*self.mRawPtr).release(); }
        }
        *self = other.forget();
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

impl<T: RefCounted> HeapSizeOf for RefPtr<T> {
    fn heap_size_of_children(&self) -> usize { 0 }
}

impl<T: RefCounted> PartialEq for RefPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

unsafe impl<T: ThreadSafeRefCounted> Send for RefPtr<T> {}
unsafe impl<T: ThreadSafeRefCounted> Sync for RefPtr<T> {}

// Companion of NS_DECL_THREADSAFE_FFI_REFCOUNTING
//
// Gets you a free RefCounted impl
macro_rules! impl_threadsafe_refcount {
    ($t:ty, $addref:ident, $release:ident) => (
        unsafe impl RefCounted for $t {
            fn addref(&self) {
                unsafe { ::gecko_bindings::bindings::$addref(self as *const _ as *mut _) }
            }
            unsafe fn release(&self) {
                ::gecko_bindings::bindings::$release(self as *const _ as *mut _)
            }
        }
        unsafe impl ThreadSafeRefCounted for $t {}
    );
}

impl_threadsafe_refcount!(::gecko_bindings::bindings::ThreadSafePrincipalHolder,
                          Gecko_AddRefPrincipalArbitraryThread,
                          Gecko_ReleasePrincipalArbitraryThread);
impl_threadsafe_refcount!(::gecko_bindings::bindings::ThreadSafeURIHolder,
                          Gecko_AddRefURIArbitraryThread,
                          Gecko_ReleaseURIArbitraryThread);
impl_threadsafe_refcount!(::gecko_bindings::structs::nsStyleQuoteValues,
                          Gecko_AddRefQuoteValuesArbitraryThread,
                          Gecko_ReleaseQuoteValuesArbitraryThread);
impl_threadsafe_refcount!(::gecko_bindings::structs::nsCSSValueSharedList,
                          Gecko_AddRefCSSValueSharedListArbitraryThread,
                          Gecko_ReleaseCSSValueSharedListArbitraryThread);
pub type GeckoArcPrincipal = RefPtr<::gecko_bindings::bindings::ThreadSafePrincipalHolder>;
pub type GeckoArcURI = RefPtr<::gecko_bindings::bindings::ThreadSafeURIHolder>;
