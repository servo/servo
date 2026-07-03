/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::mem;
use std::panic::Location;

use serde::Serialize;

use super::{GenericSender, SendResult};

/// A buffered sender that collects individual messages (`U`) and sends them
/// as a single batched message (`T`) via a user-provided packing closure.
///
/// The buffer is flushed automatically when it reaches `max_buffer` items,
/// or explicitly via [`flush`](GenericBufferedSender::flush).
/// [`send_immediate`](GenericBufferedSender::send_immediate)
/// combines the current buffer contents with the new message into a single
/// packed message, ensuring ordering without an extra flush step.
pub struct GenericBufferedSender<T, U>
where
    T: Serialize,
{
    sender: GenericSender<T>,
    buffer: RefCell<Vec<U>>,
    buffering: Box<dyn Fn(Vec<U>) -> T>,
    max_buffer: usize,
}

impl<T: Serialize, U> GenericBufferedSender<T, U> {
    /// Create a new buffered sender.
    ///
    /// * `sender` — the underlying `GenericSender<T>` that delivers packed messages.
    /// * `buffering` — closure that packs a `Vec<U>` into a single `T`.
    /// * `max_buffer` — automatic flush is triggered when the buffer reaches this size.
    pub fn new(
        sender: GenericSender<T>,
        buffering: Box<dyn Fn(Vec<U>) -> T>,
        max_buffer: usize,
    ) -> Self {
        Self {
            sender,
            buffer: RefCell::new(Vec::new()),
            buffering,
            max_buffer,
        }
    }

    /// Returns `true` if the buffer contains no pending messages.
    pub fn is_empty(&self) -> bool {
        self.buffer.borrow().is_empty()
    }

    /// Returns the number of pending messages in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.borrow().len()
    }

    /// Buffer a message for later batched delivery.
    ///
    /// If the buffer reaches `max_buffer` items an automatic flush is triggered.
    pub fn send(&self, msg: U) -> SendResult {
        if self.buffer.borrow().len() + 1 >= self.max_buffer {
            self.send_immediate(msg)
        } else {
            self.buffer.borrow_mut().push(msg);
            Ok(())
        }
    }

    #[inline]
    #[track_caller]
    /// Buffer a message for later batched delivery.
    ///
    /// If the buffer reaches `max_buffer` items an automatic flush is triggered.
    /// Errors on automatic flush are logged.
    pub fn send_or_warn(&self, msg: U) {
        if let Err(error) = self.send(msg) {
            let location = Location::caller();
            log::warn!("Failed to send buffered messages due to `{error}` at {location:?}");
        }
    }

    /// Deliver a message immediately, combining it with any
    /// buffered messages into a single packed `T`.
    pub fn send_immediate(&self, msg: U) -> SendResult {
        let mut buffer = self.buffer.borrow_mut();
        buffer.push(msg);
        let msgs = mem::take(&mut *buffer);
        drop(buffer);
        let packed = (self.buffering)(msgs);
        self.sender.send(packed)
    }

    #[inline]
    #[track_caller]
    /// Deliver a message immediately, combining it with any
    /// buffered messages into a single packed `T`.
    ///
    /// Errors are logged.
    pub fn send_immediate_or_warn(&self, msg: U) {
        if let Err(error) = self.send_immediate(msg) {
            let location = Location::caller();
            log::warn!(
                "Failed to send (immediate) buffered messages due to `{error}` at {location:?}"
            );
        }
    }

    /// Flush all buffered messages by packing them into a single `T` and
    /// sending it.
    pub fn flush(&self) -> SendResult {
        let mut buffer = self.buffer.borrow_mut();
        if buffer.is_empty() {
            return Ok(());
        }
        let msgs = mem::take(&mut *buffer);
        drop(buffer);
        let packed = (self.buffering)(msgs);
        self.sender.send(packed)
    }

    #[inline]
    #[track_caller]
    /// Flush all buffered messages by packing them into a single `T` and sending it.
    /// Errors are logged
    pub fn flush_or_warn(&self) {
        if let Err(error) = self.flush() {
            let location = Location::caller();
            log::warn!("Failed to flush buffered messages due to `{error}` at {location:?}");
        }
    }

    /// Discard all buffered messages without sending them.
    pub fn discard(&self) {
        self.buffer.borrow_mut().clear();
    }

    /// If the last buffered item satisfies `pred`, remove it and return `true`.
    pub fn pop_last_if_matches(&self, pred: impl Fn(&U) -> bool) -> bool {
        let is_match = self
            .buffer
            .borrow()
            .last()
            .map(|x| pred(x))
            .unwrap_or(false);
        if is_match {
            self.buffer.borrow_mut().pop();
        }
        is_match
    }
}

impl<T: Serialize, U> Drop for GenericBufferedSender<T, U> {
    fn drop(&mut self) {
        // Best-effort flush on drop. Ignore send failures.
        if !self.buffer.borrow().is_empty() {
            let msgs = mem::take(&mut *self.buffer.borrow_mut());
            let packed = (self.buffering)(msgs);
            let _ = self.sender.send(packed);
        }
    }
}
