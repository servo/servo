/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application entry point, runs the event loop.

use std::cell::Cell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use std::{env, fs};

use ::servo::ServoBuilder;
use crossbeam_channel::unbounded;
use log::{info, trace, warn};
use net::protocols::ProtocolRegistry;
use servo::config::opts::Opts;
use servo::config::prefs::Preferences;
use servo::servo_url::ServoUrl;
use servo::user_content_manager::{UserContentManager, UserScript};
use servo::{EventLoopWaker, WebDriverCommandMsg};
use url::Url;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use super::app_state::AppState;
use super::events_loop::{AppEvent, EventLoopProxy, EventsLoop};
use super::minibrowser::{Minibrowser, MinibrowserEvent};
use super::{headed_window, headless_window};
use crate::desktop::app_state::RunningAppState;
use crate::desktop::protocols;
use crate::desktop::tracing::trace_winit_event;
use crate::desktop::webxr::XrDiscoveryWebXrRegistry;
use crate::desktop::window_trait::WindowPortsMethods;
use crate::parser::{get_default_url, location_bar_input_to_url};
use crate::prefs::ServoShellPreferences;

pub struct App {
    opts: Opts,
    preferences: Preferences,
    servoshell_preferences: ServoShellPreferences,
    suspended: Cell<bool>,
    minibrowser: Option<Minibrowser>,
    waker: Box<dyn EventLoopWaker>,
    proxy: Option<EventLoopProxy>,
    initial_url: ServoUrl,
    t_start: Instant,
    t: Instant,
    state: AppState,

    // This is the last field of the struct to ensure that windows are dropped *after* all other
    // references to the relevant rendering contexts have been destroyed.
    // (https://github.com/servo/servo/issues/36711)
    windows: HashMap<WindowId, Rc<dyn WindowPortsMethods>>,
}

