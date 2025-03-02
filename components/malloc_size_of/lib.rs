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
//!
//!   Note: WebRender has a reduced fork of this crate, so that we can avoid
//!   publishing this crate on crates.io.

use std::cell::OnceCell;
use std::collections::BinaryHeap;
use std::hash::{BuildHasher, Hash};
use std::ops::Range;
use std::sync::Arc;

pub use style_malloc_size_of::MallocSizeOfOps;
use uuid::Uuid;

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

impl<T: MallocSizeOf> MallocSizeOf for [T] {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = 0;
        for elem in self.iter() {
            n += elem.size_of(ops);
        }
        n
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

impl MallocSizeOf for keyboard_types::Key {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match &self {
            keyboard_types::Key::Character(string) => {
                <String as MallocSizeOf>::size_of(string, ops)
            },
            _ => 0,
        }
    }
}

impl MallocSizeOf for markup5ever::QualName {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.prefix.size_of(ops) + self.ns.size_of(ops) + self.local.size_of(ops)
    }
}

impl MallocSizeOf for String {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        unsafe { ops.malloc_size_of(self.as_ptr()) }
    }
}

impl<T: ?Sized> MallocSizeOf for &'_ T {
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

impl<B: ?Sized + ToOwned> MallocSizeOf for std::borrow::Cow<'_, B>
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
                unsafe { ops.malloc_enclosing_size_of(front) }
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
where
    A: smallvec::Array,
    A::Item: MallocSizeOf,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = self.shallow_size_of(ops);
        for elem in self.iter() {
            n += elem.size_of(ops);
        }
        n
    }
}

impl<T> MallocShallowSizeOf for thin_vec::ThinVec<T> {
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if self.capacity() == 0 {
            // If it's the singleton we might not be a heap pointer.
            return 0;
        }

        assert_eq!(
            std::mem::size_of::<Self>(),
            std::mem::size_of::<*const ()>()
        );
        unsafe { ops.malloc_size_of(*(self as *const Self as *const *const ())) }
    }
}

impl<T: MallocSizeOf> MallocSizeOf for thin_vec::ThinVec<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = self.shallow_size_of(ops);
        for elem in self.iter() {
            n += elem.size_of(ops);
        }
        n
    }
}

impl<T: MallocSizeOf> MallocSizeOf for BinaryHeap<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.iter().map(|element| element.size_of(ops)).sum()
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

impl<K, V> MallocShallowSizeOf for std::collections::BTreeMap<K, V>
where
    K: Eq + Hash,
{
    fn shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if ops.has_malloc_enclosing_size_of() {
            self.values()
                .next()
                .map_or(0, |v| unsafe { ops.malloc_enclosing_size_of(v) })
        } else {
            self.len() * (size_of::<V>() + size_of::<K>() + size_of::<usize>())
        }
    }
}

impl<K, V> MallocSizeOf for std::collections::BTreeMap<K, V>
where
    K: Eq + Hash + MallocSizeOf,
    V: MallocSizeOf,
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

// PhantomData is always 0.
impl<T> MallocSizeOf for std::marker::PhantomData<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

impl<T: MallocSizeOf> MallocSizeOf for OnceCell<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.get()
            .map(|interior| interior.size_of(ops))
            .unwrap_or_default()
    }
}

// See https://github.com/rust-lang/rust/issues/68318:
// We don't want MallocSizeOf to be defined for Rc and Arc. If negative trait bounds are
// ever allowed, this code should be uncommented.  Instead, there is a compile-fail test for
// this.
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

impl<T> MallocUnconditionalShallowSizeOf for Arc<T> {
    fn unconditional_shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        unsafe { ops.malloc_size_of(Arc::as_ptr(self)) }
    }
}

impl<T: MallocSizeOf> MallocUnconditionalSizeOf for Arc<T> {
    fn unconditional_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.unconditional_shallow_size_of(ops) + (**self).size_of(ops)
    }
}

