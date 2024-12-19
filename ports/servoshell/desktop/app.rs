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
use log::{info, trace};
use servo::compositing::windowing::EmbedderEvent;
use servo::compositing::CompositeTarget;
use servo::config::{opts, set_pref};
use servo::embedder_traits::EventLoopWaker;
use servo::servo_config::pref;
use servo::url::ServoUrl;
use servo::Servo;
use surfman::GLApi;
use webxr::glwindow::GlWindowDiscovery;
#[cfg(target_os = "windows")]
use webxr::openxr::{AppInfo, OpenXrDiscovery};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use super::events_loop::{EventLoopGuard, EventsLoop, WakerEvent};
use super::minibrowser::Minibrowser;
use super::webview::WebViewManager;
use super::{headed_window, headless_window};
use crate::desktop::embedder::{EmbedderCallbacks, XrDiscovery};
use crate::desktop::events_loop::with_current_event_loop;
use crate::desktop::tracing::trace_winit_event;
use crate::desktop::window_trait::WindowPortsMethods;
use crate::parser::get_default_url;

pub struct App {
    servo: Option<Servo<dyn WindowPortsMethods>>,
    webviews: RefCell<WebViewManager<dyn WindowPortsMethods>>,
    event_queue: RefCell<Vec<EmbedderEvent>>,
    suspended: Cell<bool>,
    windows: HashMap<WindowId, Rc<dyn WindowPortsMethods>>,
    minibrowser: Option<RefCell<Minibrowser>>,
    user_agent: Option<String>,
    waker: Box<dyn EventLoopWaker>,
    initial_url: ServoUrl,
    t_start: Instant,
    t: Instant,
}

enum Present {
    Immediate,
    Deferred,
    None,
}

/// Action to be taken by the caller of [`App::handle_events`].
enum PumpResult {
    /// The caller should shut down Servo and its related context.
    Shutdown,
    Continue {
        update: bool,
        present: Present,
    },
}

impl App {
    pub fn new(
        events_loop: &EventsLoop,
        no_native_titlebar: bool,
        device_pixel_ratio_override: Option<f32>,
        user_agent: Option<String>,
        url: Option<String>,
    ) -> Self {
        // Implements window methods, used by compositor.
        let window = if opts::get().headless {
            // GL video rendering is not supported on headless windows.
            set_pref!(media.glvideo.enabled, false);
            headless_window::Window::new(
                opts::get().initial_window_size,
                device_pixel_ratio_override,
            )
        } else {
            Rc::new(headed_window::Window::new(
                opts::get().initial_window_size,
                events_loop.as_winit(),
                no_native_titlebar,
                device_pixel_ratio_override,
            ))
        };

        // Handle browser state.
        let webviews = WebViewManager::new(window.clone());
        let initial_url = get_default_url(url.as_deref(), env::current_dir().unwrap(), |path| {
            fs::metadata(path).is_ok()
        });
        let t = Instant::now();
        let mut app = App {
            event_queue: RefCell::new(vec![]),
            webviews: RefCell::new(webviews),
            servo: None,
            suspended: Cell::new(false),
            windows: HashMap::new(),
            minibrowser: None,
            user_agent,
            waker: events_loop.create_event_loop_waker(),
            initial_url: initial_url.clone(),
            t_start: t,
            t,
        };

        if window.winit_window().is_some() {
            // Make sure the gl context is made current.
            let rendering_context = window.rendering_context();
            let webrender_gl = match rendering_context.connection().gl_api() {
                GLApi::GL => unsafe {
                    gl::GlFns::load_with(|s| rendering_context.get_proc_address(s))
                },
                GLApi::GLES => unsafe {
                    gl::GlesFns::load_with(|s| rendering_context.get_proc_address(s))
                },
            };
            rendering_context.make_gl_context_current().unwrap();
            debug_assert_eq!(webrender_gl.get_error(), gleam::gl::NO_ERROR);

            app.minibrowser = Some(
                Minibrowser::new(
                    &rendering_context,
                    events_loop.as_winit(),
                    initial_url.clone(),
                )
                .into(),
            );
        }

        if let Some(mut minibrowser) = app.minibrowser() {
            // Servo is not yet initialised, so there is no `servo_framebuffer_id`.
            minibrowser.update(
                window.winit_window().unwrap(),
                &mut app.webviews.borrow_mut(),
                None,
                "init",
            );
            window.set_toolbar_height(minibrowser.toolbar_height);
        }

        app.windows.insert(window.id(), window);

        app
    }

