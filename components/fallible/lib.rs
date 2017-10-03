/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate hashglobe;
extern crate smallvec;

use hashglobe::FailedAllocationError;
#[cfg(feature = "known_system_malloc")]
use hashglobe::alloc;
use smallvec::Array;
use smallvec::SmallVec;
use std::vec::Vec;

pub trait FallibleVec<T> {
    /// Append |val| to the end of |vec|.  Returns Ok(()) on success,
    /// Err(reason) if it fails, with |reason| describing the failure.
    fn try_push(&mut self, value: T) -> Result<(), FailedAllocationError>;
}


/////////////////////////////////////////////////////////////////
// Vec

impl<T> FallibleVec<T> for Vec<T> {
    #[inline(always)]
    fn try_push(&mut self, val: T) -> Result<(), FailedAllocationError> {
        #[cfg(feature = "known_system_malloc")]
        {
            if self.capacity() == self.len() {
                try_double_vec(self)?;
                debug_assert!(self.capacity() > self.len());
            }
        }
        self.push(val);
        Ok(())
    }
}

// Double the capacity of |vec|, or fail to do so due to lack of memory.
// Returns Ok(()) on success, Err(..) on failure.
#[cfg(feature = "known_system_malloc")]
#[inline(never)]
#[cold]
fn try_double_vec<T>(vec: &mut Vec<T>) -> Result<(), FailedAllocationError> {
    use std::mem;

    let old_ptr = vec.as_mut_ptr();
    let old_len = vec.len();

    let old_cap: usize = vec.capacity();
    let new_cap: usize = if old_cap == 0 {
        4
    } else {
        old_cap.checked_mul(2).ok_or(FailedAllocationError::new(
            "capacity overflow for Vec",
        ))?
    };

    let new_size_bytes = new_cap.checked_mul(mem::size_of::<T>()).ok_or(
        FailedAllocationError::new("capacity overflow for Vec"),
    )?;

    let new_ptr = unsafe {
        if old_cap == 0 {
            alloc::alloc(new_size_bytes, 0)
        } else {
            alloc::realloc(old_ptr as *mut u8, new_size_bytes)
        }
    };

    if new_ptr.is_null() {
        return Err(FailedAllocationError::new(
            "out of memory when allocating Vec",
        ));
    }

    let new_vec = unsafe {
        Vec::from_raw_parts(new_ptr as *mut T, old_len, new_cap)
    };

    mem::forget(mem::replace(vec, new_vec));
    Ok(())
}


/////////////////////////////////////////////////////////////////
// SmallVec

impl<T: Array> FallibleVec<T::Item> for SmallVec<T> {
    #[inline(always)]
    fn try_push(&mut self, val: T::Item) -> Result<(), FailedAllocationError> {
        #[cfg(feature = "known_system_malloc")]
        {
            if self.capacity() == self.len() {
                try_double_small_vec(self)?;
                debug_assert!(self.capacity() > self.len());
            }
        }
        self.push(val);
        Ok(())
    }
}

// Double the capacity of |svec|, or fail to do so due to lack of memory.
// Returns Ok(()) on success, Err(..) on failure.
#[cfg(feature = "known_system_malloc")]
#[inline(never)]
#[cold]
fn try_double_small_vec<T>(svec: &mut SmallVec<T>)
-> Result<(), FailedAllocationError>
where
    T: Array,
{
    use std::mem;
    use std::ptr::copy_nonoverlapping;

    let old_ptr = svec.as_mut_ptr();
    let old_len = svec.len();

    let old_cap: usize = svec.capacity();
    let new_cap: usize = if old_cap == 0 {
        4
    } else {
        old_cap.checked_mul(2).ok_or(FailedAllocationError::new(
            "capacity overflow for SmallVec",
        ))?
    };

    // This surely shouldn't fail, if |old_cap| was previously accepted as a
    // valid value.  But err on the side of caution.
    let old_size_bytes = old_cap.checked_mul(mem::size_of::<T>()).ok_or(
        FailedAllocationError::new("capacity overflow for SmallVec"),
    )?;

    let new_size_bytes = new_cap.checked_mul(mem::size_of::<T>()).ok_or(
        FailedAllocationError::new("capacity overflow for SmallVec"),
    )?;

    let new_ptr;
    if svec.spilled() {
        // There's an old block to free, and, presumably, old contents to
        // copy.  realloc takes care of both aspects.
        unsafe {
            new_ptr = alloc::realloc(old_ptr as *mut u8, new_size_bytes);
        }
    } else {
        // There's no old block to free.  There may be old contents to copy.
        unsafe {
            new_ptr = alloc::alloc(new_size_bytes, 0);
            if !new_ptr.is_null() && old_size_bytes > 0 {
                copy_nonoverlapping(old_ptr as *const u8,
                                    new_ptr as *mut u8, old_size_bytes);
            }
        }
    }

    if new_ptr.is_null() {
        return Err(FailedAllocationError::new(
            "out of memory when allocating SmallVec",
        ));
    }

    let new_vec = unsafe {
        Vec::from_raw_parts(new_ptr as *mut T::Item, old_len, new_cap)
    };

    let new_svec = SmallVec::from_vec(new_vec);
    mem::forget(mem::replace(svec, new_svec));
    Ok(())
}
