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
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, Thread};
use std::time::Duration;
use util::thread::spawn_named;

pub struct TimerScheduler {
    port: Receiver<TimerEventRequest>,

    scheduled_events: BinaryHeap<ScheduledEvent>,

    /// Channel used to schedule a new timeout
    to_timeout_helper_tx: Sender<NsDuration>,

    /// Channel used to receive a timeout
    from_timeout_helper_rx: Receiver<()>,

    /// Used to wake-up the timeout helper thread
    timeout_helper: Thread,
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

// TODO(emilio): This can be vastly simplified once action is taken in
// https://github.com/rust-lang/rfcs/issues/962.
//
// Another way to do this more cleanly would be returning a
// `TimerSchedulerHandler`, that would contain the thread handle and the sender,
// and make the sender take care of `send()`ing, then waking up the thread.
//
// Unfortunately, this wouldn't be ipc-safe.
//
// Also, this could be a method in `TimerScheduler` if we wouldn't use it while
// holding the topmost event in the heap.
#[allow(unsafe_code)] // due to select!
fn recv_until<T: Send>(port: &Receiver<T>,
                       until: NsDuration,
                       timeout_rx: &Receiver<()>,
                       timeout_tx: &Sender<NsDuration>,
                       timeout_thread: &Thread) -> Option<T> {
    if let Ok(ret) = port.try_recv() {
        return Some(ret);
    }

    timeout_tx.send(until).expect("send to TimeoutHelper failed");
    loop {
        select! {
            msg = port.recv() => {
                if let Ok(ret) = msg {
                    timeout_thread.unpark();
                    let _ = timeout_rx.recv();
                    return Some(ret)
                }
                continue;
            },
            _ = timeout_rx.recv() => {
                return None;
            }
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

        let (to_timeout_helper_tx, to_timeout_helper_rx) = channel::<NsDuration>();
        let (from_timeout_helper_tx, from_timeout_helper_rx) = channel::<()>();

        let helper_thread = thread::spawn(move || {
            loop {
                let until = match to_timeout_helper_rx.recv() {
                    Ok(until) => until,
                    Err(_) => continue,
                };

                let now = precise_time_ns();
                if until < now {
                    from_timeout_helper_tx.send(())
                                          .expect("Failed to send to TimerScheduler");
                    continue;
                }

                let duration = (until - now).get();
                let duration_secs = duration / 1_000_000_000;
                let duration_ns = duration % 1_000_000_000;
                thread::park_timeout(Duration::new(duration_secs, duration_ns as u32));
                let _ = from_timeout_helper_tx.send(());
            }
        }).thread().clone();

        let mut timer_scheduler = TimerScheduler {
            port: ROUTER.route_ipc_receiver_to_new_mpsc_receiver(port),
            scheduled_events: BinaryHeap::new(),
            to_timeout_helper_tx: to_timeout_helper_tx,
            from_timeout_helper_rx: from_timeout_helper_rx,
            timeout_helper: helper_thread,
        };

        spawn_named("TimerScheduler".to_owned(), move || {
            timer_scheduler.run_event_loop();
        });

        chan
    }

    fn get_next_task(&mut self) -> Option<Task> {
        let now = precise_time_ns();
        if let Some(event) = self.scheduled_events.peek() {
            if event.for_time < now {
                return Some(Task::DispatchDueEvents);
            }

            recv_until(&self.port,
                       event.for_time,
                       &self.from_timeout_helper_rx,
                       &self.to_timeout_helper_tx,
                       &self.timeout_helper).map(Task::HandleRequest)
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
