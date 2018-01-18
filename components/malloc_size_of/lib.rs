// Copyright 2016-2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A crate for measuring the heap usage of data structures in a way that
//! integrates with Firefox's memory reporting, particularly the use of
//! mozjemalloc and DMD. In particular, it has the following features.
//! - It isn't bound to a particular heap allocator.
//! - It provides traits for both "shallow" and "deep" measurement, which gives
//!   flexibility in the cases where the traits can't be used.
//! - It allows for measuring blocks even when only an interior pointer can be
//!   obtained for heap allocations, e.g. `HashSet` and `HashMap`. (This relies
//!   on the heap allocator having suitable support, which mozjemalloc has.)
//! - It allows handling of types like `Rc` and `Arc` by providing traits that
//!   are different to the ones for non-graph structures.
//!
//! Suggested uses are as follows.
//! - When possible, use the `MallocSizeOf` trait. (Deriving support is
//!   provided by the `malloc_size_of_derive` crate.)
//! - If you need an additional synchronization argument, provide a function
//!   that is like the standard trait method, but with the extra argument.
//! - If you need multiple measurements for a type, provide a function named
//!   `add_size_of` that takes a mutable reference to a struct that contains
//!   the multiple measurement fields.
//! - When deep measurement (via `MallocSizeOf`) cannot be implemented for a
//!   type, shallow measurement (via `MallocShallowSizeOf`) in combination with
//!   iteration can be a useful substitute.
//! - `Rc` and `Arc` are always tricky, which is why `MallocSizeOf` is not (and
//!   should not be) implemented for them.
//! - If an `Rc` or `Arc` is known to be a "primary" reference and can always
//!   be measured, it should be measured via the `MallocUnconditionalSizeOf`
//!   trait.
//! - If an `Rc` or `Arc` should be measured only if it hasn't been seen
//!   before, it should be measured via the `MallocConditionalSizeOf` trait.
//! - Using universal function call syntax is a good idea when measuring boxed
//!   fields in structs, because it makes it clear that the Box is being
//!   measured as well as the thing it points to. E.g.
//!   `<Box<_> as MallocSizeOf>::size_of(field, ops)`.

extern crate app_units;
extern crate cssparser;
extern crate euclid;
extern crate hashglobe;
#[cfg(feature = "servo")]
extern crate mozjs as js;
extern crate selectors;
extern crate servo_arc;
extern crate smallbitvec;
extern crate smallvec;
#[cfg(feature = "servo")]
extern crate string_cache;
#[cfg(feature = "servo")]
extern crate url;
#[cfg(feature = "servo")]
extern crate webrender_api;
#[cfg(feature = "servo")]
extern crate xml5ever;

use std::hash::{BuildHasher, Hash};
use std::mem::size_of;
use std::ops::Range;
use std::os::raw::c_void;

/// A C function that takes a pointer to a heap allocation and returns its size.
type VoidPtrToSizeFn = unsafe extern "C" fn(ptr: *const c_void) -> usize;

/// A closure implementing a stateful predicate on pointers.
type VoidPtrToBoolFnMut = FnMut(*const c_void) -> bool;

/// Operations used when measuring heap usage of data structures.
pub struct MallocSizeOfOps {
    /// A function that returns the size of a heap allocation.
    size_of_op: VoidPtrToSizeFn,

    /// Like `size_of_op`, but can take an interior pointer. Optional because
    /// not all allocators support this operation. If it's not provided, some
    /// memory measurements will actually be computed estimates rather than
    /// real and accurate measurements.
    enclosing_size_of_op: Option<VoidPtrToSizeFn>,

    /// Check if a pointer has been seen before, and remember it for next time.
    /// Useful when measuring `Rc`s and `Arc`s. Optional, because many places
    /// don't need it.
    have_seen_ptr_op: Option<Box<VoidPtrToBoolFnMut>>,
}

