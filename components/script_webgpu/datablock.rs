/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::c_void;
use std::ops::Range;
use std::sync::Arc;

use js::context::JSContext;
use js::rooted;
use js::rust::wrappers2::{DetachArrayBuffer, NewExternalArrayBuffer};
use js::typedarray::HeapArrayBuffer;
use jstraceable_derive::JSTraceable;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::trace::RootedTraceableBox;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DataBlock {
    #[conditional_malloc_size_of]
    data: Arc<Box<[u8]>>,
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
        let data = vec![0; size];
        Self {
            data: Arc::new(data.into_boxed_slice()),
            data_views: Vec::new(),
        }
    }

    /// Panics if there is any active view or src data is not same length
    pub(crate) fn load(&mut self, src: &[u8]) {
        // `Arc::get_mut` ensures there are no views
        Arc::get_mut(&mut self.data).unwrap().clone_from_slice(src)
    }

    /// Panics if there is any active view
    pub(crate) fn data(&mut self) -> &mut [u8] {
        // `Arc::get_mut` ensures there are no views
        Arc::get_mut(&mut self.data).unwrap()
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
            rooted!(&in(cx) let b = unsafe { buffer.underlying_object().get() });
            assert!(unsafe { DetachArrayBuffer(cx, b.handle()) })
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
        assert!(range.end <= self.data.len());

        /// `freeFunc()` must be threadsafe, should be safely callable from any thread
        /// without causing conflicts, unexpected behavior.
        unsafe extern "C" fn free_func(_contents: *mut c_void, free_user_data: *mut c_void) {
            let raw: *const Box<[u8]> = free_user_data.cast();
            // SAFETY: `free_func` is called by SM and returns ownership of the Arc we
            // leaked below with `into_raw`. Hence it is safe to reconstruct the Arc,
            // and destroy it to release the reference count.
            drop(unsafe { Arc::from_raw(raw) });
        }
        let raw: *const Box<[u8]> = Arc::into_raw(Arc::clone(&self.data));
        // SAFETY: We leaked the Arc, so the underlying slice will stay alive
        // until `free_func` is called. `range.start..range.end` is inside
        // the valid range of the slice.
        let data_ptr = unsafe { (**raw).as_ptr().add(range.start) };
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
            buffer: HeapArrayBuffer::from(*object).unwrap(),
        });
        Ok(self.data_views.last().unwrap())
    }
}

/// DataView are created from `NewExternalArrayBuffer`,
/// so SM will detach the underlying buffer when the DataView is GCed.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DataView {
    #[no_trace]
    range: Range<usize>,
    #[ignore_malloc_size_of = "defined in mozjs"]
    buffer: HeapArrayBuffer,
}

impl DataView {
    pub(crate) fn array_buffer(&self) -> RootedTraceableBox<HeapArrayBuffer> {
        RootedTraceableBox::new(unsafe {
            HeapArrayBuffer::from(self.buffer.underlying_object().get()).unwrap()
        })
    }
}
