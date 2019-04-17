/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{browser, headed_window, headless_window};
use servo::{Servo, BrowserId};
use servo::config::opts::{self, parse_url_or_filename};
use servo::compositing::windowing::WindowEvent;
use servo::servo_config::pref;
use servo::servo_url::ServoUrl;
use std::env;
use crate::window_trait::ServoWindow;
use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::sync::Arc;
use servo::embedder_traits::EventLoopWaker;
use std::mem;

pub struct App {
    events_loop: Rc<RefCell<glutin::EventsLoop>>,
    window: Rc<ServoWindow>,
    servo: RefCell<Servo<ServoWindow>>,
    browser: RefCell<browser::Browser<ServoWindow>>,
    event_queue: RefCell<Vec<WindowEvent>>,
    suspended: Cell<bool>,
}

impl App {
    pub fn run() {
        let opts = opts::get();

        let events_loop = Rc::new(RefCell::new(glutin::EventsLoop::new()));

        let waker = create_event_loop_waker(&events_loop.borrow());

        let window = if opts.headless {
            headless_window::Window::new(opts.initial_window_size, waker)
        } else {
            headed_window::Window::new(opts.initial_window_size, waker, events_loop.clone())
        };

        let browser = browser::Browser::new(window.clone());

        let mut servo = Servo::new(window.clone());
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
            glutin::Event::WindowEvent { window_id, event, .. } => {
                if Some(window_id) != self.window.id() {
                    warn!("FIXME");
                } else {
                    self.window.winit_event_to_servo_event(event);
                }
            },
            glutin::Event::Suspended(suspended) => {
                self.suspended.set(suspended);
                if !suspended {
                    self.event_queue.borrow_mut().push(WindowEvent::Idle);
                }
            },
            glutin::Event::Awakened => {
                self.event_queue.borrow_mut().push(WindowEvent::Idle);
            },
            glutin::Event::DeviceEvent { .. } => {
            }
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

fn create_event_loop_waker(events_loop: &glutin::EventsLoop) -> Box<dyn EventLoopWaker> {
    struct GlutinEventLoopWaker {
        proxy: Arc<glutin::EventsLoopProxy>,
    }
    impl GlutinEventLoopWaker {
        fn new(events_loop: &glutin::EventsLoop) -> GlutinEventLoopWaker {
            let proxy = Arc::new(events_loop.create_proxy());
            GlutinEventLoopWaker { proxy }
        }
    }
    impl EventLoopWaker for GlutinEventLoopWaker {
        fn wake(&self) {
            // kick the OS event loop awake.
            if let Err(err) = self.proxy.wakeup() {
                warn!("Failed to wake up event loop ({}).", err);
            }
        }
        fn clone(&self) -> Box<dyn EventLoopWaker + Send> {
            Box::new(GlutinEventLoopWaker {
                proxy: self.proxy.clone(),
            })
        }
    }

    Box::new(GlutinEventLoopWaker::new(&events_loop))
}
