/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script_traits::{TimerEvent, TimerEventRequest};
use util::task::spawn_named;

use num::traits::Saturating;
use std::cell::RefCell;
use std::cmp::{self, Ord};
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::atomic::{self, AtomicBool};
use std::sync::mpsc::{channel, Receiver, Select, Sender};
use std::thread::{self, spawn, Thread};
use time::precise_time_ns;

/// A quick hack to work around the removal of [`std::old_io::timer::Timer`](
/// http://doc.rust-lang.org/1.0.0-beta/std/old_io/timer/struct.Timer.html )
struct CancelableOneshotTimer {
    thread: Thread,
    canceled: Arc<AtomicBool>,
    port: Receiver<()>,
}

impl CancelableOneshotTimer {
    fn new(duration_ms: u32) -> CancelableOneshotTimer {
        let (tx, rx) = channel();
        let canceled = Arc::new(AtomicBool::new(false));
        let canceled_clone = canceled.clone();

        let thread = spawn(move || {
            let duration_ns = (duration_ms as u64) * 1000 * 1000;
            let due_time = precise_time_ns() + duration_ns;

            let mut park_time = duration_ms;

            loop {
                thread::park_timeout_ms(park_time);

                if canceled_clone.load(atomic::Ordering::Relaxed) {
                    return;
                }

                // park_timeout_ms does not guarantee parking for the
                // given amout. We might have woken up early.
                let current_time = precise_time_ns();
                if current_time >= due_time {
                    let _ = tx.send(());
                    return;
                }
                park_time = ((due_time - current_time + 999999) / (1000 * 1000)) as u32;
            }
        }).thread().clone();

        CancelableOneshotTimer {
            thread: thread,
            canceled: canceled,
            port: rx,
        }
    }

    fn port(&self) -> &Receiver<()> {
        &self.port
    }

    fn cancel(&self) {
        self.canceled.store(true, atomic::Ordering::Relaxed);
        self.thread.unpark();
    }
}

pub struct TimerScheduler {
    port: Receiver<TimerEventRequest>,

    scheduled_events: RefCell<BinaryHeap<ScheduledEvent>>,

    timer: RefCell<Option<CancelableOneshotTimer>>,
}

struct ScheduledEvent {
    request: TimerEventRequest,
    for_time: u64,
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

enum Task {
    HandleRequest(TimerEventRequest),
    DispatchDueEvents,
}

impl TimerScheduler {
    pub fn start() -> Sender<TimerEventRequest> {
        let (chan, port) = channel();

        let timer_scheduler = TimerScheduler {
            port: port,

            scheduled_events: RefCell::new(BinaryHeap::new()),

            timer: RefCell::new(None),
        };

        spawn_named("TimerScheduler".to_owned(), move || {
            timer_scheduler.run_event_loop();
        });

        chan
    }

    fn run_event_loop(&self) {
        // FIXME shut down when everybody hung up
        loop {
            match self.receive_next_task() {
                Task::HandleRequest(request) => self.handle_request(request),
                Task::DispatchDueEvents => self.dispatch_due_events(),
            }
        }
    }

    #[allow(unsafe_code)]
    fn receive_next_task(&self) -> Task {
        let port = &self.port;
        let timer = self.timer.borrow();
        let timer_port = timer.as_ref().map(|timer| timer.port());

        if let Some(ref timer_port) = timer_port {
            let sel = Select::new();
            let mut scheduler_handle = sel.handle(port);
            let mut timer_handle = sel.handle(timer_port);

            unsafe {
                scheduler_handle.add();
                timer_handle.add();
            }

            let ret = sel.wait();
            if ret == scheduler_handle.id() {
                Task::HandleRequest(port.recv().unwrap())
            } else if ret == timer_handle.id() {
                Task::DispatchDueEvents
            } else {
                panic!("unexpected select result!")
            }
        } else {
            Task::HandleRequest(port.recv().unwrap())
        }
    }

    fn handle_request(&self, request: TimerEventRequest) {
        let TimerEventRequest(_, _, _, duration) = request;

        let duration = duration as u64;
        let schedule_for = precise_time_ns() + (duration * 1000 * 1000);

        let previously_earliest = self.scheduled_events.borrow().peek()
                .map(|scheduled| scheduled.for_time)
                .unwrap_or(u64::max_value());

        self.scheduled_events.borrow_mut().push(ScheduledEvent {
            request: request,
            for_time: schedule_for,
        });

        if schedule_for < previously_earliest {
            self.start_timer_for_next_event();
        }
    }

    fn dispatch_due_events(&self) {
        let now = precise_time_ns();

        {
            let mut events = self.scheduled_events.borrow_mut();

            while !events.is_empty() && events.peek().as_ref().unwrap().for_time <= now {
                let event = events.pop().unwrap();
                let TimerEventRequest(chan, source, id, _) = event.request;

                let _ = chan.send(TimerEvent(source, id));
            }
        }

        self.start_timer_for_next_event();
    }

    fn start_timer_for_next_event(&self) {
        let events = self.scheduled_events.borrow();
        let next_event = events.peek();

        let mut timer = self.timer.borrow_mut();

        if let Some(ref mut timer) = *timer {
            timer.cancel();
        }

        *timer = next_event.map(|next_event| {
            let delay = next_event.for_time.saturating_sub(precise_time_ns());
            // Round up, we'd rather be late than early…
            let delay = (delay + 999999) / (1000 * 1000);
            let delay = cmp::min(u32::max_value() as u64, delay) as u32;

            CancelableOneshotTimer::new(delay)
        });
    }
}