impl<T> MallocConditionalShallowSizeOf for Arc<T> {
    fn conditional_shallow_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if ops.have_seen_ptr(Arc::as_ptr(self)) {
            0
        } else {
            self.unconditional_shallow_size_of(ops)
        }
    }
}

impl<T: MallocSizeOf> MallocConditionalSizeOf for Arc<T> {
    fn conditional_size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if ops.have_seen_ptr(Arc::as_ptr(self)) {
            0
        } else {
            self.unconditional_size_of(ops)
        }
    }
}

/// If a mutex is stored directly as a member of a data type that is being measured,
/// it is the unique owner of its contents and deserves to be measured.
///
/// If a mutex is stored inside of an Arc value as a member of a data type that is being measured,
/// the Arc will not be automatically measured so there is no risk of overcounting the mutex's
/// contents.
impl<T: MallocSizeOf> MallocSizeOf for std::sync::Mutex<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        (*self.lock().unwrap()).size_of(ops)
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

impl MallocSizeOf for url::Host {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match *self {
            url::Host::Domain(ref s) => s.size_of(ops),
            _ => 0,
        }
    }
}

impl MallocSizeOf for url::Url {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        // TODO: This is an estimate, but a real size should be calculated in `rust-url` once
        // it has support for `malloc_size_of`.
        self.to_string().size_of(ops)
    }
}

impl<T: MallocSizeOf, U> MallocSizeOf for euclid::Vector2D<T, U> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.x.size_of(ops) + self.y.size_of(ops)
    }
}

impl<Static: string_cache::StaticAtomSet> MallocSizeOf for string_cache::Atom<Static> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

// Placeholder for unique case where internals of Sender cannot be measured.
// malloc size of is 0 macro complains about type supplied!
impl<T> MallocSizeOf for crossbeam_channel::Sender<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

impl<T> MallocSizeOf for tokio::sync::mpsc::UnboundedSender<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

impl<T> MallocSizeOf for ipc_channel::ipc::IpcSender<T> {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        0
    }
}

impl<T: MallocSizeOf> MallocSizeOf for accountable_refcell::RefCell<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.borrow().size_of(ops)
    }
}

malloc_size_of_hash_map!(indexmap::IndexMap<K, V, S>);
malloc_size_of_hash_set!(indexmap::IndexSet<T, S>);

malloc_size_of_is_0!(bool, char, str);
malloc_size_of_is_0!(f32, f64);
malloc_size_of_is_0!(i8, i16, i32, i64, i128, isize);
malloc_size_of_is_0!(u8, u16, u32, u64, u128, usize);

malloc_size_of_is_0!(Range<f32>, Range<f64>);
malloc_size_of_is_0!(Range<i8>, Range<i16>, Range<i32>, Range<i64>, Range<isize>);
malloc_size_of_is_0!(Range<u8>, Range<u16>, Range<u32>, Range<u64>, Range<usize>);

malloc_size_of_is_0!(Uuid);
malloc_size_of_is_0!(content_security_policy::Destination);
malloc_size_of_is_0!(http::StatusCode);
malloc_size_of_is_0!(app_units::Au);
malloc_size_of_is_0!(keyboard_types::Modifiers);
malloc_size_of_is_0!(std::num::NonZeroU64);
malloc_size_of_is_0!(std::num::NonZeroUsize);
malloc_size_of_is_0!(std::sync::atomic::AtomicBool);
malloc_size_of_is_0!(std::sync::atomic::AtomicIsize);
malloc_size_of_is_0!(std::sync::atomic::AtomicUsize);
malloc_size_of_is_0!(std::time::Duration);
malloc_size_of_is_0!(std::time::Instant);
malloc_size_of_is_0!(std::time::SystemTime);
malloc_size_of_is_0!(style::font_face::SourceList);

