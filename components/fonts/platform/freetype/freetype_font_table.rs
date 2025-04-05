/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ::core::ptr;
use ::std::alloc; // or extern crate alloc; use ::alloc::alloc;
use crate::font::FontTableMethods;

// Alloc and dealloc functions was taken from
// <https://users.rust-lang.org/t/how-can-i-allocate-aligned-memory-in-rust/33293/6>

/// FontTable structure should have specific allignment due to the fact that we rely
/// on it to validate checksums
/// <https://learn.microsoft.com/en-us/typography/opentype/spec/otff#calculating-checksums>

#[derive(Debug)]
pub struct FontTable {
    length: usize,
    allignment: usize,
    pointer: *mut u8,
}

impl FontTableMethods for FontTable {
    fn buffer(&self) -> &[u8] {
        unsafe {
            &*ptr::slice_from_raw_parts(self.pointer, self.length)
        }
    }
}

impl FontTable {
    pub fn new_four_alligned(length: usize) -> Result<Self, &'static str> {
        if let Some(ptr) = alloc(length, 4) {
            Ok(Self {
                length,
                allignment: 4,
                pointer: ptr.as_ptr() as *mut u8
            })
        } else {
            return Err("FontTable: Allocation error");
        }
    }

    pub fn get_mut_ptr(&self) -> *mut u8 {
        self.pointer
    }

    pub fn get_ptr(&self) -> *const u8 {
        self.pointer
    }
}

impl Drop for FontTable {
    fn drop(&mut self) {
        unsafe {
            free(
                ptr::NonNull::<()>::new(self.pointer as *mut ()),
                self.length,
                self.allignment
            );
        }
    }
}


fn alloc(numbytes: usize, alignment: usize) -> Option<ptr::NonNull<()>> {
    Some({
        if numbytes == 0 {
            return None;
        }
        let layout = alloc::Layout::from_size_align(numbytes, alignment)
            .map_err(|err| eprintln!("Layout error: {}", err))
            .ok()?;
        ptr::NonNull::new(unsafe {
            // # Safety
            //
            //   - numbytes != 0
            alloc::alloc(layout)
        })?
        .cast::<()>()
    })
}

/// # Safety
///
///   - `ptr`, when `NonNull`, must be a value returned by `alloc(numbytes, alignment)`
#[allow(unused_unsafe)]
unsafe fn free(ptr: Option<ptr::NonNull<()>>, numbytes: usize, alignment: usize) {
    let ptr = if let Some(ptr) = ptr {
        ptr
    } else {
        return;
    };
    let layout = alloc::Layout::from_size_align(numbytes, alignment).unwrap_or_else(|err| {
        // if same layout as input this should not happen,
        // so it is a very bad bug if this is reached
        eprintln!("Layout error: {}", err);
        ::std::process::abort();
    });
    unsafe {
        // # Safety
        //
        //   - `ptr` came from alloc::alloc(layout);
        alloc::dealloc(ptr.cast::<u8>().as_ptr(), layout);
    }
}
