/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use serde_json;
use servo::{self, gl, webrender_api, BrowserId, Servo};
use servo::compositing::windowing::{AnimationState, EmbedderCoordinates, MouseWindowEvent, WindowEvent, WindowMethods};
use servo::embedder_traits::EmbedderMsg;
use servo::embedder_traits::resources::{self, Resource};
use servo::euclid::{Length, TypedPoint2D, TypedScale, TypedSize2D, TypedVector2D};
use servo::msg::constellation_msg::TraversalDirection;
use servo::script_traits::{MouseButton, TouchEventType};
use servo::servo_config::opts;
use servo::servo_config::prefs::PREFS;
use servo::servo_url::ServoUrl;
use servo::style_traits::DevicePixel;
use std::cell::{Cell, RefCell};
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;

thread_local! {
    pub static SERVO: RefCell<Option<ServoGlue>> = RefCell::new(None);
}

/// The EventLoopWaker::wake function will be called from any thread.
/// It will be called to notify embedder that some events are available,
/// and that perform_updates need to be called
pub use servo::embedder_traits::EventLoopWaker;

/// Delegate resource file reading to the embedder.
pub trait ReadFileTrait {
    fn readfile(&self, file: &str) -> Vec<u8>;
}

/// Callbacks. Implemented by embedder. Called by Servo.
pub trait HostTrait {
    /// Will be called from the thread used for the init call.
    /// Will be called when the GL buffer has been updated.
    fn flush(&self);
    /// Will be called before drawing.
    /// Time to make the targetted GL context current.
    fn make_current(&self);
    /// Page starts loading.
    /// "Reload button" should be disabled.
    /// "Stop button" should be enabled.
    /// Throbber starts spinning.
    fn on_load_started(&self);
    /// Page has loaded.
    /// "Reload button" should be enabled.
    /// "Stop button" should be disabled.
    /// Throbber stops spinning.
    fn on_load_ended(&self);
    /// Page title has changed.
    fn on_title_changed(&self, title: String);
    /// Page URL has changed.
    fn on_url_changed(&self, url: String);
    /// Back/forward state has changed.
    /// Back/forward buttons need to be disabled/enabled.
    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool);
    /// Page animation state has changed. If animating, it's recommended
    /// that the embedder doesn't wait for the wake function to be called
    /// to call perform_updates. Usually, it means doing:
    /// while true { servo.perform_updates() }. This will end up calling flush
    /// which will call swap_buffer which will be blocking long enough to limit
    /// drawing at 60 FPS.
    /// If not animating, call perform_updates only when needed (when the embedder
    /// has events for Servo, or Servo has woken up the embedder event loop via
    /// EventLoopWaker).
    fn on_animating_changed(&self, animating: bool);
}

pub struct ServoGlue {
    servo: Servo<ServoCallbacks>,
    batch_mode: bool,
    callbacks: Rc<ServoCallbacks>,
    /// id of the top level browsing context. It is unique as tabs
    /// are not supported yet. None until created.
    browser_id: Option<BrowserId>,
    // A rudimentary stack of "tabs".
    // EmbedderMsg::BrowserCreated will push onto it.
    // EmbedderMsg::CloseBrowser will pop from it,
    // and exit if it is empty afterwards.
    browsers: Vec<BrowserId>,
    events: Vec<WindowEvent>,
    current_url: Option<ServoUrl>,
}

pub fn servo_version() -> String {
    servo::config::servo_version()
}

