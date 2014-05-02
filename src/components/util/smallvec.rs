/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Small vectors in various sizes. These store a certain number of elements inline and fall back
//! to the heap for larger allocations.

use i = std::mem::init;
use std::cast;
use std::cmp;
use std::intrinsics;
use std::mem;
use std::ptr;
use std::rt::global_heap;
use std::rt::local_heap;
use std::raw::Slice;

// Generic code for all small vectors

trait SmallVecPrivate<T> {
    unsafe fn set_len(&mut self, new_len: uint);
    unsafe fn set_cap(&mut self, new_cap: uint);
    fn data(&self, index: uint) -> *T;
    fn mut_data(&mut self, index: uint) -> *mut T;
    unsafe fn ptr(&self) -> *T;
    unsafe fn mut_ptr(&mut self) -> *mut T;
    unsafe fn set_ptr(&mut self, new_ptr: *mut T);
}

pub trait SmallVec<T> : SmallVecPrivate<T> {
    fn inline_size(&self) -> uint;
    fn len(&self) -> uint;
    fn cap(&self) -> uint;

    fn spilled(&self) -> bool {
        self.cap() > self.inline_size()
    }

    fn begin(&self) -> *T {
        unsafe {
            if self.spilled() {
                self.ptr()
            } else {
                self.data(0)
            }
        }
    }

    fn end(&self) -> *T {
        unsafe {
            self.begin().offset(self.len() as int)
        }
    }

    fn iter<'a>(&'a self) -> SmallVecIterator<'a,T> {
        SmallVecIterator {
            ptr: self.begin(),
            end: self.end(),
            lifetime: None,
        }
    }

    fn mut_iter<'a>(&'a mut self) -> SmallVecMutIterator<'a,T> {
        unsafe {
            SmallVecMutIterator {
                ptr: cast::transmute(self.begin()),
                end: cast::transmute(self.end()),
                lifetime: None,
            }
        }
    }

    /// NB: For efficiency reasons (avoiding making a second copy of the inline elements), this
    /// actually clears out the original array instead of moving it.
    fn move_iter<'a>(&'a mut self) -> SmallVecMoveIterator<'a,T> {
        unsafe {
            let iter = cast::transmute(self.iter());
            let ptr_opt = if self.spilled() {
                Some(cast::transmute(self.ptr()))
            } else {
                None
            };
            let inline_size = self.inline_size();
            self.set_cap(inline_size);
            self.set_len(0);
            SmallVecMoveIterator {
                allocation: ptr_opt,
                iter: iter,
                lifetime: None,
            }
        }
    }

    fn push(&mut self, value: T) {
        let cap = self.cap();
        if self.len() == cap {
            self.grow(cmp::max(cap * 2, 1))
        }
        unsafe {
            let end: &mut T = cast::transmute(self.end());
            mem::move_val_init(end, value);
            let len = self.len();
            self.set_len(len + 1)
        }
    }

    fn push_all_move<V:SmallVec<T>>(&mut self, mut other: V) {
        for value in other.move_iter() {
            self.push(value)
        }
    }

    fn pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None
        }

        unsafe {
            let mut value: T = mem::uninit();
            let last_index = self.len() - 1;

            if (last_index as int) < 0 {
                fail!("overflow")
            }
            let end_ptr = self.begin().offset(last_index as int);

            mem::swap(&mut value, cast::transmute::<*T,&mut T>(end_ptr));
            self.set_len(last_index);
            Some(value)
        }
    }

    fn grow(&mut self, new_cap: uint) {
        unsafe {
            let new_alloc: *mut T = cast::transmute(global_heap::malloc_raw(mem::size_of::<T>() *
                                                                            new_cap));
            ptr::copy_nonoverlapping_memory(new_alloc, self.begin(), self.len());

            if self.spilled() {
                if intrinsics::owns_managed::<T>() {
                    local_heap::local_free(self.ptr() as *u8)
                } else {
                    global_heap::exchange_free(self.ptr() as *u8)
                }
            } else {
                let mut_begin: *mut T = cast::transmute(self.begin());
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
            cast::transmute(self.begin().offset(index as int))
        }
    }

    fn get_mut<'a>(&'a mut self, index: uint) -> &'a mut T {
        if index >= self.len() {
            self.fail_bounds_check(index)
        }
        unsafe {
            cast::transmute(self.begin().offset(index as int))
        }
    }

    fn slice<'a>(&'a self, start: uint, end: uint) -> &'a [T] {
        assert!(start <= end);
        assert!(end <= self.len());
        unsafe {
            cast::transmute(Slice {
                data: self.begin().offset(start as int),
                len: (end - start)
            })
        }
    }

    fn as_slice<'a>(&'a self) -> &'a [T] {
        self.slice(0, self.len())
    }

    fn as_mut_slice<'a>(&'a mut self) -> &'a mut [T] {
        let len = self.len();
        self.mut_slice(0, len)
    }

    fn mut_slice<'a>(&'a mut self, start: uint, end: uint) -> &'a mut [T] {
        assert!(start <= end);
        assert!(end <= self.len());
        unsafe {
            cast::transmute(Slice {
                data: self.begin().offset(start as int),
                len: (end - start)
            })
        }
    }

    fn mut_slice_from<'a>(&'a mut self, start: uint) -> &'a mut [T] {
        let len = self.len();
        self.mut_slice(start, len)
    }

    fn fail_bounds_check(&self, index: uint) {
        fail!("index {} beyond length ({})", index, self.len())
    }
}

