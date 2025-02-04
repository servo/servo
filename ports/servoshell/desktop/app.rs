/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application entry point, runs the event loop.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;
use std::{env, fs};

use arboard::Clipboard;
use log::{info, trace, warn};
use raw_window_handle::HasDisplayHandle;
use servo::compositing::windowing::{AnimationState, WindowMethods};
use servo::compositing::CompositeTarget;
use servo::config::opts::Opts;
use servo::config::prefs::Preferences;
use servo::servo_config::pref;
use servo::servo_url::ServoUrl;
use servo::webrender_traits::SurfmanRenderingContext;
use servo::webxr::glwindow::GlWindowDiscovery;
#[cfg(target_os = "windows")]
use servo::webxr::openxr::{AppInfo, OpenXrDiscovery};
use servo::{EventLoopWaker, Servo};
use surfman::Connection;
use url::Url;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use super::events_loop::{EventsLoop, WakerEvent};
use super::minibrowser::{Minibrowser, MinibrowserEvent};
use super::webview::WebViewManager;
use super::{headed_window, headless_window};
use crate::desktop::embedder::{EmbedderCallbacks, XrDiscovery};
use crate::desktop::tracing::trace_winit_event;
use crate::desktop::window_trait::WindowPortsMethods;
use crate::parser::{get_default_url, location_bar_input_to_url};
use crate::prefs::ServoShellPreferences;

