/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::length::Length;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use num::traits::Saturating;
use script_traits::{MsDuration, NsDuration, precise_time_ms, precise_time_ns};
use script_traits::{TimerEvent, TimerEventRequest};
use std::cell::RefCell;
use std::cmp::{self, Ord};
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::sync::atomic::{self, AtomicBool};
use std::sync::mpsc::{channel, Receiver, Select};
use std::thread::{self, spawn, Thread};
use std::time::Duration;
use util::task::spawn_named;

/// A quick hack to work around the removal of [`std::old_io::timer::Timer`](
/// http://doc.rust-lang.org/1.0.0-beta/std/old_io/timer/struct.Timer.html )
struct CancelableOneshotTimer {
    thread: Thread,
    canceled: Arc<AtomicBool>,
    port: Receiver<()>,
}

impl CancelableOneshotTimer {
    fn new(duration: MsDuration) -> CancelableOneshotTimer {
        let (tx, rx) = channel();
        let canceled = Arc::new(AtomicBool::new(false));
        let canceled_clone = canceled.clone();

        let thread = spawn(move || {
            let due_time = precise_time_ms() + duration;

            let mut park_time = duration;

            loop {
                thread::park_timeout(Duration::from_millis(park_time.get()));

                if canceled_clone.load(atomic::Ordering::Relaxed) {
                    return;
                }

                // park_timeout_ms does not guarantee parking for the
                // given amout. We might have woken up early.
                let current_time = precise_time_ms();
                if current_time >= due_time {
                    let _ = tx.send(());
                    return;
                }
                park_time = due_time - current_time;
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

enum Task {
    HandleRequest(TimerEventRequest),
    DispatchDueEvents,
}

impl TimerScheduler {
    pub fn start() -> IpcSender<TimerEventRequest> {
        let (chan, port) = ipc::channel().unwrap();

        let timer_scheduler = TimerScheduler {
            port: ROUTER.route_ipc_receiver_to_new_mpsc_receiver(port),

            scheduled_events: RefCell::new(BinaryHeap::new()),

            timer: RefCell::new(None),
        };

        spawn_named("TimerScheduler".to_owned(), move || {
            timer_scheduler.run_event_loop();
        });

        chan
    }

    fn run_event_loop(&self) {
        while let Some(task) = self.receive_next_task() {
            match task {
                Task::HandleRequest(request) => self.handle_request(request),
                Task::DispatchDueEvents => self.dispatch_due_events(),
            }
        }
    }

    #[allow(unsafe_code)]
    fn receive_next_task(&self) -> Option<Task> {
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
                port.recv().ok().map(|req| Task::HandleRequest(req))
            } else if ret == timer_handle.id() {
                timer_port.recv().ok().map(|_| Task::DispatchDueEvents)
            } else {
                panic!("unexpected select result!")
            }
        } else {
            port.recv().ok().map(|req| Task::HandleRequest(req))
        }
    }

    fn handle_request(&self, request: TimerEventRequest) {
        let TimerEventRequest(_, _, _, duration_ms) = request;
        let duration_ns = Length::new(duration_ms.get() * 1000 * 1000);
        let schedule_for = precise_time_ns() + duration_ns;

        let previously_earliest = self.scheduled_events.borrow().peek()
                .map(|scheduled| scheduled.for_time)
                .unwrap_or(Length::new(u64::max_value()));

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
            let delay_ns = next_event.for_time.get().saturating_sub(precise_time_ns().get());
            // Round up, we'd rather be late than early…
            let delay_ms = Length::new(delay_ns.saturating_add(999999) / (1000 * 1000));

            CancelableOneshotTimer::new(delay_ms)
        });
    }
}