/// Action to be taken by the caller of [`App::handle_events`].
pub(crate) enum PumpResult {
    /// The caller should shut down Servo and its related context.
    Shutdown,
    Continue {
        need_update: bool,
        need_window_redraw: bool,
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
            servoshell_preferences: servo_shell_preferences,
            suspended: Cell::new(false),
            windows: HashMap::new(),
            minibrowser: None,
            waker: events_loop.create_event_loop_waker(),
            proxy: events_loop.event_loop_proxy(),
            initial_url: initial_url.clone(),
            t_start: t,
            t,
            state: AppState::Initializing,
        }
    }

    /// Initialize Application once event loop start running.
    pub fn init(&mut self, event_loop: Option<&ActiveEventLoop>) {
        let headless = self.servoshell_preferences.headless;

        assert_eq!(headless, event_loop.is_none());
        let window = match event_loop {
            Some(event_loop) => {
                let proxy = self.proxy.take().expect("Must have a proxy available");
                let window = headed_window::Window::new(&self.servoshell_preferences, event_loop);
                self.minibrowser = Some(Minibrowser::new(
                    &window,
                    event_loop,
                    proxy,
                    self.initial_url.clone(),
                ));
                Rc::new(window)
            },
            None => headless_window::Window::new(&self.servoshell_preferences),
        };

        self.windows.insert(window.id(), window);

        self.suspended.set(false);
        let (_, window) = self.windows.iter().next().unwrap();

        let mut user_content_manager = UserContentManager::new();
        for script in load_userscripts(self.servoshell_preferences.userscripts_directory.as_deref())
            .expect("Loading userscripts failed")
        {
            user_content_manager.add_script(script);
        }

        let mut protocol_registry = ProtocolRegistry::default();
        let _ = protocol_registry.register(
            "urlinfo",
            protocols::urlinfo::UrlInfoProtocolHander::default(),
        );
        let _ =
            protocol_registry.register("servo", protocols::servo::ServoProtocolHandler::default());
        let _ = protocol_registry.register(
            "resource",
            protocols::resource::ResourceProtocolHandler::default(),
        );

        let servo_builder = ServoBuilder::new(window.rendering_context())
            .opts(self.opts.clone())
            .preferences(self.preferences.clone())
            .user_content_manager(user_content_manager)
            .protocol_registry(protocol_registry)
            .event_loop_waker(self.waker.clone());

        #[cfg(feature = "webxr")]
        let servo_builder = servo_builder.webxr_registry(XrDiscoveryWebXrRegistry::new_boxed(
            window.clone(),
            event_loop,
            &self.preferences,
        ));

        let servo = servo_builder.build();
        servo.setup_logging();

        // Initialize WebDriver server here before `servo` is moved.
        let webdriver_receiver = self.servoshell_preferences.webdriver_port.map(|port| {
            let (embedder_sender, embedder_receiver) = unbounded();

            // TODO: WebDriver will no longer need this channel once all WebDriver
            // commands are executed via the Servo API.
            let constellation_sender_deprecated = servo.constellation_sender();
            webdriver_server::start_server(
                port,
                constellation_sender_deprecated,
                embedder_sender,
                self.waker.clone(),
            );

            embedder_receiver
        });

        let running_state = Rc::new(RunningAppState::new(
            servo,
            window.clone(),
            self.servoshell_preferences.clone(),
            webdriver_receiver,
        ));
        running_state.create_and_focus_toplevel_webview(self.initial_url.clone().into_url());

        if let Some(ref mut minibrowser) = self.minibrowser {
            minibrowser.update(window.winit_window().unwrap(), &running_state, "init");
            window.set_toolbar_height(minibrowser.toolbar_height);
        }

        self.state = AppState::Running(running_state);
    }

    pub(crate) fn animating(&self) -> bool {
        match self.state {
            AppState::Initializing => false,
            AppState::Running(ref running_app_state) => running_app_state.servo().animating(),
            AppState::ShuttingDown => false,
        }
    }

    /// Handle events with winit contexts
    pub fn handle_events_with_winit(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: Rc<dyn WindowPortsMethods>,
    ) {
        let AppState::Running(state) = &self.state else {
            return;
        };

        match state.pump_event_loop() {
            PumpResult::Shutdown => {
                state.shutdown();
                self.state = AppState::ShuttingDown;
            },
            PumpResult::Continue {
                need_update: update,
                need_window_redraw,
            } => {
                let updated = match (update, &mut self.minibrowser) {
                    (true, Some(minibrowser)) => minibrowser.update_webview_data(state),
                    _ => false,
                };

                // If in headed mode, request a winit redraw event, so we can paint the minibrowser.
                if updated || need_window_redraw {
                    if let Some(window) = window.winit_window() {
                        window.request_redraw();
                    }
                }
            },
        }

        if matches!(self.state, AppState::ShuttingDown) {
            event_loop.exit();
        }
    }

    /// Handle all servo events with headless mode. Return true if the application should
    /// continue.
    pub fn handle_events_with_headless(&mut self) -> bool {
        let now = Instant::now();
        let event = winit::event::Event::UserEvent(AppEvent::Waker);
        trace_winit_event!(
            event,
            "@{:?} (+{:?}) {event:?}",
            now - self.t_start,
            now - self.t
        );
        self.t = now;

        // We should always be in the running state.
        let AppState::Running(state) = &self.state else {
            return false;
        };

        match state.pump_event_loop() {
            PumpResult::Shutdown => {
                state.shutdown();
                self.state = AppState::ShuttingDown;
            },
            PumpResult::Continue { .. } => state.repaint_servo_if_necessary(),
        }

        !matches!(self.state, AppState::ShuttingDown)
    }

    /// Takes any events generated during `egui` updates and performs their actions.
    fn handle_servoshell_ui_events(&mut self) {
        let Some(minibrowser) = self.minibrowser.as_ref() else {
            return;
        };
        // We should always be in the running state.
        let AppState::Running(state) = &self.state else {
            return;
        };

        for event in minibrowser.take_events() {
            match event {
                MinibrowserEvent::Go(location) => {
                    minibrowser.update_location_dirty(false);
                    let Some(url) = location_bar_input_to_url(
                        &location.clone(),
                        &self.servoshell_preferences.searchpage,
                    ) else {
                        warn!("failed to parse location");
                        break;
                    };
                    if let Some(focused_webview) = state.focused_webview() {
                        focused_webview.load(url.into_url());
                    }
                },
                MinibrowserEvent::Back => {
                    if let Some(focused_webview) = state.focused_webview() {
                        focused_webview.go_back(1);
                    }
                },
                MinibrowserEvent::Forward => {
                    if let Some(focused_webview) = state.focused_webview() {
                        focused_webview.go_forward(1);
                    }
                },
                MinibrowserEvent::Reload => {
                    minibrowser.update_location_dirty(false);
                    if let Some(focused_webview) = state.focused_webview() {
                        focused_webview.reload();
                    }
                },
                MinibrowserEvent::NewWebView => {
                    minibrowser.update_location_dirty(false);
                    state.create_and_focus_toplevel_webview(Url::parse("servo:newtab").unwrap());
                },
                MinibrowserEvent::CloseWebView(id) => {
                    minibrowser.update_location_dirty(false);
                    state.close_webview(id);
                },
            }
        }
    }

    pub fn handle_webdriver_messages(&self) {
        let AppState::Running(running_state) = &self.state else {
            return;
        };

        let Some(webdriver_receiver) = running_state.webdriver_receiver() else {
            return;
        };

        while let Ok(msg) = webdriver_receiver.try_recv() {
            match msg {
                WebDriverCommandMsg::IsWebViewOpen(webview_id, sender) => {
                    let context = running_state.webview_by_id(webview_id);

                    if let Err(error) = sender.send(context.is_some()) {
                        warn!("Failed to send response of IsWebViewOpein: {error}");
                    }
                },
                webdriver_msg @ WebDriverCommandMsg::IsBrowsingContextOpen(..) => {
                    running_state.forward_webdriver_command(webdriver_msg);
                },
                WebDriverCommandMsg::NewWebView(response_sender, load_status_sender) => {
                    let new_webview =
                        running_state.create_toplevel_webview(Url::parse("about:blank").unwrap());

                    if let Err(error) = response_sender.send(new_webview.id()) {
                        warn!("Failed to send response of NewWebview: {error}");
                    }

                    running_state.set_load_status_sender(new_webview.id(), load_status_sender);
                },
                WebDriverCommandMsg::CloseWebView(webview_id) => {
                    running_state.close_webview(webview_id);
                },
                WebDriverCommandMsg::FocusWebView(webview_id) => {
                    if let Some(webview) = running_state.webview_by_id(webview_id) {
                        webview.focus();
                    }

                    // TODO: send a response to the WebDriver
                    // so it knows when the focus has finished.
                },
                WebDriverCommandMsg::GetWindowRect(_webview_id, response_sender) => {
                    let window = self
                        .windows
                        .values()
                        .next()
                        .expect("Should have at least one window in servoshell");

                    if let Err(error) = response_sender.send(window.window_rect()) {
                        warn!("Failed to send response of GetWindowSize: {error}");
                    }
                },
                WebDriverCommandMsg::SetWindowSize(webview_id, requested_size, size_sender) => {
                    let Some(webview) = running_state.webview_by_id(webview_id) else {
                        continue;
                    };

                    let window = self
                        .windows
                        .values()
                        .next()
                        .expect("Should have at least one window in servoshell");

                    // When None is returned, it means that the request went to the display system,
                    // and the actual size will be delivered later with the WindowEvent::Resized.
                    let returned_size = window.request_resize(&webview, requested_size);
                    // TODO: Handle None case. For now, we assume always succeed.
                    // In reality, the request may exceed available screen size.
                    if let Err(error) = size_sender.send(returned_size.unwrap_or(requested_size)) {
                        warn!("Failed to send window size: {error}");
                    }
                },
                WebDriverCommandMsg::GetViewportSize(_webview_id, response_sender) => {
                    let window = self
                        .windows
                        .values()
                        .next()
                        .expect("Should have at least one window in servoshell");

                    let size = window.rendering_context().size2d();

                    if let Err(error) = response_sender.send(size) {
                        warn!("Failed to send response of GetViewportSize: {error}");
                    }
                },
                WebDriverCommandMsg::GetFocusedWebView(sender) => {
                    let focused_webview = running_state.focused_webview();
                    if let Err(error) = sender.send(focused_webview.map(|w| w.id())) {
                        warn!("Failed to send response of GetFocusedWebView: {error}");
                    };
                },
                WebDriverCommandMsg::LoadUrl(webview_id, url, load_status_sender) => {
                    if let Some(webview) = running_state.webview_by_id(webview_id) {
                        webview.load(url.into_url());
                        running_state.set_load_status_sender(webview_id, load_status_sender);
                    }
                },
                WebDriverCommandMsg::Refresh(webview_id, load_status_sender) => {
                    if let Some(webview) = running_state.webview_by_id(webview_id) {
                        webview.reload();
                        running_state.set_load_status_sender(webview_id, load_status_sender);
                    }
                },
                WebDriverCommandMsg::GoBack(webview_id) => {
                    if let Some(webview) = running_state.webview_by_id(webview_id) {
                        webview.go_back(1);
                    }
                },
                WebDriverCommandMsg::GoForward(webview_id) => {
                    if let Some(webview) = running_state.webview_by_id(webview_id) {
                        webview.go_forward(1);
                    }
                },
                WebDriverCommandMsg::SendKeys(..) |
                WebDriverCommandMsg::KeyboardAction(..) |
                WebDriverCommandMsg::MouseButtonAction(..) |
                WebDriverCommandMsg::MouseMoveAction(..) |
                WebDriverCommandMsg::WheelScrollAction(..) |
                WebDriverCommandMsg::ScriptCommand(..) |
                WebDriverCommandMsg::TakeScreenshot(..) => {
                    warn!(
                        "WebDriverCommand {:?} is still not moved from constellation to embedder",
                        msg
                    );
                },
            };
        }
    }
}

