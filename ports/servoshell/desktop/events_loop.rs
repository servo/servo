/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An event loop implementation that works in headless mode.

use std::cell::Cell;
use std::sync::{Arc, Condvar, Mutex};
use std::time;

use log::warn;
use servo::embedder_traits::EventLoopWaker;
use winit::error::EventLoopError;
use winit::event::{Event, StartCause};
use winit::event_loop::{ActiveEventLoop, EventLoop as WinitEventLoop};
#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

use super::app::App;

/// Another process or thread has kicked the OS event loop with EventLoopWaker.
#[derive(Debug)]
pub struct WakerEvent;

/// The real or fake OS event loop.
#[allow(dead_code)]
enum EventLoop {
    /// A real Winit windowing event loop.
    Winit(Option<winit::event_loop::EventLoop<WakerEvent>>),
    /// A fake event loop which contains a signalling flag used to ensure
    /// that pending events get processed in a timely fashion, and a condition
    /// variable to allow waiting on that flag changing state.
    Headless(Arc<(Mutex<bool>, Condvar)>),
}

pub struct EventsLoop(EventLoop);

impl EventsLoop {
    // Ideally, we could use the winit event loop in both modes,
    // but on Linux, the event loop requires a X11 server.
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub fn new(_headless: bool, _has_output_file: bool) -> Result<EventsLoop, EventLoopError> {
        Ok(EventsLoop(EventLoop::Winit(Some(
            WinitEventLoop::with_user_event().build()?,
        ))))
    }
    #[cfg(target_os = "linux")]
    pub fn new(headless: bool, _has_output_file: bool) -> Result<EventsLoop, EventLoopError> {
        Ok(EventsLoop(if headless {
            EventLoop::Headless(Arc::new((Mutex::new(false), Condvar::new())))
        } else {
            EventLoop::Winit(Some(WinitEventLoop::with_user_event().build()?))
        }))
    }
    #[cfg(target_os = "macos")]
    pub fn new(headless: bool, _has_output_file: bool) -> Result<EventsLoop, EventLoopError> {
        Ok(EventsLoop(if headless {
            EventLoop::Headless(Arc::new((Mutex::new(false), Condvar::new())))
        } else {
            let mut event_loop_builder = WinitEventLoop::with_user_event();
            if _has_output_file {
                // Prevent the window from showing in Dock.app, stealing focus,
                // when generating an output file.
                event_loop_builder.with_activation_policy(ActivationPolicy::Prohibited);
            }
            EventLoop::Winit(Some(event_loop_builder.build()?))
        }))
    }
}

impl EventsLoop {
    pub fn create_event_loop_waker(&self) -> Box<dyn EventLoopWaker> {
        match self.0 {
            EventLoop::Winit(ref events_loop) => {
                let events_loop = events_loop
                    .as_ref()
                    .expect("Can't create waker for unavailable event loop.");
                Box::new(HeadedEventLoopWaker::new(events_loop))
            },
            EventLoop::Headless(ref data) => Box::new(HeadlessEventLoopWaker(data.clone())),
        }
    }
    pub fn as_winit(&self) -> &WinitEventLoop<WakerEvent> {
        match self.0 {
            EventLoop::Winit(Some(ref event_loop)) => event_loop,
            EventLoop::Winit(None) | EventLoop::Headless(..) => {
                panic!("Can't access winit event loop while using the fake headless event loop")
            },
        }
    }
    pub fn run_app(self, app: &mut App) {
        match self.0 {
            EventLoop::Winit(events_loop) => {
                let events_loop = events_loop.expect("Can't run an unavailable event loop.");
                events_loop
                    .run_app(app)
                    .expect("Failed while running events loop");
            },
            EventLoop::Headless(_) => todo!(),
        }
    }

    pub fn run_forever<F>(self, mut callback: F)
    where
        F: 'static + FnMut(Event<WakerEvent>, &mut ControlFlow),
    {
        match self.0 {
            EventLoop::Winit(events_loop) => {
                let events_loop = events_loop.expect("Can't run an unavailable event loop.");
                #[allow(deprecated)]
                events_loop
                    .run(move |e, window_target| {
                        let mut control_flow = ControlFlow::default();
                        let _guard = EventLoopGuard::new(window_target);
                        callback(e, &mut control_flow);
                        control_flow.apply_to(window_target);
                    })
                    .expect("Failed while running events loop");
            },
            EventLoop::Headless(ref data) => {
                let (flag, condvar) = &**data;
                let mut event = Event::NewEvents(StartCause::Init);
                loop {
                    self.sleep(flag, condvar);
                    let mut control_flow = ControlFlow::Poll;
                    callback(event, &mut control_flow);
                    event = Event::<WakerEvent>::UserEvent(WakerEvent);

                    if control_flow != ControlFlow::Poll {
                        *flag.lock().unwrap() = false;
                    }

                    if control_flow == ControlFlow::Exit {
                        break;
                    }
                }
            },
        }
    }