    /// Initialize Application once event loop start running.
    pub fn init(&mut self) {
        self.suspended.set(false);
        self.event_queue.borrow_mut().push(EmbedderEvent::Idle);
        let (_, window) = self.windows.iter().next().unwrap();
        let surfman = window.rendering_context();

        let openxr_discovery = if pref!(dom.webxr.openxr.enabled) && !opts::get().headless {
            #[cfg(target_os = "windows")]
            let openxr = {
                let app_info = AppInfo::new("Servoshell", 0, "Servo", 0);
                Some(XrDiscovery::OpenXr(OpenXrDiscovery::new(None, app_info)))
            };
            #[cfg(not(target_os = "windows"))]
            let openxr = None;

            openxr
        } else {
            None
        };

        let glwindow_discovery = if pref!(dom.webxr.glwindow.enabled) && !opts::get().headless {
            let window = window.clone();
            let factory = Box::new(move || {
                with_current_event_loop(|w| Ok(window.new_glwindow(w)))
                    .expect("An event loop should always be active in headed mode")
            });
            Some(XrDiscovery::GlWindow(GlWindowDiscovery::new(
                surfman.connection(),
                surfman.adapter(),
                surfman.context_attributes(),
                factory,
            )))
        } else {
            None
        };

        let xr_discovery = openxr_discovery.or(glwindow_discovery);

        let window = window.clone();
        // Implements embedder methods, used by libservo and constellation.
        let embedder = Box::new(EmbedderCallbacks::new(self.waker.clone(), xr_discovery));

        let composite_target = if self.minibrowser.is_some() {
            CompositeTarget::Fbo
        } else {
            CompositeTarget::Window
        };
        let servo_data = Servo::new(
            embedder,
            window.clone(),
            self.user_agent.clone(),
            composite_target,
        );
        let mut servo = servo_data.servo;

        servo.handle_events(vec![EmbedderEvent::NewWebView(
            self.initial_url.to_owned(),
            servo_data.browser_id,
        )]);
        servo.setup_logging();

        self.servo = Some(servo);
    }

    pub fn is_animating(&self) -> bool {
        self.windows.iter().any(|(_, window)| window.is_animating())
    }

    fn get_events(&self) -> Vec<EmbedderEvent> {
        std::mem::take(&mut *self.event_queue.borrow_mut())
    }

    /// Pumps events and messages between the embedder and Servo, where embedder events flow
    /// towards Servo and embedder messages flow away from Servo, and also runs the compositor.
    ///
    /// As the embedder, we push embedder events through our event queues, from the App queue and
    /// Window queues to the WebViewManager queue, and from the WebViewManager queue to Servo. We
    /// receive and collect embedder messages from the various Servo components, then take them out
    /// of the Servo interface so that the WebViewManager can handle them.
    fn handle_events(&mut self) -> PumpResult {
        let mut webviews = self.webviews.borrow_mut();

        // Take any outstanding embedder events from the App and its Windows.
        let mut embedder_events = self.get_events();
        for window in self.windows.values() {
            embedder_events.extend(window.get_events());
        }

        // Catch some keyboard events, and push the rest onto the WebViewManager event queue.
        webviews.handle_window_events(embedder_events);

        // If the Gamepad API is enabled, handle gamepad events from GilRs.
        // Checking for focused_webview_id should ensure we'll have a valid browsing context.
        if pref!(dom.gamepad.enabled) && webviews.focused_webview_id().is_some() {
            webviews.handle_gamepad_events();
        }

        // Take any new embedder messages from Servo itself.
        let mut embedder_messages = self.servo.as_mut().unwrap().get_events();
        let mut need_resize = false;
        let mut need_present = false;
        let mut need_update = false;
        loop {
            // Consume and handle those embedder messages.
            let servo_event_response = webviews.handle_servo_events(embedder_messages);
            need_present |= servo_event_response.need_present;
            need_update |= servo_event_response.need_update;

            // Route embedder events from the WebViewManager to the relevant Servo components,
            // receives and collects embedder messages from various Servo components,
            // and runs the compositor.
            need_resize |= self
                .servo
                .as_mut()
                .unwrap()
                .handle_events(webviews.get_events());
            if webviews.shutdown_requested() {
                return PumpResult::Shutdown;
            }

            // Take any new embedder messages from Servo itself.
            embedder_messages = self.servo.as_mut().unwrap().get_events();
            if embedder_messages.len() == 0 {
                break;
            }
        }

        let present = if need_resize {
            Present::Immediate
        } else if need_present {
            Present::Deferred
        } else {
            Present::None
        };

        PumpResult::Continue {
            update: need_update,
            present,
        }
    }

