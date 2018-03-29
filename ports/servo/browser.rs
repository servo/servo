/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::compositor_thread::EmbedderMsg;
use compositing::windowing::{WebRenderDebugOption, WindowEvent};
use euclid::{TypedPoint2D, TypedVector2D};
use glutin_app::keyutils::{CMD_OR_CONTROL, CMD_OR_ALT};
use glutin_app::window::{Window, LINE_HEIGHT};
use msg::constellation_msg::{Key, TopLevelBrowsingContextId as BrowserId};
use msg::constellation_msg::{KeyModifiers, KeyState, TraversalDirection};
use script_traits::TouchEventType;
use servo::net_traits::pub_domains::is_reg_domain;
use servo::servo_url::{ChromeReader, ServoUrl};
use servo_config::prefs::PREFS;
use std::borrow::Cow;
use std::fs::{canonicalize, File};
use std::io::Read;
use std::mem;
use std::rc::Rc;
use std::str::Utf8Error;
use tinyfiledialogs;
use servo_config::resource_files::resources_dir_path;
use url::percent_encoding::percent_decode;
use webrender_api::ScrollLocation;

pub struct Browser {
    current_url: Option<ServoUrl>,
    /// id of the top level browsing context. It is unique as tabs
    /// are not supported yet. None until created.
    browser_id: Option<BrowserId>,

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

    pub fn set_browser_id(&mut self, browser_id: BrowserId) {
        self.browser_id = Some(browser_id);
    }

    pub fn handle_window_events(&mut self, events: Vec<WindowEvent>) {
        for event in events {
            match event {
                WindowEvent::KeyEvent(ch, key, state, mods) => {
                    self.handle_key_from_window(ch, key, state, mods);
                },
                event => {
                    self.event_queue.push(event);
                }
            }
        }
    }

    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    /// Handle key events before sending them to Servo.
    fn handle_key_from_window(&mut self, ch: Option<char>, key: Key, state: KeyState, mods: KeyModifiers) {
        match (mods, ch, key) {
            (CMD_OR_CONTROL, Some('r'), _) => {
                if let Some(id) = self.browser_id {
                    self.event_queue.push(WindowEvent::Reload(id));
                }
            }
            (CMD_OR_CONTROL, Some('l'), _) => {
                if let Some(id) = self.browser_id {
                    let url: String = if let Some(ref current_url) = self.current_url {
                        current_url.to_string()
                    } else {
                        String::from("")
                    };
                    let title = "URL or search query";
                    if let Some(input) = tinyfiledialogs::input_box(title, title, &url) {
                        if let Some(url) = sanitize_url(&input) {
                            self.event_queue.push(WindowEvent::LoadUrl(id, url));
                        }
                    }
                }
            }
            (CMD_OR_CONTROL, Some('q'), _) => {
                self.event_queue.push(WindowEvent::Quit);
            }
            (_, Some('3'), _) => if mods ^ KeyModifiers::CONTROL == KeyModifiers::SHIFT {
                self.event_queue.push(WindowEvent::CaptureWebRender);
            }
            (KeyModifiers::CONTROL, None, Key::F10) => {
                let event = WindowEvent::ToggleWebRenderDebug(WebRenderDebugOption::RenderTargetDebug);
                self.event_queue.push(event);
            }
            (KeyModifiers::CONTROL, None, Key::F11) => {
                let event = WindowEvent::ToggleWebRenderDebug(WebRenderDebugOption::TextureCacheDebug);
                self.event_queue.push(event);
            }
            (KeyModifiers::CONTROL, None, Key::F12) => {
                let event = WindowEvent::ToggleWebRenderDebug(WebRenderDebugOption::Profiler);
                self.event_queue.push(event);
            }
            (CMD_OR_ALT, None, Key::Right) | (KeyModifiers::NONE, None, Key::NavigateForward) => {
                if let Some(id) = self.browser_id {
                    let event = WindowEvent::Navigation(id, TraversalDirection::Forward(1));
                    self.event_queue.push(event);
                }
            }
            (CMD_OR_ALT, None, Key::Left) | (KeyModifiers::NONE, None, Key::NavigateBackward) => {
                if let Some(id) = self.browser_id {
                    let event = WindowEvent::Navigation(id, TraversalDirection::Back(1));
                    self.event_queue.push(event);
                }
            }
            (KeyModifiers::NONE, None, Key::Escape) => {
                self.event_queue.push(WindowEvent::Quit);
            }
            _ => {
                let event = self.platform_handle_key(key, mods);
                self.event_queue.push(event.unwrap_or(WindowEvent::KeyEvent(ch, key, state, mods)));
            }
        }

    }