/// Initialize Servo. At that point, we need a valid GL context.
/// In the future, this will be done in multiple steps.
pub fn init(
    gl: Rc<gl::Gl>,
    argsline: String,
    embedder_url: Option<String>,
    waker: Box<EventLoopWaker>,
    readfile: Box<ReadFileTrait + Send + Sync>,
    callbacks: Box<HostTrait>,
    width: u32,
    height: u32,
) -> Result<(), &'static str> {
    resources::set(Box::new(ResourceReader(readfile)));

    if !argsline.is_empty() {
        let mut args: Vec<String> = serde_json::from_str(&argsline).map_err(|_| {
            "Invalid arguments. Servo arguments must be formatted as a JSON array"
        })?;
        // opts::from_cmdline_args expects the first argument to be the binary name.
        args.insert(0, "servo".to_string());
        opts::from_cmdline_args(&args);
    }

    let embedder_url = embedder_url.as_ref().and_then(|s| {
        ServoUrl::parse(s).ok()
    });
    let cmdline_url = opts::get().url.clone();
    let pref_url = PREFS.get("shell.homepage").as_string().and_then(|s| {
        ServoUrl::parse(s).ok()
    });
    let blank_url = ServoUrl::parse("about:blank").ok();

    let url = embedder_url
        .or(cmdline_url)
        .or(pref_url)
        .or(blank_url).unwrap();

    gl.clear_color(1.0, 1.0, 1.0, 1.0);
    gl.clear(gl::COLOR_BUFFER_BIT);
    gl.finish();

    let callbacks = Rc::new(ServoCallbacks {
        gl: gl.clone(),
        host_callbacks: callbacks,
        width: Cell::new(width),
        height: Cell::new(height),
        waker,
    });

    let servo = Servo::new(callbacks.clone());

    SERVO.with(|s| {
        let mut servo_glue = ServoGlue {
            servo,
            batch_mode: false,
            callbacks,
            browser_id: None,
            browsers: vec![],
            events: vec![],
            current_url: Some(url.clone()),
        };
        let browser_id = BrowserId::new();
        let _ = servo_glue.process_event(WindowEvent::NewBrowser(url, browser_id));
        *s.borrow_mut() = Some(servo_glue);
    });

    Ok(())
}

