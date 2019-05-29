// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Fork of Arc for Servo. This has the following advantages over std::sync::Arc:
//!
//! * We don't waste storage on the weak reference count.
//! * We don't do extra RMU operations to handle the possibility of weak references.
//! * We can experiment with arena allocation (todo).
//! * We can add methods to support our custom use cases [1].
//! * We have support for dynamically-sized types (see from_header_and_iter).
//! * We have support for thin arcs to unsized types (see ThinArc).
//! * We have support for references to static data, which don't do any
//!   refcounting.
//!
//! [1]: https://bugzilla.mozilla.org/show_bug.cgi?id=1360883

// The semantics of `Arc` are already documented in the Rust docs, so we don't
// duplicate those here.
#![allow(missing_docs)]

extern crate nodrop;
#[cfg(feature = "servo")]
extern crate serde;
extern crate stable_deref_trait;

use nodrop::NoDrop;
#[cfg(feature = "servo")]
use serde::{Deserialize, Serialize};
use stable_deref_trait::{CloneStableDeref, StableDeref};
use std::alloc::Layout;
use std::borrow;
use std::cmp::Ordering;
use std::convert::From;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::{ExactSizeIterator, Iterator};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::process;
use std::ptr;
use std::slice;
use std::sync::atomic;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::{isize, usize};

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

/// Special refcount value that means the data is not reference counted,
/// and that the `Arc` is really acting as a read-only static reference.
const STATIC_REFCOUNT: usize = usize::MAX;

/// An atomically reference counted shared pointer
///
/// See the documentation for [`Arc`] in the standard library. Unlike the
/// standard library `Arc`, this `Arc` does not support weak reference counting.
///
/// See the discussion in https://github.com/rust-lang/rust/pull/60594 for the
/// usage of PhantomData.
///
/// [`Arc`]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html
#[repr(C)]
pub struct Arc<T: ?Sized> {
    p: ptr::NonNull<ArcInner<T>>,
    phantom: PhantomData<T>,
}

/// An `Arc` that is known to be uniquely owned
///
/// When `Arc`s are constructed, they are known to be
/// uniquely owned. In such a case it is safe to mutate
/// the contents of the `Arc`. Normally, one would just handle
/// this by mutating the data on the stack before allocating the
/// `Arc`, however it's possible the data is large or unsized
/// and you need to heap-allocate it earlier in such a way
/// that it can be freely converted into a regular `Arc` once you're
/// done.
///
/// `UniqueArc` exists for this purpose, when constructed it performs
/// the same allocations necessary for an `Arc`, however it allows mutable access.
/// Once the mutation is finished, you can call `.shareable()` and get a regular `Arc`
/// out of it.
///
/// ```rust
/// # use servo_arc::UniqueArc;
/// let data = [1, 2, 3, 4, 5];
/// let mut x = UniqueArc::new(data);
/// x[4] = 7; // mutate!
/// let y = x.shareable(); // y is an Arc<T>
/// ```
pub struct UniqueArc<T: ?Sized>(Arc<T>);

impl<T> UniqueArc<T> {
    #[inline]
    /// Construct a new UniqueArc
    pub fn new(data: T) -> Self {
        UniqueArc(Arc::new(data))
    }

    #[inline]
    /// Convert to a shareable Arc<T> once we're done mutating it
    pub fn shareable(self) -> Arc<T> {
        self.0
    }
}

impl<T> Deref for UniqueArc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.0
    }
}

impl<T> DerefMut for UniqueArc<T> {
    fn deref_mut(&mut self) -> &mut T {
        // We know this to be uniquely owned
        unsafe { &mut (*self.0.ptr()).data }
    }
}

unsafe impl<T: ?Sized + Sync + Send> Send for Arc<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Arc<T> {}

/// The object allocated by an Arc<T>
#[repr(C)]
struct ArcInner<T: ?Sized> {
    count: atomic::AtomicUsize,
    data: T,
}

unsafe impl<T: ?Sized + Sync + Send> Send for ArcInner<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for ArcInner<T> {}

impl<T> Arc<T> {
    /// Construct an `Arc<T>`
    #[inline]
    pub fn new(data: T) -> Self {
        let x = Box::new(ArcInner {
            count: atomic::AtomicUsize::new(1),
            data,
        });
        unsafe {
            Arc {
                p: ptr::NonNull::new_unchecked(Box::into_raw(x)),
                phantom: PhantomData,
            }
        }
    }

    /// Convert the Arc<T> to a raw pointer, suitable for use across FFI
    ///
    /// Note: This returns a pointer to the data T, which is offset in the allocation.
    ///
    /// It is recommended to use RawOffsetArc for this.
    #[inline]
    fn into_raw(this: Self) -> *const T {
        let ptr = unsafe { &((*this.ptr()).data) as *const _ };
        mem::forget(this);
        ptr
    }

