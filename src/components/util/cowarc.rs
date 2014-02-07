/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An atomically reference counted type that copies itself on mutation.

use std::cast;
use std::ptr;
use std::sync::atomics::{AtomicUint, SeqCst};

struct CowArcAlloc<T> {
    ref_count: AtomicUint,
    data: T,
}

#[unsafe_no_drop_flag]
pub struct CowArc<T> {
    priv ptr: *mut CowArcAlloc<T>,
}

#[unsafe_destructor]
impl<T> Drop for CowArc<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            if self.ptr != ptr::mut_null() && (*self.ptr).ref_count.fetch_sub(1, SeqCst) == 1 {
                let _kill_it: ~CowArcAlloc<T> = cast::transmute(self.ptr);
                self.ptr = ptr::mut_null()
            }
        }
    }
}

impl<T:Eq + Freeze + Clone> Eq for CowArc<T> {
    fn eq(&self, other: &CowArc<T>) -> bool {
        self.get() == other.get()
    }
}

impl<T:Freeze + Clone> Clone for CowArc<T> {
    #[inline]
    fn clone(&self) -> CowArc<T> {
        unsafe {
            drop((*self.ptr).ref_count.fetch_add(1, SeqCst));
        }
        CowArc {
            ptr: self.ptr
        }
    }
}

impl<T:Freeze + Clone> CowArc<T> {
    #[inline]
    pub fn new(value: T) -> CowArc<T> {
        let alloc = ~CowArcAlloc {
            ref_count: AtomicUint::new(1),
            data: value,
        };
        unsafe {
            CowArc {
                ptr: cast::transmute(alloc),
            }
        }
    }

    #[inline]
    pub fn shared(&self) -> bool {
        unsafe {
            (*self.ptr).ref_count.load(SeqCst) != 1
        }
    }

    #[inline]
    pub fn get<'a>(&'a self) -> &'a T {
        unsafe {
            cast::transmute(&(*self.ptr).data)
        }
    }

    #[inline(always)]
    pub fn get_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe {
            if (*self.ptr).ref_count.load(SeqCst) == 1 {
                return cast::transmute(&mut (*self.ptr).data)
            }

            let copy = ~CowArcAlloc {
                ref_count: AtomicUint::new(1),
                data: (*self.ptr).data.clone(),
            };

            *self = CowArc {
                ptr: cast::transmute(copy),
            };

            cast::transmute(&mut (*self.ptr).data)
        }
    }
}

