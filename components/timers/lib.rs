/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A generic timer scheduler module that can be integrated into a crossbeam based event
//! loop or used to launch a background timer thread.

#![deny(unsafe_code)]

use std::cmp::{self, Ord};
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, after, never};
use malloc_size_of_derive::MallocSizeOf;

/// A callback to pass to the [`TimerScheduler`] to be called when the timer is
/// dispatched.
pub type BoxedTimerCallback = Box<dyn Fn() + Send + 'static>;

/// Requests a TimerEvent-Message be sent after the given duration.
#[derive(MallocSizeOf)]
pub struct TimerEventRequest {
    #[ignore_malloc_size_of = "Size of a boxed function"]
    pub callback: BoxedTimerCallback,
    pub duration: Duration,
}

impl TimerEventRequest {
    fn dispatch(self) {
        (self.callback)()
    }
}

#[derive(MallocSizeOf)]
struct ScheduledEvent {
    id: TimerId,
    request: TimerEventRequest,
    for_time: Instant,
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &ScheduledEvent) -> cmp::Ordering {
        self.for_time.cmp(&other.for_time).reverse()
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &ScheduledEvent) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ScheduledEvent {}
impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &ScheduledEvent) -> bool {
        std::ptr::eq(self, other)
    }
}

#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub struct TimerId(usize);

/// A queue of [`TimerEventRequest`]s that are stored in order of next-to-fire.
#[derive(Default, MallocSizeOf)]
pub struct TimerScheduler {
    /// A priority queue of future events, sorted by due time.
    queue: BinaryHeap<ScheduledEvent>,

    /// The current timer id, used to generate new ones.
    current_id: usize,
}

impl TimerScheduler {
    /// Schedule a new timer for on this [`TimerScheduler`].
    pub fn schedule_timer(&mut self, request: TimerEventRequest) -> TimerId {
        let for_time = Instant::now() + request.duration;

        let id = TimerId(self.current_id);
        self.current_id += 1;

        self.queue.push(ScheduledEvent {
            id,
            request,
            for_time,
        });
        id
    }

    /// Cancel a timer with the given [`TimerId`]. If a timer with that id is not
    /// currently waiting to fire, do nothing.
    pub fn cancel_timer(&mut self, id: TimerId) {
        self.queue.retain(|event| event.id != id);
    }

    /// Get a [`Receiver<Instant>`] that receives a message after waiting for the next timer
    /// to fire. If there are no timers, the channel will *never* send a message.
    pub fn wait_channel(&self) -> Receiver<Instant> {
        self.queue
            .peek()
            .map(|event| {
                let now = Instant::now();
                if event.for_time < now {
                    after(Duration::ZERO)
                } else {
                    after(event.for_time - now)
                }
            })
            .unwrap_or_else(never)
    }

    /// Dispatch any timer events from this [`TimerScheduler`]'s `queue` when `now` is
    /// past the due time of the event.
    pub fn dispatch_completed_timers(&mut self) {
        let now = Instant::now();
        loop {
            match self.queue.peek() {
                // Dispatch the event if its due time is past.
                Some(event) if event.for_time <= now => {},
                // Otherwise, we're done dispatching events.
                _ => break,
            }
            // Remove the event from the priority queue (Note this only executes when the
            // first event has been dispatched
            self.queue
                .pop()
                .expect("Expected request")
                .request
                .dispatch();
        }
    }
}
