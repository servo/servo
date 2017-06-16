// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Fork of Arc for Servo. This has the following advantages over std::Arc:
//! * We don't waste storage on the weak reference count.
//! * We don't do extra RMU operations to handle the possibility of weak references.
//! * We can experiment with arena allocation (todo).
//! * We can add methods to support our custom use cases [1].
//! * We have support for dynamically-sized types (see from_header_and_iter).
//! * We have support for thin arcs to unsized types (see ThinArc).
//!
//! [1] https://bugzilla.mozilla.org/show_bug.cgi?id=1360883

// The semantics of Arc are alread documented in the Rust docs, so we don't
// duplicate those here.
#![allow(missing_docs)]

#[cfg(feature = "servo")] extern crate serde;
extern crate nodrop;

#[cfg(feature = "servo")]
use heapsize::HeapSizeOf;
use nodrop::NoDrop;
#[cfg(feature = "servo")]
use serde::{Deserialize, Serialize};
use std::{isize, usize};
use std::borrow;
use std::cmp::Ordering;
use std::convert::From;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::{ExactSizeIterator, Iterator};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice;
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

/// Wrapper type for pointers to get the non-zero optimization. When
/// NonZero/Shared/Unique are stabilized, we should just use Shared
/// here to get the same effect. Gankro is working on this in [1].
///
/// It's unfortunate that this needs to infect all the caller types
/// with 'static. It would be nice to just use a &() and a PhantomData<T>
/// instead, but then the compiler can't determine whether the &() should
/// be thin or fat (which depends on whether or not T is sized). Given
/// that this is all a temporary hack, this restriction is fine for now.
///
/// [1] https://github.com/rust-lang/rust/issues/27730
pub struct NonZeroPtrMut<T: ?Sized + 'static>(&'static mut T);
impl<T: ?Sized> NonZeroPtrMut<T> {
    pub fn new(ptr: *mut T) -> Self {
        assert!(!(ptr as *mut u8).is_null());
        NonZeroPtrMut(unsafe { mem::transmute(ptr) })
    }

    pub fn ptr(&self) -> *mut T {
        self.0 as *const T as *mut T
    }
}

impl<T: ?Sized + 'static> Clone for NonZeroPtrMut<T> {
    fn clone(&self) -> Self {
        NonZeroPtrMut::new(self.ptr())
    }
}

impl<T: ?Sized + 'static> fmt::Pointer for NonZeroPtrMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr(), f)
    }
}

impl<T: ?Sized + 'static> fmt::Debug for NonZeroPtrMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Pointer>::fmt(self, f)
    }
}

impl<T: ?Sized + 'static> PartialEq for NonZeroPtrMut<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr() == other.ptr()
    }
}

impl<T: ?Sized + 'static> Eq for NonZeroPtrMut<T> {}

pub struct Arc<T: ?Sized + 'static> {
    p: NonZeroPtrMut<ArcInner<T>>,
}

/// An Arc that is known to be uniquely owned
///
/// This lets us build arcs that we can mutate before
/// freezing, without needing to change the allocation
pub struct UniqueArc<T: ?Sized + 'static>(Arc<T>);

impl<T> UniqueArc<T> {
    #[inline]
    /// Construct a new UniqueArc
    pub fn new(data: T) -> Self {
        UniqueArc(Arc::new(data))
    }

    #[inline]
    /// Convert to a shareable Arc<T> once we're done using it
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
        Arc { p: NonZeroPtrMut::new(Box::into_raw(x)) }
    }

    pub fn into_raw(this: Self) -> *const T {
        let ptr = unsafe { &((*this.ptr()).data) as *const _ };
        mem::forget(this);
        ptr
    }

    pub unsafe fn from_raw(ptr: *const T) -> Self {
        // To find the corresponding pointer to the `ArcInner` we need
        // to subtract the offset of the `data` field from the pointer.
        let ptr = (ptr as *const u8).offset(-offset_of!(ArcInner<T>, data));
        Arc {
            p: NonZeroPtrMut::new(ptr as *mut ArcInner<T>),
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


    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr() == other.ptr()
    }

    fn ptr(&self) -> *mut ArcInner<T> {
        self.p.ptr()
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

        Arc { p: NonZeroPtrMut::new(self.ptr()) }
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
            &mut (*this.ptr()).data
        }
    }
}

