// Copyright 2016-2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A reduced fork of Firefox's malloc_size_of crate, for bundling with WebRender.

extern crate app_units;
extern crate euclid;

use std::hash::{BuildHasher, Hash};
use std::mem::size_of;
use std::ops::Range;
use std::os::raw::c_void;
use std::path::PathBuf;

/// A C function that takes a pointer to a heap allocation and returns its size.
type VoidPtrToSizeFn = unsafe extern "C" fn(ptr: *const c_void) -> usize;

/// Operations used when measuring heap usage of data structures.
pub struct MallocSizeOfOps {
    /// A function that returns the size of a heap allocation.
    pub size_of_op: VoidPtrToSizeFn,

    /// Like `size_of_op`, but can take an interior pointer. Optional because
    /// not all allocators support this operation. If it's not provided, some
    /// memory measurements will actually be computed estimates rather than
    /// real and accurate measurements.
    pub enclosing_size_of_op: Option<VoidPtrToSizeFn>,
}

impl MallocSizeOfOps {
    pub fn new(
        size_of: VoidPtrToSizeFn,
        malloc_enclosing_size_of: Option<VoidPtrToSizeFn>,
    ) -> Self {
        MallocSizeOfOps {
            size_of_op: size_of,
            enclosing_size_of_op: malloc_enclosing_size_of,
        }
    }

    /// Check if an allocation is empty. This relies on knowledge of how Rust
    /// handles empty allocations, which may change in the future.
    fn is_empty<T: ?Sized>(ptr: *const T) -> bool {
        // The correct condition is this:
        //   `ptr as usize <= ::std::mem::align_of::<T>()`
        // But we can't call align_of() on a ?Sized T. So we approximate it
        // with the following. 256 is large enough that it should always be
        // larger than the required alignment, but small enough that it is
        // always in the first page of memory and therefore not a legitimate
        // address.
        ptr as *const usize as usize <= 256
    }

    /// Call `size_of_op` on `ptr`, first checking that the allocation isn't
    /// empty, because some types (such as `Vec`) utilize empty allocations.
    pub unsafe fn malloc_size_of<T: ?Sized>(&self, ptr: *const T) -> usize {
        if MallocSizeOfOps::is_empty(ptr) {
            0
        } else {
            (self.size_of_op)(ptr as *const c_void)
        }
    }

    /// Is an `enclosing_size_of_op` available?
    pub fn has_malloc_enclosing_size_of(&self) -> bool {
        self.enclosing_size_of_op.is_some()
    }

    /// Call `enclosing_size_of_op`, which must be available, on `ptr`, which
    /// must not be empty.
    pub unsafe fn malloc_enclosing_size_of<T>(&self, ptr: *const T) -> usize {
        assert!(!MallocSizeOfOps::is_empty(ptr));
        (self.enclosing_size_of_op.unwrap())(ptr as *const c_void)
    }
}

/// Trait for measuring the "deep" heap usage of a data structure. This is the
/// most commonly-used of the traits.
pub trait MallocSizeOf {
    /// Measure the heap usage of all descendant heap-allocated structures, but
    /// not the space taken up by the value itself.
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize;
}

/// Trait for measuring the "shallow" heap usage of a container.
pub trait MallocShallowSizeOf {
    /// Measure the heap usage of immediate heap-allocated descendant
    /// structures, but not the space taken up by the value itself. Anything
    /// beyond the immediate descendants must be measured separately, using
    /// iteration.
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize;
}

impl MallocSizeOf for String {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        unsafe { ops.malloc_size_of(self.as_ptr()) }
    }
}

impl<T: ?Sized> MallocShallowSizeOf for Box<T> {
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        unsafe { ops.malloc_size_of(&**self) }
    }
}

impl<T: MallocSizeOf + ?Sized> MallocSizeOf for Box<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.shallow_size_of(ops) + (**self).size_of(ops)
    }
}

impl MallocSizeOf for () {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

impl<T1, T2> MallocSizeOf for (T1, T2)
where
    T1: MallocSizeOf,
    T2: MallocSizeOf,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops) + self.1.size_of(ops)
    }
}

