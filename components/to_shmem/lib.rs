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

extern crate cssparser;
extern crate servo_arc;
extern crate smallbitvec;
extern crate smallvec;
#[cfg(feature = "string_cache")]
extern crate string_cache;
extern crate thin_vec;

use servo_arc::{Arc, ThinArc};
use smallbitvec::{InternalStorage, SmallBitVec};
use smallvec::{Array, SmallVec};
use std::alloc::Layout;
use std::collections::HashSet;
use std::ffi::CString;
use std::isize;
use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop};
use std::num::Wrapping;
use std::ops::Range;
use std::os::raw::c_char;
#[cfg(debug_assertions)]
use std::os::raw::c_void;
use std::ptr::{self, NonNull};
use std::slice;
use std::str;
use thin_vec::ThinVec;

/// Result type for ToShmem::to_shmem.
///
/// The String is an error message describing why the call failed.
pub type Result<T> = std::result::Result<ManuallyDrop<T>, String>;

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
    /// Pointers to every shareable value that we store in the shared memory
    /// buffer.  We use this to assert against encountering the same value
    /// twice, e.g. through another Arc reference, so that we don't
    /// inadvertently store duplicate copies of values.
    #[cfg(debug_assertions)]
    shared_values: HashSet<*const c_void>,
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
        }
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
    pub fn write<T: ToShmem>(&mut self, value: &T) -> std::result::Result<*mut T, String> {
        // Reserve space for the value.
        let dest: *mut T = self.alloc_value();

        // Make a clone of the value with all of its heap allocations
        // placed in the shared memory buffer.
        let value = value.to_shmem(self)?;

        unsafe {
            // Copy the value into the buffer.
            ptr::write(dest, ManuallyDrop::into_inner(value));
        }

        // Return a pointer to the shared value.
        Ok(dest)
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
        assert!(start <= std::isize::MAX as usize); // for the cast below

        // Reserve space for the value.
        let end = start.checked_add(layout.size()).unwrap();
        assert!(end <= self.capacity);

        self.index = end;
        unsafe { self.buffer.add(start) as *mut T }
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
    ///
    /// Returns a Result so that we can gracefully recover from unexpected
    /// content.
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self>;
}

#[macro_export]
macro_rules! impl_trivial_to_shmem {
    ($($ty:ty),*) => {
        $(
            impl $crate::ToShmem for $ty {
                fn to_shmem(
                    &self,
                    _builder: &mut $crate::SharedMemoryBuilder,
                ) -> $crate::Result<Self> {
                    $crate::Result::Ok(::std::mem::ManuallyDrop::new(*self))
                }
            }
        )*
    };
}

impl_trivial_to_shmem!(
    (),
    bool,
    f32,
    f64,
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
    isize,
    usize
);

impl_trivial_to_shmem!(cssparser::SourceLocation);
impl_trivial_to_shmem!(cssparser::TokenSerializationType);

impl<T> ToShmem for PhantomData<T> {
    fn to_shmem(&self, _builder: &mut SharedMemoryBuilder) -> Result<Self> {
        Ok(ManuallyDrop::new(*self))
    }
}

impl<T: ToShmem> ToShmem for Range<T> {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        Ok(ManuallyDrop::new(Range {
            start: ManuallyDrop::into_inner(self.start.to_shmem(builder)?),
            end: ManuallyDrop::into_inner(self.end.to_shmem(builder)?),
        }))
    }
}

impl ToShmem for cssparser::UnicodeRange {
    fn to_shmem(&self, _builder: &mut SharedMemoryBuilder) -> Result<Self> {
        Ok(ManuallyDrop::new(cssparser::UnicodeRange {
            start: self.start,
            end: self.end,
        }))
    }
}

impl<T: ToShmem, U: ToShmem> ToShmem for (T, U) {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        Ok(ManuallyDrop::new((
            ManuallyDrop::into_inner(self.0.to_shmem(builder)?),
            ManuallyDrop::into_inner(self.1.to_shmem(builder)?),
        )))
    }
}

impl<T: ToShmem> ToShmem for Wrapping<T> {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        Ok(ManuallyDrop::new(Wrapping(ManuallyDrop::into_inner(
            self.0.to_shmem(builder)?,
        ))))
    }
}

impl<T: ToShmem> ToShmem for Box<T> {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        // Reserve space for the boxed value.
        let dest: *mut T = builder.alloc_value();

        // Make a clone of the boxed value with all of its heap allocations
        // placed in the shared memory buffer.
        let value = (**self).to_shmem(builder)?;

        unsafe {
            // Copy the value into the buffer.
            ptr::write(dest, ManuallyDrop::into_inner(value));

            Ok(ManuallyDrop::new(Box::from_raw(dest)))
        }
    }
}

