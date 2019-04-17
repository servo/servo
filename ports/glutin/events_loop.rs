/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An event loop implementation that works in headless mode.


use glutin;
use servo::embedder_traits::EventLoopWaker;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
use std::thread;
use std::time;

pub struct EventsLoop(Option<glutin::EventsLoop>);

impl EventsLoop {
    // Ideally, we could use the winit event loop in both modes,
    // but on Linux, the event loop requires a X11 server.
    #[cfg(not(target_os = "linux"))]
    pub fn new(_headless: bool) -> Rc<RefCell<EventsLoop>> {
        Rc::new(RefCell::new(EventsLoop(Some(glutin::EventsLoop::new()))))
    }
    #[cfg(target_os = "linux")]
    pub fn new(headless: bool) -> Rc<RefCell<EventsLoop>> {
        let events_loop = if headless {
            None
        } else {
            Some(glutin::EventsLoop::new())
        };
        Rc::new(RefCell::new(EventsLoop(events_loop)))
    }
}

impl EventsLoop {
    pub fn create_event_loop_waker(&self) -> Box<dyn EventLoopWaker> {
        if let Some(ref events_loop) = self.0 {
            Box::new(HeadedEventLoopWaker::new(&events_loop))
        } else {
            Box::new(HeadlessEventLoopWaker)
        }
    }
    pub fn as_winit(&self) -> &glutin::EventsLoop {
        &self.0.as_ref().expect("Can't access winit event loop while using the fake headless event loop")
    }
    pub fn poll_events<F>(&mut self, callback: F) where F: FnMut(glutin::Event) {
        if let Some(ref mut events_loop) = self.0 {
            events_loop.poll_events(callback);
        } else {
            self.sleep();
        }
    }
    pub fn run_forever<F>(&mut self, mut callback: F) where F: FnMut(glutin::Event) -> glutin::ControlFlow {
        if let Some(ref mut events_loop) = self.0 {
            events_loop.run_forever(callback);
        } else {
            loop {
                self.sleep();
                if callback(glutin::Event::Awakened) == glutin::ControlFlow::Break {
                    break;
                }
            }
        }
    }
    fn sleep(&self) {
        thread::sleep(time::Duration::from_millis(5));
    }
}

struct HeadedEventLoopWaker {
    proxy: Arc<glutin::EventsLoopProxy>,
}
impl HeadedEventLoopWaker {
    fn new(events_loop: &glutin::EventsLoop) -> HeadedEventLoopWaker {
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
    fn clone(&self) -> Box<dyn EventLoopWaker + Send> {
        Box::new(HeadedEventLoopWaker {
            proxy: self.proxy.clone(),
        })
    }
}

struct HeadlessEventLoopWaker;
impl EventLoopWaker for HeadlessEventLoopWaker {
    fn wake(&self) {}
    fn clone(&self) -> Box<dyn EventLoopWaker + Send> { Box::new(HeadlessEventLoopWaker) }
}
