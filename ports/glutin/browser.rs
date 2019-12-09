/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::keyutils::{CMD_OR_ALT, CMD_OR_CONTROL};
use crate::window_trait::{WindowPortsMethods, LINE_HEIGHT};
use euclid::{Point2D, Vector2D};
use keyboard_types::{Key, KeyboardEvent, Modifiers, ShortcutMatcher};
use servo::compositing::windowing::{WebRenderDebugOption, WindowEvent};
use servo::embedder_traits::{EmbedderMsg, FilterPattern, PromptDefinition, PromptOrigin, PromptResult};
use servo::msg::constellation_msg::TopLevelBrowsingContextId as BrowserId;
use servo::msg::constellation_msg::TraversalDirection;
use servo::net_traits::pub_domains::is_reg_domain;
use servo::script_traits::TouchEventType;
use servo::servo_config::opts;
use servo::servo_config::pref;
use servo::servo_url::ServoUrl;
use servo::webrender_api::ScrollLocation;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::env;
use std::fs::File;
use std::io::Write;
use std::mem;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use tinyfiledialogs::{self, MessageBoxIcon, OkCancel, YesNo};

pub struct Browser<Window: WindowPortsMethods + ?Sized> {
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
    clipboard_ctx: Option<ClipboardContext>,
    shutdown_requested: bool,
}

enum LoadingState {
    Connecting,
    Loading,
    Loaded,
}

