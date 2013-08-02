/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::option::*;
use std::unstable::intrinsics;
use std::unstable::atomics::{AtomicPtr, SeqCst, AtomicUint};
use std::cast::{transmute, forget};
use std::vec;
 
pub struct WorkstealingDeque<T> {
    count: AtomicUint,
    top: AtomicUint,
    bottom: AtomicUint,
    array: AtomicPtr<DequeArray<T>>
}
 
pub struct DequeArray<T> {
    size: AtomicUint,
    raw: ~[AtomicPtr<T>]
}
 
impl<T> DequeArray<T> {
    pub fn new(init_size: uint) -> DequeArray<T> {
        unsafe {
            DequeArray {
                size: AtomicUint::new(init_size),
                raw: vec::from_elem(init_size, AtomicPtr::new(intrinsics::uninit()))
            }
        }
    }
}
 
impl<T> WorkstealingDeque<T> {
    pub fn new(init_size: uint) -> WorkstealingDeque<T> {
        unsafe {
            WorkstealingDeque {
                count: AtomicUint::new(1),
                top: AtomicUint::new(0),
                bottom: AtomicUint::new(0),
                array: AtomicPtr::new(transmute(~DequeArray::new::<T>(init_size)))
            }
        }
    }
 
    pub fn push(&mut self, value: *T) {
        unsafe {
            let b = self.bottom.load(SeqCst);
            let t = self.top.load(SeqCst);
            let a: *mut DequeArray<T> = self.array.load(SeqCst);
            let size = (*a).size.load(SeqCst); // XXX: Pick ordering.
            if b - t > size - 1 {
                fail!("deque overfilled, no resize implemented");
                //self.resize();
            }
            let size = (*a).size.load(SeqCst); // XXX: Pick ordering.
            let value: *mut T = transmute(value);
            debug!("about to push onto raw array: length is: %u", (*a).raw.len());
            (*a).raw[b % size].store(value, SeqCst);
            self.bottom.store(b+1, SeqCst);                       
        }
    }
 
    pub fn pop(&mut self) -> Option<*T> {
        unsafe {
            let b = self.bottom.load(SeqCst) - 1;
            let a = self.array.load(SeqCst);
            self.bottom.store(b, SeqCst);
            let t = self.top.load(SeqCst);
            debug!("in pop t = %u b = %u", t, b);
            let mut x: Option<*T>;
            if t <= b {
                let size = (*a).size.load(SeqCst); // XXX: Pick ordering.
                x = Some(transmute((*a).raw[b % size].load(SeqCst)));
                if t == b {
                    // XXX: "compare_exchange_strong_explicit" is what in rust?
                    if 0 != self.top.compare_and_swap(t, t+1, SeqCst) {
                        x = None;
                    }
                    self.bottom.store(b+1, SeqCst);
                }
            } else {
                x = None;
                self.bottom.store(b+1, SeqCst);
            }
            return x;
        }
    }
 
    pub fn steal(&mut self) -> Option<*T> {
        unsafe {
            let t = self.top.load(SeqCst);
            let b = self.bottom.load(SeqCst);
            let mut x: Option<*T> = None;
            if (t < b) {
                let a = self.array.load(SeqCst);
                let size = (*a).size.load(SeqCst); // XXX: Pick ordering.
                x = Some(transmute((*a).raw[t % size].load(SeqCst)));
                // XXX: "compare_exchange_strong_explicit" is what in rust?
                if 0 != self.top.compare_and_swap(t, t+1, SeqCst) {
                    return None;
                }
            }
            return x;
        }
    }
 
    pub fn is_empty(&self) -> bool {
        // XXX: This is almost certainly too restrictive an ordering.
        let b = self.bottom.load(SeqCst);
        let t = self.top.load(SeqCst);
        return b == t;
    }
}
