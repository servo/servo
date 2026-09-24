/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::c_void;
use std::ops::Range;
use std::sync::Arc;

use js::context::JSContext;
use js::rooted;
use js::rust::wrappers2::NewExternalArrayBuffer;
use js::typedarray::{ArrayBufferU8, HeapArrayBuffer};
use jstraceable_derive::JSTraceable;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::buffer_source::HeapBufferSource;
use script_bindings::trace::RootedTraceableBox;
use servo_base::generic_channel::GenericSharedMemory;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DataBlock {
    #[conditional_malloc_size_of]
    data: Arc<GenericSharedMemory>,
    /// Data views (mutable subslices of data)
    data_views: Vec<DataView>,
}

/// Returns true if two non-inclusive ranges overlap
// https://stackoverflow.com/questions/3269434/whats-the-most-efficient-way-to-test-if-two-ranges-overlap
fn range_overlap<T: std::cmp::PartialOrd>(range1: &Range<T>, range2: &Range<T>) -> bool {
    range1.start < range2.end && range2.start < range1.end
}

impl DataBlock {
    pub(crate) fn new_zeroed(size: usize) -> Self {
        Self {
            data: Arc::new(GenericSharedMemory::from_byte(0, size)),
            data_views: Vec::new(),
        }
    }

    pub(crate) fn new_from_shared_memory(data: GenericSharedMemory) -> Self {
        Self {
            data: Arc::new(data),
            data_views: Vec::new(),
        }
    }

    #[cfg_attr(
        crown,
        expect(
            crown::unrooted_must_root,
            reason = "Underlying content is rooted when GC can happen"
        )
    )]
    pub(crate) fn clear_views(&mut self, cx: &mut JSContext) {
        // we need to pop one by one so we can root one by one for detach
        while let Some(DataView { buffer, .. }) = self.data_views.pop() {
            let buffer = RootedTraceableBox::new(buffer);
            assert!(buffer.detach_buffer(cx))
        }
    }

    /// Returns error if requested range is already mapped
    pub(crate) fn view(
        &mut self,
        cx: &mut JSContext,
        range: Range<usize>,
    ) -> Result<&DataView, ()> {
        if self
            .data_views
            .iter()
            .any(|view| range_overlap(&view.range, &range))
        {
            return Err(());
        }
        let range_len = range
            .end
            .checked_sub(range.start)
            .expect("range end must be >= range start");
        assert!(range.end <= self.data.as_ref().len());

        /// `freeFunc()` must be threadsafe, should be safely callable from any thread
        /// without causing conflicts, unexpected behavior.
        unsafe extern "C" fn free_func(_contents: *mut c_void, free_user_data: *mut c_void) {
            let raw: *const GenericSharedMemory = free_user_data.cast();
            // SAFETY: `free_func` is called by SM and returns ownership of the Arc we
            // leaked below with `into_raw`. Hence it is safe to reconstruct the Arc,
            // and destroy it to release the reference count.
            drop(unsafe { Arc::from_raw(raw) });
        }
        let raw: *const GenericSharedMemory = Arc::into_raw(Arc::clone(&self.data));
        // SAFETY: We leaked the Arc, so the underlying slice will stay alive
        // until `free_func` is called. `range.start..range.end` is inside
        // the valid range of the slice.
        let data_ptr = unsafe { (*raw).as_ptr().add(range.start) };
        rooted!(&in(cx) let object = unsafe {
            NewExternalArrayBuffer(
                cx,
                range_len,
                // FIXME(jschwe): I believe casting to a mutable pointer is unsound.
                // We would need interior mutability.
                data_ptr.cast_mut().cast(),
                Some(free_func),
                raw as _,
            )
        });
        self.data_views.push(DataView {
            range,
            buffer: HeapBufferSource::new(object.handle()),
        });
        Ok(self.data_views.last().unwrap())
    }

    #[cfg_attr(
        crown,
        expect(
            crown::unrooted_must_root,
            reason = "No GC can happen when this is called"
        )
    )]
    pub(crate) fn consume(self) -> GenericSharedMemory {
        Arc::into_inner(self.data).expect("No view should be alive")
    }
}

/// DataView are created from `NewExternalArrayBuffer`,
/// so SM will detach the underlying buffer when the DataView is GCed.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DataView {
    #[no_trace]
    range: Range<usize>,
    #[ignore_malloc_size_of = "HeapBufferSource"]
    buffer: HeapBufferSource<ArrayBufferU8>,
}

impl DataView {
    pub(crate) fn array_buffer(&self) -> RootedTraceableBox<HeapArrayBuffer> {
        self.buffer.get_typed_array().unwrap()
    }
}