impl MallocSizeOfOps {
    pub fn new(size_of: VoidPtrToSizeFn,
               malloc_enclosing_size_of: Option<VoidPtrToSizeFn>,
               have_seen_ptr: Option<Box<VoidPtrToBoolFnMut>>) -> Self {
        MallocSizeOfOps {
            size_of_op: size_of,
            enclosing_size_of_op: malloc_enclosing_size_of,
            have_seen_ptr_op: have_seen_ptr,
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
        return ptr as *const usize as usize <= 256
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

    /// Call `have_seen_ptr_op` on `ptr`.
    pub fn have_seen_ptr<T>(&mut self, ptr: *const T) -> bool {
        let have_seen_ptr_op = self.have_seen_ptr_op.as_mut().expect("missing have_seen_ptr_op");
        have_seen_ptr_op(ptr as *const c_void)
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

/// Like `MallocSizeOf`, but with a different name so it cannot be used
/// accidentally with derive(MallocSizeOf). For use with types like `Rc` and
/// `Arc` when appropriate (e.g. when measuring a "primary" reference).
pub trait MallocUnconditionalSizeOf {
    /// Measure the heap usage of all heap-allocated descendant structures, but
    /// not the space taken up by the value itself.
    fn unconditional_size_of(&self, ops: &mut MallocSizeOfOps) -> usize;
}

/// `MallocUnconditionalSizeOf` combined with `MallocShallowSizeOf`.
pub trait MallocUnconditionalShallowSizeOf {
    /// `unconditional_size_of` combined with `shallow_size_of`.
    fn unconditional_shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize;
}

/// Like `MallocSizeOf`, but only measures if the value hasn't already been
/// measured. For use with types like `Rc` and `Arc` when appropriate (e.g.
/// when there is no "primary" reference).
pub trait MallocConditionalSizeOf {
    /// Measure the heap usage of all heap-allocated descendant structures, but
    /// not the space taken up by the value itself, and only if that heap usage
    /// hasn't already been measured.
    fn conditional_size_of(&self, ops: &mut MallocSizeOfOps) -> usize;
}

/// `MallocConditionalSizeOf` combined with `MallocShallowSizeOf`.
pub trait MallocConditionalShallowSizeOf {
    /// `conditional_size_of` combined with `shallow_size_of`.
    fn conditional_shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize;
}

impl MallocSizeOf for String {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        unsafe { ops.malloc_size_of(self.as_ptr()) }
    }
}

impl<'a, T: ?Sized> MallocSizeOf for &'a T {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // Zero makes sense for a non-owning reference.
        0
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
    where T1: MallocSizeOf, T2: MallocSizeOf
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops) + self.1.size_of(ops)
    }
}

impl<T1, T2, T3> MallocSizeOf for (T1, T2, T3)
    where T1: MallocSizeOf, T2: MallocSizeOf, T3: MallocSizeOf
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops) + self.1.size_of(ops) + self.2.size_of(ops)
    }
}

impl<T1, T2, T3, T4> MallocSizeOf for (T1, T2, T3, T4)
    where T1: MallocSizeOf, T2: MallocSizeOf, T3: MallocSizeOf, T4: MallocSizeOf
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
    where B::Owned: MallocSizeOf
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

impl<T> MallocShallowSizeOf for std::collections::VecDeque<T> {
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if ops.has_malloc_enclosing_size_of() {
            if let Some(front) = self.front() {
                // The front element is an interior pointer.
                unsafe { ops.malloc_enclosing_size_of(&*front) }
            } else {
                // This assumes that no memory is allocated when the VecDeque is empty.
                0
            }
        } else {
            // An estimate.
            self.capacity() * size_of::<T>()
        }
    }
}

impl<T: MallocSizeOf> MallocSizeOf for std::collections::VecDeque<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = self.shallow_size_of(ops);
        for elem in self.iter() {
            n += elem.size_of(ops);
        }
        n
    }
}

impl<A: smallvec::Array> MallocShallowSizeOf for smallvec::SmallVec<A> {
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if self.spilled() {
            unsafe { ops.malloc_size_of(self.as_ptr()) }
        } else {
            0
        }
    }
}

impl<A> MallocSizeOf for smallvec::SmallVec<A>
    where A: smallvec::Array,
          A::Item: MallocSizeOf
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = self.shallow_size_of(ops);
        for elem in self.iter() {
            n += elem.size_of(ops);
        }
        n
    }
}

impl<T, S> MallocShallowSizeOf for std::collections::HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if ops.has_malloc_enclosing_size_of() {
            // The first value from the iterator gives us an interior pointer.
            // `ops.malloc_enclosing_size_of()` then gives us the storage size.
            // This assumes that the `HashSet`'s contents (values and hashes)
            // are all stored in a single contiguous heap allocation.
            self.iter().next().map_or(0, |t| unsafe { ops.malloc_enclosing_size_of(t) })
        } else {
            // An estimate.
            self.capacity() * (size_of::<T>() + size_of::<usize>())
        }
    }
}