impl ApplicationHandler<AppEvent> for App {
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

        let AppState::Running(state) = &self.state else {
            return;
        };

        let Some(window) = self.windows.get(&window_id) else {
            return;
        };

        let window = window.clone();
        if event == WindowEvent::RedrawRequested {
            // We need to redraw the window for some reason.
            trace!("RedrawRequested");

            // WARNING: do not defer painting or presenting to some later tick of the event
            // loop or servoshell may become unresponsive! (servo#30312)
            if let Some(ref mut minibrowser) = self.minibrowser {
                minibrowser.update(window.winit_window().unwrap(), state, "RedrawRequested");
                minibrowser.paint(window.winit_window().unwrap());
            }
        }

        // Handle the event
        let mut consumed = false;
        if let Some(ref mut minibrowser) = self.minibrowser {
            match event {
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    // Intercept any ScaleFactorChanged events away from EguiGlow::on_window_event, so
                    // we can use our own logic for calculating the scale factor and set egui’s
                    // scale factor to that value manually.
                    let desired_scale_factor = window.hidpi_scale_factor().get();
                    let effective_egui_zoom_factor = desired_scale_factor / scale_factor as f32;

                    info!(
                        "window scale factor changed to {}, setting egui zoom factor to {}",
                        scale_factor, effective_egui_zoom_factor
                    );

                    minibrowser
                        .context
                        .egui_ctx
                        .set_zoom_factor(effective_egui_zoom_factor);

                    state.hidpi_scale_factor_changed();

                    // Request a winit redraw event, so we can recomposite, update and paint
                    // the minibrowser, and present the new frame.
                    window.winit_window().unwrap().request_redraw();
                },
                ref event => {
                    let response =
                        minibrowser.on_window_event(window.winit_window().unwrap(), state, event);
                    // Update minibrowser if there's resize event to sync up with window.
                    if let WindowEvent::Resized(_) = event {
                        minibrowser.update(
                            window.winit_window().unwrap(),
                            state,
                            "Sync WebView size with Window Resize event",
                        );
                    }
                    if response.repaint && *event != WindowEvent::RedrawRequested {
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
            window.handle_winit_event(state.clone(), event);
        }

        // Block until the window gets an event
        if !self.animating() || self.suspended.get() {
            event_loop.set_control_flow(ControlFlow::Wait);
        } else {
            event_loop.set_control_flow(ControlFlow::Poll);
        }

        // Consume and handle any events from the servoshell UI.
        self.handle_servoshell_ui_events();

        self.handle_events_with_winit(event_loop, window);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        if let AppEvent::Accessibility(ref event) = event {
            let Some(ref mut minibrowser) = self.minibrowser else {
                return;
            };
            if !minibrowser.handle_accesskit_event(&event.window_event) {
                return;
            }
            if let Some(window) = self.windows.get(&event.window_id) {
                window.winit_window().unwrap().request_redraw();
            }
            return;
        }

        let now = Instant::now();
        let event = winit::event::Event::UserEvent(event);
        trace_winit_event!(
            event,
            "@{:?} (+{:?}) {event:?}",
            now - self.t_start,
            now - self.t
        );
        self.t = now;

        if !matches!(self.state, AppState::Running(..)) {
            return;
        };
        let Some(window) = self.windows.values().next() else {
            return;
        };
        let window = window.clone();

        // Block until the window gets an event
        if !self.animating() || self.suspended.get() {
            event_loop.set_control_flow(ControlFlow::Wait);
        } else {
            event_loop.set_control_flow(ControlFlow::Poll);
        }

        // Consume and handle any events from the Minibrowser.
        self.handle_servoshell_ui_events();

        // Consume and handle any events from the WebDriver.
        self.handle_webdriver_messages();

        self.handle_events_with_winit(event_loop, window);
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.suspended.set(true);
    }
}

fn load_userscripts(userscripts_directory: Option<&Path>) -> std::io::Result<Vec<UserScript>> {
    let mut userscripts = Vec::new();
    if let Some(userscripts_directory) = &userscripts_directory {
        let mut files = std::fs::read_dir(userscripts_directory)?
            .map(|e| e.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        files.sort();
        for file in files {
            userscripts.push(UserScript {
                script: std::fs::read_to_string(&file)?,
                source_file: Some(file),
            });
        }
    }
    Ok(userscripts)
}
