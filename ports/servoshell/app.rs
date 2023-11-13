/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application entry point, runs the event loop.

use std::cell::{Cell, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;
use std::{env, fs};

use gleam::gl;
use log::{info, trace, warn};
use servo::compositing::windowing::EmbedderEvent;
use servo::config::opts;
use servo::servo_config::pref;
use servo::Servo;
use surfman::GLApi;
use webxr::glwindow::GlWindowDiscovery;
use winit::event::WindowEvent;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::WindowId;

use crate::browser::Browser;
use crate::embedder::EmbedderCallbacks;
use crate::events_loop::{EventsLoop, WakerEvent};
use crate::minibrowser::Minibrowser;
use crate::parser::get_default_url;
use crate::window_trait::WindowPortsMethods;
use crate::{headed_window, headless_window};

pub struct App {
    servo: Option<Servo<dyn WindowPortsMethods>>,
    browser: RefCell<Browser<dyn WindowPortsMethods>>,
    event_queue: RefCell<Vec<EmbedderEvent>>,
    suspended: Cell<bool>,
    windows: HashMap<WindowId, Rc<dyn WindowPortsMethods>>,
    minibrowser: Option<RefCell<Minibrowser>>,
}

/// Action to be taken by the caller of [`App::handle_events`].
enum PumpResult {
    /// The caller should shut down Servo and its related context.
    Shutdown,
    /// A new frame is ready to present. The caller can paint other things themselves during this
    /// period, but has to call [`Servo::present`] to perform page flip and tell Servo compositor
    /// to continue rendering.
    ReadyToPresent,
    /// The size has changed. The caller can paint other things themselves during this
    /// period, but has to call [`Servo::present`] to perform page flip and tell Servo compositor
    /// to continue rendering.
    Resize,
}

impl App {
    pub fn run(
        no_native_titlebar: bool,
        device_pixel_ratio_override: Option<f32>,
        user_agent: Option<String>,
        url: Option<String>,
    ) {
        let events_loop = EventsLoop::new(opts::get().headless, opts::get().output_file.is_some());

        // Implements window methods, used by compositor.
        let window = if opts::get().headless {
            headless_window::Window::new(
                opts::get().initial_window_size,
                device_pixel_ratio_override,
            )
        } else {
            Rc::new(headed_window::Window::new(
                opts::get().initial_window_size,
                &events_loop,
                no_native_titlebar,
                device_pixel_ratio_override,
            ))
        };

        // Handle browser state.
        let browser = Browser::new(window.clone());
        let initial_url = get_default_url(
            url.as_ref().map(String::as_str),
            env::current_dir().unwrap(),
            |path| fs::metadata(path).is_ok(),
        );

        let mut app = App {
            event_queue: RefCell::new(vec![]),
            browser: RefCell::new(browser),
            servo: None,
            suspended: Cell::new(false),
            windows: HashMap::new(),
            minibrowser: None,
        };

        if opts::get().minibrowser && window.winit_window().is_some() {
            // Make sure the gl context is made current.
            let webrender_surfman = window.webrender_surfman();
            let webrender_gl = match webrender_surfman.connection().gl_api() {
                GLApi::GL => unsafe {
                    gl::GlFns::load_with(|s| webrender_surfman.get_proc_address(s))
                },
                GLApi::GLES => unsafe {
                    gl::GlesFns::load_with(|s| webrender_surfman.get_proc_address(s))
                },
            };
            webrender_surfman.make_gl_context_current().unwrap();
            debug_assert_eq!(webrender_gl.get_error(), gleam::gl::NO_ERROR);

            app.minibrowser =
                Some(Minibrowser::new(&webrender_surfman, &events_loop, window.as_ref()).into());
        }

        if let Some(mut minibrowser) = app.minibrowser() {
            minibrowser.update(window.winit_window().unwrap(), "init");
            window.set_toolbar_height(minibrowser.toolbar_height.get());
        }

        // Whether or not to recomposite during the next RedrawRequested event.
        // Normally this is true, including for RedrawRequested events that come from the platform
        // (e.g. X11 without picom or similar) when an offscreen or obscured window becomes visible.
        // If we are calling request_redraw in response to the compositor having painted to this
        // frame, set this to false, so we can avoid an unnecessary recomposite.
        let mut need_recomposite = true;

        let t_start = Instant::now();
        let mut t = t_start;
        let ev_waker = events_loop.create_event_loop_waker();
        events_loop.run_forever(move |event, w, control_flow| {
            let now = Instant::now();
            match event {
                // Uncomment to filter out logging of common events, which can be very noisy.
                // winit::event::Event::DeviceEvent { .. } => {},
                // winit::event::Event::WindowEvent {
                //     event: WindowEvent::CursorMoved { .. },
                //     ..
                // } => {},
                // winit::event::Event::MainEventsCleared => {},
                // winit::event::Event::RedrawEventsCleared => {},
                // winit::event::Event::UserEvent(..) => {},
                // winit::event::Event::NewEvents(..) => {},
                _ => trace!("@{:?} (+{:?}) {:?}", now - t_start, now - t, event),
            }
            t = now;
            match event {
                winit::event::Event::NewEvents(winit::event::StartCause::Init) => {
                    let surfman = window.webrender_surfman();

                    let xr_discovery = if pref!(dom.webxr.glwindow.enabled) && !opts::get().headless
                    {
                        let window = window.clone();
                        // This should be safe because run_forever does, in fact,
                        // run forever. The event loop window target doesn't get
                        // moved, and does outlast this closure, and we won't
                        // ever try to make use of it once shutdown begins and
                        // it stops being valid.
                        let w = unsafe {
                            std::mem::transmute::<
                                &EventLoopWindowTarget<WakerEvent>,
                                &'static EventLoopWindowTarget<WakerEvent>,
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
                    let embedder = Box::new(EmbedderCallbacks::new(ev_waker.clone(), xr_discovery));

                    let servo_data = Servo::new(embedder, window.clone(), user_agent.clone());
                    let mut servo = servo_data.servo;

                    servo.handle_events(vec![EmbedderEvent::NewBrowser(
                        initial_url.to_owned(),
                        servo_data.browser_id,
                    )]);
                    servo.setup_logging();

                    app.windows.insert(window.id(), window.clone());
                    app.servo = Some(servo);
                },
                _ => {},
            }

            // If self.servo is None here, it means that we're in the process of shutting down,
            // let's ignore events.
            if app.servo.is_none() {
                return;
            }

            if let winit::event::Event::RedrawRequested(_) = event {
                // We need to redraw the window for some reason.
                trace!("RedrawRequested");

                // WARNING: do not defer painting or presenting to some later tick of the event
                // loop or servoshell may become unresponsive! (servo#30312)
                if need_recomposite {
                    trace!("need_recomposite");
                    app.servo.as_mut().unwrap().recomposite();
                }
                if let Some(mut minibrowser) = app.minibrowser() {
                    minibrowser.update(window.winit_window().unwrap(), "RedrawRequested");
                    minibrowser.paint(window.winit_window().unwrap());
                }
                app.servo.as_mut().unwrap().present();

                // By default, the next RedrawRequested event will need to recomposite.
                need_recomposite = true;
            }

            // Handle the event
            let mut consumed = false;
            if let Some(mut minibrowser) = app.minibrowser() {
                match event {
                    winit::event::Event::WindowEvent {
                        event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
                        ..
                    } => {
                        // Intercept any ScaleFactorChanged events away from EguiGlow::on_event, so
                        // we can use our own logic for calculating the scale factor and set egui’s
                        // scale factor to that value manually.
                        let effective_scale_factor = window.hidpi_factor().get();
                        info!(
                            "window scale factor changed to {}, setting scale factor to {}",
                            scale_factor, effective_scale_factor
                        );
                        minibrowser
                            .context
                            .egui_ctx
                            .set_pixels_per_point(effective_scale_factor);

                        // Request a winit redraw event, so we can recomposite, update and paint
                        // the minibrowser, and present the new frame.
                        window.winit_window().unwrap().request_redraw();
                    },
                    winit::event::Event::WindowEvent { ref event, .. } => {
                        let response = minibrowser.context.on_event(&event);
                        if response.repaint {
                            // Request a winit redraw event, so we can recomposite, update and paint
                            // the minibrowser, and present the new frame.
                            window.winit_window().unwrap().request_redraw();
                        }

                        // TODO how do we handle the tab key? (see doc for consumed)
                        // Note that servo doesn’t yet support tabbing through links and inputs
                        consumed = response.consumed;
                    },
                    _ => {},
                }
            }
            if !consumed {
                app.queue_embedder_events_for_winit_event(event);
            }

            let animating = app.is_animating();

            // Block until the window gets an event
            if !animating || app.suspended.get() {
                *control_flow = winit::event_loop::ControlFlow::Wait;
            } else {
                *control_flow = winit::event_loop::ControlFlow::Poll;
            }

            // Consume and handle any events from the Minibrowser.
            if let Some(mut minibrowser) = app.minibrowser() {
                let browser = &mut app.browser.borrow_mut();
                let app_event_queue = &mut app.event_queue.borrow_mut();
                minibrowser.queue_embedder_events_for_minibrowser_events(browser, app_event_queue);
                if minibrowser.update_location_in_toolbar(browser) {
                    // Update the minibrowser immediately. While we could update by requesting a
                    // redraw, doing so would delay the location update by two frames.
                    minibrowser
                        .update(window.winit_window().unwrap(), "update_location_in_toolbar");
                }
            }

            match app.handle_events() {
                Some(PumpResult::Shutdown) => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                    app.servo.take().unwrap().deinit();
                    if let Some(mut minibrowser) = app.minibrowser() {
                        minibrowser.context.destroy();
                    }
                },
                Some(PumpResult::ReadyToPresent) => {
                    // The compositor has painted to this frame.
                    trace!("PumpResult::ReadyToPresent");

                    // Request a winit redraw event, so we can paint the minibrowser and present.
                    // Otherwise, it's in headless mode and we present directly.
                    if let Some(window) = window.winit_window() {
                        window.request_redraw();
                    } else {
                        app.servo.as_mut().unwrap().present();
                    }

                    // We don’t need the compositor to paint to this frame during the redraw event.
                    // TODO(servo#30331) broken on macOS?
                    // need_recomposite = false;
                },
                Some(PumpResult::Resize) => {
                    // The window was resized.
                    trace!("PumpResult::Resize");

                    // Resizes are unusual in that we need to repaint synchronously.
                    // TODO(servo#30049) can we replace this with the simpler Servo::recomposite?
                    app.servo.as_mut().unwrap().repaint_synchronously();

                    if let Some(mut minibrowser) = app.minibrowser() {
                        minibrowser.update(window.winit_window().unwrap(), "PumpResult::Resize");
                        minibrowser.paint(window.winit_window().unwrap());
                    }
                    app.servo.as_mut().unwrap().present();
                },
                None => {},
            }
        });
    }

    fn is_animating(&self) -> bool {
        self.windows.iter().any(|(_, window)| window.is_animating())
    }

    fn get_events(&self) -> Vec<EmbedderEvent> {
        std::mem::take(&mut *self.event_queue.borrow_mut())
    }

    /// Processes the given winit Event, possibly converting it to an [EmbedderEvent] and
    /// routing that to the App or relevant Window event queues.
    fn queue_embedder_events_for_winit_event(&self, event: winit::event::Event<'_, WakerEvent>) {
        match event {
            // App level events
            winit::event::Event::Suspended => {
                self.suspended.set(true);
            },
            winit::event::Event::Resumed => {
                self.suspended.set(false);
                self.event_queue.borrow_mut().push(EmbedderEvent::Idle);
            },
            winit::event::Event::UserEvent(_) => {
                self.event_queue.borrow_mut().push(EmbedderEvent::Idle);
            },
            winit::event::Event::DeviceEvent { .. } => {},

            winit::event::Event::RedrawRequested(_) => {
                self.event_queue.borrow_mut().push(EmbedderEvent::Idle);
            },

            // Window level events
            winit::event::Event::WindowEvent {
                window_id, event, ..
            } => match self.windows.get(&window_id) {
                None => {
                    warn!("Got an event from unknown window");
                },
                Some(window) => {
                    window.queue_embedder_events_for_winit_event(event);
                },
            },

            winit::event::Event::LoopDestroyed |
            winit::event::Event::NewEvents(..) |
            winit::event::Event::MainEventsCleared |
            winit::event::Event::RedrawEventsCleared => {},
        }
    }

    /// Pumps events and messages between the embedder and Servo, where embedder events flow
    /// towards Servo and embedder messages flow away from Servo, and also runs the compositor.
    ///
    /// As the embedder, we push embedder events through our event queues, from the App queue and
    /// Window queues to the Browser queue, and from the Browser queue to Servo. We receive and
    /// collect embedder messages from the various Servo components, then take them out of the
    /// Servo interface so that the Browser can handle them.
    fn handle_events(&mut self) -> Option<PumpResult> {
        let mut browser = self.browser.borrow_mut();

        // FIXME:
        // As of now, we support only one browser (self.browser)
        // but have multiple windows (dom.webxr.glwindow). We forward
        // the events of all the windows combined to that single
        // browser instance. Pressing the "a" key on the glwindow
        // will send a key event to the servo window.

        // Take any outstanding embedder events from the App and its Windows.
        let mut embedder_events = self.get_events();
        for (_win_id, window) in &self.windows {
            embedder_events.extend(window.get_events());
        }

        // Catch some keyboard events, and push the rest onto the Browser event queue.
        browser.handle_window_events(embedder_events);

        // Take any new embedder messages from Servo itself.
        let mut embedder_messages = self.servo.as_mut().unwrap().get_events();
        let mut need_resize = false;
        let mut need_present = false;
        loop {
            // Consume and handle those embedder messages.
            need_present |= browser.handle_servo_events(embedder_messages);

            // Route embedder events from the Browser to the relevant Servo components,
            // receives and collects embedder messages from various Servo components,
            // and runs the compositor.
            need_resize |= self
                .servo
                .as_mut()
                .unwrap()
                .handle_events(browser.get_events());
            if browser.shutdown_requested() {
                return Some(PumpResult::Shutdown);
            }

            // Take any new embedder messages from Servo itself.
            embedder_messages = self.servo.as_mut().unwrap().get_events();
            if embedder_messages.is_empty() {
                break;
            }
        }

        if need_resize {
            Some(PumpResult::Resize)
        } else if need_present {
            Some(PumpResult::ReadyToPresent)
        } else {
            None
        }
    }

    fn minibrowser(&self) -> Option<RefMut<Minibrowser>> {
        self.minibrowser.as_ref().map(|x| x.borrow_mut())
    }
}