    /// Reconstruct the Arc<T> from a raw pointer obtained from into_raw()
    ///
    /// Note: This raw pointer will be offset in the allocation and must be preceded
    /// by the atomic count.
    ///
    /// It is recommended to use RawOffsetArc for this
    #[inline]
    unsafe fn from_raw(ptr: *const T) -> Self {
        // To find the corresponding pointer to the `ArcInner` we need
        // to subtract the offset of the `data` field from the pointer.
        let ptr = (ptr as *const u8).offset(-offset_of!(ArcInner<T>, data));
        Arc {
            p: ptr::NonNull::new_unchecked(ptr as *mut ArcInner<T>),
            phantom: PhantomData,
        }
    }

    /// Create a new static Arc<T> (one that won't reference count the object)
    /// and place it in the allocation provided by the specified `alloc`
    /// function.
    ///
    /// `alloc` must return a pointer into a static allocation suitable for
    /// storing data with the `Layout` passed into it. The pointer returned by
    /// `alloc` will not be freed.
    #[inline]
    pub unsafe fn new_static<F>(alloc: F, data: T) -> Arc<T>
    where
        F: FnOnce(Layout) -> *mut u8,
    {
        let ptr = alloc(Layout::new::<ArcInner<T>>()) as *mut ArcInner<T>;

        let x = ArcInner {
            count: atomic::AtomicUsize::new(STATIC_REFCOUNT),
            data,
        };

        ptr::write(ptr, x);

        Arc {
            p: ptr::NonNull::new_unchecked(ptr),
            phantom: PhantomData,
        }
    }

    /// Produce a pointer to the data that can be converted back
    /// to an Arc. This is basically an `&Arc<T>`, without the extra indirection.
    /// It has the benefits of an `&T` but also knows about the underlying refcount
    /// and can be converted into more `Arc<T>`s if necessary.
    #[inline]
    pub fn borrow_arc<'a>(&'a self) -> ArcBorrow<'a, T> {
        ArcBorrow(&**self)
    }

    /// Temporarily converts |self| into a bonafide RawOffsetArc and exposes it to the
    /// provided callback. The refcount is not modified.
    #[inline(always)]
    pub fn with_raw_offset_arc<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&RawOffsetArc<T>) -> U,
    {
        // Synthesize transient Arc, which never touches the refcount of the ArcInner.
        let transient = unsafe { NoDrop::new(Arc::into_raw_offset(ptr::read(self))) };

        // Expose the transient Arc to the callback, which may clone it if it wants.
        let result = f(&transient);

        // Forget the transient Arc to leave the refcount untouched.
        mem::forget(transient);

        // Forward the result.
        result
    }

    /// Returns the address on the heap of the Arc itself -- not the T within it -- for memory
    /// reporting.
    ///
    /// If this is a static reference, this returns null.
    pub fn heap_ptr(&self) -> *const c_void {
        if self.inner().count.load(Relaxed) == STATIC_REFCOUNT {
            ptr::null()
        } else {
            self.p.as_ptr() as *const ArcInner<T> as *const c_void
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
        unsafe { &*self.ptr() }
    }

    // Non-inlined part of `drop`. Just invokes the destructor.
    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
        let _ = Box::from_raw(self.ptr());
    }

    /// Test pointer equality between the two Arcs, i.e. they must be the _same_
    /// allocation
    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr() == other.ptr()
    }

    fn ptr(&self) -> *mut ArcInner<T> {
        self.p.as_ptr()
    }
}

impl<T: ?Sized> Clone for Arc<T> {
    #[inline]
    fn clone(&self) -> Self {
        // NOTE(emilio): If you change anything here, make sure that the
        // implementation in layout/style/ServoStyleConstsInlines.h matches!
        //
        // Using a relaxed ordering to check for STATIC_REFCOUNT is safe, since
        // `count` never changes between STATIC_REFCOUNT and other values.
        if self.inner().count.load(Relaxed) != STATIC_REFCOUNT {
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
                process::abort();
            }
        }

        unsafe {
            Arc {
                p: ptr::NonNull::new_unchecked(self.ptr()),
                phantom: PhantomData,
            }
        }
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
    /// Makes a mutable reference to the `Arc`, cloning if necessary
    ///
    /// This is functionally equivalent to [`Arc::make_mut`][mm] from the standard library.
    ///
    /// If this `Arc` is uniquely owned, `make_mut()` will provide a mutable
    /// reference to the contents. If not, `make_mut()` will create a _new_ `Arc`
    /// with a copy of the contents, update `this` to point to it, and provide
    /// a mutable reference to its contents.
    ///
    /// This is useful for implementing copy-on-write schemes where you wish to
    /// avoid copying things if your `Arc` is not shared.
    ///
    /// [mm]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html#method.make_mut
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
            &mut (*this.ptr()).data
        }
    }
}

impl<T: ?Sized> Arc<T> {
    /// Provides mutable access to the contents _if_ the `Arc` is uniquely owned.
    #[inline]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if this.is_unique() {
            unsafe {
                // See make_mut() for documentation of the threadsafety here.
                Some(&mut (*this.ptr()).data)
            }
        } else {
            None
        }
    }

    /// Whether or not the `Arc` is uniquely owned (is the refcount 1?) and not
    /// a static reference.
    #[inline]
    pub fn is_unique(&self) -> bool {
        // See the extensive discussion in [1] for why this needs to be Acquire.
        //
        // [1] https://github.com/servo/servo/issues/21186
        self.inner().count.load(Acquire) == 1
    }
}

