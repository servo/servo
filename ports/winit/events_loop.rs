/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An event loop implementation that works in headless mode.


use winit;
use servo::embedder_traits::EventLoopWaker;
use std::sync::{Arc, Condvar, Mutex};
use std::rc::Rc;
use std::cell::RefCell;
use std::time;

#[allow(dead_code)]
enum EventLoop {
    /// A real Winit windowing event loop.
    Winit(Option<winit::EventsLoop>),
    /// A fake event loop which contains a signalling flag used to ensure
    /// that pending events get processed in a timely fashion, and a condition
    /// variable to allow waiting on that flag changing state.
    Headless(Arc<(Mutex<bool>, Condvar)>),
}

pub struct EventsLoop(EventLoop);

impl EventsLoop {
    // Ideally, we could use the winit event loop in both modes,
    // but on Linux, the event loop requires a X11 server.
    #[cfg(not(target_os = "linux"))]
    pub fn new(_headless: bool) -> Rc<RefCell<EventsLoop>> {
        Rc::new(RefCell::new(EventsLoop(EventLoop::Winit(Some(winit::EventsLoop::new())))))
    }
    #[cfg(target_os = "linux")]
    pub fn new(headless: bool) -> Rc<RefCell<EventsLoop>> {
        let events_loop = if headless {
            EventLoop::Headless(Arc::new((Mutex::new(false), Condvar::new())))
        } else {
            EventLoop::Winit(Some(winit::EventsLoop::new()))
        };
        Rc::new(RefCell::new(EventsLoop(events_loop)))
    }
}

impl EventsLoop {
    pub fn create_event_loop_waker(&self) -> Box<dyn EventLoopWaker> {
        match self.0 {
            EventLoop::Winit(ref events_loop) => {
                let events_loop = events_loop
                    .as_ref()
                    .expect("Can't create waker for unavailable event loop.");
                Box::new(HeadedEventLoopWaker::new(&events_loop))
            },
            EventLoop::Headless(ref data) =>
                Box::new(HeadlessEventLoopWaker(data.clone())),
        }
    }
    pub fn as_winit(&self) -> &winit::EventsLoop {
        match self.0 {
            EventLoop::Winit(Some(ref event_loop)) => event_loop,
            EventLoop::Winit(None) | EventLoop::Headless(..) =>
                panic!("Can't access winit event loop while using the fake headless event loop"),
        }
    }

    pub fn poll_events<F>(&mut self, callback: F) where F: FnMut(winit::Event) {
        match self.0 {
            EventLoop::Winit(Some(ref mut events_loop)) => events_loop.poll_events(callback),
            EventLoop::Winit(None) => (),
            EventLoop::Headless(ref data) => {
                // This is subtle - the use of the event loop in App::run_loop
                // optionally calls run_forever, then always calls poll_events.
                // If our signalling flag is true before we call run_forever,
                // we don't want to reset it before poll_events is called or
                // we'll end up sleeping even though there are events waiting
                // to be handled. We compromise by only resetting the flag here
                // in poll_events, so that both poll_events and run_forever can
                // check it first and avoid sleeping unnecessarily.
                self.sleep(&data.0, &data.1);
                *data.0.lock().unwrap() = false;
            }
        }
    }
    pub fn run_forever<F>(&mut self, mut callback: F) where F: FnMut(winit::Event) -> winit::ControlFlow {
        match self.0 {
            EventLoop::Winit(ref mut events_loop) => {
                let events_loop = events_loop
                    .as_mut()
                    .expect("Can't run an unavailable event loop.");
                events_loop.run_forever(callback);
            }
            EventLoop::Headless(ref data) => {
                let &(ref flag, ref condvar) = &**data;
                while !*flag.lock().unwrap() {
                    self.sleep(flag, condvar);
                    if callback(winit::Event::Awakened) == winit::ControlFlow::Break {
                        break;
                    }
                }
            }
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
        let _ = condvar.wait_timeout(
            guard, time::Duration::from_millis(5)
        ).unwrap();
    }
}

struct HeadedEventLoopWaker {
    proxy: Arc<winit::EventsLoopProxy>,
}
impl HeadedEventLoopWaker {
    fn new(events_loop: &winit::EventsLoop) -> HeadedEventLoopWaker {
        let proxy = Arc::new(events_loop.create_proxy());
        HeadedEventLoopWaker { proxy }
    }
}
impl EventLoopWaker for HeadedEventLoopWaker {
    fn wake(&self) {
        // kick the OS event loop awake.
        if let Err(err) = self.proxy.wakeup() {
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
