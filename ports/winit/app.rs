/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application entry point, runs the event loop.

use crate::browser::Browser;
use crate::embedder::EmbedderCallbacks;
use crate::events_loop::{EventsLoop, ServoEvent};
use crate::window_trait::WindowPortsMethods;
use crate::{headed_window, headless_window};
use egui::TopBottomPanel;
use egui_winit::EventResponse;
use gleam::gl;
use glow::{NativeFramebuffer, HasContext};
use winit::window::WindowId;
use winit::event_loop::EventLoopWindowTarget;
use servo::compositing::windowing::WindowEvent;
use servo::config::opts::{self, parse_url_or_filename};
use servo::servo_config::pref;
use servo::servo_url::ServoUrl;
use servo::Servo;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::num::NonZeroU32;
use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use surfman::GLApi;
use webxr::glwindow::GlWindowDiscovery;

pub struct App {
    servo: Option<Servo<dyn WindowPortsMethods>>,
    browser: RefCell<Browser<dyn WindowPortsMethods>>,
    event_queue: RefCell<Vec<WindowEvent>>,
    suspended: Cell<bool>,
    windows: HashMap<WindowId, Rc<dyn WindowPortsMethods>>,
}

impl App {
    pub fn run(
        no_native_titlebar: bool,
        device_pixels_per_px: Option<f32>,
        user_agent: Option<String>,
    ) {
        let events_loop = EventsLoop::new(opts::get().headless, opts::get().output_file.is_some());

        // Implements window methods, used by compositor.
        let window = if opts::get().headless {
            headless_window::Window::new(opts::get().initial_window_size, device_pixels_per_px)
        } else {
            Rc::new(headed_window::Window::new(
                opts::get().initial_window_size,
                &events_loop,
                no_native_titlebar,
                device_pixels_per_px,
            ))
        };

        // Handle browser state.
        let browser = Browser::new(window.clone());

        let mut app = App {
            event_queue: RefCell::new(vec![]),
            browser: RefCell::new(browser),
            servo: None,
            suspended: Cell::new(false),
            windows: HashMap::new(),
        };

        struct Minibrowser {
            context: egui_glow::EguiGlow,
            location: RefCell<String>,
        }

        impl Minibrowser {
            fn update(&mut self, window: &winit::window::Window) {
                let Self { context, location } = self;
                let _duration = context.run(window, |ctx| {
                    TopBottomPanel::top("toolbar").show(ctx, |ui| {
                        ui.allocate_ui_with_layout(
                            ui.available_size(),
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.button("go").clicked() {
                                    // TODO go
                                    dbg!("go clicked");
                                }
                                ui.add_sized(
                                    ui.available_size(),
                                    egui::TextEdit::singleline(&mut *location.borrow_mut()),
                                );
                            },
                        );
                    });

                    // Get the toolbar size
                    // TODO somehow pass this to the headed Window::get_coordinates
                    dbg!(ctx.used_rect());
                });
                context.paint(window);
            }
        }

        // Create FrameBufferObject
        let webrender_surfman = window.webrender_surfman();
        let webrender_gl = match webrender_surfman.connection().gl_api() {
            GLApi::GL => unsafe { gl::GlFns::load_with(|s| webrender_surfman.get_proc_address(s)) },
            GLApi::GLES => unsafe {
                gl::GlesFns::load_with(|s| webrender_surfman.get_proc_address(s))
            },
        };

        // Make sure the gl context is made current.
        webrender_surfman.make_gl_context_current().unwrap();
        debug_assert_eq!(webrender_gl.get_error(), gleam::gl::NO_ERROR,);

        // Bind the webrender framebuffer
        let framebuffer_object = webrender_surfman
            .context_surface_info()
            .unwrap_or(None)
            .map(|info| info.framebuffer_object)
            .unwrap_or(0);

        let native_framebuffer = NativeFramebuffer(NonZeroU32::new(framebuffer_object).unwrap());

        let framebuffer_object = native_framebuffer.0.get();