impl<T: ?Sized> Drop for Arc<T> {
    #[inline]
    fn drop(&mut self) {
        // NOTE(emilio): If you change anything here, make sure that the
        // implementation in layout/style/ServoStyleConstsInlines.h matches!
        //
        // Using a relaxed ordering to check for STATIC_REFCOUNT is safe, since
        // `count` never changes between STATIC_REFCOUNT and other values.
        if self.inner().count.load(Relaxed) == STATIC_REFCOUNT {
            return;
        }

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
        Self::ptr_eq(self, other) || *(*self) == *(*other)
    }

    fn ne(&self, other: &Arc<T>) -> bool {
        !Self::ptr_eq(self, other) && *(*self) != *(*other)
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
        fmt::Pointer::fmt(&self.ptr(), f)
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
    #[inline]
    fn from(t: T) -> Self {
        Arc::new(t)
    }
}

impl<T: ?Sized> borrow::Borrow<T> for Arc<T> {
    #[inline]
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T: ?Sized> AsRef<T> for Arc<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &**self
    }
}

unsafe impl<T: ?Sized> StableDeref for Arc<T> {}
unsafe impl<T: ?Sized> CloneStableDeref for Arc<T> {}

#[cfg(feature = "servo")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Arc<T> {
    fn deserialize<D>(deserializer: D) -> Result<Arc<T>, D::Error>
    where
        D: ::serde::de::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Arc::new)
    }
}

#[cfg(feature = "servo")]
impl<T: Serialize> Serialize for Arc<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::ser::Serializer,
    {
        (**self).serialize(serializer)
    }
}

/// Structure to allow Arc-managing some fixed-sized data and a variably-sized
/// slice in a single allocation.
#[derive(Debug, Eq, PartialEq, PartialOrd)]
#[repr(C)]
pub struct HeaderSlice<H, T: ?Sized> {
    /// The fixed-sized data.
    pub header: H,

    /// The dynamically-sized data.
    pub slice: T,
}

#[inline(always)]
fn divide_rounding_up(dividend: usize, divisor: usize) -> usize {
    (dividend + divisor - 1) / divisor
}

impl<H, T> Arc<HeaderSlice<H, [T]>> {
    /// Creates an Arc for a HeaderSlice using the given header struct and
    /// iterator to generate the slice.
    ///
    /// `is_static` indicates whether to create a static Arc.
    ///
    /// `alloc` is used to get a pointer to the memory into which the
    /// dynamically sized ArcInner<HeaderSlice<H, T>> value will be
    /// written.  If `is_static` is true, then `alloc` must return a
    /// pointer into some static memory allocation.  If it is false,
    /// then `alloc` must return an allocation that can be dellocated
    /// by calling Box::from_raw::<ArcInner<HeaderSlice<H, T>>> on it.
    #[inline]
    fn from_header_and_iter_alloc<F, I>(alloc: F, header: H, mut items: I, is_static: bool) -> Self
    where
        F: FnOnce(Layout) -> *mut u8,
        I: Iterator<Item = T> + ExactSizeIterator,
    {
        use std::mem::{align_of, size_of};
        assert_ne!(size_of::<T>(), 0, "Need to think about ZST");

        let inner_align = align_of::<ArcInner<HeaderSlice<H, [T; 0]>>>();
        debug_assert!(inner_align >= align_of::<T>());

        // Compute the required size for the allocation.
        let num_items = items.len();
        let size = {
            // Next, synthesize a totally garbage (but properly aligned) pointer
            // to a sequence of T.
            let fake_slice_ptr = inner_align as *const T;

            // Convert that sequence to a fat pointer. The address component of
            // the fat pointer will be garbage, but the length will be correct.
            let fake_slice = unsafe { slice::from_raw_parts(fake_slice_ptr, num_items) };

            // Pretend the garbage address points to our allocation target (with
            // a trailing sequence of T), rather than just a sequence of T.
            let fake_ptr = fake_slice as *const [T] as *const ArcInner<HeaderSlice<H, [T]>>;
            let fake_ref: &ArcInner<HeaderSlice<H, [T]>> = unsafe { &*fake_ptr };

            // Use size_of_val, which will combine static information about the
            // type with the length from the fat pointer. The garbage address
            // will not be used.
            mem::size_of_val(fake_ref)
        };

        let ptr: *mut ArcInner<HeaderSlice<H, [T]>>;
        unsafe {
            // Allocate the buffer.
            let layout = if inner_align <= align_of::<usize>() {
                Layout::from_size_align_unchecked(size, align_of::<usize>())
            } else if inner_align <= align_of::<u64>() {
                // On 32-bit platforms <T> may have 8 byte alignment while usize
                // has 4 byte aligment.  Use u64 to avoid over-alignment.
                // This branch will compile away in optimized builds.
                Layout::from_size_align_unchecked(size, align_of::<u64>())
            } else {
                panic!("Over-aligned type not handled");
            };

            let buffer = alloc(layout);

            // Synthesize the fat pointer. We do this by claiming we have a direct
            // pointer to a [T], and then changing the type of the borrow. The key
            // point here is that the length portion of the fat pointer applies
            // only to the number of elements in the dynamically-sized portion of
            // the type, so the value will be the same whether it points to a [T]
            // or something else with a [T] as its last member.
            let fake_slice: &mut [T] = slice::from_raw_parts_mut(buffer as *mut T, num_items);
            ptr = fake_slice as *mut [T] as *mut ArcInner<HeaderSlice<H, [T]>>;

            // Write the data.
            //
            // Note that any panics here (i.e. from the iterator) are safe, since
            // we'll just leak the uninitialized memory.
            let count = if is_static {
                atomic::AtomicUsize::new(STATIC_REFCOUNT)
            } else {
                atomic::AtomicUsize::new(1)
            };
            ptr::write(&mut ((*ptr).count), count);
            ptr::write(&mut ((*ptr).data.header), header);
            if num_items != 0 {
                let mut current: *mut T = &mut (*ptr).data.slice[0];
                for _ in 0..num_items {
                    ptr::write(
                        current,
                        items
                            .next()
                            .expect("ExactSizeIterator over-reported length"),
                    );
                    current = current.offset(1);
                }
                // We should have consumed the buffer exactly, maybe accounting
                // for some padding from the alignment.
                debug_assert!(
                    (buffer.offset(size as isize) as usize - current as *mut u8 as usize) <
                        inner_align
                );
            }
            assert!(
                items.next().is_none(),
                "ExactSizeIterator under-reported length"
            );
        }

        // Return the fat Arc.
        assert_eq!(
            size_of::<Self>(),
            size_of::<usize>() * 2,
            "The Arc will be fat"
        );
        unsafe {
            Arc {
                p: ptr::NonNull::new_unchecked(ptr),
                phantom: PhantomData,
            }
        }
    }

