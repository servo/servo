/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcSender};
use script_traits::{TimerEvent, TimerEventRequest, TimerSchedulerMsg};
use std::cmp::{self, Ord};
use std::collections::BinaryHeap;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError::{Disconnected, Empty};
use std::thread;
use std::time::{Duration, Instant};

pub struct TimerScheduler;

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
        self as *const ScheduledEvent == other as *const ScheduledEvent
    }
}

impl TimerScheduler {
    pub fn start() -> IpcSender<TimerSchedulerMsg> {
        let (req_ipc_sender, req_ipc_receiver) = ipc::channel().expect("Channel creation failed.");
        let (req_sender, req_receiver) = mpsc::sync_channel(1);

        // We could do this much more directly with recv_timeout
        // (https://github.com/rust-lang/rfcs/issues/962).

        // util::thread doesn't give us access to the JoinHandle, which we need for park/unpark,
        // so we use the builder directly.
        let timeout_thread = thread::Builder::new()
            .name(String::from("TimerScheduler"))
            .spawn(move || {
                // We maintain a priority queue of future events, sorted by due time.
                let mut scheduled_events = BinaryHeap::<ScheduledEvent>::new();
                loop {
                    let now = Instant::now();
                    // Dispatch any events whose due time is past
                    loop {
                        match scheduled_events.peek() {
                            // Dispatch the event if its due time is past
                            Some(event) if event.for_time <= now => {
                                let TimerEventRequest(ref sender, source, id, _) = event.request;
                                let _ = sender.send(TimerEvent(source, id));
                            },
                            // Otherwise, we're done dispatching events
                            _ => break,
                        }
                        // Remove the event from the priority queue
                        // (Note this only executes when the first event has been dispatched
                        scheduled_events.pop();
                    }
                    // Look to see if there are any incoming events
                    match req_receiver.try_recv() {
                        // If there is an event, add it to the priority queue
                        Ok(TimerSchedulerMsg::Request(req)) => {
                            let TimerEventRequest(_, _, _, delay) = req;
                            let schedule = Instant::now() + Duration::from_millis(delay.get());
                            let event = ScheduledEvent { request: req, for_time: schedule };
                            scheduled_events.push(event);
                        },
                        // If there is no incoming event, park the thread,
                        // it will either be unparked when a new event arrives,
                        // or by a timeout.
                        Err(Empty) => match scheduled_events.peek() {
                            None => thread::park(),
                            Some(event) => thread::park_timeout(event.for_time - now),
                        },
                        // If the channel is closed or we are shutting down, we are done.
                        Ok(TimerSchedulerMsg::Exit) |
                        Err(Disconnected) => break,
                    }
                }
                // This thread can terminate if the req_ipc_sender is dropped.
                warn!("TimerScheduler thread terminated.");
            })
            .expect("Thread creation failed.")
            .thread()
            .clone();

        // A proxy that just routes incoming IPC requests over the MPSC channel to the timeout thread,
        // and unparks the timeout thread each time. Note that if unpark is called while the timeout
        // thread isn't parked, this causes the next call to thread::park by the timeout thread
        // not to block. This means that the timeout thread won't park when there is a request
        // waiting in the MPSC channel buffer.
        thread::Builder::new()
            .name(String::from("TimerProxy"))
            .spawn(move || {
                while let Ok(req) = req_ipc_receiver.recv() {
                    let mut shutting_down = false;
                    match req {
                        TimerSchedulerMsg::Exit => shutting_down = true,
                        _ => {}
                    }
                    let _ = req_sender.send(req);
                    timeout_thread.unpark();
                    if shutting_down {
                        break;
                    }
                }
                // This thread can terminate if the req_ipc_sender is dropped.
                warn!("TimerProxy thread terminated.");
            })
            .expect("Thread creation failed.");

        // Return the IPC sender
        req_ipc_sender
    }
}
