/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application entry point, runs the event loop.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;
use std::{env, fs};

use log::{info, trace};
use raw_window_handle::HasDisplayHandle;
use servo::base::id::WebViewId;
use servo::compositing::windowing::EmbedderEvent;
use servo::compositing::CompositeTarget;
use servo::config::opts::Opts;
use servo::config::prefs::Preferences;
use servo::embedder_traits::EventLoopWaker;
use servo::servo_config::pref;
use servo::url::ServoUrl;
use servo::webrender_traits::RenderingContext;
use servo::Servo;
use surfman::Connection;
use webxr::glwindow::GlWindowDiscovery;
#[cfg(target_os = "windows")]
use webxr::openxr::{AppInfo, OpenXrDiscovery};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use super::events_loop::{EventsLoop, WakerEvent};
use super::minibrowser::Minibrowser;
use super::webview::WebViewManager;
use super::{headed_window, headless_window};
use crate::desktop::embedder::{EmbedderCallbacks, XrDiscovery};
use crate::desktop::tracing::trace_winit_event;
use crate::desktop::window_trait::WindowPortsMethods;
use crate::parser::get_default_url;
use crate::prefs::ServoShellPreferences;

pub struct App {
    opts: Opts,
    preferences: Preferences,
    servo_shell_preferences: ServoShellPreferences,
    servo: Option<Servo<dyn WindowPortsMethods>>,
    webviews: Option<WebViewManager<dyn WindowPortsMethods>>,
    event_queue: Vec<EmbedderEvent>,
    suspended: Cell<bool>,
    windows: HashMap<WindowId, Rc<dyn WindowPortsMethods>>,
    minibrowser: Option<Minibrowser>,
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
        opts: Opts,
        preferences: Preferences,
        servo_shell_preferences: ServoShellPreferences,
        events_loop: &EventsLoop,
    ) -> Self {
        let initial_url = get_default_url(
            servo_shell_preferences.url.as_deref(),
            env::current_dir().unwrap(),
            |path| fs::metadata(path).is_ok(),
            &servo_shell_preferences,
        );

        let t = Instant::now();
        App {
            opts,
            preferences,
            servo_shell_preferences,
            event_queue: vec![],
            webviews: None,
            servo: None,
            suspended: Cell::new(false),
            windows: HashMap::new(),
            minibrowser: None,
            waker: events_loop.create_event_loop_waker(),
            initial_url: initial_url.clone(),
            t_start: t,
            t,
        }
    }

    /// Initialize Application once event loop start running.
    pub fn init(&mut self, event_loop: Option<&ActiveEventLoop>) {
        // Create rendering context
        let rendering_context = if self.opts.headless {
            let connection = Connection::new().expect("Failed to create connection");
            let adapter = connection
                .create_software_adapter()
                .expect("Failed to create adapter");
            RenderingContext::create(
                &connection,
                &adapter,
                Some(self.opts.initial_window_size.to_untyped().to_i32()),
            )
            .expect("Failed to create WR surfman")
        } else {
            let display_handle = event_loop
                .unwrap()
                .display_handle()
                .expect("could not get display handle from window");
            let connection = Connection::from_display_handle(display_handle)
                .expect("Failed to create connection");
            let adapter = connection
                .create_adapter()
                .expect("Failed to create adapter");
            RenderingContext::create(&connection, &adapter, None)
                .expect("Failed to create WR surfman")
        };

        let window = if self.opts.headless {
            headless_window::Window::new(
                self.opts.initial_window_size,
                self.servo_shell_preferences.device_pixel_ratio_override,
                self.opts.screen_size_override,
            )
        } else {
            Rc::new(headed_window::Window::new(
                &self.opts,
                &rendering_context,
                self.opts.initial_window_size,
                event_loop.unwrap(),
                self.servo_shell_preferences.no_native_titlebar,
                self.servo_shell_preferences.device_pixel_ratio_override,
            ))
        };

        // Create window's context
        self.webviews = Some(WebViewManager::new(window.clone()));
        if window.winit_window().is_some() {
            self.minibrowser = Some(Minibrowser::new(
                &rendering_context,
                event_loop.unwrap(),
                self.initial_url.clone(),
            ));
        }

        if let Some(ref mut minibrowser) = self.minibrowser {
            // Servo is not yet initialised, so there is no `servo_framebuffer_id`.
            minibrowser.update(
                window.winit_window().unwrap(),
                self.webviews.as_mut().unwrap(),
                None,
                "init",
            );
            window.set_toolbar_height(minibrowser.toolbar_height);
        }

        self.windows.insert(window.id(), window);

        self.suspended.set(false);
        self.event_queue.push(EmbedderEvent::Idle);
        let (_, window) = self.windows.iter().next().unwrap();

        let xr_discovery = if pref!(dom_webxr_openxr_enabled) && !self.opts.headless {
            #[cfg(target_os = "windows")]
            let openxr = {
                let app_info = AppInfo::new("Servoshell", 0, "Servo", 0);
                Some(XrDiscovery::OpenXr(OpenXrDiscovery::new(None, app_info)))
            };
            #[cfg(not(target_os = "windows"))]
            let openxr = None;

            openxr
        } else if pref!(dom_webxr_glwindow_enabled) && !self.opts.headless {
            let window = window.new_glwindow(event_loop.unwrap());
            Some(XrDiscovery::GlWindow(GlWindowDiscovery::new(window)))
        } else {
            None
        };

        let window = window.clone();
        // Implements embedder methods, used by libservo and constellation.
        let embedder = Box::new(EmbedderCallbacks::new(self.waker.clone(), xr_discovery));

        let composite_target = if self.minibrowser.is_some() {
            CompositeTarget::Fbo
        } else {
            CompositeTarget::Window
        };
        let mut servo = Servo::new(
            self.opts.clone(),
            self.preferences.clone(),
            rendering_context,
            embedder,
            window.clone(),
            self.servo_shell_preferences.user_agent.clone(),
            composite_target,
        );

        servo.handle_events(vec![EmbedderEvent::NewWebView(
            self.initial_url.to_owned(),
            WebViewId::new(),
        )]);
        servo.setup_logging();

        self.servo = Some(servo);
    }

    pub fn is_animating(&self) -> bool {
        self.windows.iter().any(|(_, window)| window.is_animating())
    }

    fn get_events(&mut self) -> Vec<EmbedderEvent> {
        std::mem::take(&mut self.event_queue)
    }

    /// Pumps events and messages between the embedder and Servo, where embedder events flow
    /// towards Servo and embedder messages flow away from Servo, and also runs the compositor.
    ///
    /// As the embedder, we push embedder events through our event queues, from the App queue and
    /// Window queues to the WebViewManager queue, and from the WebViewManager queue to Servo. We
    /// receive and collect embedder messages from the various Servo components, then take them out
    /// of the Servo interface so that the WebViewManager can handle them.
    fn handle_events(&mut self) -> PumpResult {
        let mut embedder_events = self.get_events();
        let webviews = self.webviews.as_mut().unwrap();

        // Take any outstanding embedder events from the App and its Windows.
        for window in self.windows.values() {
            embedder_events.extend(window.get_events());
        }

        // Catch some keyboard events, and push the rest onto the WebViewManager event queue.
        webviews.handle_window_events(embedder_events);

        // If the Gamepad API is enabled, handle gamepad events from GilRs.
        // Checking for focused_webview_id should ensure we'll have a valid browsing context.
        if pref!(dom_gamepad_enabled) && webviews.focused_webview_id().is_some() {
            webviews.handle_gamepad_events();
        }

        // Take any new embedder messages from Servo itself.
        let mut embedder_messages = self.servo.as_mut().unwrap().get_events();
        let mut need_resize = false;
        let mut need_present = false;
        let mut need_update = false;
        loop {
            // Consume and handle those embedder messages.
            let servo_event_response = webviews.handle_servo_events(&self.opts, embedder_messages);
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
                if let Some(ref mut minibrowser) = self.minibrowser {
                    minibrowser.context.destroy();
                }
            },
            PumpResult::Continue { update, present } => {
                if update {
                    if let Some(ref mut minibrowser) = self.minibrowser {
                        let webviews = self.webviews.as_mut().unwrap();
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
                        if let Some(ref mut minibrowser) = self.minibrowser {
                            minibrowser.update(
                                window.winit_window().unwrap(),
                                self.webviews.as_mut().unwrap(),
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
        self.event_queue.push(EmbedderEvent::Idle);

        let mut exit = false;
        match self.handle_events() {
            PumpResult::Shutdown => {
                exit = true;
                self.servo.take().unwrap().deinit();
                if let Some(ref mut minibrowser) = self.minibrowser {
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
}

impl ApplicationHandler<WakerEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init(Some(event_loop));
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
            if let Some(ref mut minibrowser) = self.minibrowser {
                minibrowser.update(
                    window.winit_window().unwrap(),
                    self.webviews.as_mut().unwrap(),
                    self.servo.as_ref().unwrap().offscreen_framebuffer_id(),
                    "RedrawRequested",
                );
                minibrowser.paint(window.winit_window().unwrap());
            }

            self.servo.as_mut().unwrap().present();
        }

        // Handle the event
        let mut consumed = false;
        if let Some(ref mut minibrowser) = self.minibrowser {
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
                            self.webviews.as_mut().unwrap(),
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
                self.event_queue.push(EmbedderEvent::Idle);
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
        if let Some(ref minibrowser) = self.minibrowser {
            let webviews = &mut self.webviews.as_mut().unwrap();
            let app_event_queue = &mut self.event_queue;
            minibrowser.queue_embedder_events_for_minibrowser_events(
                webviews,
                app_event_queue,
                &self.servo_shell_preferences,
            );
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
        self.event_queue.push(EmbedderEvent::Idle);

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
        if let Some(ref minibrowser) = self.minibrowser {
            let webviews = &mut self.webviews.as_mut().unwrap();
            let app_event_queue = &mut self.event_queue;
            minibrowser.queue_embedder_events_for_minibrowser_events(
                webviews,
                app_event_queue,
                &self.servo_shell_preferences,
            );
        }

        self.handle_events_with_winit(event_loop, window);
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.suspended.set(true);
    }
}