    /// Creates an Arc for a HeaderSlice using the given header struct and
    /// iterator to generate the slice. The resulting Arc will be fat.
    #[inline]
    pub fn from_header_and_iter<I>(header: H, items: I) -> Self
    where
        I: Iterator<Item = T> + ExactSizeIterator,
    {
        Arc::from_header_and_iter_alloc(
            |layout| {
                // align will only ever be align_of::<usize>() or align_of::<u64>()
                let align = layout.align();
                unsafe {
                    if align == mem::align_of::<usize>() {
                        Self::allocate_buffer::<usize>(layout.size())
                    } else {
                        assert_eq!(align, mem::align_of::<u64>());
                        Self::allocate_buffer::<u64>(layout.size())
                    }
                }
            },
            header,
            items,
            /* is_static = */ false,
        )
    }

    #[inline]
    unsafe fn allocate_buffer<W>(size: usize) -> *mut u8 {
        // We use Vec because the underlying allocation machinery isn't
        // available in stable Rust. To avoid alignment issues, we allocate
        // words rather than bytes, rounding up to the nearest word size.
        let words_to_allocate = divide_rounding_up(size, mem::size_of::<W>());
        let mut vec = Vec::<W>::with_capacity(words_to_allocate);
        vec.set_len(words_to_allocate);
        Box::into_raw(vec.into_boxed_slice()) as *mut W as *mut u8
    }
}

/// Header data with an inline length. Consumers that use HeaderWithLength as the
/// Header type in HeaderSlice can take advantage of ThinArc.
#[derive(Debug, Eq, PartialEq, PartialOrd)]
#[repr(C)]
pub struct HeaderWithLength<H> {
    /// The fixed-sized data.
    pub header: H,

    /// The slice length.
    length: usize,
}

impl<H> HeaderWithLength<H> {
    /// Creates a new HeaderWithLength.
    pub fn new(header: H, length: usize) -> Self {
        HeaderWithLength {
            header: header,
            length: length,
        }
    }
}

type HeaderSliceWithLength<H, T> = HeaderSlice<HeaderWithLength<H>, T>;

/// A "thin" `Arc` containing dynamically sized data
///
/// This is functionally equivalent to Arc<(H, [T])>
///
/// When you create an `Arc` containing a dynamically sized type
/// like `HeaderSlice<H, [T]>`, the `Arc` is represented on the stack
/// as a "fat pointer", where the length of the slice is stored
/// alongside the `Arc`'s pointer. In some situations you may wish to
/// have a thin pointer instead, perhaps for FFI compatibility
/// or space efficiency.
///
/// Note that we use `[T; 0]` in order to have the right alignment for `T`.
///
/// `ThinArc` solves this by storing the length in the allocation itself,
/// via `HeaderSliceWithLength`.
#[repr(C)]
pub struct ThinArc<H, T> {
    ptr: ptr::NonNull<ArcInner<HeaderSliceWithLength<H, [T; 0]>>>,
    phantom: PhantomData<(H, T)>,
}

