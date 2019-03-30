/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Trait for cloning data into a shared memory buffer.
//!
//! This module contains the SharedMemoryBuilder type and ToShmem trait.
//!
//! We put them here (and not in style_traits) so that we can derive ToShmem
//! from the selectors and style crates.

#![crate_name = "to_shmem"]
#![crate_type = "rlib"]

use std::alloc::Layout;
#[cfg(debug_assertions)]
use std::any::TypeId;
use std::isize;
#[cfg(debug_assertions)]
use std::collections::HashSet;
use std::mem::{self, ManuallyDrop};
#[cfg(debug_assertions)]
use std::os::raw::c_void;
use std::ptr::{self, NonNull};

// Various pointer arithmetic functions in this file can be replaced with
// functions on `Layout` once they have stabilized:
//
// https://github.com/rust-lang/rust/issues/55724

/// A builder object that transforms and copies values into a fixed size buffer.
pub struct SharedMemoryBuilder {
    /// The buffer into which values will be copied.
    buffer: *mut u8,
    /// The size of the buffer.
    capacity: usize,
    /// The current position in the buffer, where the next value will be written
    /// at.
    index: usize,
    /// Pointers to every sharable value that we store in the shared memory
    /// buffer.  We use this to assert against encountering the same value
    /// twice, e.g. through another Arc reference, so that we don't
    /// inadvertently store duplicate copies of values.
    #[cfg(debug_assertions)]
    shared_values: HashSet<*const c_void>,
    /// Types of values that we may duplicate in the shared memory buffer when
    /// there are shared references to them, such as in Arcs.
    #[cfg(debug_assertions)]
    allowed_duplication_types: HashSet<TypeId>,
}

/// Amount of padding needed after `size` bytes to ensure that the following
/// address will satisfy `align`.
fn padding_needed_for(size: usize, align: usize) -> usize {
    padded_size(size, align).wrapping_sub(size)
}

/// Rounds up `size` so that the following address will satisfy `align`.
fn padded_size(size: usize, align: usize) -> usize {
    size.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1)
}

impl SharedMemoryBuilder {
    /// Creates a new SharedMemoryBuilder using the specified buffer.
    pub unsafe fn new(buffer: *mut u8, capacity: usize) -> SharedMemoryBuilder {
        SharedMemoryBuilder {
            buffer,
            capacity,
            index: 0,
            #[cfg(debug_assertions)]
            shared_values: HashSet::new(),
            #[cfg(debug_assertions)]
            allowed_duplication_types: HashSet::new(),
        }
    }

    /// Notes a type as being allowed for duplication when being copied to the
    /// shared memory buffer, such as Arcs referencing the same value.
    #[inline]
    pub fn add_allowed_duplication_type<T: 'static>(&mut self) {
        #[cfg(debug_assertions)]
        self.allowed_duplication_types.insert(TypeId::of::<T>());
    }

    /// Returns the number of bytes currently used in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.index
    }

    /// Writes a value into the shared memory buffer and returns a pointer to
    /// it in the buffer.
    ///
    /// The value is cloned and converted into a form suitable for placing into
    /// a shared memory buffer by calling ToShmem::to_shmem on it.
    ///
    /// Panics if there is insufficient space in the buffer.
    pub fn write<T: ToShmem>(&mut self, value: &T) -> *mut T {
        // Reserve space for the value.
        let dest: *mut T = self.alloc_value();

        // Make a clone of the value with all of its heap allocations
        // placed in the shared memory buffer.
        let value = value.to_shmem(self);

        unsafe {
            // Copy the value into the buffer.
            ptr::write(dest, ManuallyDrop::into_inner(value));
        }

        // Return a pointer to the shared value.
        dest
    }

    /// Reserves space in the shared memory buffer to fit a value of type T,
    /// and returns a pointer to that reserved space.
    ///
    /// Panics if there is insufficient space in the buffer.
    pub fn alloc_value<T>(&mut self) -> *mut T {
        self.alloc(Layout::new::<T>())
    }

    /// Reserves space in the shared memory buffer to fit an array of values of
    /// type T, and returns a pointer to that reserved space.
    ///
    /// Panics if there is insufficient space in the buffer.
    pub fn alloc_array<T>(&mut self, len: usize) -> *mut T {
        if len == 0 {
            return NonNull::dangling().as_ptr();
        }

        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();

        self.alloc(Layout::from_size_align(padded_size(size, align) * len, align).unwrap())
    }

    /// Reserves space in the shared memory buffer that conforms to the
    /// specified layout, and returns a pointer to that reserved space.
    ///
    /// Panics if there is insufficient space in the buffer.
    pub fn alloc<T>(&mut self, layout: Layout) -> *mut T {
        // Amount of padding to align the value.
        //
        // The addition can't overflow, since self.index <= self.capacity, and
        // for us to have successfully allocated the buffer, `buffer + capacity`
        // can't overflow.
        let padding = padding_needed_for(self.buffer as usize + self.index, layout.align());

        // Reserve space for the padding.
        let start = self.index.checked_add(padding).unwrap();
        assert!(start <= std::isize::MAX as usize);  // for the cast below

        // Reserve space for the value.
        let end = start.checked_add(layout.size()).unwrap();
        assert!(end <= self.capacity);

        self.index = end;
        unsafe { self.buffer.offset(start as isize) as *mut T }
    }
}

/// A type that can be copied into a SharedMemoryBuilder.
pub trait ToShmem: Sized {
    /// Clones this value into a form suitable for writing into a
    /// SharedMemoryBuilder.
    ///
    /// If this value owns any heap allocations, they should be written into
    /// `builder` so that the return value of this function can point to the
    /// copy in the shared memory buffer.
    ///
    /// The return type is wrapped in ManuallyDrop to make it harder to
    /// accidentally invoke the destructor of the value that is produced.
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> ManuallyDrop<Self>;
}
