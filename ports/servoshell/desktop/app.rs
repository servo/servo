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
use log::warn;
use net::protocols::ProtocolRegistry;
use servo::config::opts::Opts;
use servo::config::prefs::Preferences;
use servo::servo_url::ServoUrl;
use servo::user_content_manager::{UserContentManager, UserScript};
use servo::{EventLoopWaker, WebDriverCommandMsg, WebDriverUserPromptAction};
use url::Url;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

use super::app_state::AppState;
use super::events_loop::{AppEvent, EventLoopProxy, EventsLoop};
use super::{headed_window, headless_window};
use crate::desktop::app_state::RunningAppState;
use crate::desktop::protocols;
use crate::desktop::tracing::trace_winit_event;
use crate::desktop::window_trait::WindowPortsMethods;
use crate::parser::get_default_url;
use crate::prefs::ServoShellPreferences;
use crate::running_app_state::RunningAppStateTrait;

pub struct App {
    opts: Opts,
    preferences: Preferences,
    servoshell_preferences: ServoShellPreferences,
    suspended: Cell<bool>,
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
        needs_user_interface_update: bool,
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
                let event_loop_proxy = self.proxy.take().expect("Must have a proxy available");
                Rc::new(headed_window::Window::new(
                    &self.servoshell_preferences,
                    event_loop,
                    event_loop_proxy,
                    self.initial_url.clone(),
                ))
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

        let servo_builder = ServoBuilder::default()
            .opts(self.opts.clone())
            .preferences(self.preferences.clone())
            .user_content_manager(user_content_manager)
            .protocol_registry(protocol_registry)
            .event_loop_waker(self.waker.clone());

        #[cfg(feature = "webxr")]
        let servo_builder =
            servo_builder.webxr_registry(super::webxr::XrDiscoveryWebXrRegistry::new_boxed(
                window.clone(),
                event_loop,
                &self.preferences,
            ));

        let servo = servo_builder.build();
        servo.setup_logging();

        // Initialize WebDriver server here before `servo` is moved.
        let webdriver_receiver = self.servoshell_preferences.webdriver_port.map(|port| {
            let (embedder_sender, embedder_receiver) = unbounded();
            webdriver_server::start_server(port, embedder_sender, self.waker.clone());
            embedder_receiver
        });

        let running_state = Rc::new(RunningAppState::new(
            servo,
            window.clone(),
            self.servoshell_preferences.clone(),
            webdriver_receiver,
        ));

        running_state.create_and_focus_toplevel_webview(self.initial_url.clone().into_url());
        window.rebuild_user_interface(&running_state);