pub struct App {
    opts: Opts,
    preferences: Preferences,
    servo_shell_preferences: ServoShellPreferences,
    clipboard: Option<Clipboard>,
    servo: Option<Servo>,
    webviews: WebViewManager,
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
            clipboard: Clipboard::new().ok(),
            webviews: WebViewManager::new(),
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
            SurfmanRenderingContext::create(
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
            SurfmanRenderingContext::create(&connection, &adapter, None)
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
        self.webviews.set_window(window.clone());
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
                &mut self.webviews,
                self.servo.as_ref(),
                "init",
            );
            window.set_toolbar_height(minibrowser.toolbar_height);
        }

        self.windows.insert(window.id(), window);

        self.suspended.set(false);
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

        // TODO: Remove this once dyn upcasting coercion stabilises
        // <https://github.com/rust-lang/rust/issues/65991>
        struct UpcastedWindow(Rc<dyn WindowPortsMethods>);
        impl WindowMethods for UpcastedWindow {
            fn get_coordinates(&self) -> servo::compositing::windowing::EmbedderCoordinates {
                self.0.get_coordinates()
            }
            fn set_animation_state(&self, state: AnimationState) {
                self.0.set_animation_state(state);
            }
        }
        let window = UpcastedWindow(window.clone());
        // Implements embedder methods, used by libservo and constellation.
        let embedder = Box::new(EmbedderCallbacks::new(self.waker.clone(), xr_discovery));

        let composite_target = if self.minibrowser.is_some() {
            CompositeTarget::Fbo
        } else {
            CompositeTarget::Window
        };
        let servo = Servo::new(
            self.opts.clone(),
            self.preferences.clone(),
            Rc::new(rendering_context),
            embedder,
            Rc::new(window),
            self.servo_shell_preferences.user_agent.clone(),
            composite_target,
        );

        servo.setup_logging();

        let webview = servo.new_webview(self.initial_url.clone().into_url());
        self.webviews.add(webview);

        self.servo = Some(servo);
    }

    pub fn is_animating(&self) -> bool {
        self.windows.iter().any(|(_, window)| window.is_animating())
    }

    /// Spins the Servo event loop, and (for now) handles a few other tasks:
    /// - Notifying Servo about incoming gamepad events
    /// - Receiving updates from Servo
    /// - Performing updates in the compositor, such as queued pinch zoom events
    ///
    /// In the future, these tasks may be decoupled.
    fn handle_events(&mut self) -> PumpResult {
        // If the Gamepad API is enabled, handle gamepad events from GilRs.
        // Checking for focused_webview_id should ensure we'll have a valid browsing context.
        if pref!(dom_gamepad_enabled) && self.webviews.focused_webview_id().is_some() {
            self.webviews.handle_gamepad_events();
        }

        // Take any new embedder messages from Servo.
        let servo = self.servo.as_mut().expect("Servo should be running.");
        let mut embedder_messages: Vec<_> = servo.get_events().collect();
        let mut need_resize = false;
        let mut need_present = false;
        let mut need_update = false;
        loop {
            // Consume and handle those embedder messages.
            let servo_event_response = self.webviews.handle_servo_events(
                servo,
                &mut self.clipboard,
                &self.opts,
                embedder_messages,
            );
            need_present |= servo_event_response.need_present;
            need_update |= servo_event_response.need_update;

            // Runs the compositor, and receives and collects embedder messages from various Servo components.
            need_resize |= servo.handle_events(vec![]);
            if self.webviews.shutdown_requested() {
                return PumpResult::Shutdown;
            }

            // Take any new embedder messages from Servo itself.
            embedder_messages = servo.get_events().collect();
            if embedder_messages.is_empty() {
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
                        if minibrowser.update_webview_data(&mut self.webviews) {
                            // Update the minibrowser immediately. While we could update by requesting a
                            // redraw, doing so would delay the location update by two frames.
                            minibrowser.update(
                                window.winit_window().unwrap(),
                                &mut self.webviews,
                                self.servo.as_ref(),
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
                                &mut self.webviews,
                                self.servo.as_ref(),
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

    /// Takes any events generated during `egui` updates and performs their actions.
    fn handle_servoshell_ui_events(&mut self) {
        let Some(minibrowser) = self.minibrowser.as_ref() else {
            return;
        };
        let Some(servo) = self.servo.as_ref() else {
            return;
        };

        for event in minibrowser.take_events() {
            match event {
                MinibrowserEvent::Go(location) => {
                    minibrowser.update_location_dirty(false);
                    let Some(url) = location_bar_input_to_url(
                        &location.clone(),
                        &self.servo_shell_preferences.searchpage,
                    ) else {
                        warn!("failed to parse location");
                        break;
                    };
                    if let Some(focused_webview) = self.webviews.focused_webview() {
                        focused_webview.servo_webview.load(url.into_url());
                    }
                },
                MinibrowserEvent::Back => {
                    if let Some(focused_webview) = self.webviews.focused_webview() {
                        focused_webview.servo_webview.go_back(1);
                    }
                },
                MinibrowserEvent::Forward => {
                    if let Some(focused_webview) = self.webviews.focused_webview() {
                        focused_webview.servo_webview.go_forward(1);
                    }
                },
                MinibrowserEvent::Reload => {
                    minibrowser.update_location_dirty(false);
                    if let Some(focused_webview) = self.webviews.focused_webview() {
                        focused_webview.servo_webview.reload();
                    }
                },
                MinibrowserEvent::NewWebView => {
                    minibrowser.update_location_dirty(false);
                    let webview = servo.new_webview(Url::parse("servo:newtab").unwrap());
                    self.webviews.add(webview);
                },
                MinibrowserEvent::CloseWebView(id) => {
                    minibrowser.update_location_dirty(false);
                    self.webviews.close_webview(servo, id);
                },
            }
        }
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
        let Some(ref mut servo) = self.servo else {
            return;
        };

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
                    &mut self.webviews,
                    Some(servo),
                    "RedrawRequested",
                );
                minibrowser.paint(window.winit_window().unwrap());
            }

            servo.present();
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
                            &mut self.webviews,
                            Some(servo),
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
            window.handle_winit_event(servo, &mut self.clipboard, &mut self.webviews, event);
        }

        let animating = self.is_animating();

        // Block until the window gets an event
        if !animating || self.suspended.get() {
            event_loop.set_control_flow(ControlFlow::Wait);
        } else {
            event_loop.set_control_flow(ControlFlow::Poll);
        }

        // Consume and handle any events from the servoshell UI.
        self.handle_servoshell_ui_events();

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
        self.handle_servoshell_ui_events();

        self.handle_events_with_winit(event_loop, window);
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.suspended.set(true);
    }
}
