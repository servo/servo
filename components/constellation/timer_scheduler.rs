/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ipc_channel::ipc::{self, IpcSender};
use script_traits::{TimerEvent, TimerEventRequest};
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
    pub fn start() -> IpcSender<TimerEventRequest> {
        let (req_ipc_sender, req_ipc_receiver) = ipc::channel().unwrap();
        let (req_sender, req_receiver) = mpsc::sync_channel(1);

        // We could do this much more directly with recv_timeout
        // (https://github.com/rust-lang/rfcs/issues/962).

        // util::thread doesn't give us access to the JoinHandle, which we need for park/unpark,
        // so we use the builder directly.
        let timeout_thread = thread::Builder::new()
            .name(String::from("TimerScheduler"))
            .spawn(move || {
                let mut scheduled_events = BinaryHeap::<ScheduledEvent>::new();
                loop {
                    let now = Instant::now();
                    loop {
                        match scheduled_events.peek() {
                            Some(event) if event.for_time <= now => {
                                let TimerEventRequest(ref sender, source, id, _) = event.request;
                                let _ = sender.send(TimerEvent(source, id));
                            },
                            _ => break,
                        }
                        scheduled_events.pop();
                    }
                    match req_receiver.try_recv() {
                        Ok(req) => {
                            let TimerEventRequest(_, _, _, delay) = req;
                            let schedule = Instant::now() + Duration::from_millis(delay.get());
                            let event = ScheduledEvent { request: req, for_time: schedule };
                            scheduled_events.push(event);
                        },
                        Err(Empty) => match scheduled_events.peek() {
                            None => thread::park(),
                            Some(event) => thread::park_timeout(event.for_time - now),
                        },
                        Err(Disconnected) => break,
                    }
                }
            })
            .unwrap()
            .thread()
            .clone();

        thread::Builder::new()
            .name(String::from("TimerProxy"))
            .spawn(move || {
                while let Ok(req) = req_ipc_receiver.recv() {
                    req_sender.send(req).unwrap();
                    timeout_thread.unpark();
                }
            })
            .unwrap();

        req_ipc_sender
    }
}
