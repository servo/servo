/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An event loop implementation that works in headless mode.

use std::sync::{Arc, Condvar, Mutex};
use std::time;

use log::warn;
use servo::EventLoopWaker;
use winit::event_loop::{EventLoop, EventLoop as WinitEventLoop, EventLoopProxy};

use super::app::App;

#[derive(Debug)]
pub enum AppEvent {
    /// Another process or thread has kicked the OS event loop with EventLoopWaker.
    Waker,
    Accessibility(egui_winit::accesskit_winit::Event),
}

impl From<egui_winit::accesskit_winit::Event> for AppEvent {
    fn from(event: egui_winit::accesskit_winit::Event) -> AppEvent {
        AppEvent::Accessibility(event)
    }
}

/// A headed or headless event loop. Headless event loops are necessary for environments without a
/// display server. Ideally, we could use the headed winit event loop in both modes, but on Linux,
/// the event loop requires a display server, which prevents running servoshell in a console.
#[allow(clippy::large_enum_variant)]
pub(crate) enum ServoShellEventLoop {
    /// A real Winit windowing event loop.
    Winit(EventLoop<AppEvent>),
    /// A fake event loop which contains a signalling flag used to ensure
    /// that pending events get processed in a timely fashion, and a condition
    /// variable to allow waiting on that flag changing state.
    Headless(Arc<(Mutex<bool>, Condvar)>),
}

impl ServoShellEventLoop {
    pub(crate) fn headless() -> ServoShellEventLoop {
        ServoShellEventLoop::Headless(Arc::new((Mutex::new(false), Condvar::new())))
    }

    pub(crate) fn headed() -> ServoShellEventLoop {
        ServoShellEventLoop::Winit(
            WinitEventLoop::with_user_event()
                .build()
                .expect("Could not start winit event loop"),
        )
    }
}

impl ServoShellEventLoop {
    pub(crate) fn event_loop_proxy(&self) -> Option<EventLoopProxy<AppEvent>> {
        match self {
            ServoShellEventLoop::Winit(event_loop) => Some(event_loop.create_proxy()),
            ServoShellEventLoop::Headless(..) => None,
        }
    }

    pub fn create_event_loop_waker(&self) -> Box<dyn EventLoopWaker> {
        match self {
            ServoShellEventLoop::Winit(event_loop) => {
                Box::new(HeadedEventLoopWaker::new(event_loop))
            },
            ServoShellEventLoop::Headless(data) => Box::new(HeadlessEventLoopWaker(data.clone())),
        }
    }

    pub fn run_app(self, app: &mut App) {
        match self {
            ServoShellEventLoop::Winit(event_loop) => {
                event_loop
                    .run_app(app)
                    .expect("Failed while running events loop");
            },
            ServoShellEventLoop::Headless(ref data) => {
                let (flag, condvar) = &**data;

                app.init(None);
                loop {
                    self.sleep(flag, condvar);
                    app.handle_webdriver_messages();
                    if !app.handle_events_with_headless() {
                        break;
                    }
                    *flag.lock().unwrap() = false;
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

struct HeadedEventLoopWaker {
    proxy: Arc<Mutex<EventLoopProxy<AppEvent>>>,
}
impl HeadedEventLoopWaker {
    fn new(event_loop: &EventLoop<AppEvent>) -> HeadedEventLoopWaker {
        let proxy = Arc::new(Mutex::new(event_loop.create_proxy()));
        HeadedEventLoopWaker { proxy }
    }
}
impl EventLoopWaker for HeadedEventLoopWaker {
    fn wake(&self) {
        // Kick the OS event loop awake.
        if let Err(err) = self.proxy.lock().unwrap().send_event(AppEvent::Waker) {
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