impl<H: fmt::Debug, T: fmt::Debug> fmt::Debug for ThinArc<H, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

unsafe impl<H: Sync + Send, T: Sync + Send> Send for ThinArc<H, T> {}
unsafe impl<H: Sync + Send, T: Sync + Send> Sync for ThinArc<H, T> {}

// Synthesize a fat pointer from a thin pointer.
//
// See the comment around the analogous operation in from_header_and_iter.
fn thin_to_thick<H, T>(
    thin: *mut ArcInner<HeaderSliceWithLength<H, [T; 0]>>,
) -> *mut ArcInner<HeaderSliceWithLength<H, [T]>> {
    let len = unsafe { (*thin).data.header.length };
    let fake_slice: *mut [T] = unsafe { slice::from_raw_parts_mut(thin as *mut T, len) };

    fake_slice as *mut ArcInner<HeaderSliceWithLength<H, [T]>>
}

impl<H, T> ThinArc<H, T> {
    /// Temporarily converts |self| into a bonafide Arc and exposes it to the
    /// provided callback. The refcount is not modified.
    #[inline]
    pub fn with_arc<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&Arc<HeaderSliceWithLength<H, [T]>>) -> U,
    {
        // Synthesize transient Arc, which never touches the refcount of the ArcInner.
        let transient = unsafe {
            NoDrop::new(Arc {
                p: ptr::NonNull::new_unchecked(thin_to_thick(self.ptr.as_ptr())),
                phantom: PhantomData,
            })
        };

        // Expose the transient Arc to the callback, which may clone it if it wants.
        let result = f(&transient);

        // Forget the transient Arc to leave the refcount untouched.
        // XXXManishearth this can be removed when unions stabilize,
        // since then NoDrop becomes zero overhead
        mem::forget(transient);

        // Forward the result.
        result
    }

    /// Creates a `ThinArc` for a HeaderSlice using the given header struct and
    /// iterator to generate the slice.
    pub fn from_header_and_iter<I>(header: H, items: I) -> Self
    where
        I: Iterator<Item = T> + ExactSizeIterator,
    {
        let header = HeaderWithLength::new(header, items.len());
        Arc::into_thin(Arc::from_header_and_iter(header, items))
    }

    /// Create a static `ThinArc` for a HeaderSlice using the given header
    /// struct and iterator to generate the slice, placing it in the allocation
    /// provided by the specified `alloc` function.
    ///
    /// `alloc` must return a pointer into a static allocation suitable for
    /// storing data with the `Layout` passed into it. The pointer returned by
    /// `alloc` will not be freed.
    pub unsafe fn static_from_header_and_iter<F, I>(alloc: F, header: H, items: I) -> Self
    where
        F: FnOnce(Layout) -> *mut u8,
        I: Iterator<Item = T> + ExactSizeIterator,
    {
        let header = HeaderWithLength::new(header, items.len());
        Arc::into_thin(Arc::from_header_and_iter_alloc(
            alloc, header, items, /* is_static = */ true,
        ))
    }

    /// Returns the address on the heap of the ThinArc itself -- not the T
    /// within it -- for memory reporting, and bindings.
    #[inline]
    pub fn ptr(&self) -> *const c_void {
        self.ptr.as_ptr() as *const ArcInner<T> as *const c_void
    }

    /// If this is a static ThinArc, this returns null.
    #[inline]
    pub fn heap_ptr(&self) -> *const c_void {
        let is_static =
            ThinArc::with_arc(self, |a| a.inner().count.load(Relaxed) == STATIC_REFCOUNT);
        if is_static {
            ptr::null()
        } else {
            self.ptr()
        }
    }
}

impl<H, T> Deref for ThinArc<H, T> {
    type Target = HeaderSliceWithLength<H, [T]>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &(*thin_to_thick(self.ptr.as_ptr())).data }
    }
}

impl<H, T> Clone for ThinArc<H, T> {
    #[inline]
    fn clone(&self) -> Self {
        ThinArc::with_arc(self, |a| Arc::into_thin(a.clone()))
    }
}

impl<H, T> Drop for ThinArc<H, T> {
    #[inline]
    fn drop(&mut self) {
        let _ = Arc::from_thin(ThinArc {
            ptr: self.ptr,
            phantom: PhantomData,
        });
    }
}

impl<H, T> Arc<HeaderSliceWithLength<H, [T]>> {
    /// Converts an `Arc` into a `ThinArc`. This consumes the `Arc`, so the refcount
    /// is not modified.
    #[inline]
    pub fn into_thin(a: Self) -> ThinArc<H, T> {
        assert_eq!(
            a.header.length,
            a.slice.len(),
            "Length needs to be correct for ThinArc to work"
        );
        let fat_ptr: *mut ArcInner<HeaderSliceWithLength<H, [T]>> = a.ptr();
        mem::forget(a);
        let thin_ptr = fat_ptr as *mut [usize] as *mut usize;
        ThinArc {
            ptr: unsafe {
                ptr::NonNull::new_unchecked(
                    thin_ptr as *mut ArcInner<HeaderSliceWithLength<H, [T; 0]>>,
                )
            },
            phantom: PhantomData,
        }
    }