impl<T, S> MallocSizeOf for std::collections::HashSet<T, S>
    where T: Eq + Hash + MallocSizeOf,
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

impl<T, S> MallocShallowSizeOf for hashglobe::hash_set::HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher
{
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        // See the implementation for std::collections::HashSet for details.
        if ops.has_malloc_enclosing_size_of() {
            self.iter().next().map_or(0, |t| unsafe { ops.malloc_enclosing_size_of(t) })
        } else {
            self.capacity() * (size_of::<T>() + size_of::<usize>())
        }
    }
}

impl<T, S> MallocSizeOf for hashglobe::hash_set::HashSet<T, S>
    where T: Eq + Hash + MallocSizeOf,
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

impl<T, S> MallocShallowSizeOf for hashglobe::fake::HashSet<T, S>
    where T: Eq + Hash,
          S: BuildHasher,
{
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        use std::ops::Deref;
        self.deref().shallow_size_of(ops)
    }
}

impl<T, S> MallocSizeOf for hashglobe::fake::HashSet<T, S>
    where T: Eq + Hash + MallocSizeOf,
          S: BuildHasher,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        use std::ops::Deref;
        self.deref().size_of(ops)
    }
}

impl<K, V, S> MallocShallowSizeOf for std::collections::HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        // See the implementation for std::collections::HashSet for details.
        if ops.has_malloc_enclosing_size_of() {
            self.values().next().map_or(0, |v| unsafe { ops.malloc_enclosing_size_of(v) })
        } else {
            self.capacity() * (size_of::<V>() + size_of::<K>() + size_of::<usize>())
        }
    }
}

impl<K, V, S> MallocSizeOf for std::collections::HashMap<K, V, S>
    where K: Eq + Hash + MallocSizeOf,
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

impl<K, V, S> MallocShallowSizeOf for hashglobe::hash_map::HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        // See the implementation for std::collections::HashSet for details.
        if ops.has_malloc_enclosing_size_of() {
            self.values().next().map_or(0, |v| unsafe { ops.malloc_enclosing_size_of(v) })
        } else {
            self.capacity() * (size_of::<V>() + size_of::<K>() + size_of::<usize>())
        }
    }
}

impl<K, V, S> MallocSizeOf for hashglobe::hash_map::HashMap<K, V, S>
    where K: Eq + Hash + MallocSizeOf,
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

impl<K, V, S> MallocShallowSizeOf for hashglobe::fake::HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher,
{
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        use std::ops::Deref;
        self.deref().shallow_size_of(ops)
    }
}

impl<K, V, S> MallocSizeOf for hashglobe::fake::HashMap<K, V, S>
    where K: Eq + Hash + MallocSizeOf,
          V: MallocSizeOf,
          S: BuildHasher,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        use std::ops::Deref;
        self.deref().size_of(ops)
    }
}

// PhantomData is always 0.
impl<T> MallocSizeOf for std::marker::PhantomData<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

// XXX: we don't want MallocSizeOf to be defined for Rc and Arc. If negative
// trait bounds are ever allowed, this code should be uncommented.
// (We do have a compile-fail test for this:
// rc_arc_must_not_derive_malloc_size_of.rs)
//impl<T> !MallocSizeOf for Arc<T> { }
//impl<T> !MallocShallowSizeOf for Arc<T> { }

impl<T> MallocUnconditionalShallowSizeOf for servo_arc::Arc<T> {
    fn unconditional_shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        unsafe { ops.malloc_size_of(self.heap_ptr()) }
    }
}

impl<T: MallocSizeOf> MallocUnconditionalSizeOf for servo_arc::Arc<T> {
    fn unconditional_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.unconditional_shallow_size_of(ops) + (**self).size_of(ops)
    }
}

impl<T> MallocConditionalShallowSizeOf for servo_arc::Arc<T> {
    fn conditional_shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if ops.have_seen_ptr(self.heap_ptr()) {
            0
        } else {
            self.unconditional_shallow_size_of(ops)
        }
    }
}

impl<T: MallocSizeOf> MallocConditionalSizeOf for servo_arc::Arc<T> {
    fn conditional_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if ops.have_seen_ptr(self.heap_ptr()) {
            0
        } else {
            self.unconditional_size_of(ops)
        }
    }
}