impl<Window> Browser<Window>
where
    Window: WindowPortsMethods + ?Sized,
{
    pub fn new(window: Rc<Window>) -> Browser<Window> {
        Browser {
            title: None,
            current_url: None,
            browser_id: None,
            browsers: Vec::new(),
            status: None,
            favicon: None,
            loading_state: None,
            window: window,
            clipboard_ctx: match ClipboardContext::new() {
                Ok(c) => Some(c),
                Err(e) => {
                    warn!("Error creating clipboard context ({})", e);
                    None
                },
            },
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
                WindowEvent::Keyboard(key_event) => {
                    self.handle_key_from_window(key_event);
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
    fn handle_key_from_window(&mut self, key_event: KeyboardEvent) {
        ShortcutMatcher::from_event(key_event.clone())
            .shortcut(CMD_OR_CONTROL, 'R', || {
                if let Some(id) = self.browser_id {
                    self.event_queue.push(WindowEvent::Reload(id));
                }
            })
            .shortcut(CMD_OR_CONTROL, 'L', || {
                let url: String = if let Some(ref current_url) = self.current_url {
                    current_url.to_string()
                } else {
                    String::from("")
                };
                let title = "URL or search query";
                let input = tinyfiledialogs::input_box(title, title, &tiny_dialog_escape(&url));
                if let Some(input) = input {
                    if let Some(url) = sanitize_url(&input) {
                        if let Some(id) = self.browser_id {
                            self.event_queue.push(WindowEvent::LoadUrl(id, url));
                        }
                    }
                }
            })
            .shortcut(CMD_OR_CONTROL, 'Q', || {
                self.event_queue.push(WindowEvent::Quit);
            })
            .shortcut(CMD_OR_CONTROL, 'P', || {
                let rate = env::var("SAMPLING_RATE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10);
                let duration = env::var("SAMPLING_DURATION")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10);
                self.event_queue.push(WindowEvent::ToggleSamplingProfiler(
                    Duration::from_millis(rate),
                    Duration::from_secs(duration),
                ));
            })
            .shortcut(Modifiers::CONTROL, Key::F9, || {
                self.event_queue.push(WindowEvent::CaptureWebRender)
            })
            .shortcut(Modifiers::CONTROL, Key::F10, || {
                self.event_queue.push(WindowEvent::ToggleWebRenderDebug(
                    WebRenderDebugOption::RenderTargetDebug,
                ));
            })
            .shortcut(Modifiers::CONTROL, Key::F11, || {
                self.event_queue.push(WindowEvent::ToggleWebRenderDebug(
                    WebRenderDebugOption::TextureCacheDebug,
                ));
            })
            .shortcut(Modifiers::CONTROL, Key::F12, || {
                self.event_queue.push(WindowEvent::ToggleWebRenderDebug(
                    WebRenderDebugOption::Profiler,
                ));
            })
            .shortcut(CMD_OR_ALT, Key::ArrowRight, || {
                if let Some(id) = self.browser_id {
                    let event = WindowEvent::Navigation(id, TraversalDirection::Forward(1));
                    self.event_queue.push(event);
                }
            })
            .shortcut(CMD_OR_ALT, Key::ArrowLeft, || {
                if let Some(id) = self.browser_id {
                    let event = WindowEvent::Navigation(id, TraversalDirection::Back(1));
                    self.event_queue.push(event);
                }
            })
            .shortcut(Modifiers::empty(), Key::Escape, || {
                let state = self.window.get_fullscreen();
                if state {
                    if let Some(id) = self.browser_id {
                        let event = WindowEvent::ExitFullScreen(id);
                        self.event_queue.push(event);
                    }
                } else {
                    self.event_queue.push(WindowEvent::Quit);
                }
            })
            .otherwise(|| self.platform_handle_key(key_event));
    }

    #[cfg(not(target_os = "win"))]
    fn platform_handle_key(&mut self, key_event: KeyboardEvent) {
        if let Some(id) = self.browser_id {
            if let Some(event) = ShortcutMatcher::from_event(key_event.clone())
                .shortcut(CMD_OR_CONTROL, '[', || {
                    WindowEvent::Navigation(id, TraversalDirection::Back(1))
                })
                .shortcut(CMD_OR_CONTROL, ']', || {
                    WindowEvent::Navigation(id, TraversalDirection::Forward(1))
                })
                .otherwise(|| WindowEvent::Keyboard(key_event))
            {
                self.event_queue.push(event)
            }
        }
    }

    #[cfg(target_os = "win")]
    fn platform_handle_key(&mut self, _key_event: KeyboardEvent) {}

    /// Handle key events after they have been handled by Servo.
    fn handle_key_from_servo(&mut self, _: Option<BrowserId>, event: KeyboardEvent) {
        ShortcutMatcher::from_event(event)
            .shortcut(CMD_OR_CONTROL, '=', || {
                self.event_queue.push(WindowEvent::Zoom(1.1))
            })
            .shortcut(CMD_OR_CONTROL, '+', || {
                self.event_queue.push(WindowEvent::Zoom(1.1))
            })
            .shortcut(CMD_OR_CONTROL, '-', || {
                self.event_queue.push(WindowEvent::Zoom(1.0 / 1.1))
            })
            .shortcut(CMD_OR_CONTROL, '0', || {
                self.event_queue.push(WindowEvent::ResetZoom)
            })
            .shortcut(Modifiers::empty(), Key::PageDown, || {
                let scroll_location = ScrollLocation::Delta(Vector2D::new(
                    0.0,
                    -self.window.page_height() + 2.0 * LINE_HEIGHT,
                ));
                self.scroll_window_from_key(scroll_location, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::PageUp, || {
                let scroll_location = ScrollLocation::Delta(Vector2D::new(
                    0.0,
                    self.window.page_height() - 2.0 * LINE_HEIGHT,
                ));
                self.scroll_window_from_key(scroll_location, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::Home, || {
                self.scroll_window_from_key(ScrollLocation::Start, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::End, || {
                self.scroll_window_from_key(ScrollLocation::End, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::ArrowUp, || {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(Vector2D::new(0.0, 3.0 * LINE_HEIGHT)),
                    TouchEventType::Move,
                );
            })
            .shortcut(Modifiers::empty(), Key::ArrowDown, || {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(Vector2D::new(0.0, -3.0 * LINE_HEIGHT)),
                    TouchEventType::Move,
                );
            })
            .shortcut(Modifiers::empty(), Key::ArrowLeft, || {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(Vector2D::new(LINE_HEIGHT, 0.0)),
                    TouchEventType::Move,
                );
            })
            .shortcut(Modifiers::empty(), Key::ArrowRight, || {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(Vector2D::new(-LINE_HEIGHT, 0.0)),
                    TouchEventType::Move,
                );
            });
    }

    fn scroll_window_from_key(&mut self, scroll_location: ScrollLocation, phase: TouchEventType) {
        let event = WindowEvent::Scroll(scroll_location, Point2D::zero(), phase);
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
                EmbedderMsg::Prompt(definition, origin) => {
                    let res = if opts::get().headless {
                        match definition {
                            PromptDefinition::Alert(_message, sender) => {
                                sender.send(())
                            }
                            PromptDefinition::YesNo(_message, sender) => {
                                sender.send(PromptResult::Primary)
                            }
                            PromptDefinition::OkCancel(_message, sender) => {
                                sender.send(PromptResult::Primary)
                            }
                            PromptDefinition::Input(_message, default, sender) => {
                                sender.send(Some(default.to_owned()))
                            }
                        }
                    } else {
                        thread::Builder::new()
                            .name("display alert dialog".to_owned())
                            .spawn(move || {
                                match definition {
                                    PromptDefinition::Alert(mut message, sender) => {
                                        if origin == PromptOrigin::Untrusted {
                                            message = tiny_dialog_escape(&message);
                                        }
                                        tinyfiledialogs::message_box_ok(
                                            "Alert!",
                                            &message,
                                            MessageBoxIcon::Warning,
                                        );
                                        sender.send(())
                                    }
                                    PromptDefinition::YesNo(mut message, sender) => {
                                        if origin == PromptOrigin::Untrusted {
                                            message = tiny_dialog_escape(&message);
                                        }
                                        let result = tinyfiledialogs::message_box_yes_no(
                                            "", &message, MessageBoxIcon::Warning, YesNo::No,
                                        );
                                        sender.send(match result {
                                            YesNo::Yes => PromptResult::Primary,
                                            YesNo::No => PromptResult::Secondary,
                                        })
                                    }
                                    PromptDefinition::OkCancel(mut message, sender) => {
                                        if origin == PromptOrigin::Untrusted {
                                            message = tiny_dialog_escape(&message);
                                        }
                                        let result = tinyfiledialogs::message_box_ok_cancel(
                                            "", &message, MessageBoxIcon::Warning, OkCancel::Cancel,
                                        );
                                        sender.send(match result {
                                            OkCancel::Ok => PromptResult::Primary,
                                            OkCancel::Cancel => PromptResult::Secondary,
                                        })
                                    }
                                    PromptDefinition::Input(mut message, mut default, sender) => {
                                        if origin == PromptOrigin::Untrusted {
                                            message = tiny_dialog_escape(&message);
                                            default = tiny_dialog_escape(&default);
                                        }
                                        let result = tinyfiledialogs::input_box("", &message, &default);
                                        sender.send(result)
                                    }
                                }
                            })
                            .unwrap()
                            .join()
                            .expect("Thread spawning failed")
                    };
                    if let Err(e) = res {
                        let reason = format!("Failed to send Prompt response: {}", e);
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
                EmbedderMsg::AllowNavigationRequest(pipeline_id, _url) => {
                    if let Some(_browser_id) = browser_id {
                        self.event_queue
                            .push(WindowEvent::AllowNavigationResponse(pipeline_id, true));
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
                    } else {
                        error!("Multiple top level browsing contexts not supported yet.");
                    }
                    self.event_queue
                        .push(WindowEvent::SelectBrowser(new_browser_id));
                },
                EmbedderMsg::Keyboard(key_event) => {
                    self.handle_key_from_servo(browser_id, key_event);
                },
                EmbedderMsg::GetClipboardContents(sender) => {
                    let contents = match self.clipboard_ctx {
                        Some(ref mut ctx) => {
                            match ctx.get_contents() {
                                Ok(c) => c,
                                Err(e) => {
                                    warn!("Error getting clipboard contents ({}), defaulting to empty string", e);
                                    "".to_owned()
                                },
                            }
                        },
                        None => "".to_owned(),
                    };
                    if let Err(e) = sender.send(contents) {
                        warn!("Failed to send clipboard ({})", e);
                    }
                }
                EmbedderMsg::SetClipboardContents(text) => {
                    if let Some(ref mut ctx) = self.clipboard_ctx {
                        if let Err(e) = ctx.set_contents(text) {
                            warn!("Error setting clipboard contents ({})", e);
                        }
                    }
                }
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
                EmbedderMsg::ReportProfile(bytes) => {
                    let filename = env::var("PROFILE_OUTPUT").unwrap_or("samples.json".to_string());
                    let result = File::create(&filename).and_then(|mut f| f.write_all(&bytes));
                    if let Err(e) = result {
                        error!("Failed to store profile: {}", e);
                    }
                },
                EmbedderMsg::MediaSessionEvent(_) => {
                    debug!("MediaSessionEvent received");
                    // TODO(ferjm): MediaSession support for Glutin based browsers.
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
        })
        .unwrap()
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
                filters.push(tiny_dialog_escape(&s))
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
        })
        .unwrap()
        .join()
        .expect("Thread spawning failed")
}

fn sanitize_url(request: &str) -> Option<ServoUrl> {
    let request = request.trim();
    ServoUrl::parse(&request)
        .ok()
        .or_else(|| {
            if request.contains('/') || is_reg_domain(request) {
                ServoUrl::parse(&format!("https://{}", request)).ok()
            } else {
                None
            }
        })
        .or_else(|| {
            let url = pref!(shell.searchpage).replace("%s", request);
            ServoUrl::parse(&url).ok()
        })
}

// This is a mitigation for #25498, not a verified solution.
// There may be codepaths in tinyfiledialog.c that this is
// inadquate against, as it passes the string via shell to
// different programs depending on what the user has installed.
#[cfg(target_os = "linux")]
fn tiny_dialog_escape(raw: &str) -> String {
   let s:String = raw.chars()
       .filter_map(|c| match c {
           '\n' => Some('\n'),
           '\0' ..= '\x1f' => None,
           '<' => Some('\u{FF1C}'),
           '>' => Some('\u{FF1E}'),
           '&' => Some('\u{FF06}'),
           _ => Some(c)
       })
       .collect();
   return shellwords::escape(&s);
}

#[cfg(not(target_os = "linux"))]
fn tiny_dialog_escape(raw: &str) -> String {
   raw.to_string()
}