    #[cfg(not(target_os = "win"))]
    fn platform_handle_key(&self, key: Key, mods: KeyModifiers) -> Option<WindowEvent> {
        match (mods, key, self.browser_id) {
            (CMD_OR_CONTROL, Key::LeftBracket, Some(id)) => {
                Some(WindowEvent::Navigation(id, TraversalDirection::Back(1)))
            }
            (CMD_OR_CONTROL, Key::RightBracket, Some(id)) => {
                Some(WindowEvent::Navigation(id, TraversalDirection::Forward(1)))
            }
            _ => None
        }
    }

    #[cfg(target_os = "win")]
    fn platform_handle_key(&self, key: Key, mods: KeyModifiers) -> Option<WindowEvent> {
        None
    }

    /// Handle key events after they have been handled by Servo.
    fn handle_key_from_servo(&mut self, _: Option<BrowserId>, ch: Option<char>,
                             key: Key, _: KeyState, mods: KeyModifiers) {
        match (mods, ch, key) {
            (_, Some('+'), _) => {
                if mods & !KeyModifiers::SHIFT == CMD_OR_CONTROL {
                    self.event_queue.push(WindowEvent::Zoom(1.1));
                } else if mods & !KeyModifiers::SHIFT == CMD_OR_CONTROL | KeyModifiers::ALT {
                    self.event_queue.push(WindowEvent::PinchZoom(1.1));
                }
            }
            (CMD_OR_CONTROL, Some('-'), _) => {
                self.event_queue.push(WindowEvent::Zoom(1.0 / 1.1));
            }
            (_, Some('-'), _) if mods == CMD_OR_CONTROL | KeyModifiers::ALT => {
                self.event_queue.push(WindowEvent::PinchZoom(1.0 / 1.1));
            }
            (CMD_OR_CONTROL, Some('0'), _) => {
                self.event_queue.push(WindowEvent::ResetZoom);
            }

            (KeyModifiers::NONE, None, Key::PageDown) => {
               let scroll_location = ScrollLocation::Delta(TypedVector2D::new(0.0,
                                   -self.window.page_height() + 2.0 * LINE_HEIGHT));
                self.scroll_window_from_key(scroll_location, TouchEventType::Move);
            }
            (KeyModifiers::NONE, None, Key::PageUp) => {
                let scroll_location = ScrollLocation::Delta(TypedVector2D::new(0.0,
                                   self.window.page_height() - 2.0 * LINE_HEIGHT));
                self.scroll_window_from_key(scroll_location, TouchEventType::Move);
            }

            (KeyModifiers::NONE, None, Key::Home) => {
                self.scroll_window_from_key(ScrollLocation::Start, TouchEventType::Move);
            }

            (KeyModifiers::NONE, None, Key::End) => {
                self.scroll_window_from_key(ScrollLocation::End, TouchEventType::Move);
            }

            (KeyModifiers::NONE, None, Key::Up) => {
                self.scroll_window_from_key(ScrollLocation::Delta(TypedVector2D::new(0.0, 3.0 * LINE_HEIGHT)),
                                            TouchEventType::Move);
            }
            (KeyModifiers::NONE, None, Key::Down) => {
                self.scroll_window_from_key(ScrollLocation::Delta(TypedVector2D::new(0.0, -3.0 * LINE_HEIGHT)),
                                            TouchEventType::Move);
            }
            (KeyModifiers::NONE, None, Key::Left) => {
                self.scroll_window_from_key(ScrollLocation::Delta(TypedVector2D::new(LINE_HEIGHT, 0.0)),
                                            TouchEventType::Move);
            }
            (KeyModifiers::NONE, None, Key::Right) => {
                self.scroll_window_from_key(ScrollLocation::Delta(TypedVector2D::new(-LINE_HEIGHT, 0.0)),
                                            TouchEventType::Move);
            }

            _ => {
            }
        }
    }

