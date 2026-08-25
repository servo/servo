/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::slice;
use std::sync::{Arc, Mutex};

use log::debug;
use ohos_media_sys::avbuffer::OH_AVBuffer;
use ohos_media_sys::avcodec_base::OH_AVDataSourceExt;
use ohos_media_sys::avplayer::OH_AVPlayer_SetDataSource;
use ohos_media_sys::avplayer_base::OH_AVPlayer;

use crate::ohos_media::source_builder::MediaSourceBuilder;

const DEFAULT_CACHE_SIZE: usize = 8 * 1024 * 1024; // 8MB

pub struct MediaSourceWrapper {
    pub(crate) data_src: ohos_media_sys::avcodec_base::OH_AVDataSourceExt,
    total_media_source_size: usize,
    playback_buffer: Arc<Mutex<PlaybackBuffer>>,
    closure_handle: *mut Box<dyn Fn(*mut u8, u32, i64) -> i32>,
}

impl MediaSourceWrapper {
    pub fn builder() -> MediaSourceBuilder {
        MediaSourceBuilder {
            enough_data: None,
            seek_data: None,
        }
    }
}

// AVPlayer itself already have a short buffer internally,
// we can just schedule fetch if we does not have data for that specific location.
impl MediaSourceWrapper {
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
            let buffer = unsafe { slice::from_raw_parts_mut(buffer, length as usize) };
            let (read_bytes, seek_pos) = {
                let mut playback_buffer_lock = playback_buffer_clone.lock().unwrap();
                playback_buffer_lock.read_data(buffer, pos)
            };
            // The playback_buffer lock must be released before calling the seek
            // closure, which blocks on IPC with the script thread. Holding the
            // lock here would deadlock if the script thread is simultaneously
            // trying to push_data (which also acquires this lock).
            if let Some(seek_pos) = seek_pos {
                if let Some(seek_closure) = &source_cb.seek_data {
                    seek_closure(seek_pos);
                }
            }
            read_bytes
        };

        let box_closure: Box<dyn Fn(*mut u8, u32, i64) -> i32> = Box::new(read_at_closure);
        // Double boxing is needed because we need to convert the closure into a raw pointer to pass to C,
        // but Rust does not allow us to directly convert a Box<dyn Fn> into a raw pointer, we need to first box it
        // and then convert the box into a raw pointer.
        let double_box_closure = Box::new(box_closure);

        extern "C" fn oh_avdatasource_read_at_callback(
            data: *mut OH_AVBuffer,
            length: i32,
            pos: i64,
            user_data: *mut std::ffi::c_void,
        ) -> i32 {
            assert!(
                !user_data.is_null(),
                "oh_avdatasource_read_at_callback: user_data must not be null"
            );
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
        self.playback_buffer.lock().unwrap().notify_seek_done();
    }

    pub fn push_data(&self, data: Vec<u8>) -> bool {
        let mut playback_buffer_lock = self.playback_buffer.lock().unwrap();
        playback_buffer_lock.push_buffer(data)
    }

    pub fn end_of_stream(&self) {
        self.playback_buffer.lock().unwrap().end_of_stream();
    }

    pub fn set_data_src(&mut self, av_player: *mut OH_AVPlayer) {
        unsafe {
            OH_AVPlayer_SetDataSource(
                av_player,
                &mut self.data_src as *mut OH_AVDataSourceExt,
                self.closure_handle as *mut std::ffi::c_void,
            );
        }
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
    has_active_request: bool,
    last_read_end: i64,
    is_seeking: bool,
    buffer: Vec<u8>,
}

impl PlaybackBuffer {
    pub fn new(enough_data_closure: Option<Box<dyn Fn() + Send + Sync>>) -> Self {
        PlaybackBuffer {
            enough_data_closure,
            buffer_data_head: 0,
            has_active_request: false,
            is_seeking: false,
            last_read_end: 0,
            buffer: Vec::with_capacity(DEFAULT_CACHE_SIZE),
        }
    }

    pub fn notify_seek_done(&mut self) {
        self.is_seeking = false;
    }

    /// Return (Number of Bytes read, Some(Seek Position) if no data at that position)
    pub fn read_data(&mut self, dest_slice: &mut [u8], pos: i64) -> (i32, Option<u64>) {
        if self.is_seeking {
            debug!(
                "Currently seeking, cannot read data at position {}, buffer head is at {}, buffer len is {}， has_active_request: {}",
                pos,
                self.buffer_data_head,
                self.buffer.len(),
                self.has_active_request
            );
            return (0, None);
        }
        // First check whether we have enough data at that position.
        let pos_offset = pos - self.buffer_data_head;

        let available_data = self.buffer.len() as i64 - pos_offset;
        let need_seek = pos_offset < 0 ||
            (available_data <= 0 &&
                (!self.has_active_request ||
                    pos >= self.buffer_data_head + self.buffer.capacity() as i64));
        if need_seek {
            debug!(
                "We don't have data at position {}, buffer head is at {}, buffer len is {}， has_active_request: {}",
                pos,
                self.buffer_data_head,
                self.buffer.len(),
                self.has_active_request
            );
            self.buffer.clear();
            self.buffer_data_head = pos;
            self.has_active_request = true;
            self.is_seeking = true;
            return (0, Some(pos as u64));
        }
        let read_len = available_data.clamp(0, dest_slice.len() as i64) as usize;
        if read_len == 0 {
            debug!(
                "No available data to read at position {}, buffer head is at {}, buffer len is {}， has_active_request: {}",
                pos,
                self.buffer_data_head,
                self.buffer.len(),
                self.has_active_request
            );
            return (0, None);
        }
        dest_slice[..read_len]
            .copy_from_slice(&self.buffer[(pos_offset) as usize..(pos_offset as usize + read_len)]);

        self.last_read_end = pos + read_len as i64;
        (read_len as i32, None)
    }

    /// Return False when we have enough data.
    pub fn push_buffer(&mut self, data: Vec<u8>) -> bool {
        // Reject data while a seek is in progress. Between the buffer being
        // cleared/reset for a new seek position and the old fetch being
        // cancelled, stale data from the previous fetch could arrive and
        // corrupt the buffer (it would be appended as if it started at the
        // new seek position). Silently discard it.
        if self.is_seeking {
            return true;
        }
        if self.buffer.len() + data.len() > self.buffer.capacity() {
            debug!(
                "Buffer is full, cannot push more data,current head: {}, current len: {}, incoming data len: {}, capacity: {}",
                self.buffer_data_head,
                self.buffer.len(),
                data.len(),
                self.buffer.capacity()
            );
            self.has_active_request = false;
            if let Some(enough_data_closure) = &self.enough_data_closure {
                enough_data_closure();
            }
            return false;
        }
        self.buffer.extend_from_slice(&data);
        true
    }

    pub fn end_of_stream(&mut self) {
        self.has_active_request = false;
    }
}
