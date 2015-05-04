/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data structure measurement.

use libc::{c_void, size_t};
use std::collections::LinkedList;
use std::mem::transmute;
use std::sync::Arc;

extern {
    // Get the size of a heap block.
    //
    // Ideally Rust would expose a function like this in std::rt::heap, which would avoid the
    // jemalloc dependence.
    //
    // The C prototype is `je_malloc_usable_size(JEMALLOC_USABLE_SIZE_CONST void *ptr)`. On some
    // platforms `JEMALLOC_USABLE_SIZE_CONST` is `const` and on some it is empty. But in practice
    // this function doesn't modify the contents of the block that `ptr` points to, so we use
    // `*const c_void` here.
    fn je_malloc_usable_size(ptr: *const c_void) -> size_t;
}

// A wrapper for je_malloc_usable_size that handles `EMPTY` and returns `usize`.
pub fn heap_size_of(ptr: *const c_void) -> usize {
    if ptr == ::std::rt::heap::EMPTY as *const c_void {
        0
    } else {
        unsafe { je_malloc_usable_size(ptr) as usize }
    }
}

// The simplest trait for measuring the size of heap data structures. More complex traits that
// return multiple measurements -- e.g. measure text separately from images -- are also possible,
// and should be used when appropriate.
//
// FIXME(njn): it would be nice to be able to derive this trait automatically, given that
// implementations are mostly repetitive and mechanical.
//
pub trait HeapSizeOf {
    /// Measure the size of any heap-allocated structures that hang off this value, but not the
    /// space taken up by the value itself (i.e. what size_of::<T> measures, more or less); that
    /// space is handled by the implementation of HeapSizeOf for Box<T> below.
    fn heap_size_of_children(&self) -> usize;
}

// There are two possible ways to measure the size of `self` when it's on the heap: compute it
// (with `::std::rt::heap::usable_size(::std::mem::size_of::<T>(), 0)`) or measure it directly
// using the heap allocator (with `heap_size_of`). We do the latter, for the following reasons.
//
// * The heap allocator is the true authority for the sizes of heap blocks; its measurement is
//   guaranteed to be correct. In comparison, size computations are error-prone. (For example, the
//   `rt::heap::usable_size` function used in some of Rust's non-default allocator implementations
//   underestimate the true usable size of heap blocks, which is safe in general but would cause
//   under-measurement here.)
//
// * If we measure something that isn't a heap block, we'll get a crash. This keeps us honest,
//   which is important because unsafe code is involved and this can be gotten wrong.
//
// However, in the best case, the two approaches should give the same results.
//
impl<T: HeapSizeOf> HeapSizeOf for Box<T> {
    fn heap_size_of_children(&self) -> usize {
        // Measure size of `self`.
        heap_size_of(&**self as *const T as *const c_void) + (**self).heap_size_of_children()
    }
}

impl HeapSizeOf for String {
    fn heap_size_of_children(&self) -> usize {
        heap_size_of(self.as_ptr() as *const c_void)
    }
}

impl<T: HeapSizeOf> HeapSizeOf for Option<T> {
    fn heap_size_of_children(&self) -> usize {
        match *self {
            None => 0,
            Some(ref x) => x.heap_size_of_children()
        }
    }
}

impl<T: HeapSizeOf> HeapSizeOf for Arc<T> {
    fn heap_size_of_children(&self) -> usize {
        (**self).heap_size_of_children()
    }
}

impl<T: HeapSizeOf> HeapSizeOf for Vec<T> {
    fn heap_size_of_children(&self) -> usize {
        heap_size_of(self.as_ptr() as *const c_void) +
            self.iter().fold(0, |n, elem| n + elem.heap_size_of_children())
    }
}

// FIXME(njn): We can't implement HeapSizeOf accurately for LinkedList because it requires access
// to the private Node type. Eventually we'll want to add HeapSizeOf (or equivalent) to Rust
// itself. In the meantime, we use the dirty hack of transmuting LinkedList into an identical type
// (LinkedList2) and measuring that.
impl<T: HeapSizeOf> HeapSizeOf for LinkedList<T> {
    fn heap_size_of_children(&self) -> usize {
        let list2: &LinkedList2<T> = unsafe { transmute(self) };
        list2.heap_size_of_children()
    }
}

struct LinkedList2<T> {
    _length: usize,
    list_head: Link<T>,
    _list_tail: Rawlink<Node<T>>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Rawlink<T> {
    _p: *mut T,
}

struct Node<T> {
    next: Link<T>,
    _prev: Rawlink<Node<T>>,
    value: T,
}

impl<T: HeapSizeOf> HeapSizeOf for Node<T> {
    // Unlike most heap_size_of_children() functions, this one does *not* measure descendents.
    // Instead, LinkedList2<T>::heap_size_of_children() handles that, so that it can use iteration
    // instead of recursion, which avoids potentially blowing the stack.
    fn heap_size_of_children(&self) -> usize {
        self.value.heap_size_of_children()
    }
}

impl<T: HeapSizeOf> HeapSizeOf for LinkedList2<T> {
    fn heap_size_of_children(&self) -> usize {
        let mut size = 0;
        let mut curr: &Link<T> = &self.list_head;
        while curr.is_some() {
            size += (*curr).heap_size_of_children();
            curr = &curr.as_ref().unwrap().next;
        }
        size
    }
}

// This is a basic sanity check. If the representation of LinkedList changes such that it becomes a
// different size to LinkedList2, this will fail at compile-time.
#[allow(dead_code)]
unsafe fn linked_list2_check() {
    transmute::<LinkedList<i32>, LinkedList2<i32>>(panic!());
}

// Currently, types that implement the Drop type are larger than those that don't. Because
// LinkedList implements Drop, LinkedList2 must also so that linked_list2_check() doesn't fail.
impl<T> Drop for LinkedList2<T> {
    fn drop(&mut self) {}
}