    fn scroll_window_from_key(&mut self, scroll_location: ScrollLocation, phase: TouchEventType) {
        let event = WindowEvent::Scroll(scroll_location, TypedPoint2D::zero(), phase);
        self.event_queue.push(event);
    }

    pub fn handle_servo_events(&mut self, events: Vec<EmbedderMsg>) {
        for event in events {
            match event {
                EmbedderMsg::Status(_browser_id, status) => {
                    self.status = status;
                },
                EmbedderMsg::ChangePageTitle(_browser_id, title) => {
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
                }
                EmbedderMsg::MoveTo(_browser_id, point) => {
                    self.window.set_position(point);
                }
                EmbedderMsg::ResizeTo(_browser_id, size) => {
                    self.window.set_inner_size(size);
                }
                EmbedderMsg::AllowNavigation(_browser_id, _url, response_chan) => {
                    if let Err(e) = response_chan.send(true) {
                        warn!("Failed to send allow_navigation() response: {}", e);
                    };
                }
                EmbedderMsg::KeyEvent(browser_id, ch, key, state, modified) => {
                    self.handle_key_from_servo(browser_id, ch, key, state, modified);
                }
                EmbedderMsg::SetCursor(cursor) => {
                    self.window.set_cursor(cursor);
                }
                EmbedderMsg::NewFavicon(_browser_id, url) => {
                    self.favicon = Some(url);
                }
                EmbedderMsg::HeadParsed(_browser_id, ) => {
                    self.loading_state = Some(LoadingState::Loading);
                }
                EmbedderMsg::HistoryChanged(_browser_id, entries, current) => {
                    self.current_url = Some(entries[current].url.clone());
                }
                EmbedderMsg::SetFullscreenState(_browser_id, state) => {
                    self.window.set_fullscreen(state);
                }
                EmbedderMsg::LoadStart(_browser_id) => {
                    self.loading_state = Some(LoadingState::Connecting);
                }
                EmbedderMsg::LoadComplete(_browser_id) => {
                    self.loading_state = Some(LoadingState::Loaded);
                }
                EmbedderMsg::Shutdown => {
                    self.shutdown_requested = true;
                },
                EmbedderMsg::Panic(_browser_id, _reason, _backtrace) => {
                }
            }
        }
    }

}

#[derive(Clone)]
pub struct FileChromeReader;
impl ChromeReader for FileChromeReader {
    fn clone(&self) -> Box<ChromeReader + Send> {
        Box::new(FileChromeReader)
    }
    fn resolve(&self, url: &ServoUrl) -> Result<Box<Read>, String> {
        if url.scheme() != "chrome" {
            Err("Not a chrome:// url".to_string())
        } else if url.host_str() != Some("resources") {
            Err("Not a chrome://resources/ url".to_string())
        } else {
            url.path_segments().map(|segments| {
                let segments: Vec<Result<Cow<str>, Utf8Error>> = segments.map(|s| {
                    percent_decode(s.as_bytes()).decode_utf8()
                }).collect();
                if segments.iter().any(|s| s.as_ref().map(|s| s.starts_with(".")).unwrap_or(true)) {
                    Err("Invalid Url".to_string())
                } else {
                    let resources = resources_dir_path().unwrap();
                    let mut path = resources.clone();
                    for segment in segments {
                        path.push(&*segment.unwrap());
                    }
                    match canonicalize(path) {
                        Ok(ref path) if path.starts_with(&resources) && path.exists() => {
                            match File::open(path) {
                                Err(e) => Err(format!("Error while opening file: {:?}", e)),
                                Ok(mut file) => {
                                    let reader: Box<Read> = Box::new(file);
                                    Ok(reader)
                                }
                            }
                        }
                        _ => Err("Can't find file".to_string())
                    }
                }
            }).unwrap_or(Err("Invalid Url".to_string()))
        }
    }
}

fn sanitize_url(request: &str) -> Option<ServoUrl> {
    let request = request.trim();
    ServoUrl::parse(&request).ok()
        .or_else(|| {
            if request.contains('/') || is_reg_domain(request) {
                ServoUrl::parse(&format!("http://{}", request)).ok()
            } else {
                None
            }
        }).or_else(|| {
            PREFS.get("shell.searchpage").as_string().and_then(|s: &str| {
                let url = s.replace("%s", request);
                ServoUrl::parse(&url).ok()
            })
        })
}
