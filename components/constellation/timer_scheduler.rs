/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::length::Length;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use script_traits::{NsDuration, precise_time_ns};
use script_traits::{TimerEvent, TimerEventRequest};
use std::cmp::{self, Ord};
use std::collections::BinaryHeap;
use std::sync::mpsc::Receiver;
use util::thread::spawn_named;

pub struct TimerScheduler {
    port: Receiver<TimerEventRequest>,

    scheduled_events: BinaryHeap<ScheduledEvent>,
}

struct ScheduledEvent {
    request: TimerEventRequest,
    for_time: NsDuration,
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

fn recv_with_timeout<T: Send>(port: &Receiver<T>, from: NsDuration, timeout: NsDuration) -> Option<T> {
    loop {
        if let Ok(ret) = port.try_recv() {
            return Some(ret);
        }
        let now = precise_time_ns();
        if now - from >= timeout {
            return None;
        }
    }
}

enum Task {
    HandleRequest(TimerEventRequest),
    DispatchDueEvents,
}

impl TimerScheduler {
    pub fn start() -> IpcSender<TimerEventRequest> {
        let (chan, port) = ipc::channel().unwrap();

        let mut timer_scheduler = TimerScheduler {
            port: ROUTER.route_ipc_receiver_to_new_mpsc_receiver(port),
            scheduled_events: BinaryHeap::new(),
        };

        spawn_named("TimerScheduler".to_owned(), move || {
            timer_scheduler.run_event_loop();
        });

        chan
    }

    fn get_next_task(&mut self) -> Option<Task> {
        if let Some(event) = self.scheduled_events.peek() {
            let now = precise_time_ns();
            if event.for_time < now {
                return Some(Task::DispatchDueEvents);
            }

            let timeout = event.for_time - now;
            recv_with_timeout(&self.port, now, timeout).map(Task::HandleRequest)
        } else {
            self.port.recv().ok().map(Task::HandleRequest)
        }
    }

    fn run_event_loop(&mut self) {
        loop {
            let task = self.get_next_task();
            if let Some(task) = task {
                match task {
                    Task::DispatchDueEvents => self.dispatch_due_events(),
                    Task::HandleRequest(req) => self.add_request(req),
                }
            }
        }
    }

    fn add_request(&mut self, request: TimerEventRequest) {
        let TimerEventRequest(_, _, _, duration_ms) = request;
        let duration_ns = Length::new(duration_ms.get() * 1000 * 1000);
        let schedule_for = precise_time_ns() + duration_ns;

        self.scheduled_events.push(ScheduledEvent {
            request: request,
            for_time: schedule_for,
        });
    }

    fn dispatch_due_events(&mut self) {
        let now = precise_time_ns();

        let mut events = &mut self.scheduled_events;

        {
            while !events.is_empty() && events.peek().as_ref().unwrap().for_time <= now {
                let event = events.pop().unwrap();
                let TimerEventRequest(chan, source, id, _) = event.request;

                let _ = chan.send(TimerEvent(source, id));
            }
        }
    }
}
