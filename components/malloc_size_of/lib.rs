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
//! mozjemalloc and DMD.
//!
//! This crate has a lot of overlap with the existing `heapsize` crate, and may
//! one day be merged into it. But for now, `heapsize` has the following
//! major shortcomings.
//! - It basically assumes that the `HeapSizeOf` trait can be used for every
//!   type, which is not true. Sometimes more than a single size measurement
//!   needs to be returned for a type, and sometimes additional synchronization
//!   arguments (such as lock guards) need to be passed in.
//! - It has no proper way of measuring some common types, such as `HashSet`
//!   and `HashMap`, that don't expose internal pointers.
//! - It has no proper way of handling values with multiple referents, such as
//!   `Rc` and `Arc`.
//!
//! This crate solves those problems.
//! - It provides traits for both "shallow" and "deep" measurement, which gives
//!   more flexibility in the cases where the traits can't be used.
//! - It allows for measuring blocks even when only an interior pointer can be
//!   obtained for heap allocations, e.g. `HashSet` and `HashMap`. (This relies
//!   on the heap allocator having suitable support, which mozjemalloc has.)
//! - It allows handling of types like `Rc` and `Arc` by providing special
//!   traits that are different to the ones for non-graph structures.
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
extern crate servo_arc;
extern crate smallbitvec;
extern crate smallvec;

use std::hash::{BuildHasher, Hash};
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

    /// Like `size_of_op`, but can take an interior pointer.
    enclosing_size_of_op: VoidPtrToSizeFn,

    /// Check if a pointer has been seen before, and remember it for next time.
    /// Useful when measuring `Rc`s and `Arc`s. Optional, because many places
    /// don't need it.
    have_seen_ptr_op: Option<Box<VoidPtrToBoolFnMut>>,
}

impl MallocSizeOfOps {
    pub fn new(size_of: VoidPtrToSizeFn, malloc_enclosing_size_of: VoidPtrToSizeFn,
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

    /// Call `enclosing_size_of_op` on `ptr`, which must not be empty.
    pub unsafe fn malloc_enclosing_size_of<T>(&self, ptr: *const T) -> usize {
        assert!(!MallocSizeOfOps::is_empty(ptr));
        (self.enclosing_size_of_op)(ptr as *const c_void)
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

impl<A: MallocSizeOf, B: MallocSizeOf> MallocSizeOf for (A, B) {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops) + self.1.size_of(ops)
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
        // The first value from the iterator gives us an interior pointer.
        // `ops.malloc_enclosing_size_of()` then gives us the storage size.
        // This assumes that the `HashSet`'s contents (values and hashes) are
        // all stored in a single contiguous heap allocation.
        self.iter().next().map_or(0, |t| unsafe { ops.malloc_enclosing_size_of(t) })
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
        self.iter().next().map_or(0, |t| unsafe { ops.malloc_enclosing_size_of(t) })
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

impl<K, V, S> MallocShallowSizeOf for hashglobe::hash_map::HashMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher
{
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        // See the implementation for std::collections::HashSet for details.
        self.values().next().map_or(0, |v| unsafe { ops.malloc_enclosing_size_of(v) })
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

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::TypedSize2D<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.width.size_of(ops) + self.height.size_of(ops)
    }
}

/// For use on types where size_of() returns 0.
#[macro_export]
macro_rules! size_of_is_0(
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

size_of_is_0!(bool, char, str);
size_of_is_0!(u8, u16, u32, u64, usize);
size_of_is_0!(i8, i16, i32, i64, isize);
size_of_is_0!(f32, f64);

size_of_is_0!(Range<u8>, Range<u16>, Range<u32>, Range<u64>, Range<usize>);
size_of_is_0!(Range<i8>, Range<i16>, Range<i32>, Range<i64>, Range<isize>);
size_of_is_0!(Range<f32>, Range<f64>);

size_of_is_0!(app_units::Au);
size_of_is_0!(cssparser::RGBA, cssparser::TokenSerializationType);