macro_rules! malloc_size_of_is_webrender_malloc_size_of(
    ($($ty:ty),+) => (
        $(
            impl MallocSizeOf for $ty {
                fn size_of(&self, _: &mut MallocSizeOfOps) -> usize {
                    let mut ops = wr_malloc_size_of::MallocSizeOfOps::new(servo_allocator::usable_size, None);
                    <$ty as wr_malloc_size_of::MallocSizeOf>::size_of(self, &mut ops)
                }
            }
        )+
    );
);

malloc_size_of_is_webrender_malloc_size_of!(webrender_api::BorderRadius);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::BorderStyle);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::BoxShadowClipMode);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::ColorF);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::ExtendMode);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::FontInstanceKey);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::GlyphInstance);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::GradientStop);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::ImageKey);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::ImageRendering);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::LineStyle);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::MixBlendMode);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::NormalBorder);
malloc_size_of_is_webrender_malloc_size_of!(webrender_api::RepeatMode);

macro_rules! malloc_size_of_is_style_malloc_size_of(
    ($($ty:ty),+) => (
        $(
            impl MallocSizeOf for $ty {
                fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
                    <$ty as style_malloc_size_of::MallocSizeOf>::size_of(self, ops)
                }
            }
        )+
    );
);

impl<S> MallocSizeOf for style::author_styles::GenericAuthorStyles<S>
where
    S: style::stylesheets::StylesheetInDocument
        + std::cmp::PartialEq
        + style_malloc_size_of::MallocSizeOf,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        <style::author_styles::GenericAuthorStyles<S> as style_malloc_size_of::MallocSizeOf>::size_of(self, ops)
    }
}

impl<S> MallocSizeOf for style::stylesheet_set::DocumentStylesheetSet<S>
where
    S: style::stylesheets::StylesheetInDocument
        + std::cmp::PartialEq
        + style_malloc_size_of::MallocSizeOf,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        <style::stylesheet_set::DocumentStylesheetSet<S> as style_malloc_size_of::MallocSizeOf>::size_of(self, ops)
    }
}

malloc_size_of_is_style_malloc_size_of!(style::animation::DocumentAnimationSet);
malloc_size_of_is_style_malloc_size_of!(style::attr::AttrIdentifier);
malloc_size_of_is_style_malloc_size_of!(style::attr::AttrValue);
malloc_size_of_is_style_malloc_size_of!(style::color::AbsoluteColor);
malloc_size_of_is_style_malloc_size_of!(style::computed_values::font_variant_caps::T);
malloc_size_of_is_style_malloc_size_of!(style::dom::OpaqueNode);
malloc_size_of_is_style_malloc_size_of!(style::invalidation::element::restyle_hints::RestyleHint);
malloc_size_of_is_style_malloc_size_of!(style::media_queries::MediaList);
malloc_size_of_is_style_malloc_size_of!(style::properties::style_structs::Font);
malloc_size_of_is_style_malloc_size_of!(style::queries::values::PrefersColorScheme);
malloc_size_of_is_style_malloc_size_of!(style::selector_parser::PseudoElement);
malloc_size_of_is_style_malloc_size_of!(style::selector_parser::RestyleDamage);
malloc_size_of_is_style_malloc_size_of!(style::selector_parser::Snapshot);
malloc_size_of_is_style_malloc_size_of!(style::shared_lock::SharedRwLock);
malloc_size_of_is_style_malloc_size_of!(style::stylesheets::DocumentStyleSheet);
malloc_size_of_is_style_malloc_size_of!(style::stylist::Stylist);
malloc_size_of_is_style_malloc_size_of!(style::values::computed::FontStretch);
malloc_size_of_is_style_malloc_size_of!(style::values::computed::FontStyle);
malloc_size_of_is_style_malloc_size_of!(style::values::computed::FontWeight);
malloc_size_of_is_style_malloc_size_of!(style::values::computed::font::SingleFontFamily);
malloc_size_of_is_style_malloc_size_of!(style_dom::ElementState);
