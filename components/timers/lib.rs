/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A generic timer scheduler module that can be integrated into a crossbeam based event
//! loop or used to launch a background timer thread.

#![deny(unsafe_code)]

use std::cmp::{self, Ord};
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

use base::id::PipelineId;
use crossbeam_channel::{Receiver, after, never};
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

/// Describes the source that requested the [`TimerEvent`].
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum TimerSource {
    /// The event was requested from a window (`ScriptThread`).
    FromWindow(PipelineId),
    /// The event was requested from a worker (`DedicatedGlobalWorkerScope`).
    FromWorker,
}

/// The id to be used for a [`TimerEvent`] is defined by the corresponding [`TimerEventRequest`].
#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub struct TimerEventId(pub u32);

/// A notification that a timer has fired. [`TimerSource`] must be `FromWindow` when
/// dispatched to `ScriptThread` and must be `FromWorker` when dispatched to a
/// `DedicatedGlobalWorkerScope`
#[derive(Debug, Deserialize, Serialize)]
pub struct TimerEvent(pub TimerSource, pub TimerEventId);

/// A callback to pass to the [`TimerScheduler`] to be called when the timer is
/// dispatched.
pub type BoxedTimerCallback = Box<dyn Fn(TimerEvent) + Send + 'static>;

/// Requests a TimerEvent-Message be sent after the given duration.
#[derive(MallocSizeOf)]
pub struct TimerEventRequest {
    #[ignore_malloc_size_of = "Size of a boxed function"]
    pub callback: BoxedTimerCallback,
    pub source: TimerSource,
    pub id: TimerEventId,
    pub duration: Duration,
}

impl TimerEventRequest {
    fn dispatch(self) {
        (self.callback)(TimerEvent(self.source, self.id))
    }
}

#[derive(MallocSizeOf)]
struct ScheduledEvent {
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

/// A queue of [`TimerEventRequest`]s that are stored in order of next-to-fire.
#[derive(Default, MallocSizeOf)]
pub struct TimerScheduler {
    /// A priority queue of future events, sorted by due time.
    queue: BinaryHeap<ScheduledEvent>,
}

impl TimerScheduler {
    /// Schedule a new timer for on this [`TimerScheduler`].
    pub fn schedule_timer(&mut self, request: TimerEventRequest) {
        let for_time = Instant::now() + request.duration;
        self.queue.push(ScheduledEvent { request, for_time });
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
