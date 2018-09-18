/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use winit;
use glutin_app::window::Window;
use servo::embedder_traits::EventLoopWaker;
use servo::servo_config::opts;

pub struct App {
    events_loop: winit::EventsLoop,
    windows: Option<Rc<Window>>,
}

impl App {
    pub fn new() -> App {
        App {
            events_loop: winit::EventsLoop::new(),
            windows: None,
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
        Box::new(X(self.events_loop.create_proxy()))
    }

    pub fn new_window(&self) -> Rc<Window> {

        // FIXME: is that supposed to be here?
        let opts = opts::get();
        let foreground = opts.output_file.is_none() && !opts.headless;

        let window = Window::new(&self.events_loop, foreground, opts.initial_window_size);
        // self.windows = Some(window.clone());
        window
    }

    pub fn run<T>(&mut self, mut servo_callback: T) where T: FnMut() -> bool {
        let mut stop = false;
        loop {
            match self.windows {
                Some(ref window) if window.is_animating() => {
                    // We block on compositing (servo_callback ends up calling swap_buffers)
                    self.events_loop.poll_events(|e| {
                        window.winit_event_to_servo_event(e);
                    });
                    stop = servo_callback();
                },
                Some(ref window) => {
                    // We block on winit's event loop (window events)
                    self.events_loop.run_forever(|e| {
                        window.winit_event_to_servo_event(e);
                        if !window.event_queue.borrow().is_empty() {
                            if !window.suspended.get() {
                                stop = servo_callback();
                            }
                        }
                        if stop || window.is_animating() {
                            winit::ControlFlow::Break
                        } else {
                            winit::ControlFlow::Continue
                        }
                    });
                },
                None => {
                    self.events_loop.run_forever(|e| {
                        stop = servo_callback();
                        println!("headless event: {:?}", e);
                        winit::ControlFlow::Continue
                    });
                }
            };
            if stop {
                break;
            }
        }
        // FIXME: missing headless windows
    }

}
