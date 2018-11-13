/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::{TypedPoint2D, TypedVector2D};
use glutin_app::keyutils::{CMD_OR_CONTROL, CMD_OR_ALT};
use glutin_app::window::{Window, LINE_HEIGHT};
use servo::compositing::windowing::{WebRenderDebugOption, WindowEvent};
use servo::embedder_traits::{EmbedderMsg, FilterPattern};
use servo::msg::constellation_msg::{Key, TopLevelBrowsingContextId as BrowserId};
use servo::msg::constellation_msg::{KeyModifiers, KeyState, TraversalDirection};
use servo::net_traits::pub_domains::is_reg_domain;
use servo::script_traits::TouchEventType;
use servo::servo_config::opts;
use servo::servo_config::prefs::PREFS;
use servo::servo_url::ServoUrl;
use servo::webrender_api::ScrollLocation;
use std::mem;
use std::rc::Rc;
use std::thread;
use tinyfiledialogs::{self, MessageBoxIcon};

pub struct Browser {
    current_url: Option<ServoUrl>,
    /// id of the top level browsing context. It is unique as tabs
    /// are not supported yet. None until created.
    browser_id: Option<BrowserId>,

    // A rudimentary stack of "tabs".
    // EmbedderMsg::BrowserCreated will push onto it.
    // EmbedderMsg::CloseBrowser will pop from it,
    // and exit if it is empty afterwards.
    browsers: Vec<BrowserId>,

    title: Option<String>,
    status: Option<String>,
    favicon: Option<ServoUrl>,
    loading_state: Option<LoadingState>,
    window: Rc<Window>,
    event_queue: Vec<WindowEvent>,
    shutdown_requested: bool,
}

enum LoadingState {
    Connecting,
    Loading,
    Loaded,
}

impl Browser {
    pub fn new(window: Rc<Window>) -> Browser {
        Browser {
            title: None,
            current_url: None,
            browser_id: None,
            browsers: Vec::new(),
            status: None,
            favicon: None,
            loading_state: None,
            window: window,
            event_queue: Vec::new(),
            shutdown_requested: false,
        }
    }

    pub fn get_events(&mut self) -> Vec<WindowEvent> {
        mem::replace(&mut self.event_queue, Vec::new())
    }

    pub fn handle_window_events(&mut self, events: Vec<WindowEvent>) {
        for event in events {
            match event {
                WindowEvent::KeyEvent(ch, key, state, mods) => {
                    self.handle_key_from_window(ch, key, state, mods);
                },
                event => {
                    self.event_queue.push(event);
                },
            }
        }
    }

    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    /// Handle key events before sending them to Servo.
    fn handle_key_from_window(
        &mut self,
        ch: Option<char>,
        key: Key,
        state: KeyState,
        mods: KeyModifiers,
    ) {
        let pressed = state == KeyState::Pressed;
        // We don't match the state in the parent `match` because we don't want to do anything
        // on KeyState::Released when it's a combo that we handle on Pressed. For example,
        // if we catch Alt-Left on pressed, we don't want the Release event to be sent to Servo.
        match (mods, ch, key, self.browser_id) {
            (CMD_OR_CONTROL, _, Key::R, Some(id)) => if pressed {
                self.event_queue.push(WindowEvent::Reload(id));
            },
            (CMD_OR_CONTROL, _, Key::L, Some(id)) => if pressed {
                let url: String = if let Some(ref current_url) = self.current_url {
                    current_url.to_string()
                } else {
                    String::from("")
                };
                let title = "URL or search query";
                let input = tinyfiledialogs::input_box(title, title, &url);
                if let Some(input) = input {
                    if let Some(url) = sanitize_url(&input) {
                        self.event_queue.push(WindowEvent::LoadUrl(id, url));
                    }
                }
            },
            (CMD_OR_CONTROL, _, Key::Q, _) => if pressed {
                self.event_queue.push(WindowEvent::Quit);
            },
            (_, Some('3'), _, _) if mods ^ KeyModifiers::CONTROL == KeyModifiers::SHIFT => {
                if pressed {
                    self.event_queue.push(WindowEvent::CaptureWebRender);
                }
            },
            (KeyModifiers::CONTROL, None, Key::F10, _) => if pressed {
                let event =
                    WindowEvent::ToggleWebRenderDebug(WebRenderDebugOption::RenderTargetDebug);
                self.event_queue.push(event);
            },
            (KeyModifiers::CONTROL, None, Key::F11, _) => if pressed {
                let event =
                    WindowEvent::ToggleWebRenderDebug(WebRenderDebugOption::TextureCacheDebug);
                self.event_queue.push(event);
            },
            (KeyModifiers::CONTROL, None, Key::F12, _) => if pressed {
                let event = WindowEvent::ToggleWebRenderDebug(WebRenderDebugOption::Profiler);
                self.event_queue.push(event);
            },
            (CMD_OR_ALT, None, Key::Right, Some(id)) |
            (KeyModifiers::NONE, None, Key::NavigateForward, Some(id)) => if pressed {
                let event = WindowEvent::Navigation(id, TraversalDirection::Forward(1));
                self.event_queue.push(event);
            },
            (CMD_OR_ALT, None, Key::Left, Some(id)) |
            (KeyModifiers::NONE, None, Key::NavigateBackward, Some(id)) => if pressed {
                let event = WindowEvent::Navigation(id, TraversalDirection::Back(1));
                self.event_queue.push(event);
            },
            (KeyModifiers::NONE, None, Key::Escape, _) => if pressed {
                self.event_queue.push(WindowEvent::Quit);
            },
            _ => {
                self.platform_handle_key(ch, key, mods, state);
            },
        }
    }