        self.state = AppState::Running(running_state);
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
                state.webview_collection_mut().clear();
                self.state = AppState::ShuttingDown;
            },
            PumpResult::Continue { .. } => state.repaint_servo_if_necessary(),
        }

        !matches!(self.state, AppState::ShuttingDown)
    }

    pub(crate) fn handle_webdriver_messages(&self) {
        let AppState::Running(running_state) = &self.state else {
            return;
        };

        let Some(webdriver_receiver) = running_state.webdriver_receiver() else {
            return;
        };

        while let Ok(msg) = webdriver_receiver.try_recv() {
            match msg {
                WebDriverCommandMsg::Shutdown => {
                    running_state.servo().start_shutting_down();
                },
                WebDriverCommandMsg::IsWebViewOpen(webview_id, sender) => {
                    let context = running_state.webview_by_id(webview_id);

                    if let Err(error) = sender.send(context.is_some()) {
                        warn!("Failed to send response of IsWebViewOpen: {error}");
                    }
                },
                WebDriverCommandMsg::IsBrowsingContextOpen(..) => {
                    running_state.servo().execute_webdriver_command(msg);
                },
                WebDriverCommandMsg::NewWebView(response_sender, load_status_sender) => {
                    let new_webview =
                        running_state.create_toplevel_webview(Url::parse("about:blank").unwrap());

                    if let Err(error) = response_sender.send(new_webview.id()) {
                        warn!("Failed to send response of NewWebview: {error}");
                    }
                    if let Some(load_status_sender) = load_status_sender {
                        running_state.set_load_status_sender(new_webview.id(), load_status_sender);
                    }
                },
                WebDriverCommandMsg::CloseWebView(webview_id, response_sender) => {
                    running_state.close_webview(webview_id);
                    if let Err(error) = response_sender.send(()) {
                        warn!("Failed to send response of CloseWebView: {error}");
                    }
                },
                WebDriverCommandMsg::FocusWebView(webview_id) => {
                    if let Some(webview) = running_state.webview_by_id(webview_id) {
                        webview.focus_and_raise_to_top(true);
                    }
                },
                WebDriverCommandMsg::FocusBrowsingContext(..) => {
                    running_state.servo().execute_webdriver_command(msg);
                },
                WebDriverCommandMsg::GetAllWebViews(response_sender) => {
                    let webviews = running_state.webviews().iter().map(|(id, _)| *id).collect();

                    if let Err(error) = response_sender.send(webviews) {
                        warn!("Failed to send response of GetAllWebViews: {error}");
                    }
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
                WebDriverCommandMsg::MaximizeWebView(webview_id, response_sender) => {
                    let window = self
                        .windows
                        .values()
                        .next()
                        .expect("Should have at least one window in servoshell");
                    window.maximize(
                        &running_state
                            .webview_by_id(webview_id)
                            .expect("Webview must exists as we just verified"),
                    );

                    if let Err(error) = response_sender.send(window.window_rect()) {
                        warn!("Failed to send response of GetWindowSize: {error}");
                    }
                },
                WebDriverCommandMsg::SetWindowRect(webview_id, requested_rect, size_sender) => {
                    let Some(webview) = running_state.webview_by_id(webview_id) else {
                        continue;
                    };

                    let window = self
                        .windows
                        .values()
                        .next()
                        .expect("Should have at least one window in servoshell");
                    let scale = window.hidpi_scale_factor();

                    let requested_physical_rect =
                        (requested_rect.to_f32() * scale).round().to_i32();

                    // Step 17. Set Width/Height.
                    window.request_resize(&webview, requested_physical_rect.size());

                    // Step 18. Set position of the window.
                    window.set_position(requested_physical_rect.min);

                    if let Err(error) = size_sender.send(window.window_rect()) {
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
                // This is only received when start new session.
                WebDriverCommandMsg::GetFocusedWebView(sender) => {
                    let focused_webview = running_state.focused_webview();

                    if let Err(error) = sender.send(focused_webview.map(|w| w.id())) {
                        warn!("Failed to send response of GetFocusedWebView: {error}");
                    };
                },
                WebDriverCommandMsg::LoadUrl(webview_id, url, load_status_sender) => {
                    running_state.handle_webdriver_load_url(webview_id, url, load_status_sender);
                },
                WebDriverCommandMsg::Refresh(webview_id, load_status_sender) => {
                    if let Some(webview) = running_state.webview_by_id(webview_id) {
                        running_state.set_load_status_sender(webview_id, load_status_sender);
                        webview.reload();
                    }
                },
                WebDriverCommandMsg::GoBack(webview_id, load_status_sender) => {
                    if let Some(webview) = running_state.webview_by_id(webview_id) {
                        let traversal_id = webview.go_back(1);
                        running_state.set_pending_traversal(traversal_id, load_status_sender);
                    }
                },
                WebDriverCommandMsg::GoForward(webview_id, load_status_sender) => {
                    if let Some(webview) = running_state.webview_by_id(webview_id) {
                        let traversal_id = webview.go_forward(1);
                        running_state.set_pending_traversal(traversal_id, load_status_sender);
                    }
                },
                WebDriverCommandMsg::InputEvent(webview_id, input_event, response_sender) => {
                    running_state.handle_webdriver_input_event(
                        webview_id,
                        input_event,
                        response_sender,
                    );
                },
                WebDriverCommandMsg::ScriptCommand(_, ref webdriver_script_command) => {
                    running_state.handle_webdriver_script_command(webdriver_script_command);
                    running_state.servo().execute_webdriver_command(msg);
                },
                WebDriverCommandMsg::CurrentUserPrompt(webview_id, response_sender) => {
                    let current_dialog =
                        running_state.get_current_active_dialog_webdriver_type(webview_id);
                    if let Err(error) = response_sender.send(current_dialog) {
                        warn!("Failed to send response of CurrentUserPrompt: {error}");
                    };
                },
                WebDriverCommandMsg::HandleUserPrompt(webview_id, action, response_sender) => {
                    let response = if running_state.webview_has_active_dialog(webview_id) {
                        let alert_text = running_state.alert_text_of_newest_dialog(webview_id);

                        match action {
                            WebDriverUserPromptAction::Accept => {
                                running_state.accept_active_dialogs(webview_id)
                            },
                            WebDriverUserPromptAction::Dismiss => {
                                running_state.dismiss_active_dialogs(webview_id)
                            },
                            WebDriverUserPromptAction::Ignore => {},
                        };

                        // Return success for AcceptAlert and DismissAlert commands.
                        Ok(alert_text)
                    } else {
                        // Return error for AcceptAlert and DismissAlert commands
                        // if there is no active dialog.
                        Err(())
                    };

                    if let Err(error) = response_sender.send(response) {
                        warn!("Failed to send response of HandleUserPrompt: {error}");
                    };
                },
                WebDriverCommandMsg::GetAlertText(webview_id, response_sender) => {
                    let response = match running_state.alert_text_of_newest_dialog(webview_id) {
                        Some(text) => Ok(text),
                        None => Err(()),
                    };

                    if let Err(error) = response_sender.send(response) {
                        warn!("Failed to send response of GetAlertText: {error}");
                    };
                },
                WebDriverCommandMsg::SendAlertText(webview_id, text) => {
                    running_state.set_alert_text_of_newest_dialog(webview_id, text);
                },
                WebDriverCommandMsg::TakeScreenshot(webview_id, rect, result_sender) => {
                    running_state.handle_webdriver_screenshot(webview_id, rect, result_sender);
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
        window_event: WindowEvent,
    ) {
        let now = Instant::now();
        trace_winit_event!(
            window_event,
            "@{:?} (+{:?}) {window_event:?}",
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

        self.handle_webdriver_messages();
        if !window.handle_winit_window_event(state.clone(), window_event) {
            event_loop.exit();
            self.state = AppState::ShuttingDown;
        }

        // Block until the window gets an event
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, app_event: AppEvent) {
        let AppState::Running(state) = &self.state else {
            return;
        };
        let Some(window) = self.windows.values().next() else {
            return;
        };

        self.handle_webdriver_messages();
        if !window.handle_winit_app_event(state.clone(), app_event) {
            event_loop.exit();
            self.state = AppState::ShuttingDown;
        }

        // Block until the window gets an event
        event_loop.set_control_flow(ControlFlow::Wait);
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
        files.sort_unstable();
        for file in files {
            userscripts.push(UserScript {
                script: std::fs::read_to_string(&file)?,
                source_file: Some(file),
            });
        }
    }
    Ok(userscripts)
}