impl MallocSizeOf for smallbitvec::SmallBitVec {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if let Some(ptr) = self.heap_ptr() {
            unsafe { ops.malloc_size_of(ptr) }
        } else {
            0
        }
    }
}

impl<T: MallocSizeOf, Unit> MallocSizeOf for euclid::Length<T, Unit> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops)
    }
}

impl<T: MallocSizeOf, Src, Dst> MallocSizeOf for euclid::TypedScale<T, Src, Dst> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops)
    }
}

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::TypedPoint2D<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.x.size_of(ops) + self.y.size_of(ops)
    }
}

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::TypedRect<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.origin.size_of(ops) + self.size.size_of(ops)
    }
}

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::TypedSideOffsets2D<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.top.size_of(ops) + self.right.size_of(ops) +
        self.bottom.size_of(ops) + self.left.size_of(ops)
    }
}

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::TypedSize2D<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.width.size_of(ops) + self.height.size_of(ops)
    }
}

impl<T: MallocSizeOf, Src, Dst> MallocSizeOf for euclid::TypedTransform2D<T, Src, Dst> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.m11.size_of(ops) + self.m12.size_of(ops) +
        self.m21.size_of(ops) + self.m22.size_of(ops) +
        self.m31.size_of(ops) + self.m32.size_of(ops)
    }
}

impl<T: MallocSizeOf, Src, Dst> MallocSizeOf for euclid::TypedTransform3D<T, Src, Dst> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.m11.size_of(ops) + self.m12.size_of(ops) +
        self.m13.size_of(ops) + self.m14.size_of(ops) +
        self.m21.size_of(ops) + self.m22.size_of(ops) +
        self.m23.size_of(ops) + self.m24.size_of(ops) +
        self.m31.size_of(ops) + self.m32.size_of(ops) +
        self.m33.size_of(ops) + self.m34.size_of(ops) +
        self.m41.size_of(ops) + self.m42.size_of(ops) +
        self.m43.size_of(ops) + self.m44.size_of(ops)
    }
}

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::TypedVector2D<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.x.size_of(ops) + self.y.size_of(ops)
    }
}

impl MallocSizeOf for selectors::parser::AncestorHashes {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let selectors::parser::AncestorHashes { ref packed_hashes } = *self;
        packed_hashes.size_of(ops)
    }
}

#[cfg(feature = "servo")]
impl<Static: string_cache::StaticAtomSet> MallocSizeOf for string_cache::Atom<Static> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

// This is measured properly by the heap measurement implemented in
// SpiderMonkey.
#[cfg(feature = "servo")]
impl<T: Copy + js::rust::GCMethods> MallocSizeOf for js::jsapi::Heap<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
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
malloc_size_of_is_0!(u8, u16, u32, u64, usize);
malloc_size_of_is_0!(i8, i16, i32, i64, isize);
malloc_size_of_is_0!(f32, f64);

malloc_size_of_is_0!(std::sync::atomic::AtomicBool);
malloc_size_of_is_0!(std::sync::atomic::AtomicIsize, std::sync::atomic::AtomicUsize);

malloc_size_of_is_0!(Range<u8>, Range<u16>, Range<u32>, Range<u64>, Range<usize>);
malloc_size_of_is_0!(Range<i8>, Range<i16>, Range<i32>, Range<i64>, Range<isize>);
malloc_size_of_is_0!(Range<f32>, Range<f64>);

malloc_size_of_is_0!(app_units::Au);

malloc_size_of_is_0!(cssparser::RGBA, cssparser::TokenSerializationType);

#[cfg(feature = "servo")]
impl MallocSizeOf for url::Host {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match *self {
            url::Host::Domain(ref s) => s.size_of(ops),
            _ => 0,
        }
    }
}

#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::BorderStyle);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::BoxShadowClipMode);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::ClipAndScrollInfo);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::ClipId);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::ColorF);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::ExtendMode);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::GradientStop);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::ImageKey);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::ImageRendering);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::LineStyle);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::LocalClip);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::MixBlendMode);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::NormalBorder);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::RepeatMode);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::ScrollPolicy);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::ScrollSensitivity);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::StickyOffsetBounds);
#[cfg(feature = "servo")]
malloc_size_of_is_0!(webrender_api::TransformStyle);

#[cfg(feature = "servo")]
impl MallocSizeOf for xml5ever::QualName {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.prefix.size_of(ops) +
        self.ns.size_of(ops) +
        self.local.size_of(ops)
    }
}
