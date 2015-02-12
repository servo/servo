/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Small vectors in various sizes. These store a certain number of elements inline and fall back
//! to the heap for larger allocations.

use std::mem::zeroed as i;
use std::cmp;
use std::fmt;
use std::intrinsics;
use std::iter::FromIterator;
use std::marker::ContravariantLifetime;
use std::mem;
use std::ptr;
use std::raw::Slice;
use alloc::heap;

// Generic code for all small vectors

pub trait VecLike<T> {
    fn vec_len(&self) -> uint;
    fn vec_push(&mut self, value: T);

    fn vec_slice_mut<'a>(&'a mut self, start: uint, end: uint) -> &'a mut [T];

    #[inline]
    fn vec_slice_from_mut<'a>(&'a mut self, start: uint) -> &'a mut [T] {
        let len = self.vec_len();
        self.vec_slice_mut(start, len)
    }
}

impl<T> VecLike<T> for Vec<T> {
    #[inline]
    fn vec_len(&self) -> uint {
        self.len()
    }

    #[inline]
    fn vec_push(&mut self, value: T) {
        self.push(value);
    }

    #[inline]
    fn vec_slice_mut<'a>(&'a mut self, start: uint, end: uint) -> &'a mut [T] {
        self.slice_mut(start, end)
    }
}

pub trait SmallVecPrivate<T> {
    unsafe fn set_len(&mut self, new_len: uint);
    unsafe fn set_cap(&mut self, new_cap: uint);
    fn data(&self, index: uint) -> *const T;
    fn mut_data(&mut self, index: uint) -> *mut T;
    unsafe fn ptr(&self) -> *const T;
    unsafe fn mut_ptr(&mut self) -> *mut T;
    unsafe fn set_ptr(&mut self, new_ptr: *mut T);
}

pub trait SmallVec<T> : SmallVecPrivate<T> {
    fn inline_size(&self) -> uint;
    fn len(&self) -> uint;
    fn is_empty(&self) -> bool;
    fn cap(&self) -> uint;

    fn spilled(&self) -> bool {
        self.cap() > self.inline_size()
    }

    fn begin(&self) -> *const T {
        unsafe {
            if self.spilled() {
                self.ptr()
            } else {
                self.data(0)
            }
        }
    }

    fn end(&self) -> *const T {
        unsafe {
            self.begin().offset(self.len() as int)
        }
    }