    /// Converts a `ThinArc` into an `Arc`. This consumes the `ThinArc`, so the refcount
    /// is not modified.
    #[inline]
    pub fn from_thin(a: ThinArc<H, T>) -> Self {
        let ptr = thin_to_thick(a.ptr.as_ptr());
        mem::forget(a);
        unsafe {
            Arc {
                p: ptr::NonNull::new_unchecked(ptr),
                phantom: PhantomData,
            }
        }
    }
}

impl<H: PartialEq, T: PartialEq> PartialEq for ThinArc<H, T> {
    #[inline]
    fn eq(&self, other: &ThinArc<H, T>) -> bool {
        ThinArc::with_arc(self, |a| ThinArc::with_arc(other, |b| *a == *b))
    }
}

impl<H: Eq, T: Eq> Eq for ThinArc<H, T> {}

/// An `Arc`, except it holds a pointer to the T instead of to the
/// entire ArcInner. This struct is FFI-compatible.
///
/// ```text
///  Arc<T>    RawOffsetArc<T>
///   |          |
///   v          v
///  ---------------------
/// | RefCount | T (data) | [ArcInner<T>]
///  ---------------------
/// ```
///
/// This means that this is a direct pointer to
/// its contained data (and can be read from by both C++ and Rust),
/// but we can also convert it to a "regular" Arc<T> by removing the offset.
///
/// This is very useful if you have an Arc-containing struct shared between Rust and C++,
/// and wish for C++ to be able to read the data behind the `Arc` without incurring
/// an FFI call overhead.
#[derive(Eq)]
#[repr(C)]
pub struct RawOffsetArc<T> {
    ptr: ptr::NonNull<T>,
}

unsafe impl<T: Sync + Send> Send for RawOffsetArc<T> {}
unsafe impl<T: Sync + Send> Sync for RawOffsetArc<T> {}

impl<T> Deref for RawOffsetArc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr.as_ptr() }
    }
}

impl<T> Clone for RawOffsetArc<T> {
    #[inline]
    fn clone(&self) -> Self {
        Arc::into_raw_offset(self.clone_arc())
    }
}

impl<T> Drop for RawOffsetArc<T> {
    fn drop(&mut self) {
        let _ = Arc::from_raw_offset(RawOffsetArc {
            ptr: self.ptr.clone(),
        });
    }
}

impl<T: fmt::Debug> fmt::Debug for RawOffsetArc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: PartialEq> PartialEq for RawOffsetArc<T> {
    fn eq(&self, other: &RawOffsetArc<T>) -> bool {
        *(*self) == *(*other)
    }

    fn ne(&self, other: &RawOffsetArc<T>) -> bool {
        *(*self) != *(*other)
    }
}

impl<T> RawOffsetArc<T> {
    /// Temporarily converts |self| into a bonafide Arc and exposes it to the
    /// provided callback. The refcount is not modified.
    #[inline]
    pub fn with_arc<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&Arc<T>) -> U,
    {
        // Synthesize transient Arc, which never touches the refcount of the ArcInner.
        let transient = unsafe { NoDrop::new(Arc::from_raw(self.ptr.as_ptr())) };

        // Expose the transient Arc to the callback, which may clone it if it wants.
        let result = f(&transient);

        // Forget the transient Arc to leave the refcount untouched.
        // XXXManishearth this can be removed when unions stabilize,
        // since then NoDrop becomes zero overhead
        mem::forget(transient);

        // Forward the result.
        result
    }

    /// If uniquely owned, provide a mutable reference
    /// Else create a copy, and mutate that
    ///
    /// This is functionally the same thing as `Arc::make_mut`
    #[inline]
    pub fn make_mut(&mut self) -> &mut T
    where
        T: Clone,
    {
        unsafe {
            // extract the RawOffsetArc as an owned variable
            let this = ptr::read(self);
            // treat it as a real Arc
            let mut arc = Arc::from_raw_offset(this);
            // obtain the mutable reference. Cast away the lifetime
            // This may mutate `arc`
            let ret = Arc::make_mut(&mut arc) as *mut _;
            // Store the possibly-mutated arc back inside, after converting
            // it to a RawOffsetArc again
            ptr::write(self, Arc::into_raw_offset(arc));
            &mut *ret
        }
    }

    /// Clone it as an `Arc`
    #[inline]
    pub fn clone_arc(&self) -> Arc<T> {
        RawOffsetArc::with_arc(self, |a| a.clone())
    }

    /// Produce a pointer to the data that can be converted back
    /// to an `Arc`
    #[inline]
    pub fn borrow_arc<'a>(&'a self) -> ArcBorrow<'a, T> {
        ArcBorrow(&**self)
    }
}

