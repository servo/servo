/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An in-place, dynamically borrowable slot. Useful for "mutable fields". Assuming this works out
//! well, this type should be upstreamed to the Rust standard library.

use std::cast;
use std::util;

#[unsafe_no_drop_flag]
#[no_freeze]
pub struct Slot<T> {
    // NB: Must be priv, or else someone could borrow it.
    priv value: T,
    priv immutable_borrow_count: u8,
    priv mutably_borrowed: bool,
}

impl<T:Clone> Clone for Slot<T> {
    #[inline]
    fn clone(&self) -> Slot<T> {
        Slot {
            value: self.value.clone(),
            immutable_borrow_count: 0,
            mutably_borrowed: false,
        }
    }
}

#[unsafe_destructor]
impl<T> Drop for Slot<T> {
    fn drop(&mut self) {
        // Noncopyable.
    }
}

pub struct SlotRef<'self,T> {
    ptr: &'self T,
    priv immutable_borrow_count: *mut u8,
}

#[unsafe_destructor]
impl<'self,T> Drop for SlotRef<'self,T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            *self.immutable_borrow_count -= 1
        }
    }
}

pub struct MutSlotRef<'self,T> {
    ptr: &'self mut T,
    priv mutably_borrowed: *mut bool,
}

#[unsafe_destructor]
impl<'self,T> Drop for MutSlotRef<'self,T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            *self.mutably_borrowed = false
        }
    }
}

impl<T> Slot<T> {
    #[inline]
    pub fn init(value: T) -> Slot<T> {
        Slot {
            value: value,
            immutable_borrow_count: 0,
            mutably_borrowed: false,
        }
    }

    /// Borrows the data immutably. This function is thread-safe, but *bad things will happen if
    /// you try to mutate the data while one of these pointers is held*.
    #[inline]
    pub unsafe fn borrow_unchecked<'a>(&'a self) -> &'a T {
        &self.value
    }

    #[inline]
    pub fn borrow<'a>(&'a self) -> SlotRef<'a,T> {
        unsafe {
            if self.immutable_borrow_count == 255 || self.mutably_borrowed {
                self.fail()
            }
            let immutable_borrow_count = cast::transmute_mut(&self.immutable_borrow_count);
            *immutable_borrow_count += 1;
            SlotRef {
                ptr: &self.value,
                immutable_borrow_count: immutable_borrow_count,
            }
        }
    }

    #[inline]
    pub fn mutate<'a>(&'a self) -> MutSlotRef<'a,T> {
        unsafe {
            if self.immutable_borrow_count > 0 || self.mutably_borrowed {
                self.fail()
            }
            let mutably_borrowed = cast::transmute_mut(&self.mutably_borrowed);
            *mutably_borrowed = true;
            MutSlotRef {
                ptr: cast::transmute_mut(&self.value),
                mutably_borrowed: mutably_borrowed,
            }
        }
    }

    #[inline]
    pub fn set(&self, value: T) {
        *self.mutate().ptr = value
    }

    /// Replaces the slot's value with the given value and returns the old value.
    #[inline]
    pub fn replace(&self, value: T) -> T {
        util::replace(self.mutate().ptr, value)
    }

    #[inline(never)]
    pub fn fail(&self) -> ! {
        fail!("slot is borrowed")
    }
}

impl<T:Clone> Slot<T> {
    #[inline]
    pub fn get(&self) -> T {
        self.value.clone()
    }
}