impl<T: ?Sized> Arc<T> {
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
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Arc<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Arc<T>, D::Error>
    where
        D: ::serde::de::Deserializer<'de>,
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

/// Structure to allow Arc-managing some fixed-sized data and a variably-sized
/// slice in a single allocation.
#[derive(Debug, Eq, PartialEq, PartialOrd)]
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
    /// iterator to generate the slice. The resulting Arc will be fat.
    #[inline]
    pub fn from_header_and_iter<I>(header: H, mut items: I) -> Self
        where I: Iterator<Item=T> + ExactSizeIterator
    {
        use ::std::mem::size_of;
        assert!(size_of::<T>() != 0, "Need to think about ZST");

        // Compute the required size for the allocation.
        let num_items = items.len();
        let size = {
            // First, determine the alignment of a hypothetical pointer to a
            // HeaderSlice.
            let fake_slice_ptr_align: usize = mem::align_of::<ArcInner<HeaderSlice<H, [T; 1]>>>();

            // Next, synthesize a totally garbage (but properly aligned) pointer
            // to a sequence of T.
            let fake_slice_ptr = fake_slice_ptr_align as *const T;

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
            // Allocate the buffer. We use Vec because the underlying allocation
            // machinery isn't available in stable Rust.
            //
            // To avoid alignment issues, we allocate words rather than bytes,
            // rounding up to the nearest word size.
            let buffer = if mem::align_of::<T>() <= mem::align_of::<usize>() {
                Self::allocate_buffer::<usize>(size)
            } else if mem::align_of::<T>() <= mem::align_of::<u64>() {
                // On 32-bit platforms <T> may have 8 byte alignment while usize has 4 byte aligment.
                // Use u64 to avoid over-alignment.
                // This branch will compile away in optimized builds.
                Self::allocate_buffer::<u64>(size)
            } else {
                panic!("Over-aligned type not handled");
            };

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
            ptr::write(&mut ((*ptr).count), atomic::AtomicUsize::new(1));
            ptr::write(&mut ((*ptr).data.header), header);
            let mut current: *mut T = &mut (*ptr).data.slice[0];
            for _ in 0..num_items {
                ptr::write(current, items.next().expect("ExactSizeIterator over-reported length"));
                current = current.offset(1);
            }
            assert!(items.next().is_none(), "ExactSizeIterator under-reported length");

            // We should have consumed the buffer exactly.
            debug_assert!(current as *mut u8 == buffer.offset(size as isize));
        }

        // Return the fat Arc.
        assert_eq!(size_of::<Self>(), size_of::<usize>() * 2, "The Arc will be fat");
        Arc { p: NonZeroPtrMut::new(ptr) }
    }

    #[inline]
    unsafe fn allocate_buffer<W>(size: usize) -> *mut u8 {
        let words_to_allocate = divide_rounding_up(size, mem::size_of::<W>());
        let mut vec = Vec::<W>::with_capacity(words_to_allocate);
        vec.set_len(words_to_allocate);
        Box::into_raw(vec.into_boxed_slice()) as *mut W as *mut u8
    }
}

/// Header data with an inline length. Consumers that use HeaderWithLength as the
/// Header type in HeaderSlice can take advantage of ThinArc.
#[derive(Debug, Eq, PartialEq, PartialOrd)]
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
pub struct ThinArc<H: 'static, T: 'static> {
    ptr: *mut ArcInner<HeaderSliceWithLength<H, [T; 1]>>,
}

unsafe impl<H: Sync + Send, T: Sync + Send> Send for ThinArc<H, T> {}
unsafe impl<H: Sync + Send, T: Sync + Send> Sync for ThinArc<H, T> {}