    fn iter<'a>(&'a self) -> SmallVecIterator<'a,T> {
        SmallVecIterator {
            ptr: self.begin(),
            end: self.end(),
            lifetime: ContravariantLifetime::<'a>,
        }
    }

    fn mut_iter<'a>(&'a mut self) -> SmallVecMutIterator<'a,T> {
        unsafe {
            SmallVecMutIterator {
                ptr: mem::transmute(self.begin()),
                end: mem::transmute(self.end()),
                lifetime: ContravariantLifetime::<'a>,
            }
        }
    }

    /// NB: For efficiency reasons (avoiding making a second copy of the inline elements), this
    /// actually clears out the original array instead of moving it.
    fn into_iter<'a>(&'a mut self) -> SmallVecMoveIterator<'a,T> {
        unsafe {
            let iter = mem::transmute(self.iter());
            let ptr_opt = if self.spilled() {
                Some(mem::transmute(self.ptr()))
            } else {
                None
            };
            let cap = self.cap();
            let inline_size = self.inline_size();
            self.set_cap(inline_size);
            self.set_len(0);
            SmallVecMoveIterator {
                allocation: ptr_opt,
                cap: cap,
                iter: iter,
                lifetime: ContravariantLifetime::<'a>,
            }
        }
    }

    fn push(&mut self, value: T) {
        let cap = self.cap();
        if self.len() == cap {
            self.grow(cmp::max(cap * 2, 1))
        }
        unsafe {
            let end: &mut T = mem::transmute(self.end());
            ptr::write(end, value);
            let len = self.len();
            self.set_len(len + 1)
        }
    }

    fn push_all_move<V:SmallVec<T>>(&mut self, mut other: V) {
        for value in other.into_iter() {
            self.push(value)
        }
    }

    fn pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None
        }

        unsafe {
            let mut value: T = mem::uninitialized();
            let last_index = self.len() - 1;

            if (last_index as int) < 0 {
                panic!("overflow")
            }
            let end_ptr = self.begin().offset(last_index as int);

            mem::swap(&mut value, mem::transmute::<*const T,&mut T>(end_ptr));
            self.set_len(last_index);
            Some(value)
        }
    }

    fn grow(&mut self, new_cap: uint) {
        unsafe {
            let new_alloc: *mut T = mem::transmute(heap::allocate(mem::size_of::<T>() *
                                                                            new_cap,
                                                                  mem::min_align_of::<T>()));
            ptr::copy_nonoverlapping_memory(new_alloc, self.begin(), self.len());

            if self.spilled() {
                heap::deallocate(self.mut_ptr() as *mut u8,
                                 mem::size_of::<T>() * self.cap(),
                                 mem::min_align_of::<T>())
            } else {
                let mut_begin: *mut T = mem::transmute(self.begin());
                intrinsics::set_memory(mut_begin, 0, self.len())
            }

            self.set_ptr(new_alloc);
            self.set_cap(new_cap)
        }
    }

    fn get<'a>(&'a self, index: uint) -> &'a T {
        if index >= self.len() {
            self.fail_bounds_check(index)
        }
        unsafe {
            mem::transmute(self.begin().offset(index as int))
        }
    }

    fn get_mut<'a>(&'a mut self, index: uint) -> &'a mut T {
        if index >= self.len() {
            self.fail_bounds_check(index)
        }
        unsafe {
            mem::transmute(self.begin().offset(index as int))
        }
    }

    fn slice<'a>(&'a self, start: uint, end: uint) -> &'a [T] {
        assert!(start <= end);
        assert!(end <= self.len());
        unsafe {
            mem::transmute(Slice {
                data: self.begin().offset(start as int),
                len: (end - start)
            })
        }
    }

    fn as_slice<'a>(&'a self) -> &'a [T] {
        self.slice(0, self.len())
    }

    fn as_slice_mut<'a>(&'a mut self) -> &'a mut [T] {
        let len = self.len();
        self.slice_mut(0, len)
    }

    fn slice_mut<'a>(&'a mut self, start: uint, end: uint) -> &'a mut [T] {
        assert!(start <= end);
        assert!(end <= self.len());
        unsafe {
            mem::transmute(Slice {
                data: self.begin().offset(start as int),
                len: (end - start)
            })
        }
    }

    fn slice_from_mut<'a>(&'a mut self, start: uint) -> &'a mut [T] {
        let len = self.len();
        self.slice_mut(start, len)
    }

    fn fail_bounds_check(&self, index: uint) {
        panic!("index {} beyond length ({})", index, self.len())
    }
}

pub struct SmallVecIterator<'a,T> {
    ptr: *const T,
    end: *const T,
    lifetime: ContravariantLifetime<'a>
}

impl<'a,T> Iterator for SmallVecIterator<'a,T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        unsafe {
            if self.ptr == self.end {
                return None
            }
            let old = self.ptr;
            self.ptr = if mem::size_of::<T>() == 0 {
                mem::transmute(self.ptr as uint + 1)
            } else {
                self.ptr.offset(1)
            };
            Some(mem::transmute(old))
        }
    }
}

pub struct SmallVecMutIterator<'a,T> {
    ptr: *mut T,
    end: *mut T,
    lifetime: ContravariantLifetime<'a>,
}

impl<'a,T> Iterator for SmallVecMutIterator<'a,T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        unsafe {
            if self.ptr == self.end {
                return None
            }
            let old = self.ptr;
            self.ptr = if mem::size_of::<T>() == 0 {
                mem::transmute(self.ptr as uint + 1)
            } else {
                self.ptr.offset(1)
            };
            Some(mem::transmute(old))
        }
    }
}

