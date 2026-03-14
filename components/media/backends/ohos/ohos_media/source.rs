/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

use log::warn;
use ohos_media_sys::avbuffer::OH_AVBuffer;
use ohos_media_sys::avcodec_base::OH_AVDataSourceExt;

type SeekDataClosure = Box<dyn Fn(u64) -> bool + Send + Sync>;
const DEFAULT_CACHE_SIZE: usize = 8 * 1024 * 1024; // 8MB
pub struct MediaSourceBuilder {
    pub enough_data: Option<Box<dyn Fn() + Send + Sync>>,
    pub seek_data: Option<SeekDataClosure>,
}

impl MediaSourceBuilder {
    pub fn set_enough_data<F: Fn() + Send + Sync + Clone + 'static>(mut self, callback: F) -> Self {
        self.enough_data = Some(Box::new(callback));
        self
    }

    pub fn set_seek_data<F: Fn(u64) -> bool + Send + Sync + Clone + 'static>(
        mut self,
        callback: F,
    ) -> Self {
        self.seek_data = Some(Box::new(callback));
        self
    }

    pub fn build(self) -> MediaSourceWrapper {
        MediaSourceWrapper::new(self)
    }
}

pub struct MediaSourceWrapper {
    pub(crate) data_src: ohos_media_sys::avcodec_base::OH_AVDataSourceExt,
    total_media_source_size: usize,
    playback_buffer: Arc<Mutex<PlaybackBuffer>>,
    closure_handle: *mut Box<dyn Fn(*mut u8, u32, i64) -> i32>,
}

// AVPlayer it self already have a short buffer internally,
// we can just schedule when we does not have data for that specific location.
impl MediaSourceWrapper {
    pub fn builder() -> MediaSourceBuilder {
        MediaSourceBuilder {
            enough_data: None,
            seek_data: None,
        }
    }

    pub fn new(source_cb: MediaSourceBuilder) -> Self {
        let playback_buffer = Arc::new(Mutex::new(PlaybackBuffer::new(source_cb.enough_data)));

        let playback_buffer_clone = playback_buffer.clone();

        let read_at_closure = move |buffer: *mut u8, length: u32, pos: i64| -> i32 {
            log::debug!(
                "Inside Read At Closure: {:p}, length: {}, pos: {}",
                buffer,
                length,
                pos
            );
            let mut playback_buffer_lock = playback_buffer_clone.lock().unwrap();
            let (read_bytes, seek_pos) = playback_buffer_lock.read_data(buffer, length, pos);
            if let Some(seek_pos) = seek_pos {
                if let Some(seek_closure) = &source_cb.seek_data {
                    seek_closure(seek_pos);
                }
            }
            read_bytes
        };

        let box_closure: Box<dyn Fn(*mut u8, u32, i64) -> i32> = Box::new(read_at_closure);
        // Why do we need double boxing here? Because we need to convert the closure into a raw pointer to pass to C, but Rust does not allow us to directly convert a Box<dyn Fn> into a raw pointer, we need to first box it and then convert the box into a raw pointer.
        let double_box_closure = Box::new(box_closure);

        extern "C" fn oh_avdatasource_read_at_callback(
            data: *mut OH_AVBuffer,
            length: i32,
            pos: i64,
            user_data: *mut std::ffi::c_void,
        ) -> i32 {
            let f = unsafe { &*(user_data as *mut Box<dyn Fn(*mut u8, u32, i64) -> i32>) };
            let buffer_addr = unsafe { ohos_media_sys::avbuffer::OH_AVBuffer_GetAddr(data) };
            f(buffer_addr, length as u32, pos)
        }
        let data_src = OH_AVDataSourceExt {
            size: 0,
            readAt: Some(oh_avdatasource_read_at_callback),
        };

        let raw_ptr_f = Box::into_raw(double_box_closure);

        Self {
            data_src,
            total_media_source_size: 0,
            playback_buffer,
            closure_handle: raw_ptr_f,
        }
    }

    pub fn set_input_size(&mut self, size: usize) {
        log::debug!("Setting input size to {}", size);
        if self.total_media_source_size == 0 {
            self.total_media_source_size = size;
            self.data_src.size = size as i64;
        }
    }

    pub fn push_data(&self, data: Vec<u8>) -> bool {
        let mut playback_buffer_lock = self.playback_buffer.lock().unwrap();
        playback_buffer_lock.push_buffer(data)
    }

    pub fn end_of_stream(&self) {
        self.playback_buffer.lock().unwrap().end_of_stream();
    }

    pub(crate) fn get_raw_inner_source(&mut self) -> *mut OH_AVDataSourceExt {
        &mut self.data_src as *mut OH_AVDataSourceExt // Will it be considered not safe when accessing this pointer in Other function?
    }

    pub(crate) fn get_user_data(&self) -> *mut std::ffi::c_void {
        self.closure_handle as *mut std::ffi::c_void
    }
}

impl Drop for MediaSourceWrapper {
    fn drop(&mut self) {
        unsafe {
            let box_closure = Box::from_raw(self.closure_handle);
            drop(box_closure);
        }
    }
}

/// There would be two thread interact with playbackbuffer,
/// 1. AVPlayer Client Thread, will call read.
/// 2. Script Thread, will try to push_buffer into buffer.
pub struct PlaybackBuffer {
    enough_data_closure: Option<Box<dyn Fn() + Send + Sync>>,
    buffer_data_head: i64,
    is_seeking: bool,
    buffer: Vec<u8>,
}

impl PlaybackBuffer {
    pub fn new(enough_data_closure: Option<Box<dyn Fn() + Send + Sync>>) -> Self {
        PlaybackBuffer {
            enough_data_closure,
            buffer_data_head: 0,
            is_seeking: false,
            buffer: Vec::with_capacity(DEFAULT_CACHE_SIZE),
        }
    }

    /// Return (Number of Bytes read, Some(Seek Position) if no data at that position)
    pub fn read_data(&mut self, dest: *mut u8, length: u32, pos: i64) -> (i32, Option<u64>) {
        // First check whether we have enough data at that position.
        let pos_offset = pos - self.buffer_data_head;
        let available_data = self.buffer.len() as i64 - pos_offset;
        if pos_offset < 0 || available_data <= 0 {
            warn!(
                "We don't have data at position {}, buffer head is at {}, buffer len is {}",
                pos,
                self.buffer_data_head,
                self.buffer.len()
            );

            if self.is_seeking {
                // Wait for a few seconds for seek to complete, if we still don't have data, return error to trigger seek again.
                return (0, None);
            } else {
                self.buffer.clear();
                self.buffer_data_head = pos;
                self.is_seeking = true;
                return (0, Some(pos as u64));
            }
        }
        let read_len = std::cmp::min(available_data as u32, length) as usize;
        let dest_slice = unsafe { std::slice::from_raw_parts_mut(dest, length as usize) };
        dest_slice[..read_len]
            .copy_from_slice(&self.buffer[(pos_offset) as usize..(pos_offset as usize + read_len)]);

        (read_len as i32, None)
    }

    /// Return False when we have enough data.
    pub fn push_buffer(&mut self, data: Vec<u8>) -> bool {
        if self.buffer.len() + data.len() > self.buffer.capacity() {
            if let Some(enough_data_closure) = &self.enough_data_closure {
                enough_data_closure();
            }
            return false;
        }
        self.buffer.extend_from_slice(&data);
        true
    }

    pub fn end_of_stream(&mut self) {
        self.is_seeking = false;
    }
}