        // Set up egui context for minibrowser ui
        // Adapted from https://github.com/emilk/egui/blob/9478e50d012c5138551c38cbee16b07bc1fcf283/crates/egui_glow/examples/pure_glow.rs
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                webrender_surfman.get_proc_address(s)
            })
        };

        // glow needs to set framebuffer as a target
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(NativeFramebuffer(NonZeroU32::new(framebuffer_object).unwrap())));
        };


        let mut minibrowser = window.winit_window().map(|_| Minibrowser {
            context: egui_glow::EguiGlow::new(events_loop.as_winit(), Arc::new(gl), None),
            location: RefCell::new(String::default()),
        });

        if let Some(minibrowser) = minibrowser.as_mut() {
            minibrowser.update(window.winit_window().unwrap());
        }

        let ev_waker = events_loop.create_event_loop_waker();
        events_loop.run_forever(move |e, w, control_flow| {
            if let winit::event::Event::NewEvents(winit::event::StartCause::Init) = e {
                let surfman = window.webrender_surfman();

                let xr_discovery = if pref!(dom.webxr.glwindow.enabled) && ! opts::get().headless {
                    let window = window.clone();
                    // This should be safe because run_forever does, in fact,
                    // run forever. The event loop window target doesn't get
                    // moved, and does outlast this closure, and we won't
                    // ever try to make use of it once shutdown begins and
                    // it stops being valid.
                    let w = unsafe {
                        std::mem::transmute::<
                            &EventLoopWindowTarget<ServoEvent>,
                            &'static EventLoopWindowTarget<ServoEvent>
                        >(w.unwrap())
                    };
                    let factory = Box::new(move || Ok(window.new_glwindow(w)));
                    Some(GlWindowDiscovery::new(
                        surfman.connection(),
                        surfman.adapter(),
                        surfman.context_attributes(),
                        factory,
                    ))
                } else {
                    None
                };

                let window = window.clone();
                // Implements embedder methods, used by libservo and constellation.
                let embedder = Box::new(EmbedderCallbacks::new(
                    ev_waker.clone(),
                    xr_discovery,
                ));

                let servo_data = Servo::new(embedder, window.clone(), user_agent.clone());
                let mut servo = servo_data.servo;
                servo.handle_events(vec![WindowEvent::NewBrowser(get_default_url(), servo_data.browser_id)]);
                servo.setup_logging();

                app.windows.insert(window.id(), window.clone());
                app.servo = Some(servo);
            }

            // If self.servo is None here, it means that we're in the process of shutting down,
            // let's ignore events.
            if app.servo.is_none() {
                return;
            }

            // Handle the event
            let response = match e {
                winit::event::Event::WindowEvent { ref event, .. } => {
                    if let Some(minibrowser) = minibrowser.as_mut() {
                        minibrowser.context.on_event(&event)
                    } else {
                        EventResponse { consumed: false, repaint: false }
                    }
                }
                _ => EventResponse { consumed: false, repaint: false },
            };

            // TODO how do we handle the tab key? (see doc for consumed)
            if !response.consumed {
                app.winit_event_to_servo_event(e);
            }
            if response.repaint {
                minibrowser.as_mut().unwrap().update(window.winit_window().unwrap());
            }

            let animating = app.is_animating();

            // Block until the window gets an event
            if !animating || app.suspended.get() {
                *control_flow = winit::event_loop::ControlFlow::Wait;
            } else {
                *control_flow = winit::event_loop::ControlFlow::Poll;
            }

            let stop = app.handle_events();
            if stop {
                *control_flow = winit::event_loop::ControlFlow::Exit;
                app.servo.take().unwrap().deinit();
            }
        });
    }

    fn is_animating(&self) -> bool {
        self.windows.iter().any(|(_, window)| window.is_animating())
    }

    fn get_events(&self) -> Vec<WindowEvent> {
        mem::replace(&mut *self.event_queue.borrow_mut(), Vec::new())
    }

    // This function decides whether the event should be handled during `run_forever`.
    fn winit_event_to_servo_event(&self, event: winit::event::Event<'_, ServoEvent>) {
        match event {
            // App level events
            winit::event::Event::Suspended => {
                self.suspended.set(true);
            },
            winit::event::Event::Resumed => {
                self.suspended.set(false);
                self.event_queue.borrow_mut().push(WindowEvent::Idle);
            },
            winit::event::Event::UserEvent(_) => {
                self.event_queue.borrow_mut().push(WindowEvent::Idle);
            },
            winit::event::Event::DeviceEvent { .. } => {},

            winit::event::Event::RedrawRequested(_) => {
                self.event_queue.borrow_mut().push(WindowEvent::Idle);
            },

            // Window level events
            winit::event::Event::WindowEvent {
                window_id, event, ..
            } => {
                match self.windows.get(&window_id) {
                    None => {
                        warn!("Got an event from unknown window");
                    },
                    Some(window) => {
                        window.winit_event_to_servo_event(event);
                    },
                }
            },

            winit::event::Event::LoopDestroyed |
            winit::event::Event::NewEvents(..) |
            winit::event::Event::MainEventsCleared |
            winit::event::Event::RedrawEventsCleared => {},
        }
    }

    fn handle_events(&mut self) -> bool {
        let mut browser = self.browser.borrow_mut();

        // FIXME:
        // As of now, we support only one browser (self.browser)
        // but have multiple windows (dom.webxr.glwindow). We forward
        // the events of all the windows combined to that single
        // browser instance. Pressing the "a" key on the glwindow
        // will send a key event to the servo window.

        let mut app_events = self.get_events();
        for (_win_id, window) in &self.windows {
            app_events.extend(window.get_events());
        }

        browser.handle_window_events(app_events);

        let mut servo_events = self.servo.as_mut().unwrap().get_events();
        let mut need_resize = false;
        loop {
            browser.handle_servo_events(servo_events);
            need_resize |= self.servo.as_mut().unwrap().handle_events(browser.get_events());
            if browser.shutdown_requested() {
                return true;
            }
            servo_events = self.servo.as_mut().unwrap().get_events();
            if servo_events.is_empty() {
                break;
            }
        }

        if need_resize {
            self.servo.as_mut().unwrap().repaint_synchronously();
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