/// Converts all the items in `src` into shared memory form, writes them into
/// the specified buffer, and returns a pointer to the slice.
unsafe fn to_shmem_slice_ptr<'a, T, I>(
    src: I,
    dest: *mut T,
    builder: &mut SharedMemoryBuilder,
) -> std::result::Result<*mut [T], String>
where
    T: 'a + ToShmem,
    I: ExactSizeIterator<Item = &'a T>,
{
    let dest = slice::from_raw_parts_mut(dest, src.len());

    // Make a clone of each element from the iterator with its own heap
    // allocations placed in the buffer, and copy that clone into the buffer.
    for (src, dest) in src.zip(dest.iter_mut()) {
        ptr::write(dest, ManuallyDrop::into_inner(src.to_shmem(builder)?));
    }

    Ok(dest)
}

/// Writes all the items in `src` into a slice in the shared memory buffer and
/// returns a pointer to the slice.
pub unsafe fn to_shmem_slice<'a, T, I>(
    src: I,
    builder: &mut SharedMemoryBuilder,
) -> std::result::Result<*mut [T], String>
where
    T: 'a + ToShmem,
    I: ExactSizeIterator<Item = &'a T>,
{
    let dest = builder.alloc_array(src.len());
    to_shmem_slice_ptr(src, dest, builder)
}

impl<T: ToShmem> ToShmem for Box<[T]> {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        unsafe {
            let dest = to_shmem_slice(self.iter(), builder)?;
            Ok(ManuallyDrop::new(Box::from_raw(dest)))
        }
    }
}

impl ToShmem for Box<str> {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        // Reserve space for the string bytes.
        let dest: *mut u8 = builder.alloc_array(self.len());

        unsafe {
            // Copy the value into the buffer.
            ptr::copy(self.as_ptr(), dest, self.len());

            Ok(ManuallyDrop::new(Box::from_raw(
                str::from_utf8_unchecked_mut(slice::from_raw_parts_mut(dest, self.len())),
            )))
        }
    }
}

impl ToShmem for String {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        // Reserve space for the string bytes.
        let dest: *mut u8 = builder.alloc_array(self.len());

        unsafe {
            // Copy the value into the buffer.
            ptr::copy(self.as_ptr(), dest, self.len());

            Ok(ManuallyDrop::new(String::from_raw_parts(
                dest,
                self.len(),
                self.len(),
            )))
        }
    }
}

impl ToShmem for CString {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        let len = self.as_bytes_with_nul().len();

        // Reserve space for the string bytes.
        let dest: *mut c_char = builder.alloc_array(len);

        unsafe {
            // Copy the value into the buffer.
            ptr::copy(self.as_ptr(), dest, len);

            Ok(ManuallyDrop::new(CString::from_raw(dest)))
        }
    }
}

impl<T: ToShmem> ToShmem for Vec<T> {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        unsafe {
            let dest = to_shmem_slice(self.iter(), builder)? as *mut T;
            let dest_vec = Vec::from_raw_parts(dest, self.len(), self.len());
            Ok(ManuallyDrop::new(dest_vec))
        }
    }
}

impl<T: ToShmem, A: Array<Item = T>> ToShmem for SmallVec<A> {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        let dest_vec = unsafe {
            if self.spilled() {
                // Place the items in a separate allocation in the shared memory
                // buffer.
                let dest = to_shmem_slice(self.iter(), builder)? as *mut T;
                SmallVec::from_raw_parts(dest, self.len(), self.len())
            } else {
                // Place the items inline.
                let mut s = SmallVec::new();
                to_shmem_slice_ptr(self.iter(), s.as_mut_ptr(), builder)?;
                s.set_len(self.len());
                s
            }
        };

        Ok(ManuallyDrop::new(dest_vec))
    }
}

impl<T: ToShmem> ToShmem for Option<T> {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        let v = match self {
            Some(v) => Some(ManuallyDrop::into_inner(v.to_shmem(builder)?)),
            None => None,
        };

        Ok(ManuallyDrop::new(v))
    }
}

impl<T: ToShmem, S> ToShmem for HashSet<T, S>
where
    Self: Default,
{
    fn to_shmem(&self, _builder: &mut SharedMemoryBuilder) -> Result<Self> {
        if !self.is_empty() {
            return Err(format!(
                "ToShmem failed for HashSet: We only support empty sets \
                 (we don't expect custom properties in UA sheets, they're observable by content)",
            ));
        }
        Ok(ManuallyDrop::new(Self::default()))
    }
}

impl<T: ToShmem> ToShmem for Arc<T> {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        // Assert that we don't encounter any shared references to values we
        // don't expect.
        #[cfg(debug_assertions)]
        assert!(
            !builder.shared_values.contains(&self.heap_ptr()),
            "ToShmem failed for Arc<{}>: encountered a value with multiple \
            references.",
            std::any::type_name::<T>()
        );

        // Make a clone of the Arc-owned value with all of its heap allocations
        // placed in the shared memory buffer.
        let value = (**self).to_shmem(builder)?;

        // Create a new Arc with the shared value and have it place its
        // ArcInner in the shared memory buffer.
        unsafe {
            let static_arc = Arc::new_static(
                |layout| builder.alloc(layout),
                ManuallyDrop::into_inner(value),
            );

            #[cfg(debug_assertions)]
            builder.shared_values.insert(self.heap_ptr());

            Ok(ManuallyDrop::new(static_arc))
        }
    }
}

