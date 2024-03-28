/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An event loop implementation that works in headless mode.

use std::sync::{Arc, Condvar, Mutex};
use std::time;

use log::warn;
use servo::embedder_traits::EventLoopWaker;
#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

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
    pub fn new(
        _headless: bool,
        _has_output_file: bool,
    ) -> Result<EventsLoop, winit::error::EventLoopError> {
        Ok(EventsLoop(EventLoop::Winit(Some(
            winit::event_loop::EventLoopBuilder::with_user_event().build()?,
        ))))
    }
    #[cfg(target_os = "linux")]
    pub fn new(headless: bool, _has_output_file: bool) -> EventsLoop {
        EventsLoop(if headless {
            EventLoop::Headless(Arc::new((Mutex::new(false), Condvar::new())))
        } else {
            EventLoop::Winit(Some(
                winit::event_loop::EventLoopBuilder::with_user_event().build(),
            ))
        })
    }
    #[cfg(target_os = "macos")]
    pub fn new(headless: bool, _has_output_file: bool) -> EventsLoop {
        EventsLoop(if headless {
            EventLoop::Headless(Arc::new((Mutex::new(false), Condvar::new())))
        } else {
            let mut event_loop_builder = winit::event_loop::EventLoopBuilder::with_user_event();
            if _has_output_file {
                // Prevent the window from showing in Dock.app, stealing focus,
                // when generating an output file.
                event_loop_builder.with_activation_policy(ActivationPolicy::Prohibited);
            }
            EventLoop::Winit(Some(event_loop_builder.build()))
        })
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
    pub fn as_winit(&self) -> &winit::event_loop::EventLoop<WakerEvent> {
        match self.0 {
            EventLoop::Winit(Some(ref event_loop)) => event_loop,
            EventLoop::Winit(None) | EventLoop::Headless(..) => {
                panic!("Can't access winit event loop while using the fake headless event loop")
            },
        }
    }

    pub fn run_forever<F: 'static>(self, mut callback: F)
    where
        F: FnMut(
            winit::event::Event<WakerEvent>,
            Option<&winit::event_loop::EventLoopWindowTarget<WakerEvent>>,
            &mut ControlFlow,
        ),
    {
        match self.0 {
            EventLoop::Winit(events_loop) => {
                let events_loop = events_loop.expect("Can't run an unavailable event loop.");
                events_loop
                    .run(move |e, window_target| {
                        let mut control_flow = ControlFlow::default();
                        callback(e, Some(window_target), &mut control_flow);
                        control_flow.apply_to(window_target);
                    })
                    .expect("Failed while running events loop");
            },
            EventLoop::Headless(ref data) => {
                let (flag, condvar) = &**data;
                let mut event = winit::event::Event::NewEvents(winit::event::StartCause::Init);
                loop {
                    self.sleep(flag, condvar);
                    let mut control_flow = ControlFlow::Poll;
                    callback(event, None, &mut control_flow);
                    event = winit::event::Event::<WakerEvent>::UserEvent(WakerEvent);

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
    fn apply_to(self, window_target: &winit::event_loop::EventLoopWindowTarget<WakerEvent>) {
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
