/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use winit;
use glutin_app::window::{Window, WindowId};
use servo::embedder_traits::EventLoopWaker;
use servo::servo_config::opts;
use servo::compositing::windowing::WindowEvent;
use std::cell::RefCell;

pub struct App {
    events_loop: RefCell<winit::EventsLoop>,
    // windows: RefCell<Vec<WindowId>>,
}

impl App {
    pub fn new() -> App {
        App {
            events_loop: RefCell::new(winit::EventsLoop::new()),
            // windows: RefCell::new(Vec::new()),
        }
    }

    pub fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        struct X(winit::EventsLoopProxy);
        impl EventLoopWaker for X {
            fn wake(&self) {
                self.0.wakeup();
            }
            // Maybe use clone derive
            fn clone(&self) -> Box<EventLoopWaker + Send> {
                Box::new(X(self.0.clone()))
            }
        }
        Box::new(X(self.events_loop.borrow().create_proxy()))
    }

    pub fn create_window(&mut self) -> Window {

        // FIXME: is that supposed to be here?
        let opts = opts::get();
        let foreground = opts.output_file.is_none() && !opts.headless;

        let window = Window::new(&self.events_loop.borrow(), foreground, opts.initial_window_size);
        // self.windows.borrow_mut().push(window.id());
        window
    }

    pub fn run<T>(&self, mut servo_callback: T) where T: FnMut(&Vec<WindowEvent>) -> bool {
        let mut stop = false;
        let mut events: Vec<WindowEvent> = Vec::new();
        loop {
            self.events_loop.borrow_mut().run_forever(|e| {
                stop = servo_callback(&mut events);
                events.clear();
                println!("headless event: {:?}", e);
                winit::ControlFlow::Continue
            });
            // match window {
            //     Some(ref window) if window.is_animating() => {
            //         // We block on compositing (servo_callback ends up calling swap_buffers)
            //         self.events_loop.borrow_mut().poll_events(|e| {
            //             window.winit_event_to_servo_event(&mut events, e);
            //         });
            //         stop = servo_callback(&mut events);
            //         // FIXME: how is it different from mem::replace?
            //         events.clear();
            //     },
            //     Some(ref window) => {
            //         // We block on winit's event loop (window events)
            //         self.events_loop.borrow_mut().run_forever(|e| {
            //             window.winit_event_to_servo_event(&mut events, e);
            //             if !events.is_empty() {
            //                 if !window.suspended.get() {
            //                     stop = servo_callback(&mut events);
            //                     events.clear();
            //                 }
            //             }
            //             if stop || window.is_animating() {
            //                 winit::ControlFlow::Break
            //             } else {
            //                 winit::ControlFlow::Continue
            //             }
            //         });
            //     },
            //     None => {
            //         self.events_loop.borrow_mut().run_forever(|e| {
            //             stop = servo_callback(&mut events);
            //             events.clear();
            //             println!("headless event: {:?}", e);
            //             winit::ControlFlow::Continue
            //         });
            //     }
            // };
            if stop {
                break;
            }
        }
        // FIXME: missing headless windows
    }

}