impl<H: ToShmem, T: ToShmem> ToShmem for ThinArc<H, T> {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        // We don't currently have any shared ThinArc values in stylesheets,
        // so don't support them for now.
        #[cfg(debug_assertions)]
        assert!(
            !builder.shared_values.contains(&self.heap_ptr()),
            "ToShmem failed for ThinArc<T>: encountered a value with multiple references, which \
             is not currently supported",
        );

        // Make a clone of the Arc-owned header and slice values with all of
        // their heap allocations placed in the shared memory buffer.
        let header = self.header.header.to_shmem(builder)?;
        let mut values = Vec::with_capacity(self.slice.len());
        for v in self.slice.iter() {
            values.push(v.to_shmem(builder)?);
        }

        // Create a new ThinArc with the shared value and have it place
        // its ArcInner in the shared memory buffer.
        unsafe {
            let static_arc = ThinArc::static_from_header_and_iter(
                |layout| builder.alloc(layout),
                ManuallyDrop::into_inner(header),
                values.into_iter().map(ManuallyDrop::into_inner),
            );

            #[cfg(debug_assertions)]
            builder.shared_values.insert(self.heap_ptr());

            Ok(ManuallyDrop::new(static_arc))
        }
    }
}

impl<T: ToShmem> ToShmem for ThinVec<T> {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        let len = self.len();
        if len == 0 {
            return Ok(ManuallyDrop::new(Self::new()));
        }

        assert_eq!(mem::size_of::<Self>(), mem::size_of::<*const ()>());

        // nsTArrayHeader size.
        // FIXME: Would be nice not to hard-code this, but in practice thin-vec crate also relies
        // on this.
        let header_size = 2 * mem::size_of::<u32>();
        let header_align = mem::size_of::<u32>();

        let item_size = mem::size_of::<T>();
        let item_align = mem::align_of::<T>();

        // We don't need to support underalignment for now, this could be supported if needed.
        assert!(item_align >= header_align);

        // This is explicitly unsupported by ThinVec, see:
        // https://searchfox.org/mozilla-central/rev/ad732108b073742d7324f998c085f459674a6846/third_party/rust/thin-vec/src/lib.rs#375-386
        assert!(item_align <= header_size);
        let header_padding = 0;

        let layout = Layout::from_size_align(
            header_size + header_padding + padded_size(item_size, item_align) * len,
            item_align,
        )
        .unwrap();

        let shmem_header_ptr = builder.alloc::<u8>(layout);
        let shmem_data_ptr = unsafe { shmem_header_ptr.add(header_size + header_padding) };

        let data_ptr = self.as_ptr() as *const T as *const u8;
        let header_ptr = unsafe { data_ptr.sub(header_size + header_padding) };

        unsafe {
            // Copy the header. Note this might copy a wrong capacity, but it doesn't matter,
            // because shared memory ptrs are immutable anyways, and we can't relocate.
            ptr::copy(header_ptr, shmem_header_ptr, header_size);
            // ToShmem + copy the contents into the shared buffer.
            to_shmem_slice_ptr(self.iter(), shmem_data_ptr as *mut T, builder)?;
            // Return the new ThinVec, which is just a pointer to the shared memory buffer.
            let shmem_thinvec: Self = mem::transmute(shmem_header_ptr);

            // Sanity-check that the ptr and length match.
            debug_assert_eq!(shmem_thinvec.as_ptr(), shmem_data_ptr as *const T);
            debug_assert_eq!(shmem_thinvec.len(), len);

            Ok(ManuallyDrop::new(shmem_thinvec))
        }
    }
}

impl ToShmem for SmallBitVec {
    fn to_shmem(&self, builder: &mut SharedMemoryBuilder) -> Result<Self> {
        let storage = match self.clone().into_storage() {
            InternalStorage::Spilled(vs) => {
                // Reserve space for the boxed slice values.
                let len = vs.len();
                let dest: *mut usize = builder.alloc_array(len);

                unsafe {
                    // Copy the value into the buffer.
                    let src = vs.as_ptr() as *const usize;
                    ptr::copy(src, dest, len);

                    let dest_slice =
                        Box::from_raw(slice::from_raw_parts_mut(dest, len) as *mut [usize]);
                    InternalStorage::Spilled(dest_slice)
                }
            },
            InternalStorage::Inline(x) => InternalStorage::Inline(x),
        };
        Ok(ManuallyDrop::new(unsafe {
            SmallBitVec::from_storage(storage)
        }))
    }
}

#[cfg(feature = "string_cache")]
impl<Static: string_cache::StaticAtomSet> ToShmem for string_cache::Atom<Static> {
    fn to_shmem(&self, _: &mut SharedMemoryBuilder) -> Result<Self> {
        // NOTE(emilio): In practice, this can be implemented trivially if
        // string_cache could expose the implementation detail of static atoms
        // being an index into the static table (and panicking in the
        // non-static, non-inline cases).
        unimplemented!(
            "If servo wants to share stylesheets across processes, \
             then ToShmem for Atom needs to be implemented"
        )
    }
}