impl<T> Arc<T> {
    /// Converts an `Arc` into a `RawOffsetArc`. This consumes the `Arc`, so the refcount
    /// is not modified.
    #[inline]
    pub fn into_raw_offset(a: Self) -> RawOffsetArc<T> {
        unsafe {
            RawOffsetArc {
                ptr: ptr::NonNull::new_unchecked(Arc::into_raw(a) as *mut T),
            }
        }
    }

    /// Converts a `RawOffsetArc` into an `Arc`. This consumes the `RawOffsetArc`, so the refcount
    /// is not modified.
    #[inline]
    pub fn from_raw_offset(a: RawOffsetArc<T>) -> Self {
        let ptr = a.ptr.as_ptr();
        mem::forget(a);
        unsafe { Arc::from_raw(ptr) }
    }
}

/// A "borrowed `Arc`". This is a pointer to
/// a T that is known to have been allocated within an
/// `Arc`.
///
/// This is equivalent in guarantees to `&Arc<T>`, however it is
/// a bit more flexible. To obtain an `&Arc<T>` you must have
/// an `Arc<T>` instance somewhere pinned down until we're done with it.
/// It's also a direct pointer to `T`, so using this involves less pointer-chasing
///
/// However, C++ code may hand us refcounted things as pointers to T directly,
/// so we have to conjure up a temporary `Arc` on the stack each time. The
/// same happens for when the object is managed by a `RawOffsetArc`.
///
/// `ArcBorrow` lets us deal with borrows of known-refcounted objects
/// without needing to worry about where the `Arc<T>` is.
#[derive(Debug, Eq, PartialEq)]
pub struct ArcBorrow<'a, T: 'a>(&'a T);

impl<'a, T> Copy for ArcBorrow<'a, T> {}
impl<'a, T> Clone for ArcBorrow<'a, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> ArcBorrow<'a, T> {
    /// Clone this as an `Arc<T>`. This bumps the refcount.
    #[inline]
    pub fn clone_arc(&self) -> Arc<T> {
        let arc = unsafe { Arc::from_raw(self.0) };
        // addref it!
        mem::forget(arc.clone());
        arc
    }

    /// For constructing from a reference known to be Arc-backed,
    /// e.g. if we obtain such a reference over FFI
    #[inline]
    pub unsafe fn from_ref(r: &'a T) -> Self {
        ArcBorrow(r)
    }

    /// Compare two `ArcBorrow`s via pointer equality. Will only return
    /// true if they come from the same allocation
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.0 as *const T == other.0 as *const T
    }

    /// Temporarily converts |self| into a bonafide Arc and exposes it to the
    /// provided callback. The refcount is not modified.
    #[inline]
    pub fn with_arc<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&Arc<T>) -> U,
        T: 'static,
    {
        // Synthesize transient Arc, which never touches the refcount.
        let transient = unsafe { NoDrop::new(Arc::from_raw(self.0)) };

        // Expose the transient Arc to the callback, which may clone it if it wants.
        let result = f(&transient);

        // Forget the transient Arc to leave the refcount untouched.
        // XXXManishearth this can be removed when unions stabilize,
        // since then NoDrop becomes zero overhead
        mem::forget(transient);

        // Forward the result.
        result
    }

    /// Similar to deref, but uses the lifetime |a| rather than the lifetime of
    /// self, which is incompatible with the signature of the Deref trait.
    #[inline]
    pub fn get(&self) -> &'a T {
        self.0
    }
}

impl<'a, T> Deref for ArcBorrow<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.0
    }
}

/// A tagged union that can represent `Arc<A>` or `Arc<B>` while only consuming a
/// single word. The type is also `NonNull`, and thus can be stored in an Option
/// without increasing size.
///
/// This is functionally equivalent to
/// `enum ArcUnion<A, B> { First(Arc<A>), Second(Arc<B>)` but only takes up
/// up a single word of stack space.
///
/// This could probably be extended to support four types if necessary.
pub struct ArcUnion<A, B> {
    p: ptr::NonNull<()>,
    phantom_a: PhantomData<A>,
    phantom_b: PhantomData<B>,
}

unsafe impl<A: Sync + Send, B: Send + Sync> Send for ArcUnion<A, B> {}
unsafe impl<A: Sync + Send, B: Send + Sync> Sync for ArcUnion<A, B> {}

impl<A: PartialEq, B: PartialEq> PartialEq for ArcUnion<A, B> {
    fn eq(&self, other: &Self) -> bool {
        use crate::ArcUnionBorrow::*;
        match (self.borrow(), other.borrow()) {
            (First(x), First(y)) => x == y,
            (Second(x), Second(y)) => x == y,
            (_, _) => false,
        }
    }
}