pub struct SmallVecMoveIterator<'a,T> {
    allocation: Option<*mut u8>,
    cap: uint,
    iter: SmallVecIterator<'a,T>,
    lifetime: ContravariantLifetime<'a>,
}

impl<'a, T: 'a> Iterator for SmallVecMoveIterator<'a,T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        unsafe {
            match self.iter.next() {
                None => None,
                Some(reference) => {
                    // Zero out the values as we go so they don't get double-freed.
                    let reference: &mut T = mem::transmute(reference);
                    Some(mem::replace(reference, mem::zeroed()))
                }
            }
        }
    }
}

#[unsafe_destructor]
impl<'a, T: 'a> Drop for SmallVecMoveIterator<'a,T> {
    fn drop(&mut self) {
        // Destroy the remaining elements.
        for _ in self.by_ref() {}

        match self.allocation {
            None => {}
            Some(allocation) => {
                unsafe {
                    heap::deallocate(allocation as *mut u8,
                                     mem::size_of::<T>() * self.cap,
                                     mem::min_align_of::<T>())
                }
            }
        }
    }
}

// Concrete implementations

macro_rules! def_small_vector(
    ($name:ident, $size:expr) => (
        pub struct $name<T> {
            len: uint,
            cap: uint,
            ptr: *const T,
            data: [T; $size],
        }

        impl<T> SmallVecPrivate<T> for $name<T> {
            unsafe fn set_len(&mut self, new_len: uint) {
                self.len = new_len
            }
            unsafe fn set_cap(&mut self, new_cap: uint) {
                self.cap = new_cap
            }
            fn data(&self, index: uint) -> *const T {
                let ptr: *const T = &self.data[index];
                ptr
            }
            fn mut_data(&mut self, index: uint) -> *mut T {
                let ptr: *mut T = &mut self.data[index];
                ptr
            }
            unsafe fn ptr(&self) -> *const T {
                self.ptr
            }
            unsafe fn mut_ptr(&mut self) -> *mut T {
                mem::transmute(self.ptr)
            }
            unsafe fn set_ptr(&mut self, new_ptr: *mut T) {
                self.ptr = mem::transmute(new_ptr)
            }
        }

        impl<T> SmallVec<T> for $name<T> {
            fn inline_size(&self) -> uint {
                $size
            }
            fn len(&self) -> uint {
                self.len
            }
            fn is_empty(&self) -> bool {
                self.len == 0
            }
            fn cap(&self) -> uint {
                self.cap
            }
        }

        impl<T> VecLike<T> for $name<T> {
            #[inline]
            fn vec_len(&self) -> uint {
                self.len()
            }

            #[inline]
            fn vec_push(&mut self, value: T) {
                self.push(value);
            }

            #[inline]
            fn vec_slice_mut<'a>(&'a mut self, start: uint, end: uint) -> &'a mut [T] {
                self.slice_mut(start, end)
            }
        }

        impl<T> FromIterator<T> for $name<T> {
            fn from_iter<I: Iterator<Item=T>>(mut iter: I) -> $name<T> {
                let mut v = $name::new();

                let (lower_size_bound, _) = iter.size_hint();

                if lower_size_bound > v.cap() {
                    v.grow(lower_size_bound);
                }

                for elem in iter {
                    v.push(elem);
                }

                v
            }
        }

        impl<T> $name<T> {
            pub fn extend<I: Iterator<Item=T>>(&mut self, mut iter: I) {
                let (lower_size_bound, _) = iter.size_hint();

                let target_len = self.len() + lower_size_bound;

                if target_len > self.cap() {
                   self.grow(target_len);
                }

                for elem in iter {
                    self.push(elem);
                }
            }
        }

        impl<T: fmt::Debug> fmt::Debug for $name<T> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{:?}", self.as_slice())
            }
        }

        impl<T> $name<T> {
            #[inline]
            pub fn new() -> $name<T> {
                unsafe {
                    $name {
                        len: 0,
                        cap: $size,
                        ptr: ptr::null(),
                        data: mem::zeroed(),
                    }
                }
            }
        }
    )
);

