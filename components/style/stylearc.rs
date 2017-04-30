// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Fork of Arc for the style system. This has the following advantages over std::Arc:
//! * We don't waste storage on the weak reference count.
//! * We don't do extra RMU operations to handle the possibility of weak references.
//! * We can experiment with arena allocation (todo).
//! * We can add methods to support our custom use cases [1].
//!
//! [1] https://bugzilla.mozilla.org/show_bug.cgi?id=1360883

// The semantics of Arc are alread documented in the Rust docs, so we don't
// duplicate those here.
#![allow(missing_docs)]

#[cfg(feature = "servo")]
use heapsize::HeapSizeOf;
#[cfg(feature = "servo")]
use serde::{Deserialize, Serialize};
use std::{isize, usize};
use std::borrow;
use std::cmp::Ordering;
use std::convert::From;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::Deref;
use std::sync::atomic;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

// Private macro to get the offset of a struct field in bytes from the address of the struct.
macro_rules! offset_of {
    ($container:path, $field:ident) => {{
        // Make sure the field actually exists. This line ensures that a compile-time error is
        // generated if $field is accessed through a Deref impl.
        let $container { $field: _, .. };

        // Create an (invalid) instance of the container and calculate the offset to its
        // field. Using a null pointer might be UB if `&(*(0 as *const T)).field` is interpreted to
        // be nullptr deref.
        let invalid: $container = ::std::mem::uninitialized();
        let offset = &invalid.$field as *const _ as usize - &invalid as *const _ as usize;

        // Do not run destructors on the made up invalid instance.
        ::std::mem::forget(invalid);
        offset as isize
    }};
}

/// A soft limit on the amount of references that may be made to an `Arc`.
///
/// Going above this limit will abort your program (although not
/// necessarily) at _exactly_ `MAX_REFCOUNT + 1` references.
const MAX_REFCOUNT: usize = (isize::MAX) as usize;

pub struct Arc<T: ?Sized> {
    // FIXME(bholley): When NonZero/Shared/Unique are stabilized, we should use
    // Shared here to get the NonZero optimization. Gankro is working on this.
    //
    // If we need a compact Option<Arc<T>> beforehand, we can make a helper
    // class that wraps the result of Arc::into_raw.
    //
    // https://github.com/rust-lang/rust/issues/27730
    ptr: *mut ArcInner<T>,
}

unsafe impl<T: ?Sized + Sync + Send> Send for Arc<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Arc<T> {}

struct ArcInner<T: ?Sized> {
    count: atomic::AtomicUsize,
    data: T,
}

unsafe impl<T: ?Sized + Sync + Send> Send for ArcInner<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for ArcInner<T> {}

impl<T> Arc<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        let x = Box::new(ArcInner {
            count: atomic::AtomicUsize::new(1),
            data: data,
        });
        Arc { ptr: Box::into_raw(x) }
    }

    pub fn into_raw(this: Self) -> *const T {
        let ptr = unsafe { &((*this.ptr).data) as *const _ };
        mem::forget(this);
        ptr
    }

    pub unsafe fn from_raw(ptr: *const T) -> Self {
        // To find the corresponding pointer to the `ArcInner` we need
        // to subtract the offset of the `data` field from the pointer.
        let ptr = (ptr as *const u8).offset(-offset_of!(ArcInner<T>, data));
        Arc {
            ptr: ptr as *mut ArcInner<T>,
        }
    }
}

impl<T: ?Sized> Arc<T> {
    #[inline]
    fn inner(&self) -> &ArcInner<T> {
        // This unsafety is ok because while this arc is alive we're guaranteed
        // that the inner pointer is valid. Furthermore, we know that the
        // `ArcInner` structure itself is `Sync` because the inner data is
        // `Sync` as well, so we're ok loaning out an immutable pointer to these
        // contents.
        unsafe { &*self.ptr }
    }

    // Non-inlined part of `drop`. Just invokes the destructor.
    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
        let _ = Box::from_raw(self.ptr);
    }


    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr == other.ptr
    }
}

impl<T: ?Sized> Clone for Arc<T> {
    #[inline]
    fn clone(&self) -> Self {
        // Using a relaxed ordering is alright here, as knowledge of the
        // original reference prevents other threads from erroneously deleting
        // the object.
        //
        // As explained in the [Boost documentation][1], Increasing the
        // reference counter can always be done with memory_order_relaxed: New
        // references to an object can only be formed from an existing
        // reference, and passing an existing reference from one thread to
        // another must already provide any required synchronization.
        //
        // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
        let old_size = self.inner().count.fetch_add(1, Relaxed);

        // However we need to guard against massive refcounts in case someone
        // is `mem::forget`ing Arcs. If we don't do this the count can overflow
        // and users will use-after free. We racily saturate to `isize::MAX` on
        // the assumption that there aren't ~2 billion threads incrementing
        // the reference count at once. This branch will never be taken in
        // any realistic program.
        //
        // We abort because such a program is incredibly degenerate, and we
        // don't care to support it.
        if old_size > MAX_REFCOUNT {
            // Note: std::process::abort is stable in 1.17, which we don't yet
            // require for Gecko. Panic is good enough in practice here (it will
            // trigger an abort at least in Gecko, and this case is degenerate
            // enough that Servo shouldn't have code that triggers it).
            //
            // We should fix this when we require 1.17.
            panic!();
        }

        Arc { ptr: self.ptr }
    }
}