    /// Handle events with winit contexts
    pub fn handle_events_with_winit(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: Rc<dyn WindowPortsMethods>,
    ) {
        match self.handle_events() {
            PumpResult::Shutdown => {
                event_loop.exit();
                self.servo.take().unwrap().deinit();
                if let Some(mut minibrowser) = self.minibrowser() {
                    minibrowser.context.destroy();
                }
            },
            PumpResult::Continue { update, present } => {
                if update {
                    if let Some(mut minibrowser) = self.minibrowser() {
                        let webviews = &mut self.webviews.borrow_mut();
                        if minibrowser.update_webview_data(webviews) {
                            // Update the minibrowser immediately. While we could update by requesting a
                            // redraw, doing so would delay the location update by two frames.
                            minibrowser.update(
                                window.winit_window().unwrap(),
                                webviews,
                                self.servo.as_ref().unwrap().offscreen_framebuffer_id(),
                                "update_location_in_toolbar",
                            );
                        }
                    }
                }
                match present {
                    Present::Immediate => {
                        // The window was resized.
                        trace!("PumpResult::Present::Immediate");

                        // If we had resized any of the viewports in response to this, we would need to
                        // call Servo::repaint_synchronously. At the moment we don’t, so there won’t be
                        // any paint scheduled, and calling it would hang the compositor forever.
                        if let Some(mut minibrowser) = self.minibrowser() {
                            minibrowser.update(
                                window.winit_window().unwrap(),
                                &mut self.webviews.borrow_mut(),
                                self.servo.as_ref().unwrap().offscreen_framebuffer_id(),
                                "PumpResult::Present::Immediate",
                            );
                            minibrowser.paint(window.winit_window().unwrap());
                        }
                        self.servo.as_mut().unwrap().present();
                    },
                    Present::Deferred => {
                        // The compositor has painted to this frame.
                        trace!("PumpResult::Present::Deferred");
                        // Request a winit redraw event, so we can paint the minibrowser and present.
                        // Otherwise, it's in headless mode and we present directly.
                        if let Some(window) = window.winit_window() {
                            window.request_redraw();
                        } else {
                            self.servo.as_mut().unwrap().present();
                        }
                    },
                    Present::None => {},
                }
            },
        }
    }

    /// Handle all servo events with headless mode. Return true if servo request to shutdown.
    pub fn handle_events_with_headless(&mut self) -> bool {
        let now = Instant::now();
        let event = winit::event::Event::UserEvent(WakerEvent);
        trace_winit_event!(
            event,
            "@{:?} (+{:?}) {event:?}",
            now - self.t_start,
            now - self.t
        );
        self.t = now;
        // If self.servo is None here, it means that we're in the process of shutting down,
        // let's ignore events.
        if self.servo.is_none() {
            return false;
        }
        self.event_queue.borrow_mut().push(EmbedderEvent::Idle);

        let mut exit = false;
        match self.handle_events() {
            PumpResult::Shutdown => {
                exit = true;
                self.servo.take().unwrap().deinit();
                if let Some(mut minibrowser) = self.minibrowser() {
                    minibrowser.context.destroy();
                }
            },
            PumpResult::Continue { present, .. } => {
                match present {
                    Present::Immediate => {
                        // The window was resized.
                        trace!("PumpResult::Present::Immediate");
                        self.servo.as_mut().unwrap().present();
                    },
                    Present::Deferred => {
                        // The compositor has painted to this frame.
                        trace!("PumpResult::Present::Deferred");
                        // In headless mode, we present directly.
                        self.servo.as_mut().unwrap().present();
                    },
                    Present::None => {},
                }
            },
        }
        exit
    }

    fn minibrowser(&self) -> Option<RefMut<Minibrowser>> {
        self.minibrowser.as_ref().map(|x| x.borrow_mut())
    }
}