impl ServoGlue {
    fn get_browser_id(&self) -> Result<BrowserId, &'static str> {
        let browser_id = match self.browser_id {
            Some(id) => id,
            None => return Err("No BrowserId set yet.")
        };
        Ok(browser_id)
    }
    /// This is the Servo heartbeat. This needs to be called
    /// everytime wakeup is called or when embedder wants Servo
    /// to act on its pending events.
    pub fn perform_updates(&mut self) -> Result<(), &'static str> {
        debug!("perform_updates");
        let events = mem::replace(&mut self.events, Vec::new());
        self.servo.handle_events(events);
        self.handle_servo_events()
    }

    /// In batch mode, Servo won't call perform_updates automatically.
    /// This can be useful when the embedder wants to control when Servo
    /// acts on its pending events. For example, if the embedder wants Servo
    /// to act on the scroll events only at a certain time, not everytime
    /// scroll() is called.
    pub fn set_batch_mode(&mut self, batch: bool) -> Result<(), &'static str> {
        debug!("set_batch_mode");
        self.batch_mode = batch;
        Ok(())
    }

    /// Load an URL. This needs to be a valid url.
    pub fn load_uri(&mut self, url: &str) -> Result<(), &'static str> {
        info!("load_uri: {}", url);
        ServoUrl::parse(url)
            .map_err(|_| "Can't parse URL")
            .and_then(|url| {
                let browser_id = self.get_browser_id()?;
                let event = WindowEvent::LoadUrl(browser_id, url);
                self.process_event(event)
            })
    }

    /// Reload the page.
    pub fn reload(&mut self) -> Result<(), &'static str> {
        info!("reload");
        let browser_id = self.get_browser_id()?;
        let event = WindowEvent::Reload(browser_id);
        self.process_event(event)
    }

    /// Stop loading the page.
    pub fn stop(&mut self) -> Result<(), &'static str> {
        warn!("TODO can't stop won't stop");
        Ok(())
    }

    /// Go back in history.
    pub fn go_back(&mut self) -> Result<(), &'static str> {
        info!("go_back");
        let browser_id = self.get_browser_id()?;
        let event = WindowEvent::Navigation(browser_id, TraversalDirection::Back(1));
        self.process_event(event)
    }

    /// Go forward in history.
    pub fn go_forward(&mut self) -> Result<(), &'static str> {
        info!("go_forward");
        let browser_id = self.get_browser_id()?;
        let event = WindowEvent::Navigation(browser_id, TraversalDirection::Forward(1));
        self.process_event(event)
    }

    /// Let Servo know that the window has been resized.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), &'static str> {
        info!("resize");
        self.callbacks.width.set(width);
        self.callbacks.height.set(height);
        self.process_event(WindowEvent::Resize)
    }

    /// Start scrolling.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    pub fn scroll_start(&mut self, dx: i32, dy: i32, x: u32, y: u32) -> Result<(), &'static str> {
        let delta = TypedVector2D::new(dx as f32, dy as f32);
        let scroll_location = webrender_api::ScrollLocation::Delta(delta);
        let event = WindowEvent::Scroll(
            scroll_location,
            TypedPoint2D::new(x as i32, y as i32),
            TouchEventType::Down,
        );
        self.process_event(event)
    }

    /// Scroll.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    pub fn scroll(&mut self, dx: i32, dy: i32, x: u32, y: u32) -> Result<(), &'static str> {
        let delta = TypedVector2D::new(dx as f32, dy as f32);
        let scroll_location = webrender_api::ScrollLocation::Delta(delta);
        let event = WindowEvent::Scroll(
            scroll_location,
            TypedPoint2D::new(x as i32, y as i32),
            TouchEventType::Move,
        );
        self.process_event(event)
    }

    /// End scrolling.
    /// x/y are scroll coordinates.
    /// dx/dy are scroll deltas.
    pub fn scroll_end(&mut self, dx: i32, dy: i32, x: u32, y: u32) -> Result<(), &'static str> {
        let delta = TypedVector2D::new(dx as f32, dy as f32);
        let scroll_location = webrender_api::ScrollLocation::Delta(delta);
        let event = WindowEvent::Scroll(
            scroll_location,
            TypedPoint2D::new(x as i32, y as i32),
            TouchEventType::Up,
        );
        self.process_event(event)
    }

    /// Perform a click.
    pub fn click(&mut self, x: u32, y: u32) -> Result<(), &'static str> {
        let mouse_event =
            MouseWindowEvent::Click(MouseButton::Left, TypedPoint2D::new(x as f32, y as f32));
        let event = WindowEvent::MouseWindowEventClass(mouse_event);
        self.process_event(event)
    }

    fn process_event(&mut self, event: WindowEvent) -> Result<(), &'static str> {
        self.events.push(event);
        if !self.batch_mode {
            self.perform_updates()
        } else {
            Ok(())
        }
    }

    fn handle_servo_events(&mut self) -> Result<(), &'static str> {
        for (_browser_id, event) in self.servo.get_events() {
            match event {
                EmbedderMsg::ChangePageTitle(title) => {
                    let fallback_title: String = if let Some(ref current_url) = self.current_url {
                        current_url.to_string()
                    } else {
                        String::from("Untitled")
                    };
                    let title = match title {
                        Some(ref title) if title.len() > 0 => &**title,
                        _ => &fallback_title,
                    };
                    let title = format!("{} - Servo", title);
                    self.callbacks.host_callbacks.on_title_changed(title);
                },
                EmbedderMsg::AllowNavigation(_url, response_chan) => {
                    if let Err(e) = response_chan.send(true) {
                        warn!("Failed to send allow_navigation() response: {}", e);
                    };
                },
                EmbedderMsg::HistoryChanged(entries, current) => {
                    let can_go_back = current > 0;
                    let can_go_forward = current < entries.len() - 1;
                    self.callbacks
                        .host_callbacks
                        .on_history_changed(can_go_back, can_go_forward);
                    self.callbacks
                        .host_callbacks
                        .on_url_changed(entries[current].clone().to_string());
                    self.current_url = Some(entries[current].clone());
                },
                EmbedderMsg::LoadStart => {
                    self.callbacks.host_callbacks.on_load_started();
                },
                EmbedderMsg::LoadComplete => {
                    self.callbacks.host_callbacks.on_load_ended();
                },
                EmbedderMsg::GetSelectedBluetoothDevice(_, sender) => {
                    let _ = sender.send(None);
                },
                EmbedderMsg::AllowUnload(sender) => {
                    let _ = sender.send(true);
                },
                EmbedderMsg::Alert(message, sender) => {
                    info!("Alert: {}", message);
                    let _ = sender.send(());
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
                    self.events.push(WindowEvent::SelectBrowser(new_browser_id));
                },
                EmbedderMsg::CloseBrowser => {
                    // TODO: close the appropriate "tab".
                    let _ = self.browsers.pop();
                    if let Some(prev_browser_id) = self.browsers.last() {
                        self.browser_id = Some(*prev_browser_id);
                        self.events.push(WindowEvent::SelectBrowser(*prev_browser_id));
                    } else {
                        self.events.push(WindowEvent::Quit);
                    }
                },
                EmbedderMsg::Status(..) |
                EmbedderMsg::SelectFiles(..) |
                EmbedderMsg::MoveTo(..) |
                EmbedderMsg::ResizeTo(..) |
                EmbedderMsg::KeyEvent(..) |
                EmbedderMsg::SetCursor(..) |
                EmbedderMsg::NewFavicon(..) |
                EmbedderMsg::HeadParsed |
                EmbedderMsg::SetFullscreenState(..) |
                EmbedderMsg::ShowIME(..) |
                EmbedderMsg::HideIME |
                EmbedderMsg::Shutdown |
                EmbedderMsg::Panic(..) => {},
            }
        }
        Ok(())
    }
}