impl<T: ?Sized> Deref for Arc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.inner().data
    }
}

impl<T: Clone> Arc<T> {
    #[inline]
    pub fn make_mut(this: &mut Self) -> &mut T {
        if !this.is_unique() {
            // Another pointer exists; clone
            *this = Arc::new((**this).clone());
        }

        unsafe {
            // This unsafety is ok because we're guaranteed that the pointer
            // returned is the *only* pointer that will ever be returned to T. Our
            // reference count is guaranteed to be 1 at this point, and we required
            // the Arc itself to be `mut`, so we're returning the only possible
            // reference to the inner data.
            &mut (*this.ptr).data
        }
    }
}

impl<T: ?Sized> Arc<T> {
    #[inline]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if this.is_unique() {
            unsafe {
                // See make_mut() for documentation of the threadsafety here.
                Some(&mut (*this.ptr).data)
            }
        } else {
            None
        }
    }

    #[inline]
    fn is_unique(&self) -> bool {
        // We can use Relaxed here, but the justification is a bit subtle.
        //
        // The reason to use Acquire would be to synchronize with other threads
        // that are modifying the refcount with Release, i.e. to ensure that
        // their writes to memory guarded by this refcount are flushed. However,
        // we know that threads only modify the contents of the Arc when they
        // observe the refcount to be 1, and no other thread could observe that
        // because we're holding one strong reference here.
        self.inner().count.load(Relaxed) == 1
    }
}

impl<T: ?Sized> Drop for Arc<T> {
    #[inline]
    fn drop(&mut self) {
        // Because `fetch_sub` is already atomic, we do not need to synchronize
        // with other threads unless we are going to delete the object.
        if self.inner().count.fetch_sub(1, Release) != 1 {
            return;
        }

        // FIXME(bholley): Use the updated comment when [2] is merged.
        //
        // This load is needed to prevent reordering of use of the data and
        // deletion of the data.  Because it is marked `Release`, the decreasing
        // of the reference count synchronizes with this `Acquire` load. This
        // means that use of the data happens before decreasing the reference
        // count, which happens before this load, which happens before the
        // deletion of the data.
        //
        // As explained in the [Boost documentation][1],
        //
        // > It is important to enforce any possible access to the object in one
        // > thread (through an existing reference) to *happen before* deleting
        // > the object in a different thread. This is achieved by a "release"
        // > operation after dropping a reference (any access to the object
        // > through this reference must obviously happened before), and an
        // > "acquire" operation before deleting the object.
        //
        // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
        // [2]: https://github.com/rust-lang/rust/pull/41714
        self.inner().count.load(Acquire);

        unsafe {
            self.drop_slow();
        }
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Arc<T> {
    fn eq(&self, other: &Arc<T>) -> bool {
        *(*self) == *(*other)
    }

    fn ne(&self, other: &Arc<T>) -> bool {
        *(*self) != *(*other)
    }
}
impl<T: ?Sized + PartialOrd> PartialOrd for Arc<T> {
    fn partial_cmp(&self, other: &Arc<T>) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }

    fn lt(&self, other: &Arc<T>) -> bool {
        *(*self) < *(*other)
    }

    fn le(&self, other: &Arc<T>) -> bool {
        *(*self) <= *(*other)
    }

    fn gt(&self, other: &Arc<T>) -> bool {
        *(*self) > *(*other)
    }

    fn ge(&self, other: &Arc<T>) -> bool {
        *(*self) >= *(*other)
    }
}
impl<T: ?Sized + Ord> Ord for Arc<T> {
    fn cmp(&self, other: &Arc<T>) -> Ordering {
        (**self).cmp(&**other)
    }
}
impl<T: ?Sized + Eq> Eq for Arc<T> {}

impl<T: ?Sized + fmt::Display> fmt::Display for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized> fmt::Pointer for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr, f)
    }
}

impl<T: Default> Default for Arc<T> {
    fn default() -> Arc<T> {
        Arc::new(Default::default())
    }
}

impl<T: ?Sized + Hash> Hash for Arc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl<T> From<T> for Arc<T> {
    fn from(t: T) -> Self {
        Arc::new(t)
    }
}

impl<T: ?Sized> borrow::Borrow<T> for Arc<T> {
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T: ?Sized> AsRef<T> for Arc<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

// This is what the HeapSize crate does for regular arc, but is questionably
// sound. See https://github.com/servo/heapsize/issues/37
#[cfg(feature = "servo")]
impl<T: HeapSizeOf> HeapSizeOf for Arc<T> {
    fn heap_size_of_children(&self) -> usize {
        (**self).heap_size_of_children()
    }
}

#[cfg(feature = "servo")]
impl<T: Deserialize> Deserialize for Arc<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Arc<T>, D::Error>
    where
        D: ::serde::de::Deserializer,
    {
        T::deserialize(deserializer).map(Arc::new)
    }
}

#[cfg(feature = "servo")]
impl<T: Serialize> Serialize for Arc<T>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::ser::Serializer,
    {
        (**self).serialize(serializer)
    }
}