pub struct SmallVecIterator<'a,T> {
    ptr: *T,
    end: *T,
    lifetime: Option<&'a T>
}

impl<'a,T> Iterator<&'a T> for SmallVecIterator<'a,T> {
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        unsafe {
            if self.ptr == self.end {
                return None
            }
            let old = self.ptr;
            self.ptr = if mem::size_of::<T>() == 0 {
                cast::transmute(self.ptr as uint + 1)
            } else {
                self.ptr.offset(1)
            };
            Some(cast::transmute(old))
        }
    }
}

pub struct SmallVecMutIterator<'a,T> {
    ptr: *mut T,
    end: *mut T,
    lifetime: Option<&'a mut T>
}

impl<'a,T> Iterator<&'a mut T> for SmallVecMutIterator<'a,T> {
    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        unsafe {
            if self.ptr == self.end {
                return None
            }
            let old = self.ptr;
            self.ptr = if mem::size_of::<T>() == 0 {
                cast::transmute(self.ptr as uint + 1)
            } else {
                self.ptr.offset(1)
            };
            Some(cast::transmute(old))
        }
    }
}

pub struct SmallVecMoveIterator<'a,T> {
    allocation: Option<*mut u8>,
    iter: SmallVecIterator<'static,T>,
    lifetime: Option<&'a T>,
}

impl<'a,T> Iterator<T> for SmallVecMoveIterator<'a,T> {
    #[inline]
    fn next(&mut self) -> Option<T> {
        unsafe {
            match self.iter.next() {
                None => None,
                Some(reference) => {
                    // Zero out the values as we go so they don't get double-freed.
                    let reference: &mut T = cast::transmute(reference);
                    Some(mem::replace(reference, mem::init()))
                }
            }
        }
    }
}

