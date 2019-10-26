/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application entry point, runs the event loop.

use crate::browser::Browser;
use crate::embedder::EmbedderCallbacks;
use crate::events_loop::EventsLoop;
use crate::window_trait::WindowPortsMethods;
use crate::{headed_window, headless_window};
use glutin::WindowId;
use servo::compositing::windowing::WindowEvent;
use servo::config::opts::{self, parse_url_or_filename};
use servo::servo_config::pref;
use servo::servo_url::ServoUrl;
use servo::{BrowserId, Servo};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::mem;
use std::rc::Rc;

thread_local! {
    pub static WINDOWS: RefCell<HashMap<WindowId, Rc<dyn WindowPortsMethods>>> = RefCell::new(HashMap::new());
}

pub struct App {
    events_loop: Rc<RefCell<EventsLoop>>,
    servo: RefCell<Servo<dyn WindowPortsMethods>>,
    browser: RefCell<Browser<dyn WindowPortsMethods>>,
    event_queue: RefCell<Vec<WindowEvent>>,
    suspended: Cell<bool>,
}

impl App {
    pub fn run(
        angle: bool,
        enable_vsync: bool,
        use_msaa: bool,
        no_native_titlebar: bool,
        device_pixels_per_px: Option<f32>,
    ) {
        let events_loop = EventsLoop::new(opts::get().headless);

        // Implements window methods, used by compositor.
        let window = if opts::get().headless {
            headless_window::Window::new(opts::get().initial_window_size, device_pixels_per_px)
        } else {
            Rc::new(headed_window::Window::new(
                opts::get().initial_window_size,
                None,
                events_loop.clone(),
                angle,
                enable_vsync,
                use_msaa,
                no_native_titlebar,
                device_pixels_per_px,
            ))
        };

        // Implements embedder methods, used by libservo and constellation.
        let embedder = Box::new(EmbedderCallbacks::new(
            window.clone(),
            events_loop.clone(),
            window.gl(),
            angle,
        ));

        // Handle browser state.
        let browser = Browser::new(window.clone());

        let mut servo = Servo::new(embedder, window.clone(), device_pixels_per_px);
        let browser_id = BrowserId::new();
        servo.handle_events(vec![WindowEvent::NewBrowser(get_default_url(), browser_id)]);
        servo.setup_logging();

        register_window(window);

        let app = App {
            event_queue: RefCell::new(vec![]),
            events_loop,
            browser: RefCell::new(browser),
            servo: RefCell::new(servo),
            suspended: Cell::new(false),
        };

        app.run_loop();
    }

    fn get_events(&self) -> Vec<WindowEvent> {
        mem::replace(&mut *self.event_queue.borrow_mut(), Vec::new())
    }

    // This function decides whether the event should be handled during `run_forever`.
    fn winit_event_to_servo_event(&self, event: glutin::Event) -> glutin::ControlFlow {
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
                return WINDOWS.with(|windows| {
                    match windows.borrow().get(&window_id) {
                        None => {
                            warn!("Got an event from unknown window");
                            glutin::ControlFlow::Break
                        },
                        Some(window) => {
                            // Resize events need to be handled during run_forever
                            let cont = if let glutin::WindowEvent::Resized(_) = event {
                                glutin::ControlFlow::Continue
                            } else {
                                glutin::ControlFlow::Break
                            };
                            window.winit_event_to_servo_event(event);
                            return cont;
                        },
                    }
                });
            },
        }
        glutin::ControlFlow::Break
    }

    fn run_loop(self) {
        loop {
            let animating = WINDOWS.with(|windows| {
                windows
                    .borrow()
                    .iter()
                    .any(|(_, window)| window.is_animating())
            });
            if !animating || self.suspended.get() {
                // If there's no animations running then we block on the window event loop.
                self.events_loop.borrow_mut().run_forever(|e| {
                    let cont = self.winit_event_to_servo_event(e);
                    if cont == glutin::ControlFlow::Continue {
                        // Note we need to be careful to make sure that any events
                        // that are handled during run_forever aren't re-entrant,
                        // since we are handling them while holding onto a mutable borrow
                        // of the events loop
                        self.handle_events();
                    }
                    cont
                });
            }
            // Grab any other events that may have happened
            self.events_loop.borrow_mut().poll_events(|e| {
                self.winit_event_to_servo_event(e);
            });
            // If animations are running, we block on compositing
            // (self.handle_events() ends up calling swap_buffers)
            let stop = self.handle_events();
            if stop {
                break;
            }
        }

        self.servo.into_inner().deinit()
    }

    fn handle_events(&self) -> bool {
        let mut browser = self.browser.borrow_mut();
        let mut servo = self.servo.borrow_mut();

        // FIXME:
        // As of now, we support only one browser (self.browser)
        // but have multiple windows (dom.webxr.glwindow). We forward
        // the events of all the windows combined to that single
        // browser instance. Pressing the "a" key on the glwindow
        // will send a key event to the servo window.

        let mut app_events = self.get_events();
        WINDOWS.with(|windows| {
            for (_win_id, window) in &*windows.borrow() {
                app_events.extend(window.get_events());
            }
        });

        // FIXME: this could be handled by Servo. We don't need
        // a repaint_synchronously function exposed.
        let need_resize = app_events.iter().any(|e| match *e {
            WindowEvent::Resize => true,
            _ => false,
        });

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

pub fn register_window(window: Rc<dyn WindowPortsMethods>) {
    WINDOWS.with(|w| {
        w.borrow_mut().insert(window.id(), window);
    });
}

pub fn gl_version(angle: bool) -> glutin::GlRequest {
    if angle {
        glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0))
    } else {
        glutin::GlRequest::GlThenGles {
            opengl_version: (3, 2),
            opengles_version: (3, 0),
        }
    }
}