    fn sleep(&self, lock: &Mutex<bool>, condvar: &Condvar) {
        // To avoid sleeping when we should be processing events, do two things:
        // * before sleeping, check whether our signalling flag has been set
        // * wait on a condition variable with a maximum timeout, to allow
        //   being woken up by any signals that occur while sleeping.
        let guard = lock.lock().unwrap();
        if *guard {
            return;
        }
        let _ = condvar
            .wait_timeout(guard, time::Duration::from_millis(5))
            .unwrap();
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlFlow {
    Poll,

    #[default]
    Wait,

    WaitUntil(std::time::Instant),

    /// winit removed their ControlFlow::Exit variant in 0.29.2
    Exit,
}

impl ControlFlow {
    fn apply_to(self, window_target: &ActiveEventLoop) {
        match self {
            ControlFlow::Poll => {
                window_target.set_control_flow(winit::event_loop::ControlFlow::Poll)
            },
            ControlFlow::Wait => {
                window_target.set_control_flow(winit::event_loop::ControlFlow::Wait)
            },
            ControlFlow::WaitUntil(instant) => {
                window_target.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(instant))
            },
            ControlFlow::Exit => window_target.exit(),
        }
    }

    pub fn set_poll(&mut self) {
        *self = ControlFlow::Poll;
    }

    pub fn set_wait(&mut self) {
        *self = ControlFlow::Wait;
    }

    #[allow(unused)]
    pub fn set_wait_until(&mut self, instant: std::time::Instant) {
        *self = ControlFlow::WaitUntil(instant);
    }

    pub fn set_exit(&mut self) {
        *self = ControlFlow::Exit;
    }
}

impl From<winit::event_loop::ControlFlow> for ControlFlow {
    fn from(cf: winit::event_loop::ControlFlow) -> Self {
        match cf {
            winit::event_loop::ControlFlow::Poll => Self::Poll,
            winit::event_loop::ControlFlow::Wait => Self::Wait,
            winit::event_loop::ControlFlow::WaitUntil(instant) => Self::WaitUntil(instant),
        }
    }
}

struct HeadedEventLoopWaker {
    proxy: Arc<Mutex<winit::event_loop::EventLoopProxy<WakerEvent>>>,
}
impl HeadedEventLoopWaker {
    fn new(events_loop: &winit::event_loop::EventLoop<WakerEvent>) -> HeadedEventLoopWaker {
        let proxy = Arc::new(Mutex::new(events_loop.create_proxy()));
        HeadedEventLoopWaker { proxy }
    }
}
impl EventLoopWaker for HeadedEventLoopWaker {
    fn wake(&self) {
        // Kick the OS event loop awake.
        if let Err(err) = self.proxy.lock().unwrap().send_event(WakerEvent) {
            warn!("Failed to wake up event loop ({}).", err);
        }
    }
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(HeadedEventLoopWaker {
            proxy: self.proxy.clone(),
        })
    }
}

struct HeadlessEventLoopWaker(Arc<(Mutex<bool>, Condvar)>);
impl EventLoopWaker for HeadlessEventLoopWaker {
    fn wake(&self) {
        // Set the signalling flag and notify the condition variable.
        // This ensures that any sleep operation is interrupted,
        // and any non-sleeping operation will have a change to check
        // the flag before going to sleep.
        let (ref flag, ref condvar) = *self.0;
        let mut flag = flag.lock().unwrap();
        *flag = true;
        condvar.notify_all();
    }
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(HeadlessEventLoopWaker(self.0.clone()))
    }
}

thread_local! {
    static CURRENT_EVENT_LOOP: Cell<Option<*const ActiveEventLoop>> = const { Cell::new(None) };
}

struct EventLoopGuard;

impl EventLoopGuard {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        CURRENT_EVENT_LOOP.with(|cell| {
            assert!(
                cell.get().is_none(),
                "Attempted to set a new event loop while one is already set"
            );
            cell.set(Some(event_loop as *const ActiveEventLoop));
        });
        Self
    }
}

impl Drop for EventLoopGuard {
    fn drop(&mut self) {
        CURRENT_EVENT_LOOP.with(|cell| cell.set(None));
    }
}

// Helper function to safely use the current event loop
#[allow(unsafe_code)]
pub fn with_current_event_loop<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&ActiveEventLoop) -> R,
{
    CURRENT_EVENT_LOOP.with(|cell| {
        cell.get().map(|ptr| {
            // SAFETY:
            // 1. The pointer is guaranteed to be valid when it's Some, as the EventLoopGuard that created it
            //    lives at least as long as the reference, and clears it when it's dropped. Only run_forever creates
            //    a new EventLoopGuard, and does not leak it.
            // 2. Since the pointer was created from a borrow which lives at least as long as this pointer there are
            //    no mutable references to the ActiveEventLoop.
            let event_loop = unsafe { &*ptr };
            f(event_loop)
        })
    })
}
