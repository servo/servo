/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application entry point, runs the event loop.

use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use std::{env, fs};

use log::warn;
use servo::protocol_handler::ProtocolRegistry;
use servo::{
    EventLoopWaker, Opts, PrefValue, Preferences, ServoBuilder, ServoUrl, UserContentManager,
    UserScript,
};
use url::Url;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::window::WindowId;

use super::event_loop::AppEvent;
use super::{headed_window, headless_window};
use crate::desktop::event_loop::ServoShellEventLoop;
use crate::desktop::protocols;
use crate::desktop::tracing::trace_winit_event;
use crate::parser::{get_default_url, location_bar_input_to_url};
use crate::prefs::ServoShellPreferences;
use crate::running_app_state::{RunningAppState, UserInterfaceCommand};
use crate::window::{PlatformWindow, ServoShellWindow};

pub(crate) enum AppState {
    Initializing,
    Running(Rc<RunningAppState>),
    ShuttingDown,
}

pub struct App {
    opts: Opts,
    preferences: Preferences,
    servoshell_preferences: ServoShellPreferences,
    waker: Box<dyn EventLoopWaker>,
    event_loop_proxy: Option<EventLoopProxy<AppEvent>>,
    initial_url: ServoUrl,
    t_start: Instant,
    t: Instant,
    state: AppState,
}

impl App {
    pub fn new(
        opts: Opts,
        preferences: Preferences,
        servo_shell_preferences: ServoShellPreferences,
        event_loop: &ServoShellEventLoop,
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
            waker: event_loop.create_event_loop_waker(),
            event_loop_proxy: event_loop.event_loop_proxy(),
            initial_url: initial_url.clone(),
            t_start: t,
            t,
            state: AppState::Initializing,
        }
    }

    /// Initialize Application once event loop start running.
    pub fn init(&mut self, active_event_loop: Option<&ActiveEventLoop>) {
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
        if let PrefValue::Str(mailto_handler) = self.preferences.get_value("mailto_handler") {
            let _ = protocol_registry
                .register_page_content_handler("mailto".to_owned(), mailto_handler);
        }

        let servo_builder = ServoBuilder::default()
            .opts(self.opts.clone())
            .preferences(self.preferences.clone())
            .user_content_manager(user_content_manager)
            .protocol_registry(protocol_registry)
            .event_loop_waker(self.waker.clone());

        let url = self.initial_url.as_url().clone();
        let platform_window = self.create_platform_window(url, active_event_loop);

        #[cfg(feature = "webxr")]
        let servo_builder =
            servo_builder.webxr_registry(super::webxr::XrDiscoveryWebXrRegistry::new_boxed(
                platform_window.clone(),
                active_event_loop,
                &self.preferences,
            ));

        let servo = servo_builder.build();
        servo.setup_logging();

        let running_state = Rc::new(RunningAppState::new(
            servo,
            self.servoshell_preferences.clone(),
            self.waker.clone(),
        ));
        running_state.open_window(platform_window, self.initial_url.as_url().clone());

        self.state = AppState::Running(running_state);
    }

    fn create_platform_window(
        &self,
        url: Url,
        active_event_loop: Option<&ActiveEventLoop>,
    ) -> Rc<dyn PlatformWindow> {
        assert_eq!(
            self.servoshell_preferences.headless,
            active_event_loop.is_none()
        );

        let Some(active_event_loop) = active_event_loop else {
            return headless_window::Window::new(&self.servoshell_preferences);
        };

        headed_window::Window::new(
            &self.servoshell_preferences,
            active_event_loop,
            self.event_loop_proxy
                .clone()
                .expect("Should always have event loop proxy in headed mode."),
            url,
        )
    }

    pub fn pump_servo_event_loop(&mut self, active_event_loop: Option<&ActiveEventLoop>) -> bool {
        let AppState::Running(state) = &self.state else {
            return false;
        };

        state.foreach_window_and_interface_commands(|window, commands| {
            self.handle_interface_commands_for_window(active_event_loop, state, window, commands);
        });

        if !state.spin_event_loop() {
            self.state = AppState::ShuttingDown;
            return false;
        }
        true
    }

    /// Takes any events generated during `egui` updates and performs their actions.
    fn handle_interface_commands_for_window(
        &self,
        active_event_loop: Option<&ActiveEventLoop>,
        state: &Rc<RunningAppState>,
        window: &ServoShellWindow,
        commands: Vec<UserInterfaceCommand>,
    ) {
        for event in commands {
            match event {
                UserInterfaceCommand::Go(location) => {
                    window.set_needs_update();
                    let Some(url) = location_bar_input_to_url(
                        &location.clone(),
                        &state.servoshell_preferences.searchpage,
                    ) else {
                        warn!("failed to parse location");
                        break;
                    };
                    if let Some(active_webview) = window.active_webview() {
                        active_webview.load(url.into_url());
                    }
                },
                UserInterfaceCommand::Back => {
                    if let Some(active_webview) = window.active_webview() {
                        active_webview.go_back(1);
                    }
                },
                UserInterfaceCommand::Forward => {
                    if let Some(active_webview) = window.active_webview() {
                        active_webview.go_forward(1);
                    }
                },
                UserInterfaceCommand::Reload => {
                    window.set_needs_update();
                    if let Some(active_webview) = window.active_webview() {
                        active_webview.reload();
                    }
                },
                UserInterfaceCommand::ReloadAll => {
                    for window in state.windows().values() {
                        window.set_needs_update();
                        for (_, webview) in window.webviews() {
                            webview.reload();
                        }
                    }
                },
                UserInterfaceCommand::NewWebView => {
                    window.set_needs_update();
                    let url = Url::parse("servo:newtab").expect("Should always be able to parse");
                    window.create_and_activate_toplevel_webview(state.clone(), url);
                },
                UserInterfaceCommand::CloseWebView(id) => {
                    window.set_needs_update();
                    window.close_webview(id);
                },
                UserInterfaceCommand::NewWindow => {
                    let url = Url::parse("servo:newtab").unwrap();
                    let platform_window =
                        self.create_platform_window(url.clone(), active_event_loop);
                    state.open_window(platform_window, url);
                },
            }
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

        {
            let AppState::Running(state) = &self.state else {
                return;
            };
            let window_id: u64 = window_id.into();
            if let Some(window) = state.window(window_id.into()) {
                window.platform_window().handle_winit_window_event(
                    state.clone(),
                    &window,
                    window_event,
                );
            }
        }

        if !self.pump_servo_event_loop(event_loop.into()) {
            event_loop.exit();
        }
        // Block until the window gets an event
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, app_event: AppEvent) {
        {
            let AppState::Running(state) = &self.state else {
                return;
            };
            if let Some(window_id) = app_event.window_id() {
                let window_id: u64 = window_id.into();
                if let Some(window) = state.window(window_id.into()) {
                    window.platform_window().handle_winit_app_event(app_event);
                }
            }
        }

        if !self.pump_servo_event_loop(event_loop.into()) {
            event_loop.exit();
        }

        // Block until the window gets an event
        event_loop.set_control_flow(ControlFlow::Wait);
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