// Synthesize a fat pointer from a thin pointer.
//
// See the comment around the analogous operation in from_header_and_iter.
fn thin_to_thick<H, T>(thin: *mut ArcInner<HeaderSliceWithLength<H, [T; 1]>>)
    -> *mut ArcInner<HeaderSliceWithLength<H, [T]>>
{
    let len = unsafe { (*thin).data.header.length };
    let fake_slice: *mut [T] = unsafe {
        slice::from_raw_parts_mut(thin as *mut T, len)
    };

    fake_slice as *mut ArcInner<HeaderSliceWithLength<H, [T]>>
}

impl<H: 'static, T: 'static> ThinArc<H, T> {
    /// Temporarily converts |self| into a bonafide Arc and exposes it to the
    /// provided callback. The refcount is not modified.
    #[inline(always)]
    pub fn with_arc<F, U>(&self, f: F) -> U
        where F: FnOnce(&Arc<HeaderSliceWithLength<H, [T]>>) -> U
    {
        // Synthesize transient Arc, which never touches the refcount of the ArcInner.
        let transient = NoDrop::new(Arc {
            p: NonZeroPtrMut::new(thin_to_thick(self.ptr))
        });

        // Expose the transient Arc to the callback, which may clone it if it wants.
        let result = f(&transient);

        // Forget the transient Arc to leave the refcount untouched.
        mem::forget(transient);

        // Forward the result.
        result
    }
}

impl<H, T> Deref for ThinArc<H, T> {
    type Target = HeaderSliceWithLength<H, [T]>;
    fn deref(&self) -> &Self::Target {
        unsafe { &(*thin_to_thick(self.ptr)).data }
    }
}

impl<H: 'static, T: 'static> Clone for ThinArc<H, T> {
    fn clone(&self) -> Self {
        ThinArc::with_arc(self, |a| Arc::into_thin(a.clone()))
    }
}

impl<H: 'static, T: 'static> Drop for ThinArc<H, T> {
    fn drop(&mut self) {
        let _ = Arc::from_thin(ThinArc { ptr: self.ptr });
    }
}

impl<H: 'static, T: 'static> Arc<HeaderSliceWithLength<H, [T]>> {
    /// Converts an Arc into a ThinArc. This consumes the Arc, so the refcount
    /// is not modified.
    pub fn into_thin(a: Self) -> ThinArc<H, T> {
        assert!(a.header.length == a.slice.len(),
                "Length needs to be correct for ThinArc to work");
        let fat_ptr: *mut ArcInner<HeaderSliceWithLength<H, [T]>> = a.ptr();
        mem::forget(a);
        let thin_ptr = fat_ptr as *mut [usize] as *mut usize;
        ThinArc {
            ptr: thin_ptr as *mut ArcInner<HeaderSliceWithLength<H, [T; 1]>>
        }
    }

    /// Converts a ThinArc into an Arc. This consumes the ThinArc, so the refcount
    /// is not modified.
    pub fn from_thin(a: ThinArc<H, T>) -> Self {
        let ptr = thin_to_thick(a.ptr);
        mem::forget(a);
        Arc {
            p: NonZeroPtrMut::new(ptr)
        }
    }
}

impl<H: PartialEq + 'static, T: PartialEq + 'static> PartialEq for ThinArc<H, T> {
    fn eq(&self, other: &ThinArc<H, T>) -> bool {
        ThinArc::with_arc(self, |a| {
            ThinArc::with_arc(other, |b| {
                *a == *b
            })
        })
    }
}

impl<H: Eq + 'static, T: Eq + 'static> Eq for ThinArc<H, T> {}

#[cfg(test)]
mod tests {
    use std::clone::Clone;
    use std::ops::Drop;
    use std::sync::atomic;
    use std::sync::atomic::Ordering::{Acquire, SeqCst};
    use super::{Arc, HeaderWithLength, ThinArc};

    #[derive(PartialEq)]
    struct Canary(*mut atomic::AtomicUsize);

    impl Drop for Canary {
        fn drop(&mut self) {
            unsafe { (*self.0).fetch_add(1, SeqCst); }
        }
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
        assert!(canary.load(Acquire) == 1);
    }
}