impl<T1, T2, T3> MallocSizeOf for (T1, T2, T3)
where
    T1: MallocSizeOf,
    T2: MallocSizeOf,
    T3: MallocSizeOf,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops) + self.1.size_of(ops) + self.2.size_of(ops)
    }
}

impl<T1, T2, T3, T4> MallocSizeOf for (T1, T2, T3, T4)
where
    T1: MallocSizeOf,
    T2: MallocSizeOf,
    T3: MallocSizeOf,
    T4: MallocSizeOf,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops) + self.1.size_of(ops) + self.2.size_of(ops) + self.3.size_of(ops)
    }
}

impl<T: MallocSizeOf> MallocSizeOf for Option<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if let Some(val) = self.as_ref() {
            val.size_of(ops)
        } else {
            0
        }
    }
}

impl<T: MallocSizeOf, E: MallocSizeOf> MallocSizeOf for Result<T, E> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match *self {
            Ok(ref x) => x.size_of(ops),
            Err(ref e) => e.size_of(ops),
        }
    }
}

impl<T: MallocSizeOf + Copy> MallocSizeOf for std::cell::Cell<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.get().size_of(ops)
    }
}

impl<T: MallocSizeOf> MallocSizeOf for std::cell::RefCell<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.borrow().size_of(ops)
    }
}

impl<'a, B: ?Sized + ToOwned> MallocSizeOf for std::borrow::Cow<'a, B>
where
    B::Owned: MallocSizeOf,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match *self {
            std::borrow::Cow::Borrowed(_) => 0,
            std::borrow::Cow::Owned(ref b) => b.size_of(ops),
        }
    }
}

impl<T: MallocSizeOf> MallocSizeOf for [T] {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = 0;
        for elem in self.iter() {
            n += elem.size_of(ops);
        }
        n
    }
}

impl<T> MallocShallowSizeOf for Vec<T> {
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        unsafe { ops.malloc_size_of(self.as_ptr()) }
    }
}

impl<T: MallocSizeOf> MallocSizeOf for Vec<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = self.shallow_size_of(ops);
        for elem in self.iter() {
            n += elem.size_of(ops);
        }
        n
    }
}

macro_rules! malloc_size_of_hash_set {
    ($ty:ty) => {
        impl<T, S> MallocShallowSizeOf for $ty
        where
            T: Eq + Hash,
            S: BuildHasher,
        {
            fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
                if ops.has_malloc_enclosing_size_of() {
                    // The first value from the iterator gives us an interior pointer.
                    // `ops.malloc_enclosing_size_of()` then gives us the storage size.
                    // This assumes that the `HashSet`'s contents (values and hashes)
                    // are all stored in a single contiguous heap allocation.
                    self.iter()
                        .next()
                        .map_or(0, |t| unsafe { ops.malloc_enclosing_size_of(t) })
                } else {
                    // An estimate.
                    self.capacity() * (size_of::<T>() + size_of::<usize>())
                }
            }
        }

        impl<T, S> MallocSizeOf for $ty
        where
            T: Eq + Hash + MallocSizeOf,
            S: BuildHasher,
        {
            fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
                let mut n = self.shallow_size_of(ops);
                for t in self.iter() {
                    n += t.size_of(ops);
                }
                n
            }
        }
    };
}

malloc_size_of_hash_set!(std::collections::HashSet<T, S>);

macro_rules! malloc_size_of_hash_map {
    ($ty:ty) => {
        impl<K, V, S> MallocShallowSizeOf for $ty
        where
            K: Eq + Hash,
            S: BuildHasher,
        {
            fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
                // See the implementation for std::collections::HashSet for details.
                if ops.has_malloc_enclosing_size_of() {
                    self.values()
                        .next()
                        .map_or(0, |v| unsafe { ops.malloc_enclosing_size_of(v) })
                } else {
                    self.capacity() * (size_of::<V>() + size_of::<K>() + size_of::<usize>())
                }
            }
        }

        impl<K, V, S> MallocSizeOf for $ty
        where
            K: Eq + Hash + MallocSizeOf,
            V: MallocSizeOf,
            S: BuildHasher,
        {
            fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
                let mut n = self.shallow_size_of(ops);
                for (k, v) in self.iter() {
                    n += k.size_of(ops);
                    n += v.size_of(ops);
                }
                n
            }
        }
    };
}