#[unsafe_destructor]
impl<'a,T> Drop for SmallVecMoveIterator<'a,T> {
    fn drop(&mut self) {
        // Destroy the remaining elements.
        for _ in *self {}

        match self.allocation {
            None => {}
            Some(allocation) => {
                unsafe {
                    if intrinsics::owns_managed::<T>() {
                        local_heap::local_free(allocation as *u8)
                    } else {
                        global_heap::exchange_free(allocation as *u8)
                    }
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
            ptr: *T,
            data: [T, ..$size],
        }
    )
)

macro_rules! def_small_vector_private_trait_impl(
    ($name:ident, $size:expr) => (
        impl<T> SmallVecPrivate<T> for $name<T> {
            unsafe fn set_len(&mut self, new_len: uint) {
                self.len = new_len
            }
            unsafe fn set_cap(&mut self, new_cap: uint) {
                self.cap = new_cap
            }
            fn data(&self, index: uint) -> *T {
                let ptr: *T = &self.data[index];
                ptr
            }
            fn mut_data(&mut self, index: uint) -> *mut T {
                let ptr: *mut T = &mut self.data[index];
                ptr
            }
            unsafe fn ptr(&self) -> *T {
                self.ptr
            }
            unsafe fn mut_ptr(&mut self) -> *mut T {
                cast::transmute(self.ptr)
            }
            unsafe fn set_ptr(&mut self, new_ptr: *mut T) {
                self.ptr = cast::transmute(new_ptr)
            }
        }
    )
)

macro_rules! def_small_vector_trait_impl(
    ($name:ident, $size:expr) => (
        impl<T> SmallVec<T> for $name<T> {
            fn inline_size(&self) -> uint {
                $size
            }
            fn len(&self) -> uint {
                self.len
            }
            fn cap(&self) -> uint {
                self.cap
            }
        }
    )
)

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
                        *ptr.offset(i as int) = mem::uninit();
                    }

                    if intrinsics::owns_managed::<T>() {
                        local_heap::local_free(self.ptr() as *u8)
                    } else {
                        global_heap::exchange_free(self.ptr() as *u8)
                    }
                }
            }
        }
    )
)

macro_rules! def_small_vector_clone_impl(
    ($name:ident) => (
        impl<T:Clone> Clone for $name<T> {
            fn clone(&self) -> $name<T> {
                let mut new_vector = $name::new();
                for element in self.iter() {
                    new_vector.push((*element).clone())
                }
                new_vector
            }
        }
    )
)

macro_rules! def_small_vector_impl(
    ($name:ident, $size:expr) => (
        impl<T> $name<T> {
            #[inline]
            pub fn new() -> $name<T> {
                unsafe {
                    $name {
                        len: 0,
                        cap: $size,
                        ptr: ptr::null(),
                        data: mem::init(),
                    }
                }
            }
        }
    )
)

/// TODO(pcwalton): Remove in favor of `vec_ng` after a Rust upgrade.
pub struct SmallVec0<T> {
    len: uint,
    cap: uint,
    ptr: *mut T,
}

impl<T> SmallVecPrivate<T> for SmallVec0<T> {
    unsafe fn set_len(&mut self, new_len: uint) {
        self.len = new_len
    }
    unsafe fn set_cap(&mut self, new_cap: uint) {
        self.cap = new_cap
    }
    fn data(&self, _: uint) -> *T {
        ptr::null()
    }
    fn mut_data(&mut self, _: uint) -> *mut T {
        ptr::mut_null()
    }
    unsafe fn ptr(&self) -> *T {
        cast::transmute(self.ptr)
    }
    unsafe fn mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
    unsafe fn set_ptr(&mut self, new_ptr: *mut T) {
        self.ptr = new_ptr
    }
}

impl<T> SmallVec<T> for SmallVec0<T> {
    fn inline_size(&self) -> uint {
        0
    }
    fn len(&self) -> uint {
        self.len
    }
    fn cap(&self) -> uint {
        self.cap
    }
}

impl<T> SmallVec0<T> {
    pub fn new() -> SmallVec0<T> {
        SmallVec0 {
            len: 0,
            cap: 0,
            ptr: ptr::mut_null(),
        }
    }
}

def_small_vector_drop_impl!(SmallVec0, 0)
def_small_vector_clone_impl!(SmallVec0)

def_small_vector!(SmallVec1, 1)
def_small_vector_private_trait_impl!(SmallVec1, 1)
def_small_vector_trait_impl!(SmallVec1, 1)
def_small_vector_drop_impl!(SmallVec1, 1)
def_small_vector_clone_impl!(SmallVec1)
def_small_vector_impl!(SmallVec1, 1)

