/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application entry point, runs the event loop.

use crate::browser::Browser;
use crate::embedder::EmbedderCallbacks;
use crate::window_trait::WindowPortsMethods;
use crate::{headed_window, headless_window};
use servo::compositing::windowing::WindowEvent;
use servo::config::opts::{self, parse_url_or_filename};
use servo::servo_config::pref;
use servo::servo_url::ServoUrl;
use servo::{BrowserId, Servo};
use std::cell::{Cell, RefCell};
use std::env;
use std::mem;
use std::rc::Rc;

pub struct App {
    events_loop: Rc<RefCell<glutin::EventsLoop>>,
    window: Rc<WindowPortsMethods>,
    servo: RefCell<Servo<WindowPortsMethods>>,
    browser: RefCell<Browser<WindowPortsMethods>>,
    event_queue: RefCell<Vec<WindowEvent>>,
    suspended: Cell<bool>,
}

impl App {
    pub fn run() {
        let opts = opts::get();

        let events_loop = glutin::EventsLoop::new();

        let window = if opts.headless {
            headless_window::Window::new(opts.initial_window_size)
        } else {
            headed_window::Window::new(opts.initial_window_size, &events_loop)
        };

        let events_loop = Rc::new(RefCell::new(events_loop));

        let embedder = Box::new(EmbedderCallbacks::new(
            events_loop.clone(),
            window.gl(),
        ));
        let browser = Browser::new(window.clone());

        let mut servo = Servo::new(embedder, window.clone());
        let browser_id = BrowserId::new();
        servo.handle_events(vec![WindowEvent::NewBrowser(get_default_url(), browser_id)]);
        servo.setup_logging();

        let app = App {
            event_queue: RefCell::new(vec![]),
            events_loop,
            window: window,
            browser: RefCell::new(browser),
            servo: RefCell::new(servo),
            suspended: Cell::new(false),
        };

        app.run_loop();
    }

    fn get_events(&self) -> Vec<WindowEvent> {
        mem::replace(&mut *self.event_queue.borrow_mut(), Vec::new())
    }

    fn has_events(&self) -> bool {
        !self.event_queue.borrow().is_empty() || self.window.has_events()
    }

    fn winit_event_to_servo_event(&self, event: glutin::Event) {
        match event {
            // App level events
            glutin::Event::Suspended(suspended) => {
                self.suspended.set(suspended);
                if !suspended {
                    self.event_queue.borrow_mut().push(WindowEvent::Idle);
                }
            },
            glutin::Event::Awakened => {
                self.event_queue.borrow_mut().push(WindowEvent::Idle);
            },
            glutin::Event::DeviceEvent { .. } => {},
            // Window level events
            glutin::Event::WindowEvent {
                window_id, event, ..
            } => {
                if Some(window_id) != self.window.id() {
                    warn!("Got an event from unknown window");
                } else {
                    self.window.winit_event_to_servo_event(event);
                }
            },
        }
    }

    fn run_loop(self) {
        let mut stop = false;
        loop {
            let mut events_loop = self.events_loop.borrow_mut();
            if self.window.is_animating() && !self.suspended.get() {
                // We block on compositing (self.handle_events() ends up calling swap_buffers)
                events_loop.poll_events(|e| {
                    self.winit_event_to_servo_event(e);
                });
                stop = self.handle_events();
            } else {
                // We block on winit's event loop (window events)
                events_loop.run_forever(|e| {
                    self.winit_event_to_servo_event(e);
                    if self.has_events() {
                        if !self.suspended.get() {
                            stop = self.handle_events();
                        }
                    }
                    if stop || self.window.is_animating() && !self.suspended.get() {
                        glutin::ControlFlow::Break
                    } else {
                        glutin::ControlFlow::Continue
                    }
                });
            }
            if stop {
                break;
            }
        }

        self.servo.into_inner().deinit()
    }

    fn handle_events(&self) -> bool {
        let mut browser = self.browser.borrow_mut();
        let mut servo = self.servo.borrow_mut();

        let win_events = self.window.get_events();

        // FIXME: this could be handled by Servo. We don't need
        // a repaint_synchronously function exposed.
        let need_resize = win_events.iter().any(|e| match *e {
            WindowEvent::Resize => true,
            _ => false,
        });

        let mut app_events = self.get_events();
        app_events.extend(win_events);

        browser.handle_window_events(app_events);

        let mut servo_events = servo.get_events();
        loop {
            browser.handle_servo_events(servo_events);
            servo.handle_events(browser.get_events());
            if browser.shutdown_requested() {
                return true;
            }
            servo_events = servo.get_events();
            if servo_events.is_empty() {
                break;
            }
        }

        if need_resize {
            servo.repaint_synchronously();
        }
        false
    }
}

fn get_default_url() -> ServoUrl {
    // If the url is not provided, we fallback to the homepage in prefs,
    // or a blank page in case the homepage is not set either.
    let cwd = env::current_dir().unwrap();
    let cmdline_url = opts::get().url.clone();
    let pref_url = {
        let homepage_url = pref!(shell.homepage);
        parse_url_or_filename(&cwd, &homepage_url).ok()
    };
    let blank_url = ServoUrl::parse("about:blank").ok();

    cmdline_url.or(pref_url).or(blank_url).unwrap()
}

#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
pub fn gl_version() -> glutin::GlRequest {
    glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2))
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub fn gl_version() -> glutin::GlRequest {
    glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0))
}