impl ApplicationHandler<WakerEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let _guard = EventLoopGuard::new(event_loop);
        self.init();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let now = Instant::now();
        trace_winit_event!(
            event,
            "@{:?} (+{:?}) {event:?}",
            now - self.t_start,
            now - self.t
        );
        self.t = now;
        // If self.servo is None here, it means that we're in the process of shutting down,
        // let's ignore events.
        if self.servo.is_none() {
            return;
        }

        let Some(window) = self.windows.get(&window_id) else {
            return;
        };
        let window = window.clone();

        if event == winit::event::WindowEvent::RedrawRequested {
            // We need to redraw the window for some reason.
            trace!("RedrawRequested");

            // WARNING: do not defer painting or presenting to some later tick of the event
            // loop or servoshell may become unresponsive! (servo#30312)
            if let Some(mut minibrowser) = self.minibrowser() {
                minibrowser.update(
                    window.winit_window().unwrap(),
                    &mut self.webviews.borrow_mut(),
                    self.servo.as_ref().unwrap().offscreen_framebuffer_id(),
                    "RedrawRequested",
                );
                minibrowser.paint(window.winit_window().unwrap());
            }

            self.servo.as_mut().unwrap().present();
        }

        // Handle the event
        let mut consumed = false;
        if let Some(mut minibrowser) = self.minibrowser() {
            match event {
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    // Intercept any ScaleFactorChanged events away from EguiGlow::on_window_event, so
                    // we can use our own logic for calculating the scale factor and set egui’s
                    // scale factor to that value manually.
                    let desired_scale_factor = window.hidpi_factor().get();
                    let effective_egui_zoom_factor = desired_scale_factor / scale_factor as f32;

                    info!(
                        "window scale factor changed to {}, setting egui zoom factor to {}",
                        scale_factor, effective_egui_zoom_factor
                    );

                    minibrowser
                        .context
                        .egui_ctx
                        .set_zoom_factor(effective_egui_zoom_factor);

                    // Request a winit redraw event, so we can recomposite, update and paint
                    // the minibrowser, and present the new frame.
                    window.winit_window().unwrap().request_redraw();
                },
                ref event => {
                    let response =
                        minibrowser.on_window_event(window.winit_window().unwrap(), event);
                    // Update minibrowser if there's resize event to sync up with window.
                    if let WindowEvent::Resized(_) = event {
                        minibrowser.update(
                            window.winit_window().unwrap(),
                            &mut self.webviews.borrow_mut(),
                            self.servo.as_ref().unwrap().offscreen_framebuffer_id(),
                            "Sync WebView size with Window Resize event",
                        );
                    }
                    if response.repaint && *event != winit::event::WindowEvent::RedrawRequested {
                        // Request a winit redraw event, so we can recomposite, update and paint
                        // the minibrowser, and present the new frame.
                        window.winit_window().unwrap().request_redraw();
                    }

                    // TODO how do we handle the tab key? (see doc for consumed)
                    // Note that servo doesn’t yet support tabbing through links and inputs
                    consumed = response.consumed;
                },
            }
        }
        if !consumed {
            if event == winit::event::WindowEvent::RedrawRequested {
                self.event_queue.borrow_mut().push(EmbedderEvent::Idle);
            }

            window.queue_embedder_events_for_winit_event(event);
        }

        let animating = self.is_animating();

        // Block until the window gets an event
        if !animating || self.suspended.get() {
            event_loop.set_control_flow(ControlFlow::Wait);
        } else {
            event_loop.set_control_flow(ControlFlow::Poll);
        }

        // Consume and handle any events from the Minibrowser.
        if let Some(minibrowser) = self.minibrowser() {
            let webviews = &mut self.webviews.borrow_mut();
            let app_event_queue = &mut self.event_queue.borrow_mut();
            minibrowser.queue_embedder_events_for_minibrowser_events(webviews, app_event_queue);
        }

        self.handle_events_with_winit(event_loop, window);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: WakerEvent) {
        let now = Instant::now();
        let event = winit::event::Event::UserEvent(event);
        trace_winit_event!(
            event,
            "@{:?} (+{:?}) {event:?}",
            now - self.t_start,
            now - self.t
        );
        self.t = now;
        // If self.servo is None here, it means that we're in the process of shutting down,
        // let's ignore events.
        if self.servo.is_none() {
            return;
        }
        self.event_queue.borrow_mut().push(EmbedderEvent::Idle);

        let Some(window) = self.windows.values().next() else {
            return;
        };
        let window = window.clone();

        let animating = self.is_animating();

        // Block until the window gets an event
        if !animating || self.suspended.get() {
            event_loop.set_control_flow(ControlFlow::Wait);
        } else {
            event_loop.set_control_flow(ControlFlow::Poll);
        }

        // Consume and handle any events from the Minibrowser.
        if let Some(minibrowser) = self.minibrowser() {
            let webviews = &mut self.webviews.borrow_mut();
            let app_event_queue = &mut self.event_queue.borrow_mut();
            minibrowser.queue_embedder_events_for_minibrowser_events(webviews, app_event_queue);
        }

        self.handle_events_with_winit(event_loop, window);
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.suspended.set(true);
    }
}