def_small_vector!(SmallVec2, 2)
def_small_vector_private_trait_impl!(SmallVec2, 2)
def_small_vector_trait_impl!(SmallVec2, 2)
def_small_vector_drop_impl!(SmallVec2, 2)
def_small_vector_clone_impl!(SmallVec2)
def_small_vector_impl!(SmallVec2, 2)

def_small_vector!(SmallVec4, 4)
def_small_vector_private_trait_impl!(SmallVec4, 4)
def_small_vector_trait_impl!(SmallVec4, 4)
def_small_vector_drop_impl!(SmallVec4, 4)
def_small_vector_clone_impl!(SmallVec4)
def_small_vector_impl!(SmallVec4, 4)

def_small_vector!(SmallVec8, 8)
def_small_vector_private_trait_impl!(SmallVec8, 8)
def_small_vector_trait_impl!(SmallVec8, 8)
def_small_vector_drop_impl!(SmallVec8, 8)
def_small_vector_clone_impl!(SmallVec8)
def_small_vector_impl!(SmallVec8, 8)

def_small_vector!(SmallVec16, 16)
def_small_vector_private_trait_impl!(SmallVec16, 16)
def_small_vector_trait_impl!(SmallVec16, 16)
def_small_vector_drop_impl!(SmallVec16, 16)
def_small_vector_clone_impl!(SmallVec16)
def_small_vector_impl!(SmallVec16, 16)

def_small_vector!(SmallVec24, 24)
def_small_vector_private_trait_impl!(SmallVec24, 24)
def_small_vector_trait_impl!(SmallVec24, 24)
def_small_vector_drop_impl!(SmallVec24, 24)
def_small_vector_clone_impl!(SmallVec24)
def_small_vector_impl!(SmallVec24, 24)

def_small_vector!(SmallVec32, 32)
def_small_vector_private_trait_impl!(SmallVec32, 32)
def_small_vector_trait_impl!(SmallVec32, 32)
def_small_vector_drop_impl!(SmallVec32, 32)
def_small_vector_clone_impl!(SmallVec32)
def_small_vector_impl!(SmallVec32, 32)

#[cfg(test)]
pub mod tests {
    use smallvec::{SmallVec, SmallVec0, SmallVec2, SmallVec16};

    // We heap allocate all these strings so that double frees will show up under valgrind.

    #[test]
    pub fn test_inline() {
        let mut v = SmallVec16::new();
        v.push(~"hello");
        v.push(~"there");
        assert_eq!(v.as_slice(), &[~"hello", ~"there"]);
    }

    #[test]
    pub fn test_spill() {
        let mut v = SmallVec2::new();
        v.push(~"hello");
        v.push(~"there");
        v.push(~"burma");
        v.push(~"shave");
        assert_eq!(v.as_slice(), &[~"hello", ~"there", ~"burma", ~"shave"]);
    }

    #[test]
    pub fn test_double_spill() {
        let mut v = SmallVec2::new();
        v.push(~"hello");
        v.push(~"there");
        v.push(~"burma");
        v.push(~"shave");
        v.push(~"hello");
        v.push(~"there");
        v.push(~"burma");
        v.push(~"shave");
        assert_eq!(v.as_slice(), &[
            ~"hello", ~"there", ~"burma", ~"shave", ~"hello", ~"there", ~"burma", ~"shave",
        ]);
    }

    #[test]
    pub fn test_smallvec0() {
        let mut v = SmallVec0::new();
        v.push(~"hello");
        v.push(~"there");
        v.push(~"burma");
        v.push(~"shave");
        v.push(~"hello");
        v.push(~"there");
        v.push(~"burma");
        v.push(~"shave");
        assert_eq!(v.as_slice(), &[
            ~"hello", ~"there", ~"burma", ~"shave", ~"hello", ~"there", ~"burma", ~"shave",
        ]);
    }
}

