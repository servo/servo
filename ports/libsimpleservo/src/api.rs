/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo::{self, gl, webrender_api, BrowserId, Servo};
use servo::compositing::windowing::{AnimationState, EmbedderCoordinates, MouseWindowEvent, WindowEvent, WindowMethods};
use servo::embedder_traits::EmbedderMsg;
use servo::embedder_traits::resources::{self, Resource};
use servo::euclid::{Length, TypedPoint2D, TypedScale, TypedSize2D, TypedVector2D};
use servo::ipc_channel::ipc;
use servo::msg::constellation_msg::TraversalDirection;
use servo::script_traits::{MouseButton, TouchEventType};
use servo::servo_config::opts;
use servo::servo_url::ServoUrl;
use servo::style_traits::DevicePixel;
use std::cell::{Cell, RefCell};
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;

thread_local! {
    pub static SERVO: RefCell<Option<ServoGlue>> = RefCell::new(None);
}

/// The wake function will be called from any thread.
/// Will be called to notify embedder that some events
/// are available, and that perform_updates need to be called
pub use servo::embedder_traits::EventLoopWaker;

/// Delegate resource file reading to embedder.
pub trait ReadFileTrait {
    fn readfile(&self, file: &str) -> Vec<u8>;
}

/// Callbacks. Implemented by emebedder. Called by Servo.
pub trait HostTrait {
    /// Will be called from the thread used for the init call
    /// Will be called when the GL buffer has been updated.
    fn flush(&self);
    /// Page starts loading.
    /// "Reload button" becomes "Stop button".
    /// Throbber starts spinning.
    fn on_load_started(&self);
    /// Page has loaded.
    /// "Stop button" becomes "Reload button".
    /// Throbber stops spinning.
    fn on_load_ended(&self);
    /// Page title changed.
    fn on_title_changed(&self, title: String);
    /// Page URL changed.
    fn on_url_changed(&self, url: String);
    /// Back/forward state changed.
    /// Back/forward buttons need to be disabled/enabled.
    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool);
    /// Page animation state has changed. If animating, it's recommended
    /// that the embedder redraws based on vsync (usually, the swap_buffer
    /// call should be blocking the event loop). Usually, that means just doing:
    /// while true { servo.perform_updates() }. This will end up calling flush
    /// which will call swap_buffer which will be blocking long enough to limit
    /// drawing at 60 FPS.
    /// If not animating, call perform_updates only when needed (embedder has
    /// events for Servo, or Servo has woke up the embedder event loop via
    /// EventLoopWaker).
    fn on_animating_changed(&self, animating: bool);
}

pub struct ServoGlue {
    servo: Servo<ServoCallbacks>,
    callbacks: Rc<ServoCallbacks>,
    browser_id: BrowserId,
    events: Vec<WindowEvent>,
    current_url: Option<ServoUrl>,
}

pub fn servo_version() -> String {
    servo::config::servo_version()
}

/// Initialize Servo. At that point, we already need a URL and valid GL context.
/// In the future, this will be done in multiple steps.
pub fn init(
    gl: Rc<gl::Gl>,
    url: String,
    waker: Box<EventLoopWaker>,
    readfile: Box<ReadFileTrait + Send + Sync>,
    callbacks: Box<HostTrait>,
    width: u32,
    height: u32,
) -> Result<(), &'static str> {
    resources::set(Box::new(ResourceReader(readfile)));

    let mut opts = opts::default_opts();
    opts.enable_subpixel_text_antialiasing = true; // FIXME: If VR, false.
    opts::set_defaults(opts);

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

    let mut servo = Servo::new(callbacks.clone());

    let url = ServoUrl::parse(&url).map_err(|_| "Can't parse URL")?;
    let (sender, receiver) = ipc::channel().map_err(|_| "Can't create ipc::channel")?;
    servo.handle_events(vec![WindowEvent::NewBrowser(url.clone(), sender)]);
    let browser_id = receiver.recv().map_err(|_| "Can't receive browser_id")?;
    servo.handle_events(vec![WindowEvent::SelectBrowser(browser_id)]);

    SERVO.with(|s| {
        *s.borrow_mut() = Some(ServoGlue {
            servo,
            callbacks,
            browser_id,
            events: vec![],
            current_url: Some(url),
        });
    });

    Ok(())
}

impl ServoGlue {
    /// This is the Servo heartbeat. This needs to be called
    /// everytime wakeup is called.
    pub fn perform_updates(&mut self) -> Result<(), &'static str> {
        debug!("perform_updates");
        let events = mem::replace(&mut self.events, Vec::new());
        self.servo.handle_events(events);
        self.handle_servo_events()
    }

    /// Load an URL. This needs to be a valid url.
    pub fn load_uri(&mut self, url: &str) -> Result<(), &'static str> {
        info!("load_uri: {}", url);
        ServoUrl::parse(url)
            .map_err(|_| "Can't parse URL")
            .map(|url| {
                self.servo
                    .handle_events(vec![WindowEvent::LoadUrl(self.browser_id, url)])
            })
    }

    /// Reload the page.
    pub fn reload(&mut self) -> Result<(), &'static str> {
        info!("reload");
        let event = WindowEvent::Reload(self.browser_id);
        self.servo.handle_events(vec![event]);
        Ok(())
    }

    /// Go back in history.
    pub fn go_back(&mut self) -> Result<(), &'static str> {
        info!("go_back");
        let event = WindowEvent::Navigation(self.browser_id, TraversalDirection::Back(1));
        self.servo.handle_events(vec![event]);
        Ok(())
    }

    /// Go forward in history.
    pub fn go_forward(&mut self) -> Result<(), &'static str> {
        info!("go_forward");
        let event = WindowEvent::Navigation(self.browser_id, TraversalDirection::Forward(1));
        self.servo.handle_events(vec![event]);
        Ok(())
    }

    /// Let Servo know that the window has been resized.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), &'static str> {
        info!("resize");
        self.callbacks.width.set(width);
        self.callbacks.height.set(height);
        let event = WindowEvent::Resize;
        self.servo.handle_events(vec![event]);
        Ok(())
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
        self.events.push(event);
        Ok(())
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
        self.events.push(event);
        Ok(())
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
        self.events.push(event);
        Ok(())
    }

    /// Perform a click.
    pub fn click(&mut self, x: u32, y: u32) -> Result<(), &'static str> {
        let mouse_event =
            MouseWindowEvent::Click(MouseButton::Left, TypedPoint2D::new(x as f32, y as f32));
        let event = WindowEvent::MouseWindowEventClass(mouse_event);
        self.servo.handle_events(vec![event]);
        Ok(())
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
                EmbedderMsg::CloseBrowser |
                EmbedderMsg::Alert(..) |
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
        debug!("ResourceReader::read");
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
        self.0.readfile(file)
    }
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![]
    }
    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }
}