/// This represents a borrow of an `ArcUnion`.
#[derive(Debug)]
pub enum ArcUnionBorrow<'a, A: 'a, B: 'a> {
    First(ArcBorrow<'a, A>),
    Second(ArcBorrow<'a, B>),
}

impl<A, B> ArcUnion<A, B> {
    unsafe fn new(ptr: *mut ()) -> Self {
        ArcUnion {
            p: ptr::NonNull::new_unchecked(ptr),
            phantom_a: PhantomData,
            phantom_b: PhantomData,
        }
    }

    /// Returns true if the two values are pointer-equal.
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.p == other.p
    }

    /// Returns an enum representing a borrow of either A or B.
    pub fn borrow(&self) -> ArcUnionBorrow<A, B> {
        if self.is_first() {
            let ptr = self.p.as_ptr() as *const A;
            let borrow = unsafe { ArcBorrow::from_ref(&*ptr) };
            ArcUnionBorrow::First(borrow)
        } else {
            let ptr = ((self.p.as_ptr() as usize) & !0x1) as *const B;
            let borrow = unsafe { ArcBorrow::from_ref(&*ptr) };
            ArcUnionBorrow::Second(borrow)
        }
    }

    /// Creates an `ArcUnion` from an instance of the first type.
    pub fn from_first(other: Arc<A>) -> Self {
        unsafe { Self::new(Arc::into_raw(other) as *mut _) }
    }

    /// Creates an `ArcUnion` from an instance of the second type.
    pub fn from_second(other: Arc<B>) -> Self {
        unsafe { Self::new(((Arc::into_raw(other) as usize) | 0x1) as *mut _) }
    }

    /// Returns true if this `ArcUnion` contains the first type.
    pub fn is_first(&self) -> bool {
        self.p.as_ptr() as usize & 0x1 == 0
    }

    /// Returns true if this `ArcUnion` contains the second type.
    pub fn is_second(&self) -> bool {
        !self.is_first()
    }

    /// Returns a borrow of the first type if applicable, otherwise `None`.
    pub fn as_first(&self) -> Option<ArcBorrow<A>> {
        match self.borrow() {
            ArcUnionBorrow::First(x) => Some(x),
            ArcUnionBorrow::Second(_) => None,
        }
    }

    /// Returns a borrow of the second type if applicable, otherwise None.
    pub fn as_second(&self) -> Option<ArcBorrow<B>> {
        match self.borrow() {
            ArcUnionBorrow::First(_) => None,
            ArcUnionBorrow::Second(x) => Some(x),
        }
    }
}

impl<A, B> Clone for ArcUnion<A, B> {
    fn clone(&self) -> Self {
        match self.borrow() {
            ArcUnionBorrow::First(x) => ArcUnion::from_first(x.clone_arc()),
            ArcUnionBorrow::Second(x) => ArcUnion::from_second(x.clone_arc()),
        }
    }
}

impl<A, B> Drop for ArcUnion<A, B> {
    fn drop(&mut self) {
        match self.borrow() {
            ArcUnionBorrow::First(x) => unsafe {
                let _ = Arc::from_raw(&*x);
            },
            ArcUnionBorrow::Second(x) => unsafe {
                let _ = Arc::from_raw(&*x);
            },
        }
    }
}

impl<A: fmt::Debug, B: fmt::Debug> fmt::Debug for ArcUnion<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.borrow(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Arc, HeaderWithLength, ThinArc};
    use std::clone::Clone;
    use std::ops::Drop;
    use std::sync::atomic;
    use std::sync::atomic::Ordering::{Acquire, SeqCst};

    #[derive(PartialEq)]
    struct Canary(*mut atomic::AtomicUsize);

    impl Drop for Canary {
        fn drop(&mut self) {
            unsafe {
                (*self.0).fetch_add(1, SeqCst);
            }
        }
    }

    #[test]
    fn empty_thin() {
        let header = HeaderWithLength::new(100u32, 0);
        let x = Arc::from_header_and_iter(header, std::iter::empty::<i32>());
        let y = Arc::into_thin(x.clone());
        assert_eq!(y.header.header, 100);
        assert!(y.slice.is_empty());
        assert_eq!(x.header.header, 100);
        assert!(x.slice.is_empty());
    }

    #[test]
    fn thin_assert_padding() {
        #[derive(Clone, Default)]
        #[repr(C)]
        struct Padded {
            i: u16,
        }

        // The header will have more alignment than `Padded`
        let header = HeaderWithLength::new(0i32, 2);
        let items = vec![Padded { i: 0xdead }, Padded { i: 0xbeef }];
        let a = ThinArc::from_header_and_iter(header, items.into_iter());
        assert_eq!(a.slice.len(), 2);
        assert_eq!(a.slice[0].i, 0xdead);
        assert_eq!(a.slice[1].i, 0xbeef);
    }

    #[test]
    fn slices_and_thin() {
        let mut canary = atomic::AtomicUsize::new(0);
        let c = Canary(&mut canary as *mut atomic::AtomicUsize);
        let v = vec![5, 6];
        let header = HeaderWithLength::new(c, v.len());
        {
            let x = Arc::into_thin(Arc::from_header_and_iter(header, v.into_iter()));
            let y = ThinArc::with_arc(&x, |q| q.clone());
            let _ = y.clone();
            let _ = x == x;
            Arc::from_thin(x.clone());
        }
        assert_eq!(canary.load(Acquire), 1);
    }
}
