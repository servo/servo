/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A timer thread that composites near the end of the frame.
//!
//! This is useful when we need to composite next frame but we want to opportunistically give the
//! painting thread time to paint if it can.

use compositor_thread::{CompositorProxy, Msg};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, Builder};
use std::time::Duration;
use std::u32;
use time;

/// The amount of time in nanoseconds that we give to the painting thread to paint. When this
/// expires, we give up and composite anyway.
static TIMEOUT: u64 = 12_000_000;

pub struct DelayedCompositionTimerProxy {
    sender: Sender<ToDelayedCompositionTimerMsg>,
}

struct DelayedCompositionTimer {
    compositor_proxy: CompositorProxy,
    receiver: Receiver<ToDelayedCompositionTimerMsg>,
}

enum ToDelayedCompositionTimerMsg {
    Exit,
    ScheduleComposite(u64),
}

impl DelayedCompositionTimerProxy {
    pub fn new(compositor_proxy: CompositorProxy) -> DelayedCompositionTimerProxy {
        let (to_timer_sender, to_timer_receiver) = channel();
        Builder::new().spawn(move || {
            let mut timer = DelayedCompositionTimer {
                compositor_proxy: compositor_proxy,
                receiver: to_timer_receiver,
            };
            timer.run();
        }).unwrap();
        DelayedCompositionTimerProxy {
            sender: to_timer_sender,
        }
    }

    pub fn schedule_composite(&mut self, timestamp: u64) {
        self.sender.send(ToDelayedCompositionTimerMsg::ScheduleComposite(timestamp)).unwrap()
    }

    pub fn shutdown(&mut self) {
        self.sender.send(ToDelayedCompositionTimerMsg::Exit).unwrap()
    }
}

impl DelayedCompositionTimer {
    fn run(&mut self) {
        'outer: loop {
            let mut timestamp;
            loop {
                match self.receiver.recv() {
                    Ok(ToDelayedCompositionTimerMsg::ScheduleComposite(this_timestamp)) => {
                        timestamp = this_timestamp;
                        break
                    }
                    Ok(ToDelayedCompositionTimerMsg::Exit) => break 'outer,
                    _ => break 'outer,
                }
            }

            // Drain all messages from the queue.
            loop {
                match self.receiver.try_recv() {
                    Ok(ToDelayedCompositionTimerMsg::ScheduleComposite(this_timestamp)) => {
                        timestamp = this_timestamp;
                        break
                    }
                    _ => break,
                }
            }

            let target = timestamp + TIMEOUT;
            let now = time::precise_time_ns();
            if target > now {
                let delta_ns = target - now;
                thread::sleep(duration_from_nanoseconds(delta_ns));
            }
            self.compositor_proxy.send(Msg::DelayedCompositionTimeout(timestamp));
        }
    }
}

fn duration_from_nanoseconds(nanos: u64) -> Duration {
    pub const NANOS_PER_SEC: u32 = 1_000_000_000;

    // Get number of seconds.
    let secs = nanos / NANOS_PER_SEC as u64;

    // Get number of extra nanoseconds. This should always fit in a u32, but check anyway.
    let subsec_nanos = nanos % NANOS_PER_SEC as u64;
    assert!(subsec_nanos <= u32::MAX as u64);

    Duration::new(secs, subsec_nanos as u32)
}