def_small_vector!(SmallVec1, 1);
def_small_vector!(SmallVec2, 2);
def_small_vector!(SmallVec4, 4);
def_small_vector!(SmallVec8, 8);
def_small_vector!(SmallVec16, 16);
def_small_vector!(SmallVec24, 24);
def_small_vector!(SmallVec32, 32);

macro_rules! def_small_vector_drop_impl(
    ($name:ident, $size:expr) => (
        #[unsafe_destructor]
        impl<T> Drop for $name<T> {
            fn drop(&mut self) {
                if !self.spilled() {
                    return
                }

                unsafe {
                    let ptr = self.mut_ptr();
                    for i in range(0, self.len()) {
                        *ptr.offset(i as int) = mem::uninitialized();
                    }

                    heap::deallocate(self.mut_ptr() as *mut u8,
                                     mem::size_of::<T>() * self.cap(),
                                     mem::min_align_of::<T>())
                }
            }
        }
    )
);

def_small_vector_drop_impl!(SmallVec1, 1);
def_small_vector_drop_impl!(SmallVec2, 2);
def_small_vector_drop_impl!(SmallVec4, 4);
def_small_vector_drop_impl!(SmallVec8, 8);
def_small_vector_drop_impl!(SmallVec16, 16);
def_small_vector_drop_impl!(SmallVec24, 24);
def_small_vector_drop_impl!(SmallVec32, 32);

macro_rules! def_small_vector_clone_impl(
    ($name:ident) => (
        impl<T: Clone> Clone for $name<T> {
            fn clone(&self) -> $name<T> {
                let mut new_vector = $name::new();
                for element in self.iter() {
                    new_vector.push((*element).clone())
                }
                new_vector
            }
        }
    )
);

def_small_vector_clone_impl!(SmallVec1);
def_small_vector_clone_impl!(SmallVec2);
def_small_vector_clone_impl!(SmallVec4);
def_small_vector_clone_impl!(SmallVec8);
def_small_vector_clone_impl!(SmallVec16);
def_small_vector_clone_impl!(SmallVec24);
def_small_vector_clone_impl!(SmallVec32);

#[cfg(test)]
pub mod tests {
    use smallvec::{SmallVec, SmallVec2, SmallVec16};
    use std::borrow::ToOwned;

    // We heap allocate all these strings so that double frees will show up under valgrind.

    #[test]
    pub fn test_inline() {
        let mut v = SmallVec16::new();
        v.push("hello".to_owned());
        v.push("there".to_owned());
        assert_eq!(v.as_slice(), vec![
            "hello".to_owned(),
            "there".to_owned(),
        ].as_slice());
    }

    #[test]
    pub fn test_spill() {
        let mut v = SmallVec2::new();
        v.push("hello".to_owned());
        v.push("there".to_owned());
        v.push("burma".to_owned());
        v.push("shave".to_owned());
        assert_eq!(v.as_slice(), vec![
            "hello".to_owned(),
            "there".to_owned(),
            "burma".to_owned(),
            "shave".to_owned(),
        ].as_slice());
    }

    #[test]
    pub fn test_double_spill() {
        let mut v = SmallVec2::new();
        v.push("hello".to_owned());
        v.push("there".to_owned());
        v.push("burma".to_owned());
        v.push("shave".to_owned());
        v.push("hello".to_owned());
        v.push("there".to_owned());
        v.push("burma".to_owned());
        v.push("shave".to_owned());
        assert_eq!(v.as_slice(), vec![
            "hello".to_owned(),
            "there".to_owned(),
            "burma".to_owned(),
            "shave".to_owned(),
            "hello".to_owned(),
            "there".to_owned(),
            "burma".to_owned(),
            "shave".to_owned(),
        ].as_slice());
    }
}
