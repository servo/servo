/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Paint timer thread.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{Sender, select};
use embedder_traits::RefreshDriver;
use timers::{BoxedTimerCallback, TimerEventRequest, TimerScheduler};

/// Thread-based timer driver.
pub(crate) struct TimerDriver {
    join_handle: Option<JoinHandle<()>>,
    sender: Sender<TimerMessage>,
    next_timer_id: AtomicU32,
}

impl TimerDriver {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<TimerMessage>();
        let join_handle = thread::Builder::new()
            .name(String::from("PaintTimerThread"))
            .spawn(move || {
                let mut scheduler = TimerScheduler::default();
                let mut ids = HashMap::new();

                loop {
                    select! {
                        recv(receiver) -> message => {
                            match message {
                                Ok(TimerMessage::ScheduleTimer(id, request)) => {
                                    let timer_id = scheduler.schedule_timer(request);
                                    ids.insert(id, timer_id);
                                },
                                Ok(TimerMessage::CancelTimer(id)) => {
                                    if let Some(timer_id) = ids.get(&id) {
                                        scheduler.cancel_timer(*timer_id);
                                    }
                                },
                                _ => return,
                            }
                        },
                        recv(scheduler.wait_channel()) -> _message => {
                            scheduler.dispatch_completed_timers();
                        },
                    };
                }
            })
            .expect("Could not create paint timer thread.");

        Self {
            sender,
            join_handle: Some(join_handle),
            next_timer_id: Default::default(),
        }
    }

    /// Schedule `callback` to be executed after `duration` has elapsed.
    pub(crate) fn queue_timer(
        &self,
        duration: Duration,
        callback: BoxedTimerCallback,
    ) -> PaintTimerId {
        let timer_id = PaintTimerId(self.next_timer_id.fetch_add(1, Ordering::Relaxed));
        let event = TimerEventRequest { callback, duration };
        let message = TimerMessage::ScheduleTimer(timer_id, event);
        let _ = self.sender.send(message);
        timer_id
    }

    /// Cancel an existing timer using its ID.
    ///
    /// This will cancel a pending timer callback, assuming the timeout has not elapsed already.
    pub(crate) fn cancel_timer(&self, timer_id: PaintTimerId) {
        let _ = self.sender.send(TimerMessage::CancelTimer(timer_id));
    }
}

impl Default for TimerDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TimerDriver {
    fn drop(&mut self) {
        let _ = self.sender.send(TimerMessage::Quit);
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

/// A timer-based refresh driver is necessary, because we need a reliable way of waking up the
/// embedder's main thread, which may just be sleeping until the `EventLoopWaker` asks it to wake
/// up.
///
/// It would be nice to integrate this somehow into the embedder thread, but it would
/// require both some communication with the embedder and for all embedders to be well
/// behave respecting wakeup timeouts -- a bit too much to ask at the moment.
impl RefreshDriver for TimerDriver {
    fn observe_next_frame(&self, new_start_frame_callback: Box<dyn Fn() + Send + 'static>) {
        const FRAME_DURATION: Duration = Duration::from_millis(1000 / 120);
        self.queue_timer(FRAME_DURATION, new_start_frame_callback);
    }
}

/// ID of a timer created by the [`TimerDriver`].
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct PaintTimerId(u32);

/// Message for the [`TimerDriver`] channel.
enum TimerMessage {
    ScheduleTimer(PaintTimerId, TimerEventRequest),
    CancelTimer(PaintTimerId),
    Quit,
}
