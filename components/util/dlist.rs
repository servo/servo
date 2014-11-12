/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utility functions for doubly-linked lists.

use std::collections::DList;
use std::mem;
use std::ptr;

struct RawDList<T> {
    length: uint,
    head: *mut RawNode<T>,
    tail: *mut RawNode<T>,
}

#[allow(dead_code)]
struct RawNode<T> {
    next: *mut RawNode<T>,
    prev: *mut RawNode<T>,
    value: T,
}

#[unsafe_destructor]
impl<T> Drop for RawDList<T> {
    fn drop(&mut self) {
        panic!("shouldn't happen")
    }
}

/// Workaround for a missing method on Rust's `DList` type. Splits the head off a list in O(1)
/// time.
pub fn split<T>(list: &mut DList<T>) -> DList<T> {
    let list = unsafe {
        mem::transmute::<&mut DList<T>,&mut RawDList<T>>(list)
    };

    if list.length == 0 {
        panic!("split_dlist(): empty list")
    }
    let head_node = mem::replace(&mut list.head, ptr::null_mut());
    let head_list = RawDList {
        length: 1,
        head: head_node,
        tail: head_node,
    };
    debug_assert!(list.head.is_null());

    unsafe {
        mem::swap(&mut (*head_list.head).next, &mut list.head);
        debug_assert!((*head_list.head).next.is_null());
        debug_assert!((*head_list.head).prev.is_null());
        (*head_list.head).prev = ptr::null_mut();
    }

    list.length -= 1;
    if list.length == 0 {
        list.tail = ptr::null_mut()
    } else {
        if list.length == 1 {
            list.tail = list.head
        }

        unsafe {
            (*list.head).prev = ptr::null_mut()
        }
    }

    unsafe {
        mem::transmute::<RawDList<T>,DList<T>>(head_list)
    }
}

/// Appends the items in the other list to this one, leaving the other list empty.
#[inline]
pub fn append_from<T>(this: &mut DList<T>, other: &mut DList<T>) {
    unsafe {
        let this = mem::transmute::<&mut DList<T>,&mut RawDList<T>>(this);
        let other = mem::transmute::<&mut DList<T>,&mut RawDList<T>>(other);
        if this.length == 0 {
            this.head = mem::replace(&mut other.head, ptr::null_mut());
            this.tail = mem::replace(&mut other.tail, ptr::null_mut());
            this.length = mem::replace(&mut other.length, 0);
            return
        }

        let old_other_head = mem::replace(&mut other.head, ptr::null_mut());
        if old_other_head.is_null() {
            return
        }
        (*old_other_head).prev = this.tail;
        (*this.tail).next = old_other_head;

        this.tail = mem::replace(&mut other.tail, ptr::null_mut());
        this.length += other.length;
        other.length = 0;
    }
}

/// Prepends the items in the other list to this one, leaving the other list empty.
#[inline]
pub fn prepend_from<T>(this: &mut DList<T>, other: &mut DList<T>) {
    unsafe {
        let this = mem::transmute::<&mut DList<T>,&mut RawDList<T>>(this);
        let other = mem::transmute::<&mut DList<T>,&mut RawDList<T>>(other);
        if this.length == 0 {
            this.head = mem::replace(&mut other.head, ptr::null_mut());
            this.tail = mem::replace(&mut other.tail, ptr::null_mut());
            this.length = mem::replace(&mut other.length, 0);
            return
        }

        let old_other_tail = mem::replace(&mut other.tail, ptr::null_mut());
        if old_other_tail.is_null() {
            return
        }
        (*old_other_tail).next = this.head;
        (*this.head).prev = old_other_tail;

        this.head = mem::replace(&mut other.head, ptr::null_mut());
        this.length += other.length;
        other.length = 0;
    }
}