struct ServoCallbacks {
    waker: Box<EventLoopWaker>,
    gl: Rc<gl::Gl>,
    host_callbacks: Box<HostTrait>,
    width: Cell<u32>,
    height: Cell<u32>,
}

impl WindowMethods for ServoCallbacks {
    fn prepare_for_composite(
        &self,
        _width: Length<u32, DevicePixel>,
        _height: Length<u32, DevicePixel>,
    ) -> bool {
        debug!("WindowMethods::prepare_for_composite");
        self.host_callbacks.make_current();
        true
    }

    fn present(&self) {
        debug!("WindowMethods::present");
        self.host_callbacks.flush();
    }

    fn supports_clipboard(&self) -> bool {
        debug!("WindowMethods::supports_clipboard");
        false
    }

    fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        debug!("WindowMethods::create_event_loop_waker");
        self.waker.clone()
    }

    fn gl(&self) -> Rc<gl::Gl> {
        debug!("WindowMethods::gl");
        self.gl.clone()
    }

    fn set_animation_state(&self, state: AnimationState) {
        debug!("WindowMethods::set_animation_state");
        self.host_callbacks.on_animating_changed(state == AnimationState::Animating);
    }

    fn get_coordinates(&self) -> EmbedderCoordinates {
        let size = TypedSize2D::new(self.width.get(), self.height.get());
        EmbedderCoordinates {
            viewport: webrender_api::DeviceUintRect::new(TypedPoint2D::zero(), size),
            framebuffer: size,
            window: (size, TypedPoint2D::new(0, 0)),
            screen: size,
            screen_avail: size,
            hidpi_factor: TypedScale::new(2.0),
        }
    }
}

struct ResourceReader(Box<ReadFileTrait + Send + Sync>);

impl resources::ResourceReaderMethods for ResourceReader {
    fn read(&self, file: Resource) -> Vec<u8> {
        let file = match file {
            Resource::Preferences => "prefs.json",
            Resource::BluetoothBlocklist => "gatt_blocklist.txt",
            Resource::DomainList => "public_domains.txt",
            Resource::HstsPreloadList => "hsts_preload.json",
            Resource::SSLCertificates => "certs",
            Resource::BadCertHTML => "badcert.html",
            Resource::NetErrorHTML => "neterror.html",
            Resource::UserAgentCSS => "user-agent.css",
            Resource::ServoCSS => "servo.css",
            Resource::PresentationalHintsCSS => "presentational-hints.css",
            Resource::QuirksModeCSS => "quirks-mode.css",
            Resource::RippyPNG => "rippy.png",
        };
        info!("ResourceReader::read({})", file);
        self.0.readfile(file)
    }
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![]
    }
    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }
}