    #[cfg(not(target_os = "win"))]
    fn platform_handle_key(
        &mut self,
        ch: Option<char>,
        key: Key,
        mods: KeyModifiers,
        state: KeyState,
    ) {
        let pressed = state == KeyState::Pressed;
        match (mods, key, self.browser_id) {
            (CMD_OR_CONTROL, Key::LeftBracket, Some(id)) => if pressed {
                let event = WindowEvent::Navigation(id, TraversalDirection::Back(1));
                self.event_queue.push(event);
            },
            (CMD_OR_CONTROL, Key::RightBracket, Some(id)) => if pressed {
                let event = WindowEvent::Navigation(id, TraversalDirection::Back(1));
                self.event_queue.push(event);
            },
            _ => {
                self.event_queue
                    .push(WindowEvent::KeyEvent(ch, key, state, mods));
            },
        }
    }

    #[cfg(target_os = "win")]
    fn platform_handle_key(
        &mut self,
        _ch: Option<char>,
        _key: Key,
        _mods: KeyModifiers,
        _state: KeyState,
    ) {
    }

    /// Handle key events after they have been handled by Servo.
    fn handle_key_from_servo(
        &mut self,
        _: Option<BrowserId>,
        ch: Option<char>,
        key: Key,
        state: KeyState,
        mods: KeyModifiers,
    ) {
        if state == KeyState::Released {
            return;
        }

        match (mods, ch, key) {
            (CMD_OR_CONTROL, Some('='), _) | (CMD_OR_CONTROL, Some('+'), _) => {
                self.event_queue.push(WindowEvent::Zoom(1.1));
            },
            (_, Some('='), _) if mods == (CMD_OR_CONTROL | KeyModifiers::SHIFT) => {
                self.event_queue.push(WindowEvent::Zoom(1.1));
            },
            (CMD_OR_CONTROL, Some('-'), _) => {
                self.event_queue.push(WindowEvent::Zoom(1.0 / 1.1));
            },
            (CMD_OR_CONTROL, Some('0'), _) => {
                self.event_queue.push(WindowEvent::ResetZoom);
            },

            (KeyModifiers::NONE, None, Key::PageDown) => {
                let scroll_location = ScrollLocation::Delta(TypedVector2D::new(
                    0.0,
                    -self.window.page_height() + 2.0 * LINE_HEIGHT,
                ));
                self.scroll_window_from_key(scroll_location, TouchEventType::Move);
            },
            (KeyModifiers::NONE, None, Key::PageUp) => {
                let scroll_location = ScrollLocation::Delta(TypedVector2D::new(
                    0.0,
                    self.window.page_height() - 2.0 * LINE_HEIGHT,
                ));
                self.scroll_window_from_key(scroll_location, TouchEventType::Move);
            },

            (KeyModifiers::NONE, None, Key::Home) => {
                self.scroll_window_from_key(ScrollLocation::Start, TouchEventType::Move);
            },

            (KeyModifiers::NONE, None, Key::End) => {
                self.scroll_window_from_key(ScrollLocation::End, TouchEventType::Move);
            },

            (KeyModifiers::NONE, None, Key::Up) => {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(TypedVector2D::new(0.0, 3.0 * LINE_HEIGHT)),
                    TouchEventType::Move,
                );
            },
            (KeyModifiers::NONE, None, Key::Down) => {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(TypedVector2D::new(0.0, -3.0 * LINE_HEIGHT)),
                    TouchEventType::Move,
                );
            },
            (KeyModifiers::NONE, None, Key::Left) => {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(TypedVector2D::new(LINE_HEIGHT, 0.0)),
                    TouchEventType::Move,
                );
            },
            (KeyModifiers::NONE, None, Key::Right) => {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(TypedVector2D::new(-LINE_HEIGHT, 0.0)),
                    TouchEventType::Move,
                );
            },

            _ => {},
        }
    }

    fn scroll_window_from_key(&mut self, scroll_location: ScrollLocation, phase: TouchEventType) {
        let event = WindowEvent::Scroll(scroll_location, TypedPoint2D::zero(), phase);
        self.event_queue.push(event);
    }

    pub fn handle_servo_events(&mut self, events: Vec<(Option<BrowserId>, EmbedderMsg)>) {
        for (browser_id, msg) in events {
            match msg {
                EmbedderMsg::Status(status) => {
                    self.status = status;
                },
                EmbedderMsg::ChangePageTitle(title) => {
                    self.title = title;

                    let fallback_title: String = if let Some(ref current_url) = self.current_url {
                        current_url.to_string()
                    } else {
                        String::from("Untitled")
                    };
                    let title = match self.title {
                        Some(ref title) if title.len() > 0 => &**title,
                        _ => &fallback_title,
                    };
                    let title = format!("{} - Servo", title);
                    self.window.set_title(&title);
                },
                EmbedderMsg::MoveTo(point) => {
                    self.window.set_position(point);
                },
                EmbedderMsg::ResizeTo(size) => {
                    self.window.set_inner_size(size);
                },
                EmbedderMsg::Alert(message, sender) => {
                    if !opts::get().headless {
                        let _ = thread::Builder::new()
                            .name("display alert dialog".to_owned())
                            .spawn(move || {
                                tinyfiledialogs::message_box_ok(
                                    "Alert!",
                                    &message,
                                    MessageBoxIcon::Warning,
                                );
                            }).unwrap()
                            .join()
                            .expect("Thread spawning failed");
                    }
                    if let Err(e) = sender.send(()) {
                        let reason = format!("Failed to send Alert response: {}", e);
                        self.event_queue
                            .push(WindowEvent::SendError(browser_id, reason));
                    }
                },
                EmbedderMsg::AllowUnload(sender) => {
                    // Always allow unload for now.
                    if let Err(e) = sender.send(true) {
                        let reason = format!("Failed to send AllowUnload response: {}", e);
                        self.event_queue
                            .push(WindowEvent::SendError(browser_id, reason));
                    }
                },
                EmbedderMsg::AllowNavigation(_url, sender) => {
                    if let Err(e) = sender.send(true) {
                        warn!("Failed to send AllowNavigation response: {}", e);
                    }
                },
                EmbedderMsg::AllowOpeningBrowser(response_chan) => {
                    // Note: would be a place to handle pop-ups config.
                    // see Step 7 of #the-rules-for-choosing-a-browsing-context-given-a-browsing-context-name
                    if let Err(e) = response_chan.send(true) {
                        warn!("Failed to send AllowOpeningBrowser response: {}", e);
                    };
                },
                EmbedderMsg::BrowserCreated(new_browser_id) => {
                    // TODO: properly handle a new "tab"
                    self.browsers.push(new_browser_id);
                    if self.browser_id.is_none() {
                        self.browser_id = Some(new_browser_id);
                    }
                    self.event_queue
                        .push(WindowEvent::SelectBrowser(new_browser_id));
                },
                EmbedderMsg::KeyEvent(ch, key, state, modified) => {
                    self.handle_key_from_servo(browser_id, ch, key, state, modified);
                },
                EmbedderMsg::SetCursor(cursor) => {
                    self.window.set_cursor(cursor);
                },
                EmbedderMsg::NewFavicon(url) => {
                    self.favicon = Some(url);
                },
                EmbedderMsg::HeadParsed => {
                    self.loading_state = Some(LoadingState::Loading);
                },
                EmbedderMsg::HistoryChanged(urls, current) => {
                    self.current_url = Some(urls[current].clone());
                },
                EmbedderMsg::SetFullscreenState(state) => {
                    self.window.set_fullscreen(state);
                },
                EmbedderMsg::LoadStart => {
                    self.loading_state = Some(LoadingState::Connecting);
                },
                EmbedderMsg::LoadComplete => {
                    self.loading_state = Some(LoadingState::Loaded);
                },
                EmbedderMsg::CloseBrowser => {
                    // TODO: close the appropriate "tab".
                    let _ = self.browsers.pop();
                    if let Some(prev_browser_id) = self.browsers.last() {
                        self.browser_id = Some(*prev_browser_id);
                        self.event_queue
                            .push(WindowEvent::SelectBrowser(*prev_browser_id));
                    } else {
                        self.event_queue.push(WindowEvent::Quit);
                    }
                },
                EmbedderMsg::Shutdown => {
                    self.shutdown_requested = true;
                },
                EmbedderMsg::Panic(_reason, _backtrace) => {},
                EmbedderMsg::GetSelectedBluetoothDevice(devices, sender) => {
                    let selected = platform_get_selected_devices(devices);
                    if let Err(e) = sender.send(selected) {
                        let reason =
                            format!("Failed to send GetSelectedBluetoothDevice response: {}", e);
                        self.event_queue.push(WindowEvent::SendError(None, reason));
                    };
                },
                EmbedderMsg::SelectFiles(patterns, multiple_files, sender) => {
                    let res = match (
                        opts::get().headless,
                        get_selected_files(patterns, multiple_files),
                    ) {
                        (true, _) | (false, None) => sender.send(None),
                        (false, Some(files)) => sender.send(Some(files)),
                    };
                    if let Err(e) = res {
                        let reason = format!("Failed to send SelectFiles response: {}", e);
                        self.event_queue.push(WindowEvent::SendError(None, reason));
                    };
                },
                EmbedderMsg::ShowIME(_kind) => {
                    debug!("ShowIME received");
                },
                EmbedderMsg::HideIME => {
                    debug!("HideIME received");
                },
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn platform_get_selected_devices(devices: Vec<String>) -> Option<String> {
    let picker_name = "Choose a device";

    thread::Builder::new()
        .name(picker_name.to_owned())
        .spawn(move || {
            let dialog_rows: Vec<&str> = devices.iter().map(|s| s.as_ref()).collect();
            let dialog_rows: Option<&[&str]> = Some(dialog_rows.as_slice());

            match tinyfiledialogs::list_dialog("Choose a device", &["Id", "Name"], dialog_rows) {
                Some(device) => {
                    // The device string format will be "Address|Name". We need the first part of it.
                    device.split("|").next().map(|s| s.to_string())
                },
                None => None,
            }
        }).unwrap()
        .join()
        .expect("Thread spawning failed")
}

#[cfg(not(target_os = "linux"))]
fn platform_get_selected_devices(devices: Vec<String>) -> Option<String> {
    for device in devices {
        if let Some(address) = device.split("|").next().map(|s| s.to_string()) {
            return Some(address);
        }
    }
    None
}

fn get_selected_files(patterns: Vec<FilterPattern>, multiple_files: bool) -> Option<Vec<String>> {
    let picker_name = if multiple_files {
        "Pick files"
    } else {
        "Pick a file"
    };
    thread::Builder::new()
        .name(picker_name.to_owned())
        .spawn(move || {
            let mut filters = vec![];
            for p in patterns {
                let s = "*.".to_string() + &p.0;
                filters.push(s)
            }
            let filter_ref = &(filters.iter().map(|s| s.as_str()).collect::<Vec<&str>>()[..]);
            let filter_opt = if filters.len() > 0 {
                Some((filter_ref, ""))
            } else {
                None
            };

            if multiple_files {
                tinyfiledialogs::open_file_dialog_multi(picker_name, "", filter_opt)
            } else {
                let file = tinyfiledialogs::open_file_dialog(picker_name, "", filter_opt);
                file.map(|x| vec![x])
            }
        }).unwrap()
        .join()
        .expect("Thread spawning failed")
}

fn sanitize_url(request: &str) -> Option<ServoUrl> {
    let request = request.trim();
    ServoUrl::parse(&request)
        .ok()
        .or_else(|| {
            if request.contains('/') || is_reg_domain(request) {
                ServoUrl::parse(&format!("http://{}", request)).ok()
            } else {
                None
            }
        }).or_else(|| {
            PREFS
                .get("shell.searchpage")
                .as_string()
                .and_then(|s: &str| {
                    let url = s.replace("%s", request);
                    ServoUrl::parse(&url).ok()
                })
        })
}