malloc_size_of_hash_map!(std::collections::HashMap<K, V, S>);

// PhantomData is always 0.
impl<T> MallocSizeOf for std::marker::PhantomData<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

impl MallocSizeOf for PathBuf {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match self.to_str() {
            Some(s) => unsafe { ops.malloc_size_of(s.as_ptr()) },
            None => self.as_os_str().len(),
        }
    }
}

impl<T: MallocSizeOf, Unit> MallocSizeOf for euclid::Length<T, Unit> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops)
    }
}

impl<T: MallocSizeOf, Src, Dst> MallocSizeOf for euclid::Scale<T, Src, Dst> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops)
    }
}

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::Point2D<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.x.size_of(ops) + self.y.size_of(ops)
    }
}

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::Rect<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.origin.size_of(ops) + self.size.size_of(ops)
    }
}

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::SideOffsets2D<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.top.size_of(ops) +
            self.right.size_of(ops) +
            self.bottom.size_of(ops) +
            self.left.size_of(ops)
    }
}

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::Size2D<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.width.size_of(ops) + self.height.size_of(ops)
    }
}

impl<T: MallocSizeOf, Src, Dst> MallocSizeOf for euclid::Transform2D<T, Src, Dst> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.m11.size_of(ops) +
            self.m12.size_of(ops) +
            self.m21.size_of(ops) +
            self.m22.size_of(ops) +
            self.m31.size_of(ops) +
            self.m32.size_of(ops)
    }
}

impl<T: MallocSizeOf, Src, Dst> MallocSizeOf for euclid::Transform3D<T, Src, Dst> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.m11.size_of(ops) +
            self.m12.size_of(ops) +
            self.m13.size_of(ops) +
            self.m14.size_of(ops) +
            self.m21.size_of(ops) +
            self.m22.size_of(ops) +
            self.m23.size_of(ops) +
            self.m24.size_of(ops) +
            self.m31.size_of(ops) +
            self.m32.size_of(ops) +
            self.m33.size_of(ops) +
            self.m34.size_of(ops) +
            self.m41.size_of(ops) +
            self.m42.size_of(ops) +
            self.m43.size_of(ops) +
            self.m44.size_of(ops)
    }
}

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::Vector2D<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.x.size_of(ops) + self.y.size_of(ops)
    }
}

/// For use on types where size_of() returns 0.
#[macro_export]
macro_rules! malloc_size_of_is_0(
    ($($ty:ty),+) => (
        $(
            impl $crate::MallocSizeOf for $ty {
                #[inline(always)]
                fn size_of(&self, _: &mut $crate::MallocSizeOfOps) -> usize {
                    0
                }
            }
        )+
    );
    ($($ty:ident<$($gen:ident),+>),+) => (
        $(
        impl<$($gen: $crate::MallocSizeOf),+> $crate::MallocSizeOf for $ty<$($gen),+> {
            #[inline(always)]
            fn size_of(&self, _: &mut $crate::MallocSizeOfOps) -> usize {
                0
            }
        }
        )+
    );
);

malloc_size_of_is_0!(bool, char, str);
malloc_size_of_is_0!(u8, u16, u32, u64, u128, usize);
malloc_size_of_is_0!(i8, i16, i32, i64, i128, isize);
malloc_size_of_is_0!(f32, f64);

malloc_size_of_is_0!(std::sync::atomic::AtomicBool);
malloc_size_of_is_0!(std::sync::atomic::AtomicIsize);
malloc_size_of_is_0!(std::sync::atomic::AtomicUsize);

malloc_size_of_is_0!(std::num::NonZeroUsize);
malloc_size_of_is_0!(std::num::NonZeroU32);

malloc_size_of_is_0!(std::time::Duration);
malloc_size_of_is_0!(std::time::Instant);
malloc_size_of_is_0!(std::time::SystemTime);

malloc_size_of_is_0!(Range<u8>, Range<u16>, Range<u32>, Range<u64>, Range<usize>);
malloc_size_of_is_0!(Range<i8>, Range<i16>, Range<i32>, Range<i64>, Range<isize>);
malloc_size_of_is_0!(Range<f32>, Range<f64>);

malloc_size_of_is_0!(app_units::Au);
